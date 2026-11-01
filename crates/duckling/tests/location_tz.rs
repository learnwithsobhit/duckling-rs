//! Opt-in location timezone dimension. Flag off must match today's Time parse.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use duckling::dims::time::calendar::{from_gregorian, PSECS_PER_SEC};
use duckling::service::{handle_parse, make_reftime, ParseParams};
use duckling::time_zone::{DucklingTime, TimeZoneSeries, UtcTime};
use duckling::tzif;
use duckling::{parse, Context, Dim, Lang, Locale, Options};

fn load_zones() -> HashMap<String, TimeZoneSeries> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/tz/tzdata");
    let mut m = HashMap::new();
    for name in ["Asia/Kolkata", "Europe/London", "America/Los_Angeles"] {
        let bytes = std::fs::read(base.join(name)).unwrap();
        m.insert(name.to_string(), tzif::parse_series(&bytes).unwrap());
    }
    m.insert("America/Toronto".into(), TimeZoneSeries::fixed(-300));
    m.insert("America/New_York".into(), TimeZoneSeries::fixed(-300));
    m.insert("Australia/Sydney".into(), TimeZoneSeries::fixed(660));
    m
}

fn ctx_at(y: i64, mo: i64, d: i64, h: i64, mi: i64, request_tz: &str) -> Context {
    let tzs = load_zones();
    let wall = UtcTime {
        mjd: from_gregorian(y, mo, d),
        psecs: (h * 3600 + mi * 60) * PSECS_PER_SEC,
    };
    let series = tzs.get(request_tz).cloned().unwrap_or_else(|| TimeZoneSeries::fixed(-480));
    Context {
        reference_time: DucklingTime { utc: wall, series },
        locale: Locale::new(Lang::EN, None),
        time_zones: Some(Arc::new(tzs)),
    }
}

fn ctx_la_late() -> Context {
    // 2026-11-01 23:00 in America/Los_Angeles (PST). Delhi is already 2026-11-02.
    ctx_at(2026, 11, 1, 23, 0, "America/Los_Angeles")
}

fn ctx_morning() -> Context {
    ctx_at(2013, 2, 12, 4, 30, "America/Los_Angeles")
}

fn opts() -> Options {
    Options { with_latent: false }
}

fn parse_on(ctx: &Context, input: &str, dims: &[Dim]) -> String {
    let ents = parse(input, ctx, &opts(), dims);
    format!(
        "[{}]",
        ents.iter().map(|e| e.to_json().encode()).collect::<Vec<_>>().join(",")
    )
}

fn time_only(ctx: &Context, input: &str) -> String {
    parse_on(ctx, input, &[Dim::Time])
}

fn with_loc(ctx: &Context, input: &str) -> String {
    parse_on(ctx, input, &[Dim::Time, Dim::Timezone])
}

#[test]
fn flag_off_matches_time_only() {
    let ctx = ctx_morning();
    for input in ["now", "today", "in 2 days", "5pm PST", "tomorrow at 5pm"] {
        assert_eq!(time_only(&ctx, input), with_loc(&ctx, input), "{input}");
    }
}

#[test]
fn unknown_city_leaves_now() {
    let ctx = ctx_morning();
    assert_eq!(time_only(&ctx, "now"), with_loc(&ctx, "now in narnia"));
}

#[test]
fn live_in_london_is_not_an_entity() {
    let out = with_loc(&ctx_morning(), "I live in london");
    assert_eq!(out, "[]", "{out}");
}

#[test]
fn now_in_delhi_uses_kolkata() {
    let out = with_loc(&ctx_la_late(), "now in delhi");
    assert!(out.contains(r#""dim":"time""#), "{out}");
    assert!(out.contains(r#""body":"now in delhi""#), "{out}");
    assert!(out.contains("+05:30"), "{out}");
    assert!(out.contains(r#""timezone":"Asia/Kolkata""#), "{out}");
    assert!(out.contains(r#""city":"Delhi""#), "{out}");
    assert!(out.contains(r#""dim":"timezone""#), "{out}");
    let time_at = out.find(r#""dim":"time""#).unwrap();
    let tz_at = out.find(r#""dim":"timezone""#).unwrap();
    assert!(time_at < tz_at, "time should be listed before timezone: {out}");
}

#[test]
fn date_line_today_and_tomorrow_in_delhi() {
    let ctx = ctx_la_late();
    let today = with_loc(&ctx, "today in delhi");
    assert!(today.contains("2026-11-02T00:00:00.000+05:30"), "{today}");
    let tomorrow = with_loc(&ctx, "tomorrow in delhi");
    assert!(tomorrow.contains("2026-11-03T00:00:00.000+05:30"), "{tomorrow}");
    let date = with_loc(&ctx, "date in delhi");
    assert!(date.contains("2026-11-02T00:00:00.000+05:30"), "{date}");
    let date_tom = with_loc(&ctx, "date in delhi tomorrow");
    assert!(date_tom.contains("2026-11-03T00:00:00.000+05:30"), "{date_tom}");
}

#[test]
fn five_pm_in_london_is_civil_time_there() {
    let out = with_loc(&ctx_morning(), "5pm in london");
    assert!(out.contains("T17:00:00.000"), "{out}");
    assert!(out.contains("Europe/London"), "{out}");
    assert!(out.contains("America/Toronto"), "{out}");
}

#[test]
fn london_dst_offset() {
    let winter = with_loc(&ctx_morning(), "5pm in london");
    assert!(winter.contains("2013-02-12T17:00:00.000+00:00"), "{winter}");
    let summer = with_loc(&ctx_at(2013, 7, 12, 4, 30, "America/Los_Angeles"), "5pm in london");
    assert!(summer.contains("2013-07-12T17:00:00.000+01:00"), "{summer}");
}

#[test]
fn pst_abbreviation_does_not_stack_a_city() {
    let ctx = ctx_morning();
    let off = time_only(&ctx, "5pm PST");
    let on = with_loc(&ctx, "5pm PST");
    assert_eq!(off, on);
    assert!(!on.contains("Asia/Kolkata"), "{on}");
}

#[test]
fn london_unqualified_returns_both_cities() {
    let out = with_loc(&ctx_morning(), "now in london");
    let uk = out.find("London, UK").expect(&out);
    let on = out.find("London, Ontario").expect(&out);
    assert!(uk < on, "UK before Ontario: {out}");
    assert_eq!(out.matches(r#""dim":"time""#).count(), 2, "{out}");
    assert_eq!(out.matches(r#""dim":"timezone""#).count(), 2, "{out}");
}

#[test]
fn london_ontario_is_only_ontario() {
    let out = with_loc(&ctx_morning(), "now in london, ontario");
    assert!(out.contains("London, Ontario"), "{out}");
    assert!(!out.contains("London, UK"), "{out}");
    assert!(out.contains(r#""body":"now in london, ontario""#), "{out}");
}

#[test]
fn unknown_qualifier_does_not_guess() {
    let out = with_loc(&ctx_morning(), "now in london, narnia");
    assert!(!out.contains("Europe/London"), "{out}");
    assert!(!out.contains("America/Toronto"), "{out}");
}

#[test]
fn time_in_sydney() {
    let out = with_loc(&ctx_morning(), "time in sydney");
    assert!(out.contains("Australia/Sydney"), "{out}");
    assert!(out.contains(r#""body":"time in sydney""#), "{out}");
}

#[test]
fn handle_parse_location_tz_flag() {
    let tzs = load_zones();
    let on = handle_parse(
        &ParseParams {
            text: Some("now in delhi".into()),
            locale: Some("en_US".into()),
            tz: Some("America/Los_Angeles".into()),
            location_tz: Some("true".into()),
            dims: Some(r#"["time"]"#.into()),
            ..Default::default()
        },
        &tzs,
        0,
    )
    .unwrap();
    assert!(on.contains("Asia/Kolkata"), "{on}");
    let off = handle_parse(
        &ParseParams {
            text: Some("now in delhi".into()),
            locale: Some("en_US".into()),
            tz: Some("America/Los_Angeles".into()),
            dims: Some(r#"["time"]"#.into()),
            ..Default::default()
        },
        &tzs,
        0,
    )
    .unwrap();
    assert!(!off.contains("Asia/Kolkata"), "{off}");
    assert!(off.contains(r#""body":"now""#), "{off}");
}

#[test]
fn us_dst_fallback_does_not_shift_mumbai() {
    // 2026-11-01 16:18:28 UTC = 21:48:28 IST. US DST ended at 09:00 UTC, so LA is PST.
    // Looking up America/Los_Angeles at the stored wall clock (08:18 as if UTC) still
    // sees PDT; converting through that offset would print 20:48 IST.
    let tzs = load_zones();
    let utc = UtcTime {
        mjd: from_gregorian(2026, 11, 1),
        psecs: (16 * 3600 + 18 * 60 + 28) * PSECS_PER_SEC,
    };
    let ctx = Context {
        reference_time: make_reftime(&tzs, "America/Los_Angeles", utc),
        locale: Locale::new(Lang::EN, None),
        time_zones: Some(Arc::new(tzs)),
    };
    let out = with_loc(&ctx, "now in mumbai");
    assert!(out.contains("2026-11-01T21:48:28.000+05:30"), "{out}");
}
