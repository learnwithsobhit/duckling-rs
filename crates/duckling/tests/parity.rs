//! Parity test: replays fixtures captured from the Haskell implementation (see `tools/oracle`)
//! and requires byte-identical `entities` JSON.
//!
//! Fixtures live in `<repo>/fixtures/*.jsonl`. Records whose language/dimension combination has
//! not been ported yet are counted as "skipped (unported)"; set `DUCKLING_PARITY_STRICT=1` to
//! fail when anything is skipped.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use duckling::time_zone::{DucklingTime, TimeZone, TimeZoneSeries, UtcTime};
use duckling::{analyze, format_token, Context, Dim, Lang, Locale, Options, Region};
use serde::Deserialize;
use serde_json::value::RawValue;

#[derive(Deserialize)]
struct Utc {
    mjd: i64,
    psecs: i64,
}

#[derive(Deserialize)]
struct Zone {
    offset_min: i32,
    summer: bool,
    name: String,
}

#[derive(Deserialize)]
struct Transition {
    at: Utc,
    zone: Zone,
}

#[derive(Deserialize)]
struct Record {
    lang: String,
    region: Option<String>,
    ref_utc: Utc,
    tz_default: Zone,
    tz_transitions: Vec<Transition>,
    latent: bool,
    targets: Vec<String>,
    input: String,
    entities: Box<RawValue>,
}

fn zone(z: &Zone) -> TimeZone {
    TimeZone { offset_minutes: z.offset_min, summer_only: z.summer, name: z.name.clone() }
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

#[test]
fn replay_haskell_fixtures() {
    let dir = fixtures_dir();
    let mut files: Vec<PathBuf> = match fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map(|x| x == "jsonl").unwrap_or(false))
            .collect(),
        Err(_) => Vec::new(),
    };
    files.sort();
    if files.is_empty() {
        eprintln!("no fixtures found in {:?}; run tools/oracle/capture.sh", dir);
        return;
    }

    let (mut total, mut checked, mut skipped) = (0usize, 0usize, 0usize);
    let mut failures: Vec<String> = Vec::new();
    let mut skipped_by: HashMap<String, usize> = HashMap::new();
    let mut seen: HashSet<String> = HashSet::new();
    // DUCKLING_PARITY_LANG=ES,FR restricts the replay to those languages (fast inner loop).
    let only_langs: Option<Vec<String>> = std::env::var("DUCKLING_PARITY_LANG")
        .ok()
        .map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect());
    // DUCKLING_PARITY_SHOW=N prints N failures (default 25).
    let show: usize = std::env::var("DUCKLING_PARITY_SHOW").ok().and_then(|v| v.parse().ok()).unwrap_or(25);
    let mut failed_by: HashMap<String, usize> = HashMap::new();

    for f in files {
        let text = fs::read_to_string(&f).unwrap();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            if !seen.insert(line.to_string()) {
                continue; // identical record already replayed
            }
            let rec: Record = serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("bad fixture line in {:?}: {}", f, e));
            if let Some(only) = &only_langs {
                if !only.iter().any(|l| l.eq_ignore_ascii_case(&rec.lang)) {
                    continue;
                }
            }
            total += 1;
            let lang = Lang::parse(&rec.lang).expect("lang");
            let region = rec.region.as_deref().and_then(Region::parse);
            let locale = Locale::new(lang, region);
            let targets: Vec<Dim> = rec
                .targets
                .iter()
                .map(|n| Dim::from_name(n).unwrap_or_else(|| panic!("unknown dim {}", n)))
                .collect();
            let in_scope = !targets.is_empty() && targets.iter().all(|d| duckling::ported::is_ported(locale, *d));
            if !in_scope {
                skipped += 1;
                *skipped_by.entry(format!("{}:{}", locale, rec.targets.join(","))).or_default() += 1;
                continue;
            }
            checked += 1;
            let series = TimeZoneSeries {
                default_zone: zone(&rec.tz_default),
                transitions: rec
                    .tz_transitions
                    .iter()
                    .map(|t| (UtcTime::new(t.at.mjd, t.at.psecs), zone(&t.zone)))
                    .collect(),
            };
            let ctx = Context {
                reference_time: DucklingTime { utc: UtcTime::new(rec.ref_utc.mjd, rec.ref_utc.psecs), series },
                locale,
                time_zones: None,
            };
            let opts = Options { with_latent: rec.latent };
            let tset: HashSet<Dim> = targets.iter().copied().collect();
            let got: Vec<_> = analyze(&rec.input, &ctx, &opts, &tset)
                .iter()
                .map(|r| format_token(&rec.input, r).to_json())
                .collect();
            let got_text = duckling::json::Json::Arr(got).encode();
            if got_text != rec.entities.get() {
                *failed_by.entry(format!("{}:{}", locale, rec.targets.join(","))).or_default() += 1;
                failures.push(format!(
                    "{} {:?} targets={:?} latent={}\n  haskell: {}\n  rust:    {}",
                    locale,
                    rec.input,
                    rec.targets,
                    rec.latent,
                    rec.entities.get(),
                    got_text
                ));
            }
        }
    }

    let mut skipped_sorted: Vec<_> = skipped_by.into_iter().collect();
    skipped_sorted.sort_by(|a, b| b.1.cmp(&a.1));
    eprintln!(
        "parity: {} unique records, {} checked, {} failed, {} skipped (unported)",
        total,
        checked,
        failures.len(),
        skipped
    );
    for (k, n) in skipped_sorted.iter().take(15) {
        eprintln!("  skipped {:>6}  {}", n, k);
    }
    if !failures.is_empty() {
        let mut fb: Vec<_> = failed_by.into_iter().collect();
        fb.sort_by(|a, b| b.1.cmp(&a.1));
        for (k, n) in fb.iter().take(40) {
            eprintln!("  failed  {:>6}  {}", n, k);
        }
        for f in failures.iter().take(show) {
            eprintln!("FAIL {}", f);
        }
        panic!("{} parity failures (showing up to {} above)", failures.len(), show);
    }
    if std::env::var("DUCKLING_PARITY_STRICT").is_ok() && skipped > 0 {
        panic!("{} fixture records skipped", skipped);
    }
}
