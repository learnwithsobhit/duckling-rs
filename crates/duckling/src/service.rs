//! The request handling of upstream's example server (`exe/ExampleMain.hs`), independent of any
//! HTTP stack so that the native server and the WASM package share it exactly.

use std::collections::HashMap;

use crate::api::{parse, supported_dimensions};
use crate::dims::time::calendar::add_seconds;
use crate::json::Json;
use crate::locale::{Lang, Locale, Region};
use crate::time_zone::{DucklingTime, TimeZone, TimeZoneSeries, UtcTime};
use crate::types::{Context, Dim, Options};

/// The form parameters of `POST /parse`.
#[derive(Clone, Debug, Default)]
pub struct ParseParams {
    pub text: Option<String>,
    pub lang: Option<String>,
    pub locale: Option<String>,
    pub dims: Option<String>,
    pub tz: Option<String>,
    pub reftime: Option<String>,
    pub latent: Option<String>,
}

/// A failed request: HTTP status plus body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpError {
    pub status: u16,
    pub body: String,
}

const DEFAULT_TIMEZONE: &str = "America/Los_Angeles";

fn utc_series() -> TimeZoneSeries {
    TimeZoneSeries {
        default_zone: TimeZone { offset_minutes: 0, summer_only: false, name: "UTC".into() },
        transitions: Vec::new(),
    }
}

/// `makeReftime`: `utcTime` seen as wall-clock time in zone `tz` ("we work as if we were in UTC").
pub fn make_reftime(tzs: &HashMap<String, TimeZoneSeries>, tz: &str, utc: UtcTime) -> DucklingTime {
    let series = tzs.get(tz).cloned().unwrap_or_else(utc_series);
    let offset = series.zone_at(utc).offset_minutes as i128;
    DucklingTime { utc: add_seconds(utc, offset * 60), series }
}

/// Haskell's `readMaybe :: Maybe Lang` after `Text.toUpper`.
fn parse_lang(l: &Option<String>) -> Lang {
    l.as_deref().and_then(|s| Lang::parse(s.trim().to_uppercase().as_str())).unwrap_or(Lang::EN)
}

fn parse_locale(x: &str) -> Locale {
    let upper = x.to_uppercase();
    let chunks: Vec<&str> = upper.split('_').collect();
    let (lang, region) = match chunks.as_slice() {
        [a, b] => (Lang::parse(a.trim()), Region::parse(b.trim())),
        _ => (None, None),
    };
    match lang {
        Some(l) => Locale::new(l, region),
        None => Locale::new(Lang::EN, None),
    }
}

fn parse_latent(x: &Option<String>) -> bool {
    match x.as_deref() {
        None => false,
        Some(s) => {
            let s = s.trim();
            // `Text.toTitle` then `readMaybe @Bool`.
            s.eq_ignore_ascii_case("true")
        }
    }
}

/// The `dims` form value: strip surrounding quotes and all backslashes, decode a JSON array of
/// strings and keep the known dimension names. `None` when it cannot be decoded.
fn parse_dims(raw: &str) -> Option<Vec<Dim>> {
    let mut s = raw;
    if let Some(r) = s.strip_prefix('"') {
        s = r;
    }
    if let Some(r) = s.strip_suffix('"') {
        s = r;
    }
    let cleaned: String = s.chars().filter(|&c| c != '\\').collect();
    let v: serde_json::Value = serde_json::from_str(&cleaned).ok()?;
    let arr = v.as_array()?;
    let mut names = Vec::new();
    for x in arr {
        names.push(x.as_str()?);
    }
    Some(names.into_iter().filter_map(Dim::from_name).collect())
}

/// `parseHandler`. `now_unix_ms` supplies the current time when no `reftime` is given.
pub fn handle_parse(
    p: &ParseParams,
    tzs: &HashMap<String, TimeZoneSeries>,
    now_unix_ms: i64,
) -> Result<String, HttpError> {
    let text = match &p.text {
        Some(t) => t,
        None => {
            return Err(HttpError { status: 422, body: "Need a 'text' parameter to parse".into() });
        }
    };
    let tz = p.tz.as_deref().unwrap_or(DEFAULT_TIMEZONE);
    // An unparsable `reftime` is an `error` thunk upstream: it only blows up (HTTP 500) if a
    // Time entity actually needs it, so parse with a placeholder and check afterwards.
    let (reftime_ms, reftime_invalid) = match &p.reftime {
        Some(r) => match r.trim().parse::<i64>() {
            Ok(ms) => (ms, false),
            Err(_) => (0, true),
        },
        None => (now_unix_ms, false),
    };
    let reference_time = make_reftime(tzs, tz, UtcTime::from_unix_millis(reftime_ms));
    let lang = parse_lang(&p.lang);
    let locale = match &p.locale {
        Some(l) => parse_locale(l),
        None => Locale::new(lang, None),
    };
    let options = Options { with_latent: parse_latent(&p.latent) };
    let dims: Vec<Dim> = p
        .dims
        .as_deref()
        .and_then(parse_dims)
        .unwrap_or_else(|| crate::api::all_dimensions(lang));
    let context = Context { reference_time, locale };
    let (entities, used_reftime) =
        crate::dims::time::types::probe_reference_time_use(|| parse(text, &context, &options, &dims));
    if reftime_invalid && used_reftime {
        return Err(HttpError { status: 500, body: "Please use milliseconds since epoch for reftime".into() });
    }
    Ok(Json::Arr(entities.iter().map(|e| e.to_json()).collect()).encode())
}

/// `targetsHandler`: language (lower case) to dimension names.
pub fn handle_targets() -> String {
    let mut langs: Vec<(String, Vec<Json>)> = Lang::ALL
        .iter()
        .map(|&l| (l.as_str().to_lowercase(), supported_dimensions(l).into_iter().map(|d| Json::str(d.name())).collect()))
        .collect();
    langs.sort_by(|a, b| a.0.cmp(&b.0));
    Json::obj(langs.into_iter().map(|(k, v)| (k, Json::Arr(v)))).encode()
}

impl ParseParams {
    /// Build parameters from a JSON object with the same field names as the HTTP form
    /// (`text`, `lang`, `locale`, `dims`, `tz`, `reftime`, `latent`). Besides strings, `dims` may
    /// be an array of names, `reftime` a number (milliseconds since the epoch) and `latent` a
    /// boolean, which is how JavaScript callers naturally pass them.
    pub fn from_json(v: &serde_json::Value) -> Result<ParseParams, String> {
        let obj = v.as_object().ok_or("request must be a JSON object")?;
        fn text(v: &serde_json::Value) -> Option<String> {
            match v {
                serde_json::Value::Null => None,
                serde_json::Value::String(s) => Some(s.clone()),
                other => Some(other.to_string()),
            }
        }
        let get = |k: &str| obj.get(k).and_then(text);
        Ok(ParseParams {
            text: get("text"),
            lang: get("lang"),
            locale: get("locale"),
            dims: get("dims"),
            tz: get("tz"),
            reftime: get("reftime"),
            latent: get("latent"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_params_round_trip() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"text":"in 2 days","dims":["time"],"reftime":1700000000000,"latent":true,"tz":"UTC"}"#,
        )
        .unwrap();
        let p = ParseParams::from_json(&v).unwrap();
        assert_eq!(p.dims.as_deref(), Some(r#"["time"]"#));
        assert_eq!(p.reftime.as_deref(), Some("1700000000000"));
        assert_eq!(p.latent.as_deref(), Some("true"));
        let out = handle_parse(&p, &HashMap::new(), 0).unwrap();
        assert!(out.contains(r#""value":"2023-11-16T22:00:00.000+00:00""#), "{out}");
    }

    #[test]
    fn dims_decoding_matches_upstream_quirks() {
        assert_eq!(parse_dims(r#""[\"time\",\"number\"]""#), Some(vec![Dim::Time, Dim::Numeral]));
        assert_eq!(parse_dims("[\"bogus\"]"), Some(vec![]));
        assert_eq!(parse_dims("not json"), None);
        assert_eq!(parse_dims("[1]"), None);
    }

    #[test]
    fn locale_parsing() {
        assert_eq!(parse_locale("en_gb"), Locale::new(Lang::EN, Some(Region::GB)));
        assert_eq!(parse_locale("fr"), Locale::new(Lang::EN, None));
        assert_eq!(parse_locale("de_zz"), Locale::new(Lang::DE, None));
    }
}
