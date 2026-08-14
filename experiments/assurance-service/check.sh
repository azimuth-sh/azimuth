#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
cargo test --quiet --manifest-path experiments/assurance-service/Cargo.toml
