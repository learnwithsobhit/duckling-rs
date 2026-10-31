//! Small helpers shared by dimension modules (ports of Haskell `Data.Text` / `Data.Char` behaviour).

use crate::types::{hash_f64, Token};
use std::hash::Hasher;

/// `Data.Text.toLower`: per-character full lowercase mapping without context
/// (e.g. final sigma is not special-cased).
pub fn to_lower(s: &str) -> String {
    s.chars().flat_map(|c| c.to_lowercase()).collect()
}

pub fn to_upper(s: &str) -> String {
    s.chars().flat_map(|c| c.to_uppercase()).collect()
}

pub fn hash_opt_f64<H: Hasher>(x: Option<f64>, h: &mut H) {
    use std::hash::Hash;
    match x {
        None => 0u8.hash(h),
        Some(v) => {
            1u8.hash(h);
            hash_f64(v, h);
        }
    }
}

/// Type of `Token -> Maybe Token` used with `>>=` chains in rule productions.
pub type TokOpt = Option<Token>;

/// `Atto.parseOnly (Atto.signed Atto.decimal)` followed by `fromIntegral` to `Int` (wraps).
pub fn parse_integer(s: &str) -> Option<i128> {
    let b = s.as_bytes();
    let (neg, digits) = match b.first()? {
        b'+' => (false, &b[1..]),
        b'-' => (true, &b[1..]),
        _ => (false, b),
    };
    if digits.is_empty() || !digits.iter().all(|c| c.is_ascii_digit()) {
        return None;
    }
    // Saturate on absurdly long inputs; Haskell uses arbitrary precision then wraps to Int.
    let mut v: i128 = 0;
    for c in digits {
        v = v.wrapping_mul(10).wrapping_add((c - b'0') as i128);
    }
    Some(if neg { -v } else { v })
}

/// `parseInt`: Integer parsed then truncated to a 64-bit `Int`.
pub fn parse_int(s: &str) -> Option<i64> {
    parse_integer(s).map(|v| v as i64)
}

/// Mirrors `Atto.parseOnly Atto.double`: `[+-]?\d+(\.\d+)?([eE][+-]?\d+)?` fully consumed.
pub fn atto_double(s: &str) -> Option<f64> {
    let b = s.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let ds = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == ds {
        return None;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let fs = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == fs {
            return None;
        }
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        let es = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > es {
            i = j;
        }
    }
    if i != b.len() {
        return None;
    }
    s.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles() {
        assert_eq!(atto_double("1.5"), Some(1.5));
        assert_eq!(atto_double("5."), None);
        assert_eq!(atto_double(".5"), None);
        assert_eq!(atto_double("-2e3"), Some(-2000.0));
        assert_eq!(atto_double("1e"), None);
        assert_eq!(parse_int("+12"), Some(12));
        assert_eq!(parse_int("a"), None);
    }
}
