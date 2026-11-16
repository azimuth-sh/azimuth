#!/bin/sh
# Passes when the tag is `v` plus the version in core/Cargo.toml, so a release is never built
# from code that names another version. Run from the root of the repository.

tag="${1:-}"
if [ -z "$tag" ]; then
  echo "check-tag: no tag given; usage: release/check-tag.sh v<version>" >&2
  exit 1
fi

# The version of the package is the first `version` key in its [package] table.
version="$(sed -n -e '/^\[package\]/,/^\[/s/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' core/Cargo.toml | sed -n 1p)"
if [ -z "$version" ]; then
  echo "check-tag: core/Cargo.toml names no version in its [package] table" >&2
  exit 1
fi

if [ "$tag" != "v$version" ]; then
  echo "check-tag: the tag $tag differs from v$version, the version in core/Cargo.toml" >&2
  exit 1
fi
echo "check-tag: the tag $tag matches the version in core/Cargo.toml"
