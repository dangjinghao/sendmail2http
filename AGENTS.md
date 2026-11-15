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

Named Docker volumes preserve `/usr/local/cargo` downloads and workspace `target/` build artifacts across container rebuilds. The creation command installs `clang`/`libclang-dev` and makes the target volume writable by `vscode`. Existing host build files are hidden while the target volume is mounted; deleting these volumes discards their caches.

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

Unit tests live beside argument parsing and configuration encoding; `tests/cli.rs` exercises the real executable with disposable copies and local HTTP servers. Run `cargo test --locked`. There is no numeric coverage requirement. Cover configuration persistence, repeated updates, `--truncate`, corrupt payloads, HTTP requests and failure exit codes. After successful self replacement, exit before calling another self-reading API. Tests must bound subprocess and socket waits, bypass proxies for loopback requests, and never contact external webhooks.

`.github/workflows/ci.yml` runs on branch pushes and pull requests, plus manual dispatch. It uses Rust 1.100.0, checks formatting and Clippy, builds and tests optimized `x86_64-unknown-linux-musl` binaries, and verifies static linking. Reproduce with `CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc cargo test --locked --release --target x86_64-unknown-linux-musl` after installing `musl-tools` and the target standard library. CI artifacts are verification builds, not published releases.

CI uses `Swatinem/rust-cache@v2` after selecting the toolchain to cache Cargo downloads and compiled dependencies. Keep `shared-key: x86_64-unknown-linux-musl` for compatible future release jobs. Let the action include toolchain, lockfile and compiler environment in its cache keys; do not key by commit SHA or skip validation on a cache hit. These caches are separate from local development-container volumes.

Prioritize broad x86-64 Linux compatibility over CPU-specific optimization. CI and release jobs use `ubuntu-24.04` to avoid automatic OS migrations; installed packages still receive updates. Musl builds explicitly set `CFLAGS_x86_64_unknown_linux_musl="-march=x86-64 -mtune=generic"` so C dependencies do not inherit a newer CPU baseline from the build environment. The v0.2.1 release executed an unconditional BMI2 `shlx` instruction during HTTP client initialization and crashed on Ivy Bridge CPUs, although `--help` worked. The exact old compiler configuration that introduced it has not been confirmed. `bash scripts/test-cpu-compatibility.sh BINARY` uses `qemu-user` with the conservative `qemu64` model (without AVX, AVX2 or BMI2) to exercise client initialization with a non-network URL and require the expected error. CI checks the musl binary; release preparation checks both plain and UPX files before packaging. This smoke check supplements the full native integration tests; it is not a full OS or TLS compatibility matrix.

## Release Drafts

`.github/workflows/release.yml` handles pushed `v*` tags. The tag is the authoritative release version: the script writes it into `Cargo.toml` and updates the workspace package in `Cargo.lock` before locked builds. Run in a disposable tagged checkout; these generated version changes are not committed. The checked-in version is a development fallback. It builds musl once, preserves the original, and compresses a copy using checksum-pinned UPX 5.3.0. Both exact files run all CLI integration tests through the runtime `SENDMAIL2HTTP_TEST_BINARY` override; without it, tests use Cargo's executable. Mutation tests still use disposable copies.

Keep workflows focused on setup, caching, orchestration and uploading. Use `bash scripts/prepare-release.sh vVERSION` as the release entry point; it calls `scripts/set-release-version.py` to write the package version, then builds, tests, packages and generates a plain commit list with `git log`. Keep Python logic in separate files rather than embedded in shell scripts. These scripts do not require GitHub environment variables. `UPX_BINARY` may point to an already installed binary for local runs. Outputs `dist/` and `release-notes.md` are ignored. Keep implementation details here, not in the README.

Only after both files pass, the workflow packages two executable-preserving tarballs and creates a draft with commit subjects and short hashes since the previous reachable version tag (all history for the first release). Rely on GitHub's asset SHA-256 digests rather than generating or uploading a separate checksum file. Review notes and files and publish manually. An existing release for the tag makes creation fail rather than overwriting reviewed or published content; inspect any partial draft before retrying. Never automatically publish drafts.

## Commit & Pull Request Guidelines

Recent commits use `feat:`, `refactor!:`, and descriptive imperative subjects; prefer that style and mark breaking CLI changes explicitly. Keep commits focused. PR descriptions should explain behavior changes, reference relevant issues, list validation performed, and document platform limitations. Update `readme.md` when CLI behavior changes. Never commit real webhook credentials or configured executable binaries.
