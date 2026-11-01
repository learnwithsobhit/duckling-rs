#!/usr/bin/env bash
# Compare Haskell DucklingExpensive cases against the Rust Criterion benches.
#
# Usage:
#   tools/oracle/bench_expensive.sh [REPEAT]
#
# REPEAT (default 30) is the number of loop iterations after one warmup.
# Requires the duckling-oracle-base image and the volumes created by tools/oracle/run.sh.
#
# Rust side (from the repo root):
#   cargo bench -p duckling --bench expensive
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
REPEAT="${1:-30}"
RUN="$ROOT/tools/oracle/run.sh"

echo "== Haskell duckling-expensive (analyze + debug-tree I/O), ${REPEAT} iters =="
"$RUN" "cd /work/src && TIMEFORMAT='real %R user %U sys %S' && { time stack --resolver lts-20.0 exec duckling-expensive -- ${REPEAT} >/tmp/hs-expensive.out; } 2>&1 && wc -l /tmp/hs-expensive.out"

echo
echo "== Haskell analyze-only (no parse-tree I/O), ${REPEAT} iters after warmup =="
# Compile against the installed library in the oracle work tree.
"$RUN" "
  cd /work/src
  cp /oracle/BenchExpensiveParse.hs /tmp/BenchExpensiveParse.hs
  stack --resolver lts-20.0 exec -- ghc -O2 -threaded -rtsopts -with-rtsopts=-N \
    -package duckling -package text -package unordered-containers \
    /tmp/BenchExpensiveParse.hs -o /tmp/bench-expensive-parse
  TIMEFORMAT='real %R user %U sys %S'
  { time /tmp/bench-expensive-parse ${REPEAT}; } 2>&1
"

echo
echo "Rust: cargo bench -p duckling --bench expensive"
echo "Compare the Criterion 'both' mean to (analyze-only wall time / ${REPEAT})."
