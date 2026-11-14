# sendmail2http

Forward stdin email content and sendmail arguments to an HTTP webhook as JSON (`content` and shell-quoted `args`).

## Usage

```bash
printf 'hello\n' | ./sendmail2http --url http://localhost:8080/webhook --auth "Basic xxx" -- -i recipient@example.com
```

## Embedded configuration

Configure the executable itself on Linux/macOS, then use it without repeating webhook options:

```bash
./sendmail2http --pack-self --url http://localhost:8080/webhook --auth "Basic xxx"
printf 'hello\n' | ./sendmail2http -i recipient@example.com
./sendmail2http --truncate
```

Explicit `--url` and `--auth` override stored values. Put webhook options before sendmail arguments, or separate them with `--`. Run `--help` for options.
