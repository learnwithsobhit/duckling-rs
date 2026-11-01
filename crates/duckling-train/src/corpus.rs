//! Time training corpora, parsed from `vendor/duckling/Duckling/Time/**/Corpus.hs`.
//!
//! Mirrors `Duckling.Ranking.Generate.getCorpus`: language `defaultCorpus` for EN/NL
//! default locales, language `corpus` otherwise, plus locale-specific examples for
//! EN_GB / EN_US. Languages that `getCorpusForLang` maps to `[]` stay empty.

use std::path::{Path, PathBuf};

use duckling::dims::time::calendar::{from_gregorian, time_of_day_to_psecs, PSECS_PER_SEC};
use duckling::dims::time::types::{to_rfc3339, IntervalDirection};
use duckling::dims::time_grain::Grain;
use duckling::json::Json;
use duckling::time_zone::{DucklingTime, TimeZone, TimeZoneSeries, UtcTime};
use duckling::{Context, Lang, Locale, Options, Region};

/// One labeled training sentence (`Duckling.Testing.Types.Example`).
#[derive(Clone, Debug)]
pub struct Example {
    pub text: String,
    pub expected: Expected,
}

/// The Time.Corpus constructor that becomes the `check` predicate.
#[derive(Clone, Debug)]
pub enum Expected {
    Datetime { dt: Datetime, grain: Grain, holiday: Option<String> },
    Interval { from: Datetime, to: Datetime, grain: Grain, holiday: Option<String> },
    Open { dir: IntervalDirection, dt: Datetime, grain: Grain },
}

#[derive(Clone, Copy, Debug)]
pub struct Datetime {
    pub year: i64,
    pub month: i64,
    pub day: i64,
    pub hour: i64,
    pub minute: i64,
    pub second: i64,
}

/// Training set for one locale (`Duckling.Testing.Types.Corpus`).
#[derive(Clone, Debug)]
pub struct TrainCorpus {
    pub locale: Locale,
    pub context: Context,
    pub options: Options,
    pub examples: Vec<Example>,
}

/// Locales exported by `tools/oracle/export/ExportClassifiers.hs`.
pub fn exported_locales() -> Vec<Locale> {
    let mut v: Vec<Locale> = Lang::ALL.iter().map(|l| Locale { lang: *l, region: None }).collect();
    v.push(Locale { lang: Lang::EN, region: Some(Region::GB) });
    v.push(Locale { lang: Lang::EN, region: Some(Region::US) });
    v
}

/// Languages whose `getCorpusForLang` is hard-coded to `[]` in Generate.hs.
fn empty_train_lang(lang: Lang) -> bool {
    matches!(
        lang,
        Lang::AF
            | Lang::BN
            | Lang::CS
            | Lang::ET
            | Lang::FI
            | Lang::FA
            | Lang::HI
            | Lang::ID
            | Lang::IS
            | Lang::KA
            | Lang::KM
            | Lang::KN
            | Lang::LO
            | Lang::ML
            | Lang::MN
            | Lang::MY
            | Lang::NE
            | Lang::SK
            | Lang::SW
            | Lang::TA
            | Lang::TE
            | Lang::TH
    )
}

/// `testContext` reference time: Tuesday 2013-02-12 04:30, offset −2h.
pub fn test_reftime() -> DucklingTime {
    reftime(2013, 2, 12, 4, 30, 0, -2)
}

fn reftime(y: i64, mo: i64, d: i64, h: i64, mi: i64, s: i64, offset_hours: i32) -> DucklingTime {
    DucklingTime {
        utc: utc_of(y, mo, d, h, mi, s),
        series: TimeZoneSeries {
            default_zone: TimeZone {
                offset_minutes: offset_hours * 60,
                summer_only: false,
                name: String::new(),
            },
            transitions: Vec::new(),
        },
    }
}

fn utc_of(y: i64, mo: i64, d: i64, h: i64, mi: i64, s: i64) -> UtcTime {
    UtcTime { mjd: from_gregorian(y, mo, d), psecs: time_of_day_to_psecs(h, mi, s * PSECS_PER_SEC) }
}

fn test_context(locale: Locale, reference_time: DucklingTime) -> Context {
    Context { locale, reference_time }
}

/// `Duckling.Time.Corpus.check`: compare JSON after dropping the `values` key.
pub fn predicate_matches(expected: &Expected, ctx: &Context, actual: &Json) -> bool {
    strip_values(&expected_json(expected, ctx)) == strip_values(actual)
}

pub fn strip_values(j: &Json) -> Json {
    match j {
        Json::Obj(m) => {
            let mut m = m.clone();
            m.remove("values");
            Json::Obj(m)
        }
        other => other.clone(),
    }
}

/// `datetime` / `datetimeInterval` / `datetimeOpenInterval` JSON (including `values`).
pub fn expected_json(expected: &Expected, ctx: &Context) -> Json {
    let series = &ctx.reference_time.series;
    match expected {
        Expected::Datetime { dt, grain, holiday } => {
            let mut obj = match simple_value(series, *dt, *grain) {
                Json::Obj(m) => m,
                _ => Default::default(),
            };
            let tv = Json::Obj(obj.clone());
            obj.insert("values".into(), Json::Arr(vec![tv]));
            if let Some(h) = holiday {
                obj.insert("holidayBeta".into(), Json::str(h.clone()));
            }
            Json::Obj(obj)
        }
        Expected::Interval { from, to, grain, holiday } => {
            let tv = Json::obj([
                ("type", Json::str("interval")),
                ("from", instant(series, *from, *grain)),
                ("to", instant(series, *to, *grain)),
            ]);
            let mut obj = match &tv {
                Json::Obj(m) => m.clone(),
                _ => Default::default(),
            };
            obj.insert("values".into(), Json::Arr(vec![tv]));
            if let Some(h) = holiday {
                obj.insert("holidayBeta".into(), Json::str(h.clone()));
            }
            Json::Obj(obj)
        }
        Expected::Open { dir, dt, grain } => {
            let inst = instant(series, *dt, *grain);
            let tv = match dir {
                IntervalDirection::Before => Json::obj([("type", Json::str("interval")), ("to", inst)]),
                IntervalDirection::After => Json::obj([("type", Json::str("interval")), ("from", inst)]),
            };
            let mut obj = match &tv {
                Json::Obj(m) => m.clone(),
                _ => Default::default(),
            };
            obj.insert("values".into(), Json::Arr(vec![tv]));
            Json::Obj(obj)
        }
    }
}

fn instant(series: &TimeZoneSeries, dt: Datetime, grain: Grain) -> Json {
    let utc = utc_of(dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second);
    Json::obj([("value", Json::str(to_rfc3339(series, utc))), ("grain", grain.json())])
}

fn simple_value(series: &TimeZoneSeries, dt: Datetime, grain: Grain) -> Json {
    let mut j = instant(series, dt, grain);
    if let Json::Obj(m) = &mut j {
        m.insert("type".into(), Json::str("value"));
    }
    j
}

pub fn vendor_time_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/duckling/Duckling/Time")
}

/// Load the training corpus Generate.hs would use for `locale`.
///
/// Catalan (`CA_XX`) is known to retrain off the exported table: several CA Time
/// regexes were tightened vs Haskell to reject zero-width matches (the Rust
/// engine drops `start==end` regex hits). Feature bags therefore differ. Do not
/// "fix" that here — restoring the Haskell patterns needs an engine change.
pub fn load_corpus(locale: Locale) -> Result<TrainCorpus, String> {
    load_corpus_from(&vendor_time_root(), locale)
}

pub fn load_corpus_from(time_root: &Path, locale: Locale) -> Result<TrainCorpus, String> {
    let reference_time = match locale.lang {
        Lang::VI => reftime(2017, 2, 2, 3, 55, 0, -2),
        _ => test_reftime(),
    };
    let context = test_context(locale, reference_time);
    let options = Options { with_latent: false };

    if empty_train_lang(locale.lang) {
        return Ok(TrainCorpus { locale, context, options, examples: Vec::new() });
    }

    let lang_file = time_root.join(format!("{}/Corpus.hs", locale.lang));
    let src = std::fs::read_to_string(&lang_file)
        .map_err(|e| format!("read {}: {e}", lang_file.display()))?;
    let mut examples = parse_concat_binding(&src, "allExamples")
        .map_err(|e| format!("{}: {e}", lang_file.display()))?;

    // EN / NL default locales use `defaultCorpus` = allExamples ++ custom.
    if locale.region.is_none() && matches!(locale.lang, Lang::EN | Lang::NL) {
        if let Ok(mut extra) = parse_concat_binding(&src, "custom") {
            examples.append(&mut extra);
        }
    }

    if let Some(region) = locale.region {
        let loc_file = time_root.join(format!("{}/{}/Corpus.hs", locale.lang, region));
        if loc_file.exists() {
            let loc_src = std::fs::read_to_string(&loc_file)
                .map_err(|e| format!("read {}: {e}", loc_file.display()))?;
            let mut extra = parse_concat_binding(&loc_src, "allExamples")
                .map_err(|e| format!("{}: {e}", loc_file.display()))?;
            examples.append(&mut extra);
        }
    }

    Ok(TrainCorpus { locale, context, options, examples })
}

/// Parse `{name} = concat [ examples ... ]`.
pub fn parse_concat_binding(src: &str, name: &str) -> Result<Vec<Example>, String> {
    let needle = format!("{name} = concat");
    let start = src.find(&needle).ok_or_else(|| format!("binding `{name} = concat` not found"))?;
    let after = &src[start + needle.len()..];
    let list_start = after.find('[').ok_or_else(|| format!("`{name}`: missing '[' after concat"))?;
    let list = take_bracket_list(&after[list_start..])?;
    parse_examples_list(list)
}

fn take_bracket_list(s: &str) -> Result<&str, String> {
    if !s.starts_with('[') {
        return Err("expected '['".into());
    }
    let mut depth = 0i32;
    let mut in_str = false;
    let mut escape = false;
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if in_str {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '{' && chars.peek().map(|(_, n)| *n) == Some('-') {
            let end = skip_block_comment(s, i)?;
            while chars.peek().map(|(j, _)| *j < end).unwrap_or(false) {
                chars.next();
            }
            continue;
        }
        if c == '-' && chars.peek().map(|(_, n)| *n) == Some('-') {
            for (_, n) in chars.by_ref() {
                if n == '\n' {
                    break;
                }
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(&s[..=i]);
                }
            }
            _ => {}
        }
    }
    Err("unclosed '['".into())
}

/// Nested Haskell `{- … -}` comment. `start` points at `{`.
fn skip_block_comment(s: &str, start: usize) -> Result<usize, String> {
    let b = s.as_bytes();
    if start + 1 >= b.len() || b[start] != b'{' || b[start + 1] != b'-' {
        return Ok(start + 1);
    }
    let mut depth = 1i32;
    let mut i = start + 2;
    while i + 1 < b.len() {
        if b[i] == b'{' && b[i + 1] == b'-' {
            depth += 1;
            i += 2;
            continue;
        }
        if b[i] == b'-' && b[i + 1] == b'}' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return Ok(i);
            }
            continue;
        }
        i += 1;
    }
    Err("unclosed block comment".into())
}

fn parse_examples_list(list: &str) -> Result<Vec<Example>, String> {
    let inner = list.strip_prefix('[').and_then(|s| s.strip_suffix(']')).ok_or("not a list")?;
    let mut out = Vec::new();
    let mut i = 0;
    let bytes = inner.as_bytes();
    while i < bytes.len() {
        i = skip_ws_and_comma(inner, i);
        if i >= bytes.len() {
            break;
        }
        if inner[i..].starts_with("examples") {
            i += "examples".len();
            i = skip_ws_and_comma(inner, i);
            let (expr, next) = take_parens(inner, i)?;
            let expected = parse_expected(expr)?;
            i = skip_ws_and_comma(inner, next);
            let (strs, next) = take_string_list(inner, i)?;
            i = next;
            for text in strs {
                out.push(Example { text, expected: expected.clone() });
            }
        } else {
            return Err(format!("expected `examples` at offset {i}: {:?}", &inner[i..].chars().take(40).collect::<String>()));
        }
    }
    Ok(out)
}

fn skip_ws_and_comma(s: &str, mut i: usize) -> usize {
    let b = s.as_bytes();
    while i < s.len() {
        let rest = &s[i..];
        let c = match rest.chars().next() {
            Some(c) => c,
            None => break,
        };
        if c.is_whitespace() || c == ',' {
            i += c.len_utf8();
            continue;
        }
        if c == '-' && i + 1 < b.len() && b[i + 1] == b'-' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c == '{' && i + 1 < b.len() && b[i + 1] == b'-' {
            match skip_block_comment(s, i) {
                Ok(n) => {
                    i = n;
                    continue;
                }
                Err(_) => break,
            }
        }
        break;
    }
    i
}

fn take_parens<'a>(s: &'a str, start: usize) -> Result<(&'a str, usize), String> {
    if !s[start..].starts_with('(') {
        return Err(format!("expected '(' at {start}"));
    }
    let mut depth = 0i32;
    let mut in_str = false;
    let mut escape = false;
    for (off, c) in s[start..].char_indices() {
        if in_str {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let end = start + off + 1;
                    return Ok((&s[start..end], end));
                }
            }
            _ => {}
        }
    }
    Err("unclosed '('".into())
}

fn take_string_list(s: &str, start: usize) -> Result<(Vec<String>, usize), String> {
    let list = take_bracket_list(&s[start..])?;
    let end = start + list.len();
    let inner = &list[1..list.len() - 1];
    let mut out = Vec::new();
    let mut i = 0;
    while i < inner.len() {
        i = skip_ws_and_comma(inner, i);
        if i >= inner.len() {
            break;
        }
        let (st, next) = take_hs_string(inner, i)?;
        out.push(st);
        i = next;
    }
    Ok((out, end))
}

fn take_hs_string(s: &str, start: usize) -> Result<(String, usize), String> {
    if !s[start..].starts_with('"') {
        return Err(format!("expected string at {start}"));
    }
    let mut out = String::new();
    let mut escape = false;
    for (off, c) in s[start + 1..].char_indices() {
        if escape {
            out.push(c);
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if c == '"' {
            return Ok((out, start + 1 + off + 1));
        }
        out.push(c);
    }
    Err("unclosed string".into())
}

fn parse_expected(paren: &str) -> Result<Expected, String> {
    let inner = paren.strip_prefix('(').and_then(|s| s.strip_suffix(')')).ok_or("expected (…)")?;
    let inner = inner.trim();
    if let Some(rest) = inner.strip_prefix("datetimeHoliday") {
        let (dt, rest) = parse_datetime_prefix(rest.trim())?;
        let (grain, rest) = parse_grain_prefix(rest.trim())?;
        let (holiday, _) = take_hs_string(rest.trim(), 0)?;
        Ok(Expected::Datetime { dt, grain, holiday: Some(holiday) })
    } else if let Some(rest) = inner.strip_prefix("datetimeIntervalHoliday") {
        let (from, to, rest) = parse_interval_prefix(rest.trim())?;
        let (grain, rest) = parse_grain_prefix(rest.trim())?;
        let (holiday, _) = take_hs_string(rest.trim(), 0)?;
        Ok(Expected::Interval { from, to, grain, holiday: Some(holiday) })
    } else if let Some(rest) = inner.strip_prefix("datetimeInterval") {
        let (from, to, rest) = parse_interval_prefix(rest.trim())?;
        let (grain, _) = parse_grain_prefix(rest.trim())?;
        Ok(Expected::Interval { from, to, grain, holiday: None })
    } else if let Some(rest) = inner.strip_prefix("datetimeOpenInterval") {
        let rest = rest.trim();
        let (dir, rest) = if rest.starts_with("Before") {
            (IntervalDirection::Before, rest["Before".len()..].trim())
        } else if rest.starts_with("After") {
            (IntervalDirection::After, rest["After".len()..].trim())
        } else {
            return Err(format!("open interval direction: {rest}"));
        };
        let (dt, rest) = parse_datetime_prefix(rest)?;
        let (grain, _) = parse_grain_prefix(rest.trim())?;
        Ok(Expected::Open { dir, dt, grain })
    } else if let Some(rest) = inner.strip_prefix("datetime") {
        let (dt, rest) = parse_datetime_prefix(rest.trim())?;
        let (grain, _) = parse_grain_prefix(rest.trim())?;
        Ok(Expected::Datetime { dt, grain, holiday: None })
    } else {
        Err(format!("unknown constructor: {}", &inner.chars().take(40).collect::<String>()))
    }
}

fn parse_datetime_prefix(s: &str) -> Result<(Datetime, &str), String> {
    if !s.starts_with('(') {
        return Err(format!("expected datetime tuple, got {s}"));
    }
    let (body, end) = take_parens(s, 0)?;
    let nums = parse_int_tuple(body)?;
    if nums.len() != 6 {
        return Err(format!("datetime wants 6 fields, got {nums:?}"));
    }
    Ok((
        Datetime {
            year: nums[0],
            month: nums[1],
            day: nums[2],
            hour: nums[3],
            minute: nums[4],
            second: nums[5],
        },
        &s[end..],
    ))
}

fn parse_interval_prefix(s: &str) -> Result<(Datetime, Datetime, &str), String> {
    // ((y,m,d,h,mi,s), (y,m,d,h,mi,s))
    let (pair, end) = take_parens(s, 0)?;
    let inner = pair.strip_prefix('(').and_then(|x| x.strip_suffix(')')).unwrap();
    let (from, mid) = parse_datetime_prefix(inner.trim())?;
    let mid = mid.trim().strip_prefix(',').unwrap_or(mid).trim();
    let (to, _) = parse_datetime_prefix(mid)?;
    Ok((from, to, &s[end..]))
}

fn parse_int_tuple(paren: &str) -> Result<Vec<i64>, String> {
    let inner = paren.trim().strip_prefix('(').and_then(|s| s.strip_suffix(')')).ok_or("tuple")?;
    inner
        .split(',')
        .map(|p| {
            p.trim()
                .parse::<f64>()
                .map(|x| x as i64)
                .map_err(|e| format!("int {p}: {e}"))
        })
        .collect()
}

fn parse_grain_prefix(s: &str) -> Result<(Grain, &str), String> {
    const GRAINS: &[(&str, Grain)] = &[
        ("NoGrain", Grain::NoGrain),
        ("Second", Grain::Second),
        ("Minute", Grain::Minute),
        ("Hour", Grain::Hour),
        ("Day", Grain::Day),
        ("Week", Grain::Week),
        ("Month", Grain::Month),
        ("Quarter", Grain::Quarter),
        ("Year", Grain::Year),
    ];
    for (name, g) in GRAINS {
        if s.starts_with(name) {
            return Ok((*g, &s[name.len()..]));
        }
    }
    Err(format!("grain: {s}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNIP: &str = r#"
allExamples = concat
  [ examples (datetime (2013, 2, 12, 4, 30, 0) Second)
             [ "now"
             , "right now"
             ]
  , examples (datetimeHoliday (2013, 11, 28, 0, 0, 0) Day "Thanksgiving Day")
             [ "thanksgiving"
             ]
  , examples (datetimeInterval ((2013, 2, 12, 18, 0, 0), (2013, 2, 13, 0, 0, 0)) Hour)
             [ "tonight"
             ]
  , examples (datetimeOpenInterval Before (2013, 2, 12, 14, 0, 0) Hour)
             [ "until 2pm"
             ]
  ]
"#;

    #[test]
    fn parses_all_constructors() {
        let xs = parse_concat_binding(SNIP, "allExamples").unwrap();
        assert_eq!(xs.len(), 5);
        assert_eq!(xs[0].text, "now");
        assert_eq!(xs[1].text, "right now");
        assert!(matches!(xs[2].expected, Expected::Datetime { holiday: Some(ref h), .. } if h == "Thanksgiving Day"));
        assert!(matches!(xs[3].expected, Expected::Interval { .. }));
        assert!(matches!(xs[4].expected, Expected::Open { dir: IntervalDirection::Before, .. }));
    }

    #[test]
    fn escaped_quote() {
        let src = r#"
allExamples = concat
  [ examples (datetime (2013, 2, 12, 4, 30, 1) Second)
             [ "in 1\""
             ]
  ]
"#;
        let xs = parse_concat_binding(src, "allExamples").unwrap();
        assert_eq!(xs[0].text, "in 1\"");
    }

    #[test]
    fn parses_every_vendor_file_used_for_training() {
        let root = vendor_time_root();
        let mut total = 0usize;
        for loc in exported_locales() {
            let c = load_corpus_from(&root, loc).unwrap_or_else(|e| panic!("{loc}: {e}"));
            total += c.examples.len();
            if loc.lang == Lang::EN && loc.region.is_none() {
                assert!(c.examples.len() > 400, "EN_XX too small: {}", c.examples.len());
            }
        }
        assert!(total > 1000, "too few examples: {total}");
    }
}
