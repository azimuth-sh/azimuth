#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
PACKAGE_ROOT="$PWD/tools/azimuth"
OUTPUT_ROOT="$PWD/dist/azimuth"
TARGET="${1:-$(rustc -vV | sed -n 's/^host: //p')}"
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$PACKAGE_ROOT/Cargo.toml" | head -n 1)"

cargo test --quiet --manifest-path "$PACKAGE_ROOT/Cargo.toml"
cargo package --manifest-path "$PACKAGE_ROOT/Cargo.toml" --allow-dirty
cargo build --release --target "$TARGET" --manifest-path "$PACKAGE_ROOT/Cargo.toml"

mkdir -p "$OUTPUT_ROOT"
STAGING="$OUTPUT_ROOT/azimuth-$VERSION-$TARGET"
mkdir -p "$STAGING"
cp "$PACKAGE_ROOT/target/$TARGET/release/azimuth" "$STAGING/azimuth"
cp "$PACKAGE_ROOT/README.md" "$PACKAGE_ROOT/LICENSE" "$STAGING/"
tar -C "$OUTPUT_ROOT" -czf "$STAGING.tar.gz" "$(basename "$STAGING")"
shasum -a 256 "$STAGING.tar.gz" > "$STAGING.tar.gz.sha256"

printf '%s\n' "$STAGING.tar.gz"
