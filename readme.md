# sendmail2http

A utility that sends stdin email content and sendmail arguments to an HTTP webhook.

## Usage

Send directly with an explicit URL:

```bash
printf 'Subject: test\n\nhello\n' | ./sendmail2http --url http://localhost:8080/webhook --auth "Basic xxx" -- -i a@example.com
```

The request is a JSON POST with `content` (stdin text) and `args` (shell-quoted sendmail arguments). Request, input and configuration errors return a nonzero exit status.

## Embedded configuration

On Linux/macOS, embed configuration into the executable itself:

```bash
./sendmail2http --pack-self --url http://localhost:8080/webhook --auth "Basic xxx"
printf 'hello\n' | ./sendmail2http -i a@example.com
```

`--pack-self` calls `rtad_set_self_data` directly and exits after replacement. The next invocation uses the new configuration. Repeated updates replace the existing payload:

```bash
./sendmail2http --pack-self --url http://localhost:8080/new-webhook
```

Explicit `--url` and `--auth` options override embedded values; omitted options retain their embedded values. A URL is required unless already embedded. **Put configuration options before sendmail arguments**, or use `--` to mark where sendmail arguments start.

Updates must run serially, with permission to replace the executable in its directory.

Clear the executable's embedded configuration without a URL or destination path:

```bash
./sendmail2http --truncate
```

`--truncate` calls `rtad_truncate_self_data(NULL)` and exits after replacement.
It is mutually exclusive with `--pack-self`. Clearing an executable without
embedded data also succeeds. Self replacement is supported on Linux/macOS;
Windows reports an unsupported operation.
