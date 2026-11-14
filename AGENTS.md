# Repository Guidelines

## Documentation & Collaboration

Treat `AGENTS.md` as agent working memory. Do not mention or link it in `readme.md`.

Keep `readme.md` concise: purpose, basic usage, and essential user-facing limitations. Store contributor instructions, implementation details, project decisions, and persistent collaboration preferences in `AGENTS.md`. Update these notes when decisions change; do not restore removed features or tests without agreement. Prefer simple, focused changes and reuse existing tools.

## Project Structure & Module Organization

`sendmail2http` is a Rust 2024 CLI that forwards stdin email content and sendmail arguments to an HTTP webhook. `src/main.rs` coordinates execution; `args.rs`, `sendmail.rs`, and `http.rs` handle CLI parsing, input, and requests. `pack.rs` manages embedded configuration. Keep unsafe FFI calls in `rtad_wrapper.rs`; `rtad.rs` includes generated bindings.

`build.rs` clones the pinned `rtad` dependency into ignored `rtad/`, compiles its C source, and generates bindings under `target/`. Do not edit generated bindings. Development environment configuration lives in `.devcontainer/`; user documentation is `readme.md`.

## Build, Test, and Development Commands

Use the development container, which supplies Rust and installs `pre-commit`, formatting/lint components, and the commit hook. Native builds also require a C compiler and `libclang`. Initial builds may download dependencies and compile vendored `OpenSSL`.

The container uses `ghcr.io/devcontainers-extra/features/pre-commit:2`; its `postCreateCommand` installs `rustfmt`, `clippy`, and the commit hook. Rebuild the container after changing its configuration. Outside the container, install the tools and run `pre-commit install` once per clone.

- `cargo build --locked`: compile the development binary.
- `cargo build --release --locked`: produce the optimized executable in `target/release/`.
- `cargo test --locked`: run Rust tests and documentation tests.
- `cargo fmt --all`: format Rust sources.
- `cargo clippy --locked --all-targets -- -D warnings`: reject compilation and lint warnings.
- `pre-commit run --all-files`: run all configured commit checks.
- `printf 'hello\n' | cargo run -- --url http://localhost:8080/webhook -- -i recipient@example.com`: exercise local forwarding.

## Coding Style & Naming Conventions

Follow `rustfmt` defaults, including four-space indentation. Use `snake_case` for modules/functions and `PascalCase` for types. Prefer small, focused changes and existing dependencies. Propagate errors with useful messages rather than panicking on expected failures. Commit checks also validate whitespace, final newlines, conflict markers, and TOML/YAML syntax.

The format hook only checks; use `cargo fmt --all` to fix formatting. Whitespace and final-newline hooks may modify files: review and stage fixes before retrying. Run the same checks in CI; keep tests and release builds out of the commit hook.

## Embedded Configuration Behavior

`--pack-self` directly calls `rtad_set_self_data`; `--truncate` calls `rtad_truncate_self_data(NULL)`. Both replace the current executable and exit immediately, are mutually exclusive, and support Linux/macOS. Windows reports an unsupported operation. No destination-path arguments or application-managed temporary-file layer are needed.

Truncation requires neither a URL nor valid embedded CLI configuration; clearing an executable without a payload also succeeds. Explicit URL/authentication options override stored values, while omitted options retain them. Updates require replacement permission in the executable's directory and must run serially. Request, input, and configuration failures return nonzero exit status.

## Testing Guidelines

There is currently no dedicated test suite or numeric coverage requirement. Add meaningful Rust unit tests beside modules or integration tests under `tests/` when changing behavior; use descriptive `snake_case` names. Test executable mutation only on disposable copies. Verify configuration persistence, repeated updates, `--truncate`, and HTTP failure exit codes. After successful self replacement, exit before calling another self-reading API.

## Commit & Pull Request Guidelines

Recent commits use `feat:`, `refactor!:`, and descriptive imperative subjects; prefer that style and mark breaking CLI changes explicitly. Keep commits focused. PR descriptions should explain behavior changes, reference relevant issues, list validation performed, and document platform limitations. Update `readme.md` when CLI behavior changes. Never commit real webhook credentials or configured executable binaries.
