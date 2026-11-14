use std::env;
use std::path::PathBuf;

const RTAD_REV: &str = "d2b734b05b18b0c2c3c10ecfbad95dec5c7ff539";

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let rtad_root = manifest_dir.join("rtad");

    // Auto-clone rtad repository if it doesn't exist
    if !rtad_root.exists() {
        println!("cargo:warning=Cloning rtad repository...");
        let repo = git2::Repository::clone("https://github.com/dangjinghao/rtad.git", &rtad_root)
            .expect("Failed to clone rtad");
        let commit = repo
            .revparse_single(RTAD_REV)
            .expect("Missing pinned rtad commit");
        repo.checkout_tree(&commit, None)
            .expect("Failed to check out rtad");
        repo.set_head_detached(commit.id())
            .expect("Failed to pin rtad HEAD");
    }

    let repo = git2::Repository::open(&rtad_root).expect("Failed to open rtad repository");
    let head = repo
        .head()
        .expect("Failed to read rtad HEAD")
        .peel_to_commit()
        .unwrap();
    assert_eq!(
        head.id().to_string(),
        RTAD_REV,
        "rtad checkout is outdated. Run: git -C rtad fetch origin && git -C rtad checkout {RTAD_REV}"
    );

    let rtad_src = rtad_root.join("src/rtad.c");
    let rtad_include = rtad_root.join("include");
    let rtad_src_dir = rtad_root.join("src");
    let header_path = rtad_include.join("rtad.h");

    // Compile rtad C library
    cc::Build::new()
        .file(&rtad_src)
        .include(&rtad_include)
        .include(&rtad_src_dir)
        .warnings(false)
        .compile("rtad");

    println!("cargo:rustc-link-lib=static=rtad");
    println!("cargo:rerun-if-changed={}", rtad_src.display());
    println!("cargo:rerun-if-changed={}", header_path.display());
    println!(
        "cargo:rerun-if-changed={}",
        rtad_src_dir.join("rtad_def.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        repo.path().join("HEAD").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        repo.path().join("refs").display()
    );

    // Generate Rust bindings using bindgen
    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .clang_arg(format!("-I{}", rtad_include.display()))
        .allowlist_function("rtad_.*")
        .allowlist_type("rtad_error")
        .generate_comments(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Failed to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Failed to write bindings");
}
