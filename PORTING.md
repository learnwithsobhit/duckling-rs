# Porting a language (Phase 4 guide)

Goal: a language's Rust rules must produce **byte-identical** JSON to upstream Haskell Duckling on every fixture
in `fixtures/<LANG>.jsonl` (recorded from the Haskell oracle). Upstream source is vendored in `vendor/duckling`
(`Duckling/<Dim>/<LANG>/Rules.hs`, `Duckling/Rules/<LANG>.hs`). Port faithfully — **do not fix upstream bugs**
(bug fixes are Phase 7, after parity). Keep rule *order* and rule *names* identical to the Haskell lists: the
classifiers are keyed by rule name, and ordering affects ranking/tie-breaks.

## Where things live

```
crates/duckling/src/langs/<xx>/mod.rs   <- your module (add files next to it: numeral.rs, time.rs, ...)
    PORTED: &[Dim]                       dims fully ported + fixture-verified (all locales)
    lang_rules(dim)                      == Duckling.Rules.<LANG>.langRules
    locale_rules(region, dim)            == localeRules
    default_rules(dim)                   == defaultRules (used when the request has no region)
crates/duckling/src/langs/en.rs          reference wiring (EN is complete)
crates/duckling/src/dims/**              shared types/helpers (Numeral/Time/... Types + Helpers) — EN rules for examples
crates/duckling/src/dims/numeral/en.rs, dims/ordinal.rs, dims/time/en.rs   examples of ported rule sets
crates/duckling/src/types.rs             Rule, rule(), regex(), dimension(), pred(), groups(), Token
```

Rules are `rule("name", vec![pattern items], |tokens| -> Option<Token>)`. Look at how EN's Haskell was ported
(`dims/numeral/en.rs`, `dims/ordinal.rs`, `dims/duration.rs`, `dims/time/en.rs`) and copy the idioms: `Lazy<Vec<Rule>>`
statics returning `&'static [Rule]`, helpers from the dim's Types/Helpers port (`dims::numeral::*`,
`dims::time::helpers`, `dims::duration`, ...), `to_lower` from `dims::util` (Haskell `Text.toLower` semantics).
Rust `regex()` goes through `regex_tr.rs` (Haskell PCRE-ish → fancy-regex); check that your patterns compile
(any panic on first use shows up in the parity run). Prefer adding new shared helpers to a *new* file/function
rather than changing existing shared code; if you must change shared code, keep behaviour for existing
languages identical (run the EN parity before committing) and mention it in your final report.

## Order of work (dependencies)

Numeral → Ordinal → TimeGrain → Duration → Quantity, Distance, Volume, Temperature, AmountOfMoney → Time.
Email/Url/PhoneNumber/CreditCardNumber use the common rules plus any language-specific ones (check
`Duckling/Rules/<LANG>.hs` and e.g. `Duckling/Email/<LANG>`); once the fixtures pass add them to `PORTED`.
Only the dimensions listed in `Duckling/Dimensions/<LANG>.hs` (`allDimensions`) have rules/fixtures.
Locale-specific rule sets (e.g. `Numeral/ES/MX`, `Time/NL/BE`, `Time/ZH/CN`) go in `locale_rules`; `defaultRules` may
prepend `rulesBackwardCompatible` (read the Haskell `Rules/<LANG>.hs`).

Add a dimension to `PORTED` only when **all** fixture records whose targets are that dimension (or sets of
dimensions) pass. Records are gated on *all* targets being ported.

## Verifying

```
# in your worktree, with your own target dir
export CARGO_TARGET_DIR=$PWD/target
DUCKLING_PARITY_LANG=ES cargo test --release -p duckling --test parity -- --nocapture 2>&1 | tail -60
# DUCKLING_PARITY_SHOW=100 prints more failures. Output lists haskell vs rust JSON and per-target failure counts.
cargo test --release -p duckling --lib --test parity        # full run, must stay at 0 failures (EN etc.)
```

The summary line `parity: N unique records, C checked, F failed, S skipped` — F must be 0 and your language's
`skipped` must drop to 0 when the language is done. Unported-but-listed dims never get checked, so be honest in `PORTED`.
Do not modify `fixtures/` ever. If a fixture seems wrong, it is right: the oracle is upstream Haskell; find your bug.
Also add the language's feature to nothing else — `lang-xx` features already exist and `all-langs` is the default.

## Workflow rules

- Work in your own git worktree/branch (`git worktree add ../duckling-rs-wt/<xx> -b port/<xx>`), commit often,
  small commits (`<XX>: numeral`, `<XX>: time`, ...). Do not touch other languages' folders.
- When done (or if you run out of budget) leave the branch committed and report: dims ported, `PORTED` list,
  parity summary line for your language, shared files you changed, and anything that does not match.
- Haskell semantic gotchas that have bitten before: `Data.Text.toLower`/`T.words`/`T.splitOn` semantics, `Double`
  arithmetic order (keep the same operation order), `round`/`floor` (banker's rounding for `round`), `Scientific`
  JSON encoding, regex alternation order (leftmost-alternative, not longest), `(?i)` handling, Unicode classes.
