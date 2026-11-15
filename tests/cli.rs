use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

fn binary() -> PathBuf {
    std::env::var_os("SENDMAIL2HTTP_TEST_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_sendmail2http")))
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "sendmail2http-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn binary(&self) -> PathBuf {
        let path = self.0.join("sendmail");
        fs::copy(binary(), &path).unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(binary: &Path, args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(binary)
        .args(args)
        .env("NO_PROXY", "*")
        .env("no_proxy", "*")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let output = child.wait_with_output().unwrap();
            panic!(
                "command timed out: {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
}
fn ok(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn append_payload(path: &Path, data: &[u8]) {
    use sendmail2http::rtad;
    #[cfg(unix)]
    use std::os::unix::ffi::OsStrExt;
    #[cfg(unix)]
    let bytes = path.as_os_str().as_bytes();
    #[cfg(not(unix))]
    let bytes = path.to_str().unwrap().as_bytes();
    let path = std::ffi::CString::new(bytes).unwrap();
    assert_eq!(
        unsafe { rtad::rtad_set_data(path.as_ptr(), data.as_ptr().cast(), data.len()) },
        0
    );
}

#[test]
fn cli_validation_and_help() {
    for args in [
        vec![],
        vec!["--pack-self"],
        vec!["--pack-self", "--truncate"],
        vec!["--url", "not-a-url"],
    ] {
        assert_eq!(run(&binary(), &args, b"").status.code(), Some(2));
    }
    for flag in ["--help", "--version"] {
        ok(run(&binary(), &[flag], b""));
    }
}

#[cfg(unix)]
#[test]
fn self_configuration_round_trip_and_repeated_updates() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let path = fixture.binary();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o751)).unwrap();
    ok(run(
        &path,
        &[
            "--pack-self",
            "--url",
            "http://localhost/one",
            "--auth",
            "Bearer space ' quote",
        ],
        b"",
    ));
    let size = fs::metadata(&path).unwrap().len();
    ok(run(
        &path,
        &["--pack-self", "--url", "http://localhost/two"],
        b"",
    ));
    assert_eq!(fs::metadata(&path).unwrap().len(), size);
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o751
    );
    ok(run(&path, &["--truncate"], b""));
    assert_eq!(fs::read(&path).unwrap(), fs::read(binary()).unwrap());
    ok(run(&path, &["--truncate"], b""));
    assert_eq!(fs::read(&path).unwrap(), fs::read(binary()).unwrap());
    assert_eq!(run(&path, &[], b"").status.code(), Some(2));
    ok(run(&path, &["--version"], b""));
}

#[cfg(unix)]
#[test]
fn invalid_embedded_configuration_is_rejected_and_can_be_cleared() {
    for payload in [
        &b"\xff"[..],
        b"'unterminated",
        b"--auth only",
        b"--url http://localhost --truncate",
        b"--url http://localhost --pack-self",
        b"--url http://localhost recipient",
        b"--url invalid",
    ] {
        let fixture = Fixture::new();
        let path = fixture.binary();
        append_payload(&path, payload);
        assert_eq!(run(&path, &[], b"").status.code(), Some(1), "{payload:?}");
        ok(run(&path, &["--truncate"], b""));
        assert_eq!(fs::read(&path).unwrap(), fs::read(binary()).unwrap());
    }
}

#[test]
fn malformed_rtad_trailer_is_rejected_without_changing_file() {
    let fixture = Fixture::new();
    let path = fixture.binary();
    let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
    file.write_all(&u32::MAX.to_ne_bytes()).unwrap();
    file.write_all(b"\x01*RTAD").unwrap();
    drop(file);
    let before = fs::read(&path).unwrap();
    for args in [&["--url", "http://localhost"][..], &["--truncate"][..]] {
        assert_eq!(run(&path, args, b"").status.code(), Some(1));
        assert_eq!(fs::read(&path).unwrap(), before);
    }
}

fn read_request(stream: &mut TcpStream) -> (String, serde_json::Value) {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut bytes = Vec::new();
    let mut buf = [0; 4096];
    let (end, length) = loop {
        let count = stream.read(&mut buf).unwrap();
        assert!(count > 0);
        bytes.extend_from_slice(&buf[..count]);
        if let Some(pos) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&bytes[..pos]).to_lowercase();
            let length: usize = headers
                .lines()
                .find_map(|line| line.strip_prefix("content-length: "))
                .unwrap()
                .parse()
                .unwrap();
            break (pos + 4, length);
        }
    };
    while bytes.len() < end + length {
        let count = stream.read(&mut buf).unwrap();
        assert!(count > 0);
        bytes.extend_from_slice(&buf[..count]);
    }
    (
        String::from_utf8(bytes[..end].to_vec()).unwrap(),
        serde_json::from_slice(&bytes[end..end + length]).unwrap(),
    )
}
fn server(status: u16) -> (String, thread::JoinHandle<(String, serde_json::Value)>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/webhook", listener.local_addr().unwrap());
    listener.set_nonblocking(true).unwrap();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
                {
                    thread::sleep(Duration::from_millis(10))
                }
                Err(e) => panic!("accept: {e}"),
            }
        };
        let request = read_request(&mut stream);
        write!(
            stream,
            "HTTP/1.1 {status} Test\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        request
    });
    (url, handle)
}

#[test]
fn http_request_preserves_content_arguments_and_optional_auth() {
    for auth in [None, Some("Bearer test")] {
        let (url, handle) = server(200);
        let mut args = vec!["--url", &url];
        if let Some(auth) = auth {
            args.extend(["--auth", auth]);
        }
        args.extend([
            "--",
            "-i",
            "name with spaces@example.com",
            "quote'@example.com",
            "",
        ]);
        ok(run(
            &binary(),
            &args,
            "Subject: 测试\r\n\r\nhello\n".as_bytes(),
        ));
        let (headers, body) = handle.join().unwrap();
        assert!(headers.starts_with("POST /webhook HTTP/1.1\r\n"));
        assert!(
            headers
                .to_lowercase()
                .contains("content-type: application/json\r\n")
        );
        assert_eq!(
            headers.to_lowercase().contains("authorization:"),
            auth.is_some()
        );
        if let Some(auth) = auth {
            assert!(
                headers
                    .to_lowercase()
                    .contains(&format!("authorization: {}\r\n", auth.to_lowercase()))
            );
        }
        assert_eq!(body["content"], "Subject: 测试\r\n\r\nhello\n");
        assert_eq!(
            shlex::split(body["args"].as_str().unwrap()).unwrap(),
            [
                "-i",
                "name with spaces@example.com",
                "quote'@example.com",
                ""
            ]
        );
    }
}

#[cfg(unix)]
#[test]
fn embedded_defaults_and_runtime_overrides() {
    let fixture = Fixture::new();
    let path = fixture.binary();
    let (url, handle) = server(200);
    ok(run(
        &path,
        &["--pack-self", "--url", &url, "--auth", "Bearer saved"],
        b"",
    ));
    ok(run(&path, &["-t"], b""));
    let (headers, body) = handle.join().unwrap();
    assert!(
        headers
            .to_lowercase()
            .contains("authorization: bearer saved\r\n")
    );
    assert_eq!(body["content"], "");
    let (url, handle) = server(200);
    ok(run(
        &path,
        &["--url", &url, "--auth", "Bearer override", "-t"],
        b"message",
    ));
    let (headers, _) = handle.join().unwrap();
    assert!(
        headers
            .to_lowercase()
            .contains("authorization: bearer override\r\n")
    );
    // Persist a new URL while keeping saved authentication, then update only auth.
    let (url, handle) = server(200);
    ok(run(&path, &["--pack-self", "--url", &url], b""));
    ok(run(&path, &["-t"], b"message"));
    let (headers, _) = handle.join().unwrap();
    assert!(
        headers
            .to_lowercase()
            .contains("authorization: bearer saved\r\n")
    );
    let (url, handle) = server(200);
    ok(run(&path, &["--pack-self", "--url", &url], b""));
    ok(run(
        &path,
        &["--pack-self", "--auth", "Bearer updated"],
        b"",
    ));
    ok(run(&path, &["-t"], b"message"));
    let (headers, _) = handle.join().unwrap();
    assert!(
        headers
            .to_lowercase()
            .contains("authorization: bearer updated\r\n")
    );
}

#[test]
fn input_header_http_and_transport_errors_fail() {
    for status in [400, 401, 500] {
        let (url, handle) = server(status);
        assert_eq!(
            run(&binary(), &["--url", &url], b"message").status.code(),
            Some(1)
        );
        handle.join().unwrap();
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    for (args, input, error) in [
        (
            vec!["--url", url.as_str()],
            &b"message"[..],
            "error sending request",
        ),
        (
            vec!["--url", url.as_str(), "--auth", "bad\nheader"],
            &b""[..],
            "builder error",
        ),
        (vec!["--url", url.as_str()], &b"\xff"[..], "UTF-8"),
    ] {
        let output = run(&binary(), &args, input);
        assert_eq!(output.status.code(), Some(1));
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(error),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
