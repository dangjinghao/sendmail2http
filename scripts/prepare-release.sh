#!/usr/bin/env bash
# Build, test and package ordinary and UPX musl binaries; never publish a release.
set -euo pipefail
cd "$(dirname "$0")/.."
tag=${1:?Usage: bash scripts/prepare-release.sh vVERSION}
target=x86_64-unknown-linux-musl
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=${CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER:-musl-gcc}

# Release tags are authoritative; use a disposable checkout for this script.
[[ "$(git rev-parse HEAD)" == "$(git rev-parse --verify "refs/tags/$tag^{commit}")" ]] || { echo "Check out the release tag first" >&2; exit 1; }
python3 scripts/set-release-version.py "$tag"
cargo update --workspace
cargo build --locked --release --target "$target"
cargo test --locked --release --target "$target"
file "target/$target/release/sendmail2http" | grep -E 'statically linked|static-pie linked'

work_dir=$(mktemp -d)
trap 'rm -rf "$work_dir"' EXIT
upx=${UPX_BINARY:-}
if [[ -z "$upx" ]]; then
    curl --fail --location --retry 3 https://github.com/upx/upx/releases/download/v5.3.0/upx-5.3.0-amd64_linux.tar.xz -o "$work_dir/upx.tar.xz"
    echo '79cea8a50b6fa102d10c7b79a419cd77dead3cc2d78661de2dc5e5815dc505c1  '"$work_dir/upx.tar.xz" | sha256sum --check
    tar -xJf "$work_dir/upx.tar.xz" -C "$work_dir"
    upx="$work_dir/upx-5.3.0-amd64_linux/upx"
fi
mkdir -p "$work_dir/plain" "$work_dir/upx"
cp "target/$target/release/sendmail2http" "$work_dir/plain/sendmail2http"
cp "$work_dir/plain/sendmail2http" "$work_dir/upx/sendmail2http"
"$upx" --best "$work_dir/upx/sendmail2http"
"$upx" --test "$work_dir/upx/sendmail2http"
for variant in plain upx; do
    SENDMAIL2HTTP_TEST_BINARY="$work_dir/$variant/sendmail2http" \
        cargo test --locked --release --target "$target" --test cli
done

# Package only after both exact executables pass; list only this version's files.
mkdir -p dist
plain="sendmail2http-${tag}-x86_64-linux-musl.tar.gz"
compressed="sendmail2http-${tag}-x86_64-linux-musl-upx.tar.gz"
tar -C "$work_dir/plain" -czf "dist/$plain" sendmail2http
tar -C "$work_dir/upx" -czf "dist/$compressed" sendmail2http
(cd dist && sha256sum "$plain" "$compressed" > SHA256SUMS)

# First release includes all history; subsequent releases start at the nearest ancestor tag.
previous=$(git describe --tags --match 'v*' --abbrev=0 "$tag^" 2>/dev/null || true)
git log --reverse --format='- %s (%h)' "${previous:+$previous..}$tag" > release-notes.md
