#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."
REPO_ROOT="$PWD"
EXPERIMENT_TMP="$(mktemp -d "${TMPDIR:-/tmp}/azimuth-mechanism-identities.XXXXXX")"
trap 'rm -rf "$EXPERIMENT_TMP"' EXIT
FIXTURE="$EXPERIMENT_TMP/fixture"
BUILD="$EXPERIMENT_TMP/build"
GO_CACHE="${AZIMUTH_GO_CACHE:-/tmp/azimuth-mechanism-go-build}"
GO_MOD_CACHE="${AZIMUTH_GO_MOD_CACHE:-/tmp/azimuth-mechanism-go-mod}"

python3 experiments/mechanism-identities/generate.py init "$FIXTURE" "$REPO_ROOT"
FIXTURE="$(cd "$FIXTURE" && pwd -P)"
OUTPUTS="$FIXTURE/outputs"
mkdir -p "$BUILD/bin" "$BUILD/jvm/annotations" "$BUILD/jvm/emitter"

cargo build --quiet --manifest-path tools/azimuth/Cargo.toml
AZIMUTH="$REPO_ROOT/tools/azimuth/target/debug/azimuth"
(cd tools/extractors/go && \
  GOCACHE="$GO_CACHE" GOMODCACHE="$GO_MOD_CACHE" \
  go build -o "$BUILD/bin/azimuth-emit-go" .)
cargo build --quiet --manifest-path tools/extractors/rust/Cargo.toml
RUST_EMITTER="$REPO_ROOT/tools/extractors/rust/target/debug/azimuth-emit-rust"
(cd tools/extractors/typescript && npm run build --silent)
dotnet build --nologo --verbosity quiet tools/extractors/dotnet/Azimuth.Emit/Azimuth.Emit.csproj
DOTNET_EMITTER="$REPO_ROOT/tools/extractors/dotnet/Azimuth.Emit/bin/Debug/net10.0/"
DOTNET_EMITTER+="azimuth-emit-dotnet.dll"
javac -d "$BUILD/jvm/annotations" packages/jvm/src/main/java/dev/drim/azimuth/Azimuth.java
javac -cp "$BUILD/jvm/annotations" -d "$BUILD/jvm/emitter" \
  tools/extractors/jvm/src/main/java/dev/drim/azimuth/emit/Main.java
JVM_CP="$BUILD/jvm/annotations:$BUILD/jvm/emitter"

for phase in before after; do
  mkdir -p "$OUTPUTS/$phase" "$BUILD/$phase/jvm/classes"
  dotnet build --nologo --verbosity quiet "$FIXTURE/$phase/dotnet/Fixture.csproj"
  javac -cp "$BUILD/jvm/annotations" -d "$BUILD/$phase/jvm/classes" \
    "$FIXTURE/$phase/jvm/src/conformance/Guard.java"

  python3 tools/extractors/cpp/azimuth_emit.py \
    --output "$OUTPUTS/$phase/cpp.json" --root "$FIXTURE" \
    --include packages/cpp "$FIXTURE/$phase/cpp/src"
  python3 tools/extractors/python/azimuth_emit.py \
    --output "$OUTPUTS/$phase/python.json" --root "$FIXTURE/$phase/python" \
    "$FIXTURE/$phase/python"
  GOCACHE="$GO_CACHE" GOMODCACHE="$GO_MOD_CACHE" \
    "$BUILD/bin/azimuth-emit-go" --output "$OUTPUTS/$phase/go.json" \
    --root "$FIXTURE" "$FIXTURE/$phase/go"
  "$RUST_EMITTER" --output "$OUTPUTS/$phase/rust.json" --root "$FIXTURE" \
    "$FIXTURE/$phase/rust/src"
  dotnet "$DOTNET_EMITTER" --output "$OUTPUTS/$phase/dotnet.json" \
    --root "$FIXTURE" \
    "$FIXTURE/$phase/dotnet/bin/Debug/net10.0/MechanismIdentityFixture.dll"
  java -cp "$JVM_CP" dev.drim.azimuth.emit.Main \
    --output "$OUTPUTS/$phase/jvm.json" --root "$FIXTURE" \
    --source-root "$FIXTURE/$phase/jvm/src" \
    --classes "$BUILD/$phase/jvm/classes"
  node tools/extractors/typescript/dist/cli.js \
    --output "$OUTPUTS/$phase/typescript.json" --root "$FIXTURE" \
    "$FIXTURE/$phase/typescript"
done

python3 tools/extractors/cpp/azimuth_emit.py \
  --output "$OUTPUTS/collision-first.json" --root "$FIXTURE" \
  --include packages/cpp "$FIXTURE/collision/first/src"
python3 tools/extractors/cpp/azimuth_emit.py \
  --output "$OUTPUTS/collision-second.json" --root "$FIXTURE" \
  --include packages/cpp "$FIXTURE/collision/second/src"
python3 experiments/mechanism-identities/generate.py verify-extractors "$FIXTURE"
python3 experiments/mechanism-identities/generate.py legacy-profiles "$FIXTURE"

expect_failure() {
  local output="$1"
  shift
  rm -f "$output"
  set +e
  "$@" >"$EXPERIMENT_TMP/invalid.stdout" 2>"$EXPERIMENT_TMP/invalid.stderr"
  local status="$?"
  set -e
  if [[ "$status" -eq 0 || -e "$output" ]]; then
    printf 'expected strict failure without output: %s\n' "$*" >&2
    sed -n '1,120p' "$EXPERIMENT_TMP/invalid.stdout" >&2
    sed -n '1,120p' "$EXPERIMENT_TMP/invalid.stderr" >&2
    exit 1
  fi
}

expect_failure "$OUTPUTS/invalid-cpp.json" \
  python3 tools/extractors/cpp/azimuth_emit.py \
  --output "$OUTPUTS/invalid-cpp.json" --root "$FIXTURE/invalid/cpp" \
  --include packages/cpp "$FIXTURE/invalid/cpp/internal.cpp"
expect_failure "$OUTPUTS/invalid-python.json" \
  python3 tools/extractors/python/azimuth_emit.py \
  --output "$OUTPUTS/invalid-python.json" --root "$FIXTURE/invalid/python" \
  "$FIXTURE/invalid/python/guard.py"
expect_failure "$OUTPUTS/invalid-go.json" env \
  GOCACHE="$GO_CACHE" GOMODCACHE="$GO_MOD_CACHE" \
  "$BUILD/bin/azimuth-emit-go" --output "$OUTPUTS/invalid-go.json" \
  --root "$FIXTURE/invalid/go" "$FIXTURE/invalid/go"
expect_failure "$OUTPUTS/invalid-rust.json" \
  "$RUST_EMITTER" --output "$OUTPUTS/invalid-rust.json" \
  --root "$FIXTURE/invalid/rust" "$FIXTURE/invalid/rust/custom.rs"

dotnet build --nologo --verbosity quiet "$FIXTURE/invalid/dotnet/Invalid.csproj"
expect_failure "$OUTPUTS/invalid-dotnet.json" dotnet "$DOTNET_EMITTER" \
  --output "$OUTPUTS/invalid-dotnet.json" --root "$FIXTURE/invalid/dotnet" \
  "$FIXTURE/invalid/dotnet/bin/Debug/net10.0/Invalid.dll"
mkdir -p "$BUILD/invalid/jvm/classes"
javac -cp "$BUILD/jvm/annotations" -d "$BUILD/invalid/jvm/classes" \
  "$FIXTURE/invalid/jvm/src/invalid/Guard.java"
expect_failure "$OUTPUTS/invalid-jvm.json" java -cp "$JVM_CP" \
  dev.drim.azimuth.emit.Main --output "$OUTPUTS/invalid-jvm.json" \
  --root "$FIXTURE/invalid/jvm" --source-root "$FIXTURE/invalid/jvm/src" \
  --classes "$BUILD/invalid/jvm/classes"
expect_failure "$OUTPUTS/invalid-typescript.json" node \
  tools/extractors/typescript/dist/cli.js \
  --output "$OUTPUTS/invalid-typescript.json" --root "$FIXTURE/invalid/typescript" \
  "$FIXTURE/invalid/typescript/src"

for phase in before after; do
  python3 experiments/mechanism-identities/generate.py model "$FIXTURE" "$phase"
  CORE="$FIXTURE/core/$phase"
  manifests=()
  for family in cpp python go rust dotnet jvm typescript; do
    manifests+=(--manifest "$OUTPUTS/$phase/$family.json")
  done
  "$AZIMUTH" export --model "$CORE/model" --standards "$CORE/standards.md" \
    --workspace "$CORE/workspace.json" "${manifests[@]}" \
    --out "$CORE/initial-export.json"
  python3 experiments/mechanism-identities/generate.py seal "$FIXTURE" "$phase"
  "$AZIMUTH" validate --model "$CORE/model" --standards "$CORE/standards.md" \
    --workspace "$CORE/workspace.json" "${manifests[@]}"
  "$AZIMUTH" export --model "$CORE/model" --standards "$CORE/standards.md" \
    --workspace "$CORE/workspace.json" "${manifests[@]}" --out "$CORE/export.json"
  "$AZIMUTH" run plan --request "$CORE/request.json" --model "$CORE/model" \
    --standards "$CORE/standards.md" --workspace "$CORE/workspace.json" \
    "${manifests[@]}" --config "$CORE/adapters.json" --out "$CORE/launch.json"
done

LEGACY_CORE="$FIXTURE/core/before"
legacy_other_manifests=()
for family in python go rust dotnet jvm typescript; do
  legacy_other_manifests+=(--manifest "$OUTPUTS/before/$family.json")
done
for profile in missing-site file-binding mismatched-companion; do
  expect_failure "$LEGACY_CORE/pre-d48-$profile-output.json" "$AZIMUTH" export \
    --model "$LEGACY_CORE/model" --standards "$LEGACY_CORE/standards.md" \
    --workspace "$LEGACY_CORE/workspace.json" \
    --manifest "$OUTPUTS/pre-d48-$profile.json" "${legacy_other_manifests[@]}" \
    --out "$LEGACY_CORE/pre-d48-$profile-output.json"
done

python3 experiments/mechanism-identities/generate.py verify-core "$FIXTURE"

python3 experiments/mechanism-identities/generate.py collision-model "$FIXTURE"
COLLISION="$FIXTURE/collision-model"
collision_manifests=(
  --manifest "$OUTPUTS/collision-first.json"
  --manifest "$OUTPUTS/collision-second.json"
)
"$AZIMUTH" validate --model "$COLLISION/model" --standards "$COLLISION/standards.md" \
  --workspace "$COLLISION/cross-area.json" "${collision_manifests[@]}"
expect_failure "$COLLISION/must-not-exist.json" "$AZIMUTH" export \
  --model "$COLLISION/model" --standards "$COLLISION/standards.md" \
  --workspace "$COLLISION/same-area.json" "${collision_manifests[@]}" \
  --out "$COLLISION/must-not-exist.json"

printf '{"covers":[]}\n' >"$FIXTURE/legacy.json"
expect_failure "$COLLISION/legacy-output.json" "$AZIMUTH" export \
  --model "$COLLISION/model" --standards "$COLLISION/standards.md" \
  --workspace "$COLLISION/cross-area.json" --manifest "$FIXTURE/legacy.json" \
  --out "$COLLISION/legacy-output.json"

printf 'mechanism identity conformance passed for seven public emitters\n'
