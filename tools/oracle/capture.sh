#!/usr/bin/env bash
# Regenerate fixtures/*.jsonl and crates/duckling/data/classifiers/*.json from the pinned
# upstream Haskell implementation (vendor/duckling), instrumented via tools/oracle/overlay.
#
# Requires docker. Behind an IPv4-only network, first run `python3 tools/oracle/connect_proxy.py 3128`.
set -euo pipefail
cd "$(dirname "$0")/../.."

docker image inspect duckling-oracle-base >/dev/null 2>&1 || docker build -t duckling-oracle-base tools/oracle

# 1. Run the upstream test-suite with the instrumented Duckling.Api; every `analyze` call is dumped.
tools/oracle/run.sh ': > /out/raw.jsonl && mkdir -p /work/src && cp -r /upstream/. /work/src/ 2>/dev/null; cd /work/src && cp -r /oracle/overlay/. . && DUCKLING_DUMP=/out/raw.jsonl stack --resolver lts-20.0 test -j2 --ghc-options=-O0 --test-arguments="--hide-successes" > /out/haskell_test_output.txt 2>&1; echo exit=$? >> /out/haskell_test_output.txt'
tail -5 fixtures/haskell_test_output.txt

# 2. Dedupe + split per language.
python3 tools/oracle/split_fixtures.py
rm -f fixtures/raw.jsonl

# 3. Export the Naive Bayes classifiers (only the "ok" tables are used by the ranker).
tools/oracle/run.sh 'cd /work/src && mkdir -p /work/export && stack --resolver lts-20.0 exec -- ghc -O0 -package duckling -package aeson -package unordered-containers -package text -package bytestring -outputdir /work/export /oracle/export/ExportClassifiers.hs -o /work/export/export-classifiers && /work/export/export-classifiers /data/classifiers'
python3 tools/gen_classifier_index.py
