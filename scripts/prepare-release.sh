#!/usr/bin/env bash
# Build, test and package ordinary and UPX musl binaries; never publish a release.
set -euo pipefail
cd "$(dirname "$0")/.."
tag=${1:?Usage: bash scripts/prepare-release.sh vVERSION}
target=x86_64-unknown-linux-musl
upx=${UPX_BINARY:-upx}
"$upx" --version
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=${CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER:-musl-gcc}
export CFLAGS_x86_64_unknown_linux_musl=${CFLAGS_x86_64_unknown_linux_musl:--march=x86-64 -mtune=generic}

# Release tags are authoritative; use a disposable checkout for this script.
[[ "$(git rev-parse HEAD)" == "$(git rev-parse --verify "refs/tags/$tag^{commit}")" ]] || { echo "Check out the release tag first" >&2; exit 1; }
python3 scripts/set-release-version.py "$tag"
cargo update --workspace
cargo build --locked --release --target "$target"
cargo test --locked --release --target "$target"
file "target/$target/release/sendmail2http" | grep -E 'statically linked|static-pie linked'

work_dir=$(mktemp -d)
trap 'rm -rf "$work_dir"' EXIT
mkdir -p "$work_dir/plain" "$work_dir/upx"
cp "target/$target/release/sendmail2http" "$work_dir/plain/sendmail2http"
cp "$work_dir/plain/sendmail2http" "$work_dir/upx/sendmail2http"
"$upx" --best "$work_dir/upx/sendmail2http"
"$upx" --test "$work_dir/upx/sendmail2http"
for variant in plain upx; do
    python3 scripts/test-cpu-compatibility.py "$work_dir/$variant/sendmail2http"
    SENDMAIL2HTTP_TEST_BINARY="$work_dir/$variant/sendmail2http" \
        cargo test --locked --release --target "$target" --test cli
done

# Package only after both exact executables pass; list only this version's files.
mkdir -p dist
plain="sendmail2http-${tag}-x86_64-linux-musl.tar.gz"
compressed="sendmail2http-${tag}-x86_64-linux-musl-upx.tar.gz"
tar -C "$work_dir/plain" -czf "dist/$plain" sendmail2http
tar -C "$work_dir/upx" -czf "dist/$compressed" sendmail2http

# First release includes all history; subsequent releases start at the nearest ancestor tag.
previous=$(git describe --tags --match 'v*' --abbrev=0 "$tag^" 2>/dev/null || true)
git log --reverse --format='- %s (%h)' "${previous:+$previous..}$tag" > release-notes.md
