#!/bin/sh
# Writes VERSION and SHA256SUMS beside the two binaries of a release, so the folder holds every
# file the install script reads.

version="${1:-}"
folder="${2:-}"
if [ -z "$version" ] || [ -z "$folder" ]; then
  echo "assemble: usage: release/assemble.sh <version> <folder>" >&2
  exit 1
fi

files=""
for target in aarch64-apple-darwin x86_64-unknown-linux-musl; do
  file="azimuth-$version-$target"
  if [ ! -f "$folder/$file" ]; then
    echo "assemble: $folder holds no $file" >&2
    exit 1
  fi
  files="$files $file"
done

if command -v sha256sum >/dev/null 2>&1; then
  sum="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
  sum="shasum -a 256"
else
  echo "assemble: found no tool for SHA-256, neither sha256sum nor shasum" >&2
  exit 1
fi

# Both tools write `<hex>  <file name>`, the form the install script reads. The names are
# relative because the sums are run inside the folder.
(cd "$folder" && $sum $files > SHA256SUMS) || { rm -f "$folder/SHA256SUMS"; echo "assemble: could not write $folder/SHA256SUMS" >&2; exit 1; }
echo "$version" > "$folder/VERSION" || { echo "assemble: could not write $folder/VERSION" >&2; exit 1; }
echo "assemble: wrote VERSION and SHA256SUMS for azimuth $version in $folder"
