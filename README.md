# duckling-rs

Rust port of Meta’s [Duckling](https://github.com/facebook/duckling): parse natural-language text into structured entities.

```text
"tomorrow at 5pm"  →  2026-11-02T17:00:00.000-08:00  (hour)
"this week"        →  Monday of the current week     (week grain)
"diwali 2026"      →  2026-11-08                     (holiday)
"$20"              →  20 USD
```

The library crate, HTTP server, and WASM bindings all share the same parser. On the recorded Haskell fixtures the JSON is byte-identical to upstream Duckling (`59a13ff`): 33,555 records checked, 0 failures.

Not on crates.io yet. Depend on this repo by path or git.

## What’s in the workspace

| Crate | What it is |
|---|---|
| [`duckling`](crates/duckling) | Parser library |
| [`duckling-server`](crates/duckling-server) | Drop-in for upstream’s `POST /parse` example server |
| [`duckling-wasm`](crates/duckling-wasm) | `wasm-bindgen` wrapper around the same parse API |
| [`duckling-train`](crates/duckling-train) | Naive Bayes classifier trainer |

## Dimensions

| Dimension | Example input | Notes |
|---|---|---|
| `time` | `tomorrow at 5pm`, `now`, `this week` | Needs a reference time and timezone |
| `timezone` | `now in mumbai` | Opt-in only (`location_tz=true`); not in `GET /targets` |
| `duration` | `two hours`, `3 days` | |
| `numeral` | `twenty-one`, `3.5` | |
| `ordinal` | `first`, `22nd` | |
| `amount-of-money` | `$20`, `10 euros` | Locale picks the default currency |
| `distance` | `3 miles` | |
| `volume` | `2 liters` | |
| `temperature` | `72 degrees` | |
| `quantity` | `3 cups of flour` | |
| `email` | `a@b.com` | |
| `url` | `https://example.com` | |
| `phone-number` | `+1 415-555-0100` | |
| `credit-card-number` | `4111…` | |

Omit `dims` to parse every dimension the language supports.

## HTTP server

```bash
cargo run --release -p duckling-server
# duckling-server listening on port 8000
```

Port comes from `-p` / `--port`, then `$PORT`, then `8000`. Time zones are loaded from `/usr/share/zoneinfo` (or `$DUCKLING_ZONEINFO`).

```bash
# health
curl -s http://127.0.0.1:8000/
# quack!

# parse
curl -s -XPOST http://127.0.0.1:8000/parse \
  --data 'locale=en_US&text=tomorrow at 5pm&tz=America/Los_Angeles'

# India wall clock, Indian English date/holiday rules
curl -s -XPOST http://127.0.0.1:8000/parse \
  --data 'locale=en_IN&text=now&tz=Asia/Kolkata'

# Current time in a named city (opt-in; off by default)
curl -s -XPOST http://127.0.0.1:8000/parse \
  --data 'locale=en_US&text=now in mumbai&dims=["time"]&location_tz=true'
```

Form fields (same as upstream):

| Field | Default | Meaning |
|---|---|---|
| `text` | required | Input to parse |
| `locale` | `en` | Language + region (`en_US`, `en_IN`, `en_GB`, `fr_XX`, …) |
| `lang` | from `locale`, else `EN` | Language only |
| `tz` | `America/Los_Angeles` | Olson zone for the **request clock** (`now`, `today`, printed offset) |
| `reftime` | server `now` | Unix milliseconds; the “now” the parser resolves against |
| `dims` | all | JSON array, e.g. `["time"]` |
| `latent` | `false` | Also return weak parses (`7` as an hour) |
| `location_tz` | `false` | Opt-in: resolve `now/today/5pm in <city>` in that city’s zone (English) |

`locale` chooses **rules** (date order, holidays, currency). `tz` chooses **which clock** the parser treats as “here”. They are independent: `locale=en_IN&tz=Asia/Kolkata` is Indian English in IST; `locale=en_US&tz=Asia/Kolkata` is US English rules on an IST clock.

### Location timezone (`location_tz=true`)

Off by default so `/parse` and Haskell fixtures stay unchanged. When set, English phrases that name a city on the allowlist are resolved **in that city’s zone**, using the same instant as `reftime` (server `now` if omitted). You do **not** set `tz` to the city; `tz` is still the request’s own clock (default `America/Los_Angeles`). DST comes from the server’s Olson `TZif` files.

```bash
curl -s -XPOST http://127.0.0.1:8000/parse \
  --data 'text=now in mumbai&dims=["time"]&location_tz=true'
```

```json
[
  {
    "dim": "time",
    "body": "now in mumbai",
    "value": {
      "city": "Mumbai",
      "timezone": "Asia/Kolkata",
      "value": "2026-11-01T21:48:28.000+05:30",
      "grain": "second",
      "type": "value"
    }
  },
  {
    "dim": "timezone",
    "body": "now in mumbai",
    "value": { "city": "Mumbai", "timezone": "Asia/Kolkata", "offset": "+05:30" }
  }
]
```

| Input | What you get |
|---|---|
| `now in mumbai` / `time in delhi` | Same instant as request `now`, printed with the city offset |
| `today in delhi` / `tomorrow in delhi` | That calendar day **in the city** (date-line included) |
| `5pm in london` / `tomorrow at 5pm in sydney` | That civil time in the city, not “5pm here, then converted” |
| `now in london` | Two hits: `London, UK` then `London, Ontario` |
| `now in london, ontario` | Ontario only |
| `now` / `5pm PST` / `I live in london` | Unchanged; no city timezone entity |

Cities are a closed English allowlist (Delhi, Mumbai, London, Sydney, …), not live geocoding. Unknown places leave the normal time parse. `GET /targets` does not list `timezone`. WASM accepts the same `location_tz` field on the JSON request.

## Use as a crate

```toml
[dependencies]
duckling = { path = "../duckling-rs/crates/duckling" }
# English only, smaller binary:
# duckling = { path = "../duckling-rs/crates/duckling", default-features = false, features = ["lang-en"] }
```

A git dependency works the same way once this repo has a remote (`git = "…"`). It is not on crates.io yet.

Same request shape as `POST /parse`:

```rust
use duckling::service::{handle_parse, ParseParams};
use duckling::time_zone::TimeZoneSeries;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let params = ParseParams {
        text: Some("tomorrow at 5pm".into()),
        locale: Some("en_US".into()),
        tz: Some("UTC".into()),
        dims: Some(r#"["time"]"#.into()),
        ..Default::default()
    };

    // Empty map: unknown zones behave like UTC. Load TZif files for real offsets.
    let tzs: HashMap<String, TimeZoneSeries> = HashMap::new();
    let json = handle_parse(&params, &tzs, now).expect("parse");
    println!("{json}");
}
```

Or call the parser directly:

```rust
use duckling::{parse, Context, Dim, Lang, Locale, Options};
use duckling::service::make_reftime;
use duckling::time_zone::{TimeZoneSeries, UtcTime};
use std::collections::HashMap;

let tzs = HashMap::<String, TimeZoneSeries>::new();
let ctx = Context {
    locale: Locale::new(Lang::EN, None),
    reference_time: make_reftime(&tzs, "UTC", UtcTime::from_unix_millis(now_ms)),
    time_zones: None,
};
let hits = parse("in 2 days", &ctx, &Options { with_latent: false }, &[Dim::Time]);
```

To use named zones (`Asia/Kolkata`, `America/Los_Angeles`), parse Olson `TZif` files with `duckling::tzif::parse_series` (the server does this from `/usr/share/zoneinfo`).

## WebAssembly

```js
const d = new Duckling();
d.addTimeZone("America/Los_Angeles", tzifBytes); // optional; unknown zones are UTC
d.parse(JSON.stringify({
  text: "tomorrow at 5pm",
  dims: ["time"],
  tz: "America/Los_Angeles",
}));
d.parse(JSON.stringify({
  text: "now in mumbai",
  dims: ["time"],
  location_tz: true,
}));
```

Default feature is `lang-en` so the wasm bundle stays small. Enable more with `lang-fr`, `all-langs`, etc.

```bash
cargo build --release -p duckling-wasm --target wasm32-unknown-unknown
```

## Languages

49 languages, matching upstream. Enable one with `features = ["lang-xx"]`; the library default is `all-langs`.

AF AR BG BN CA CS DA DE EL EN ES ET FA FI FR GA HE HI HR HU
ID IS IT JA KA KN KM KO LO ML MN MY NB NE NL PL PT RO RU SK
SV SW TA TE TH TR UK VI ZH

English regions with extra rules: `AU BZ CA GB IE IN JM NZ PH TT US ZA` (date order, holidays, currency). Other `en_*` locales fall back to `EN_XX`.

## Build and test

Rust 1.70+ (edition 2021). Release is the intended profile; debug is slow on Time.

```bash
cargo test --release --workspace --exclude duckling-wasm
cargo test --release -p duckling --test parity -- --nocapture

# one language
DUCKLING_PARITY_LANG=EN cargo test --release -p duckling --test parity -- --nocapture
```

CI also runs `cargo fmt`, `clippy -D warnings`, a wasm32 build, and a Criterion smoke bench.

Haskell fixtures live in `fixtures/*.jsonl`. They are captured from vendored upstream (`vendor/duckling`) via [`tools/oracle`](tools/oracle/README.md). Do not edit gold files by hand.

Intentional differences from Haskell (HTTP envelope, a few parser bug fixes) are listed in [`DIVERGENCES.md`](DIVERGENCES.md).

## Layout

```text
crates/duckling/           parser, rules, classifiers
crates/duckling-server/    axum HTTP front-end
crates/duckling-wasm/      wasm-bindgen
crates/duckling-train/     classifier trainer
fixtures/                  Haskell oracle JSONL
vendor/duckling/           upstream snapshot (submodule)
```

## License

BSD-3-Clause, same family as [upstream Duckling](https://github.com/facebook/duckling).
