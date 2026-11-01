//! Regression tests for Phase 7 documented divergences from upstream Haskell.

use duckling::dims::time::calendar::from_gregorian;
use duckling::time_zone::{DucklingTime, TimeZoneSeries, UtcTime};
use duckling::{parse, Context, Dim, Lang, Locale, Options};

/// Tuesday 2013-02-12 04:30 UTC, zone series UTC−2 (`Duckling.Testing.Types.testContext`).
fn test_context(lang: Lang) -> Context {
    let mjd = from_gregorian(2013, 2, 12);
    let psecs = (4 * 3600 + 30 * 60) * 1_000_000_000_000i64;
    Context {
        reference_time: DucklingTime {
            utc: UtcTime { mjd, psecs },
            series: TimeZoneSeries::fixed(-120),
        },
        locale: Locale::new(lang, None),
        time_zones: None,
    }
}

fn opts() -> Options {
    Options { with_latent: false }
}

fn encoded(lang: Lang, input: &str, dims: &[Dim]) -> String {
    let ents = parse(input, &test_context(lang), &opts(), dims);
    format!(
        "[{}]",
        ents.iter()
            .map(|e| e.to_json().encode())
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn money(input: &str) -> String {
    encoded(Lang::EN, input, &[Dim::AmountOfMoney])
}

fn time_en(input: &str) -> String {
    encoded(Lang::EN, input, &[Dim::Time])
}

fn number(lang: Lang, input: &str) -> String {
    encoded(lang, input, &[Dim::Numeral])
}

#[test]
fn p1_georgian_numeral_is_not_a_substring() {
    // https://github.com/facebook/duckling/issues/439
    assert!(!number(Lang::KA, "პათიზ").contains(r#""value":10"#));
    assert!(number(Lang::KA, "ათი").contains(r#""value":10"#));
}

#[test]
fn p2_german_compound_large_numbers() {
    // https://github.com/facebook/duckling/issues/481
    assert!(number(Lang::DE, "fünfzigtausend").contains(r#""value":50000"#));
    assert!(number(Lang::DE, "viertausendvierhundert").contains(r#""value":4400"#));
    let dur = encoded(Lang::DE, "in fünfzigtausend sekunden", &[Dim::Duration]);
    assert!(dur.contains(r#""value":50000"#), "{dur}");
}

#[test]
fn p4_now_composes_with_time_of_day() {
    // https://github.com/facebook/duckling/issues/440
    // Three days from now (Tue 12 Feb) at 6pm → Fri 15 Feb 18:00 (−02:00).
    let got = time_en("Three days from now at 6pm");
    assert!(got.contains("2013-02-15T18:00:00.000-02:00"), "{got}");
}

#[test]
fn p5_before_the_end_of_the_month_uses_last_day() {
    // https://github.com/facebook/duckling/issues/275
    // Ref is 12 Feb 2013; last day of February is the 28th. Open/before bound is the 28th 00:00
    // or the 1st of March depending on exclusive end.
    let got = time_en("before the end of the month");
    assert!(
        got.contains("2013-02-28") || got.contains("2013-03-01"),
        "expected last day of February, got {got}"
    );
    assert!(!got.contains("2013-02-21"), "{got}");
    let dec = time_en("before end of dec");
    assert!(
        dec.contains("2013-12-31") || dec.contains("2014-01-01"),
        "{dec}"
    );
    assert!(!dec.contains("2013-12-21"), "{dec}");
}

#[test]
fn p6_number_dollar_number_is_not_cents() {
    // https://github.com/facebook/duckling/issues/268
    let two = money("two $ 500");
    assert!(!two.contains(r#""value":7"#), "{two}");
    assert!(two.contains(r#""value":500"#), "{two}");
    let a = money("a $ 100");
    assert!(!a.contains(r#""value":2"#), "{a}");
    assert!(a.contains(r#""value":100"#), "{a}");
    // Valid cents compose is unchanged.
    assert!(money("$20 43").contains(r#""value":20.43"#));
}

#[test]
fn p7_last_four_months_of_2016() {
    // https://github.com/facebook/duckling/issues/250
    let got = time_en("last four months of 2016");
    assert!(got.contains("2016"), "{got}");
    assert!(
        !got.contains("2013-11") && !got.contains("2012-11"),
        "{got}"
    );
}

#[test]
fn p10_arabic_does_not_match_sunday_inside_a_verb() {
    // https://github.com/facebook/duckling/issues/131
    let got = encoded(Lang::AR, "وأحدث ذلك مشكلة كبيرة", &[Dim::Time]);
    assert!(
        !got.contains("Sunday") && !got.to_lowercase().contains("أحد"),
        "{got}"
    );
}
