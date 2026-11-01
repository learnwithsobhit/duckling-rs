#!/usr/bin/env bash
# Differential fuzz: randomised POST /parse against Haskell (docker) and Rust.
# Skips (exit 0) when the oracle image is missing unless REQUIRE_ORACLE=1.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

IMAGE="${DUCKLING_ORACLE_IMAGE:-duckling-oracle-base}"
COUNT="${DUCKLING_FUZZ_COUNT:-200}"
SEED="${DUCKLING_FUZZ_SEED:-1}"
RUST_PORT="${DUCKLING_RUST_PORT:-8008}"
HS_PORT="${DUCKLING_HS_PORT:-8009}"

if ! command -v docker >/dev/null 2>&1; then
  if [ "${REQUIRE_ORACLE:-}" = 1 ]; then
    echo "nightly_fuzz: docker is required" >&2
    exit 1
  fi
  echo "nightly_fuzz: docker not installed; skip"
  exit 0
fi

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  if [ "${REQUIRE_ORACLE:-}" = 1 ]; then
    echo "nightly_fuzz: image $IMAGE not found" >&2
    exit 1
  fi
  echo "nightly_fuzz: image $IMAGE not found; skip"
  exit 0
fi

cleanup() {
  kill "${RUST_PID:-}" "${HS_PID:-}" 2>/dev/null || true
  wait "${RUST_PID:-}" "${HS_PID:-}" 2>/dev/null || true
}
trap cleanup EXIT

cargo build --release -p duckling-server
./target/release/duckling-server --port "$RUST_PORT" >/tmp/duckling-rust.log 2>&1 &
RUST_PID=$!

# Best-effort Haskell example server from the oracle image.
# The overlay ExampleMain listens on 8000 inside the container.
docker run --rm -d --name duckling-hs-fuzz -p "${HS_PORT}:8000" "$IMAGE" \
  >/tmp/duckling-hs-cid 2>/tmp/duckling-hs.log || true
if ! docker ps --format '{{.Names}}' | grep -q '^duckling-hs-fuzz$'; then
  echo "nightly_fuzz: could not start Haskell server; skip"
  if [ "${REQUIRE_ORACLE:-}" = 1 ]; then
    exit 1
  fi
  exit 0
fi
HS_PID=""
trap 'docker rm -f duckling-hs-fuzz >/dev/null 2>&1 || true; cleanup' EXIT

for _ in $(seq 1 30); do
  if curl -sf "http://127.0.0.1:${RUST_PORT}/" >/dev/null \
    && curl -sf "http://127.0.0.1:${HS_PORT}/" >/dev/null; then
    break
  fi
  sleep 1
done

python3 "$ROOT/tools/diff_server.py" \
  "http://127.0.0.1:${HS_PORT}" \
  "http://127.0.0.1:${RUST_PORT}" \
  "$COUNT" \
  "$SEED"
