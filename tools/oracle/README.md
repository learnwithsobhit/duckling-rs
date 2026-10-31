# Haskell oracle

The upstream test-suite is made of Haskell closures, so it cannot be consumed directly. Instead
`overlay/Duckling/Api.hs` replaces upstream `Duckling.Api` and, when `DUCKLING_DUMP=<file>` is set,
appends one JSON line per `analyze` call: locale, reference time (+ time-zone series), latent flag,
target dims, input and the exact `entities` JSON. The Rust parity test (`crates/duckling/tests/parity.rs`)
replays those records and compares the JSON text byte-for-byte.

`./capture.sh` rebuilds everything. Baseline (upstream `59a13ff`): `All 364 tests passed`
(see `fixtures/haskell_test_output.txt`), 33,556 unique records over 49 languages.
