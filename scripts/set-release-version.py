#!/usr/bin/env python3
"""Write the release tag's version into Cargo.toml in the current directory."""

import argparse
import re
from pathlib import Path


TAG_PATTERN = r"v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?"
# Match only the version inside [package], preserving surrounding formatting.
PACKAGE_VERSION_PATTERN = (
    r'(\[package\]\s*\n(?:(?!\[)[^\n]*\n)*?version\s*=\s*)"[^"]*"'
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tag", help="Release tag, for example v0.2.0")
    args = parser.parse_args()
    if not re.fullmatch(TAG_PATTERN, args.tag):
        parser.error("Expected a version tag such as v0.2.0")

    version = args.tag[1:]
    manifest = Path("Cargo.toml")
    text, count = re.subn(
        PACKAGE_VERSION_PATTERN,
        lambda match: f'{match[1]}"{version}"',
        manifest.read_text(encoding="utf-8"),
        count=1,
    )
    if count != 1:
        raise SystemExit("Cannot locate package version in Cargo.toml")
    manifest.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
