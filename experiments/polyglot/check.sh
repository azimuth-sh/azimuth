#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
REPO_ROOT="$PWD"
BUILD_ROOT="$REPO_ROOT/build/polyglot"
OUTPUT_ROOT="$REPO_ROOT/.azimuth/polyglot"
AZIMUTH_GO_CACHE="${AZIMUTH_GO_CACHE:-/tmp/azimuth-go-build}"
AZIMUTH_GO_MOD_CACHE="${AZIMUTH_GO_MOD_CACHE:-/tmp/azimuth-go-mod}"
AZIMUTH_GRADLE_CACHE="${AZIMUTH_GRADLE_CACHE:-/tmp/azimuth-gradle-home}"

mkdir -p "$BUILD_ROOT/extractors" "$BUILD_ROOT/jvm/azimuth-annotations" \
  "$BUILD_ROOT/jvm/extractor" "$BUILD_ROOT/jvm/extractor-test" \
  "$BUILD_ROOT/jvm/java-service" "$BUILD_ROOT/cpp" "$OUTPUT_ROOT"

(cd tools/extractors/typescript && npm run build --silent && npm test --silent)
(cd tools/extractors/python && python3 -m unittest -v test_azimuth_emit.py)
(cd tools/extractors/go && GOCACHE="$AZIMUTH_GO_CACHE" GOMODCACHE="$AZIMUTH_GO_MOD_CACHE" go test ./...)
(cd tools/extractors/rust && cargo fmt -- --check && cargo test)
(cd tools/extractors/cpp && python3 -m unittest -v test_azimuth_emit.py)

javac -d "$BUILD_ROOT/jvm/azimuth-annotations" \
  packages/jvm/src/main/java/dev/drim/azimuth/Azimuth.java
javac -cp "$BUILD_ROOT/jvm/azimuth-annotations" -d "$BUILD_ROOT/jvm/extractor" \
  tools/extractors/jvm/src/main/java/dev/drim/azimuth/emit/Main.java
javac -cp "$BUILD_ROOT/jvm/azimuth-annotations:$BUILD_ROOT/jvm/extractor" \
  -d "$BUILD_ROOT/jvm/extractor-test" \
  tools/extractors/jvm/src/test/java/dev/drim/azimuth/emit/MainTest.java
java -cp "$BUILD_ROOT/jvm/azimuth-annotations:$BUILD_ROOT/jvm/extractor:$BUILD_ROOT/jvm/extractor-test" \
  dev.drim.azimuth.emit.MainTest

(cd experiments/polyglot/services/go && \
  GOCACHE="$AZIMUTH_GO_CACHE" GOMODCACHE="$AZIMUTH_GO_MOD_CACHE" go test ./...)

javac -cp "$BUILD_ROOT/jvm/azimuth-annotations" -d "$BUILD_ROOT/jvm/java-service" \
  experiments/polyglot/services/java/src/polyglot/IdentityService.java \
  experiments/polyglot/services/java/test/polyglot/IdentityServiceTest.java
java -cp "$BUILD_ROOT/jvm/azimuth-annotations:$BUILD_ROOT/jvm/java-service" \
  polyglot.IdentityServiceTest

(cd experiments/polyglot/services/kotlin && \
  GRADLE_USER_HOME="$AZIMUTH_GRADLE_CACHE" gradle test --no-daemon)

PYTHONPATH="$REPO_ROOT/packages/python:$REPO_ROOT/experiments/polyglot/services/python" \
  python3 -m unittest -v experiments/polyglot/services/python/test_service.py
(cd experiments/polyglot/services/javascript && npm test --silent)
(cd experiments/polyglot/services/rust && cargo fmt -- --check && cargo test)

clang++ -std=c++20 -Ipackages/cpp \
  experiments/polyglot/services/cpp/identity.cpp \
  experiments/polyglot/services/cpp/service.cpp \
  -o "$BUILD_ROOT/cpp/identity-service"
clang++ -std=c++20 -Ipackages/cpp \
  experiments/polyglot/services/cpp/identity.cpp \
  experiments/polyglot/services/cpp/service_test.cpp \
  -o "$BUILD_ROOT/cpp/identity-test"
"$BUILD_ROOT/cpp/identity-test"

(cd tools/extractors/go && \
  GOCACHE="$AZIMUTH_GO_CACHE" GOMODCACHE="$AZIMUTH_GO_MOD_CACHE" \
  go build -o "$BUILD_ROOT/extractors/azimuth-emit-go" .)
"$BUILD_ROOT/extractors/azimuth-emit-go" --output "$OUTPUT_ROOT/go.json" \
  --root "$REPO_ROOT" experiments/polyglot/services/go

python3 tools/extractors/python/azimuth_emit.py --output "$OUTPUT_ROOT/python.json" \
  --root "$REPO_ROOT" experiments/polyglot/services/python
node tools/extractors/typescript/dist/cli.js --output "$OUTPUT_ROOT/javascript.json" \
  --root "$REPO_ROOT" experiments/polyglot/services/javascript
cargo run --quiet --manifest-path tools/extractors/rust/Cargo.toml -- \
  --output "$OUTPUT_ROOT/rust.json" --root "$REPO_ROOT" \
  experiments/polyglot/services/rust/src
python3 tools/extractors/cpp/azimuth_emit.py --output "$OUTPUT_ROOT/cpp.json" \
  --root "$REPO_ROOT" --include packages/cpp experiments/polyglot/services/cpp

java -cp "$BUILD_ROOT/jvm/azimuth-annotations:$BUILD_ROOT/jvm/extractor" \
  dev.drim.azimuth.emit.Main --output "$OUTPUT_ROOT/java.json" --root "$REPO_ROOT" \
  --source-root experiments/polyglot/services/java/src \
  --source-root experiments/polyglot/services/java/test \
  --classes "$BUILD_ROOT/jvm/java-service"
java -cp "$BUILD_ROOT/jvm/azimuth-annotations:$BUILD_ROOT/jvm/extractor" \
  dev.drim.azimuth.emit.Main --output "$OUTPUT_ROOT/kotlin.json" --root "$REPO_ROOT" \
  --source-root experiments/polyglot/services/kotlin/src/main/kotlin \
  --source-root experiments/polyglot/services/kotlin/src/test/kotlin \
  --classes experiments/polyglot/services/kotlin/build/classes/kotlin/main \
  --classes experiments/polyglot/services/kotlin/build/classes/kotlin/test

cargo run --quiet --manifest-path tools/azimuth/Cargo.toml -- check \
  --model experiments/polyglot/model \
  --standards experiments/polyglot/standards/verification.md \
  --manifest "$OUTPUT_ROOT/go.json" \
  --manifest "$OUTPUT_ROOT/java.json" \
  --manifest "$OUTPUT_ROOT/kotlin.json" \
  --manifest "$OUTPUT_ROOT/python.json" \
  --manifest "$OUTPUT_ROOT/javascript.json" \
  --manifest "$OUTPUT_ROOT/rust.json" \
  --manifest "$OUTPUT_ROOT/cpp.json"
