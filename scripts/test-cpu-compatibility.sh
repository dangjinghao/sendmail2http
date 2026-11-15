#!/usr/bin/env bash
# Exercise HTTP client initialization without AVX, AVX2 or BMI2.
set -euo pipefail
ulimit -c 0
binary=${1:?Usage: bash scripts/test-cpu-compatibility.sh BINARY}
status=0
# An unsupported URL scheme initializes the client but never sends a request.
output=$(timeout 30s qemu-x86_64 -cpu qemu64 "$binary" \
    --url file:///cpu-compatibility-test </dev/null 2>&1) || status=$?
if [[ "$status" != 1 || "$output" != 'Error: builder error for url (file:///cpu-compatibility-test)' ]]; then
    printf 'CPU compatibility check failed (exit %s):\n%s\n' "$status" "$output" >&2
    exit 1
fi
printf 'CPU compatibility check passed: %s\n' "$binary"
