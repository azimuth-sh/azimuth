#!/bin/sh
# Staging needs a newer npm than the runner provides, and its system prefix belongs to root.
set -eu

version=11.21.3
prefix="${1:?usage: release/install-npm.sh <folder>}"
: "${GITHUB_PATH:?GITHUB_PATH must name the job's path file}"
: "${GITHUB_ENV:?GITHUB_ENV must name the job's environment file}"

mkdir -p "$prefix"
prefix="$(cd "$prefix" && pwd -P)"
export npm_config_cache="$prefix/cache"
export npm_config_userconfig="$prefix/user.npmrc"
export npm_config_globalconfig="$prefix/global.npmrc"

cd "$prefix"
npm install --global --prefix "$prefix" --no-audit --no-fund "npm@$version"
actual="$("$prefix/bin/npm" --version)"
if [ "$actual" != "$version" ]; then
  echo "install-npm: installed npm prints $actual, expected $version" >&2
  exit 1
fi

printf '%s\n' "$prefix/bin" >> "$GITHUB_PATH"
printf 'AZIMUTH_NPM_VERSION=%s\n' "$version" >> "$GITHUB_ENV"
