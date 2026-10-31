//! Minimal JSON value with an encoder that is byte-compatible with Haskell's
//! `aeson` (2.x) `encode`:
//!
//! * object keys are emitted in sorted order (aeson 2 `KeyMap` is a `Map`),
//! * numbers follow `Data.Scientific`'s `Generic` format (see [`encode_f64`]),
//! * strings escape only `"`, `\`, control characters and DEL.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    /// Haskell `Int` / `Integer`.
    Int(i64),
    /// Haskell `Double`.
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(BTreeMap<String, Json>),
}

impl Json {
    pub fn str(s: impl Into<String>) -> Json {
        Json::Str(s.into())
    }

    /// Builds an object from `(key, value)` pairs.
    pub fn obj<K: Into<String>>(pairs: impl IntoIterator<Item = (K, Json)>) -> Json {
        Json::Obj(pairs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(m) => m.get(key),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Int(i) => Some(*i as f64),
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Compact encoding identical to aeson's `encode`.
    pub fn encode(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    pub fn write(&self, out: &mut String) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(true) => out.push_str("true"),
            Json::Bool(false) => out.push_str("false"),
            Json::Int(i) => out.push_str(&i.to_string()),
            Json::Num(n) => out.push_str(&encode_f64(*n)),
            Json::Str(s) => write_string(s, out),
            Json::Arr(a) => {
                out.push('[');
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    v.write(out);
                }
                out.push(']');
            }
            Json::Obj(m) => {
                out.push('{');
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_string(k, out);
                    out.push(':');
                    v.write(out);
                }
                out.push('}');
            }
        }
    }

    /// Converts from `serde_json` (used by tests and fixtures).
    pub fn from_serde(v: &serde_json::Value) -> Json {
        match v {
            serde_json::Value::Null => Json::Null,
            serde_json::Value::Bool(b) => Json::Bool(*b),
            serde_json::Value::Number(n) => match n.as_i64() {
                Some(i) => Json::Int(i),
                None => Json::Num(n.as_f64().unwrap_or(f64::NAN)),
            },
            serde_json::Value::String(s) => Json::Str(s.clone()),
            serde_json::Value::Array(a) => Json::Arr(a.iter().map(Json::from_serde).collect()),
            serde_json::Value::Object(o) => {
                Json::Obj(o.iter().map(|(k, v)| (k.clone(), Json::from_serde(v))).collect())
            }
        }
    }
}

impl From<bool> for Json {
    fn from(b: bool) -> Json {
        Json::Bool(b)
    }
}
impl From<i64> for Json {
    fn from(i: i64) -> Json {
        Json::Int(i)
    }
}
impl From<f64> for Json {
    fn from(n: f64) -> Json {
        Json::Num(n)
    }
}
impl From<&str> for Json {
    fn from(s: &str) -> Json {
        Json::Str(s.to_string())
    }
}
impl From<String> for Json {
    fn from(s: String) -> Json {
        Json::Str(s)
    }
}

fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Shortest round-trip decimal digits of a positive finite `x`:
/// returns `(digits, e)` with `x = 0.d1d2...dn * 10^e` (Haskell `floatToDigits 10`).
fn float_to_digits(x: f64) -> (Vec<u8>, i32) {
    debug_assert!(x > 0.0 && x.is_finite());
    let s = format!("{:e}", x); // e.g. "1.2345e-3" (shortest round-trip)
    let (mant, exp) = s.split_once('e').expect("exp format");
    let exp: i32 = exp.parse().expect("exp");
    let digits: Vec<u8> = mant.bytes().filter(|b| b.is_ascii_digit()).map(|b| b - b'0').collect();
    (digits, exp + 1)
}

/// Encodes a `Double` exactly like aeson (via `Data.Scientific`).
///
/// * `NaN` and infinities become `null`.
/// * Numbers whose exponent is non-negative and at most 1024 print as integers.
/// * Others use the `Generic` format: fixed notation if `0 <= e <= 7`
///   (where `x = 0.ds * 10^e`), exponent notation otherwise.
pub fn encode_f64(x: f64) -> String {
    if x.is_nan() || x.is_infinite() {
        return "null".to_string();
    }
    if x == 0.0 {
        return "0".to_string();
    }
    let neg = x < 0.0;
    let (ds, e) = float_to_digits(x.abs());
    // Scientific coefficient/exponent: value = coefficient * 10^base10_exp.
    let n = ds.len() as i32;
    let base10_exp = e - n;
    let digit_str: String = ds.iter().map(|d| (b'0' + d) as char).collect();
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    if (0..=1024).contains(&base10_exp) {
        out.push_str(&digit_str);
        for _ in 0..base10_exp {
            out.push('0');
        }
        return out;
    }
    // formatScientificBuilder Generic Nothing
    if !(0..=7).contains(&e) {
        // Exponent
        if ds.len() == 1 {
            out.push_str(&format!("{}.0e{}", digit_str, e - 1));
        } else {
            out.push_str(&format!("{}.{}e{}", &digit_str[..1], &digit_str[1..], e - 1));
        }
    } else if e <= 0 {
        out.push_str("0.");
        for _ in 0..(-e) {
            out.push('0');
        }
        out.push_str(&digit_str);
    } else {
        let e = e as usize;
        let mut int_part = String::new();
        let mut frac_part = String::new();
        for (i, c) in digit_str.chars().enumerate() {
            if i < e {
                int_part.push(c);
            } else {
                frac_part.push(c);
            }
        }
        while int_part.len() < e {
            int_part.push('0');
        }
        out.push_str(if int_part.is_empty() { "0" } else { &int_part });
        out.push('.');
        out.push_str(if frac_part.is_empty() { "0" } else { &frac_part });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_match_aeson() {
        assert_eq!(encode_f64(3.0), "3");
        assert_eq!(encode_f64(-3.0), "-3");
        assert_eq!(encode_f64(0.0), "0");
        assert_eq!(encode_f64(0.5), "0.5");
        assert_eq!(encode_f64(0.03), "3.0e-2");
        assert_eq!(encode_f64(0.05), "5.0e-2");
        assert_eq!(encode_f64(0.001), "1.0e-3");
        assert_eq!(encode_f64(12.5), "12.5");
        assert_eq!(encode_f64(10.99), "10.99");
        assert_eq!(encode_f64(1e21), "1000000000000000000000");
        assert_eq!(encode_f64(123456789.5), "1.234567895e8");
        assert_eq!(encode_f64(0.125), "0.125");
        assert_eq!(encode_f64(1.5e-7), "1.5e-7");
        assert_eq!(encode_f64(f64::NAN), "null");
    }

    #[test]
    fn object_keys_sorted() {
        let o = Json::obj([("value", Json::Int(1)), ("type", Json::str("value"))]);
        assert_eq!(o.encode(), r#"{"type":"value","value":1}"#);
    }
}
