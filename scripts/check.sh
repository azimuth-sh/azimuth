#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

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
./experiments/assurance-extensions/check.sh
./experiments/assurance-service/check.sh

cargo test --manifest-path services/assurance/Cargo.toml --lib
(
  cd services/assurance/web
  npm ci
  npm run typecheck
  npm run build
)

if docker info >/dev/null 2>&1; then
  cargo test --manifest-path services/assurance/Cargo.toml --test lifecycle_api
else
  printf '%s\n' 'Docker is unavailable; assurance lifecycle integration evidence was not run.' >&2
fi
