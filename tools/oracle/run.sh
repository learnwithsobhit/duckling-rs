#!/usr/bin/env bash
# Usage: tools/oracle/run.sh '<shell command run inside the oracle container>'
# /src is a writable copy of vendor/duckling (with overlay applied) living in the `duckling-work` volume.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
docker run --rm \
  -e https_proxy=http://host.docker.internal:3128 -e http_proxy=http://host.docker.internal:3128 \
  -v duckling-stack:/stack -v duckling-work:/work \
  -v "$ROOT/vendor/duckling":/upstream:ro -v "$ROOT/tools/oracle":/oracle:ro -v "$ROOT/fixtures":/out -v "$ROOT/crates/duckling/data":/data \
  -w /work duckling-oracle-base bash -c "$1"
