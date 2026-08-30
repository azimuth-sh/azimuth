#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

RELEASE_IMAGES=false
MODE="all"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --release-images)
      RELEASE_IMAGES=true
      ;;
    source|assurance|deployment)
      [[ "$MODE" == "all" ]] || {
        printf '%s\n' 'only one check mode may be selected' >&2
        exit 2
      }
      MODE="$1"
      ;;
    *)
      printf '%s\n' \
        'usage: ./scripts/check.sh [source|assurance|deployment] [--release-images]' >&2
      exit 2
      ;;
  esac
  shift
done

check_source() {
  cargo test --manifest-path tools/azimuth/Cargo.toml
  dotnet test tools/extractors/dotnet/Azimuth.Emit.Tests/Azimuth.Emit.Tests.csproj
  dotnet build packages/dotnet/Azimuth.Annotations/Azimuth.Annotations.csproj

  (
    cd tools/extractors/typescript
    npm install --ignore-scripts
    npm run build
    npm test
  )

  (
    cd packages/typescript
    npm install --ignore-scripts
    npm run build
  )

  python3 -m unittest discover -s tools/extractors/python -p 'test_*.py'
  python3 -m unittest discover -s tools/extractors/cpp -p 'test_*.py'
  (cd tools/extractors/go && go test ./...)
  cargo test --manifest-path tools/extractors/rust/Cargo.toml

  ./experiments/polyglot/check.sh
  ./experiments/assurance-service/check.sh
  ./experiments/run-bundles/check.sh
  ./experiments/adapter-capabilities/check.sh
  ./experiments/challenge-planning/check.sh
  ./experiments/mechanism-identities/check.sh
}

check_assurance() {
  cargo test --manifest-path services/assurance/Cargo.toml --lib
  (
    cd services/assurance/web
    npm ci
    npm run typecheck
    npm run build
  )

  python3 -m unittest -v services.assurance.deployment.test_qualify
  python3 services/assurance/deployment/qualify.py

  if docker info >/dev/null 2>&1; then
    cargo test --manifest-path services/assurance/Cargo.toml --test lifecycle_api
  else
    printf '%s\n' 'Docker is unavailable; assurance lifecycle integration evidence was not run.' >&2
  fi
}

check_deployment() {
  if docker info >/dev/null 2>&1; then
    python3 services/assurance/deployment/qualify.py --lifecycle
    if [[ "$RELEASE_IMAGES" == true ]]; then
      python3 services/assurance/deployment/qualify.py --images
    fi
  else
    printf '%s\n' 'Docker is unavailable; private deployment lifecycle evidence was not run.' >&2
    if [[ "$RELEASE_IMAGES" == true ]]; then
      printf '%s\n' 'Docker is unavailable; release image evidence was not run.' >&2
    fi
  fi
}

if [[ "$MODE" == "all" || "$MODE" == "source" ]]; then
  check_source
fi
if [[ "$MODE" == "all" || "$MODE" == "assurance" ]]; then
  check_assurance
fi
if [[ "$MODE" == "all" || "$MODE" == "deployment" ]]; then
  check_deployment
fi
if [[ "$MODE" == "all" ]]; then
  ./release/check.sh --experiments-executed
fi
