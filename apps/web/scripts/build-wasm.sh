#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
wasm-pack build crates/hewn-wasm --target web --out-dir ../../apps/web/src/generated --release
