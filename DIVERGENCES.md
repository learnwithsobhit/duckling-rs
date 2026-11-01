# Intentional divergences from upstream Duckling

The Rust port is byte-exact with upstream Haskell Duckling on every recorded fixture. This file lists every place
where behaviour intentionally differs. Parser bug fixes (Phase 7) are added here, each with a regression test.

## HTTP server (`duckling-server`) — transport-level only

Response bodies are byte-identical to the Haskell example server (verified on ~12k randomised requests with
`tools/diff_server.py`). Differences are limited to the HTTP envelope:

| # | Area | Upstream (Snap) | Rust (axum/hyper) | Why |
|---|------|-----------------|-------------------|-----|
| S1 | Reason phrase for 422 | `Bad Input` | `Unprocessable Entity` | hyper emits canonical phrases |
| S2 | `Server` header, chunked encoding | Snap sets `Server: Snap/…` | absent | not part of the API contract |
| S3 | Invalid `reftime` when a Time entity is resolved | connection dropped (lazy exception) | HTTP 500 with a message | clients get an error rather than a reset |
| S4 | `/targets` key order | Haskell `HashMap` order | sorted by language code | JSON objects are unordered |

## Parser bug fixes

Each row is an intentional change from Haskell. The new behaviour is locked by
`crates/duckling/tests/phase7.rs`.

| # | Issue | Before (Haskell) | After (Rust) | Notes |
|---|-------|------------------|--------------|-------|
| P1 | [#439](https://github.com/facebook/duckling/issues/439) Georgian numerals | `ათი` matches inside `პათიზ` because GHC `isLower`/`isUpper` do not treat Mkhedruli as letters | Intra-word Georgian spans are rejected (`ka_is_range_valid`) | [#541](https://github.com/facebook/duckling/issues/541) is an upstream GHC ≥8.8 test flake, not a Rust bug |
| P2 | [#481](https://github.com/facebook/duckling/issues/481) large DE numbers | Compound words such as `fünfzigtausend` failed in some deployments | `rule_all_numeral_words` + `parse_numeral` accept the official compound spelling | Covered by Phase 7 tests; no fixture change |
| P3 | [#456](https://github.com/facebook/duckling/issues/456) "this week" | `this week` is Monday of the current week at Week grain | Unchanged | Official EN Time corpus requires Week grain Monday; changing it would break `monday of this week` / `beginning of this week` |
| P4 | [#440](https://github.com/facebook/duckling/issues/440) "now" | `three days from now at 6pm` does not intersect the time of day (`now` is NoGrain) | New EN rule `<duration> from now at|after <time-of-day>` shifts from today, then intersects | Standalone `a day from now` stays Haskell-identical |
| P5 | [#275](https://github.com/facebook/duckling/issues/275) end of month | `before the end of the month` uses day 21 (start of the "end of month" interval) | Dedicated rules bound on the last day of the month | `end of the month` as a standalone interval is unchanged |
| P6 | [#268](https://github.com/facebook/duckling/issues/268) `<number> $ <number>` | `two $ 500` → $7 (`500` treated as cents) | Bare numerals after an amount are cents only when `0 ≤ n < 100` | `$20 43` → $20.43 still holds |
| P7 | [#250](https://github.com/facebook/duckling/issues/250) last four months of 2016 | `last four months` resolved against the current year | `last <duration> of <time>` intersects the given year | New EN Time rule |
| P8 | [#174](https://github.com/facebook/duckling/issues/174) weekday/hour ranges | Overlapping parses of `Mon-Fri 7am-7pm CST, Sat-Sun 8am-6pm CST` | Unchanged | Ranking/overlap; no safe local fix without regressing other interval corpora |
| P9 | [#13](https://github.com/facebook/duckling/issues/13) second Sunday vs Thursday | `the second sunday of October last year` resolves inconsistently | Unchanged | Same ranking/intersect family as P8 |
| P10 | [#131](https://github.com/facebook/duckling/issues/131) Arabic intra-word matches | `أحد` (Sunday) matched inside `وأحدث` | Arabic letters are `Alpha` plus proclitic rules; intra-word Sunday is rejected | Already implied by the AR range-valid port; locked by test |
| P11 | location timezone | `now in Delhi` / `today in Delhi` empty | Opt-in `location_tz=true` city allowlist (`Dim::Timezone`); default parse unchanged | Not upstream |

## Port notes (not parser divergences)

| # | Item | Notes |
|---|------|-------|
| N1 | EN empty-targets fixture | `EN_XX` record for `the day after tomorrow 5pm` has empty `targets` and is skipped by parity (`in_scope` requires non-empty targets). Do not edit the gold file. |
| N2 | CA_XX classifier tables | Trainer verify is exact for 50/51 locales. `CA_XX` differs because CA Time regexes were tightened to avoid Haskell empty-match loops; not a trainer bug. |

## Fixture allowlist

CI fails if any `fixtures/*.jsonl` file changes unless this document names that file
(basename or path). Phase 7 bug fixes list the regenerated fixtures here.
