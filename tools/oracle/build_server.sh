#!/usr/bin/env bash
# Build upstream's example HTTP server (exe/ExampleMain.hs) inside the oracle container and keep
# the binary in the `duckling-work` volume as /work/duckling-example-exe.
#
# Behind an IPv4-only network, first run `python3 tools/oracle/connect_proxy.py 3128`.
set -euo pipefail
cd "$(dirname "$0")/../.."
tools/oracle/run.sh 'mkdir -p /work/src && cp -r /upstream/. /work/src/ 2>/dev/null; cd /work/src && cp -r /oracle/overlay/. . && stack --resolver lts-20.0 build duckling:exe:duckling-example-exe -j2 --ghc-options=-O0 && cp "$(stack --resolver lts-20.0 path --local-install-root)/bin/duckling-example-exe" /work/duckling-example-exe && ls -la /work/duckling-example-exe'
