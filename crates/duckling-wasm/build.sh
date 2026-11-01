#!/usr/bin/env bash
# Builds the JS package into crates/duckling-wasm/pkg and runs the smoke test.
# Requires: rustup target wasm32-unknown-unknown, wasm-bindgen-cli 0.2.130 (matches Cargo.lock), node.
# Usage: build.sh [nodejs|web|bundler] [cargo features, e.g. "all-langs"]
set -euo pipefail
cd "$(dirname "$0")"
target="${1:-nodejs}"
features="${2:-}"
args=(--release -p duckling-wasm --target wasm32-unknown-unknown)
[ -n "$features" ] && args+=(--features "$features")
cargo build "${args[@]}"
tdir="$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys;print(json.load(sys.stdin)["target_directory"])')"
mkdir -p pkg
wasm-bindgen --target "$target" --out-dir pkg "$tdir/wasm32-unknown-unknown/release/duckling_wasm.wasm"
[ "$target" = nodejs ] && node test/smoke.cjs
