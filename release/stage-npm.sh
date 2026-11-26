#!/bin/sh
# Stages one packed npm package, or keeps the version that npm already publishes from the same
# file. Staging waits at npm for a person's approval, so this script never replaces a staged
# version and never publishes one.
set -eu

package="${1:-}"
version="${2:-}"
file="${3:-}"
if [ -z "$package" ] || [ -z "$version" ] || [ -z "$file" ]; then
  echo "stage-npm: usage: release/stage-npm.sh <package> <version> <packed-file>" >&2
  exit 1
fi
if [ ! -f "$file" ]; then
  echo "stage-npm: $file is not a packed file, so $package $version cannot be staged" >&2
  exit 1
fi
npm="${AZIMUTH_NPM:-npm}"

if command -v sha1sum >/dev/null 2>&1; then
  shasum="$(sha1sum "$file" | cut -d ' ' -f 1)"
else
  shasum="$(shasum -a 1 "$file" | cut -d ' ' -f 1)"
fi
integrity="sha512-$(openssl dgst -sha512 -binary "$file" | openssl base64 -A)"
echo "stage-npm: $package $version is packed in $file with the shasum $shasum"

# A read of a published version needs no credential, so the job's own are left out of it.
without_credentials=""
for name in $(env | sed -n 's/^\([A-Za-z_][A-Za-z0-9_]*\)=.*/\1/p' | grep -iE 'token|auth|password|otp' || true); do
  without_credentials="$without_credentials -u $name"
done

# shellcheck disable=SC2086
if published="$(env $without_credentials "$npm" view "$package@$version" dist.integrity 2>&1)"; then
  if [ "$published" = "$integrity" ]; then
    echo "stage-npm: $package $version is already published from this file, nothing is staged"
    exit 0
  fi
  echo "stage-npm: $package $version is published with the integrity $published, and the packed file has $integrity" >&2
  exit 1
fi
case "$published" in
  *E404*) ;;
  *)
    echo "stage-npm: could not read whether $package $version is published: $published" >&2
    exit 1
    ;;
esac

if staged="$("$npm" stage publish "$file" --access public --tag latest 2>&1)"; then
  echo "$staged"
  echo "stage-npm: staged $package $version with the shasum $shasum, to be approved at npm"
  exit 0
fi
echo "$staged" >&2
case "$staged" in
  *409* | *[Aa]lready*)
    echo "stage-npm: npm refuses $package $version as already staged; compare its shasum at npm with $shasum, then approve or reject it there" >&2
    ;;
  *)
    echo "stage-npm: npm did not stage $package $version" >&2
    ;;
esac
exit 1
