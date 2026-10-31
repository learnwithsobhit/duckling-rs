//! `Duckling.Numeral.Types` and `Duckling.Numeral.Helpers`.

use std::hash::{Hash, Hasher};

use crate::dims::util::{atto_double, parse_int};
use crate::json::Json;
use crate::types::{hash_f64, pred, PatternItem, Token};

pub mod en;
pub mod rules;

#[derive(Clone, Debug, PartialEq)]
pub struct NumeralData {
    pub value: f64,
    pub grain: Option<i64>,
    pub multipliable: bool,
    pub ok_for_any_time: bool,
}

impl Eq for NumeralData {}

impl Hash for NumeralData {
    fn hash<H: Hasher>(&self, h: &mut H) {
        hash_f64(self.value, h);
        self.grain.hash(h);
        self.multipliable.hash(h);
        self.ok_for_any_time.hash(h);
    }
}

impl NumeralData {
    pub fn resolve(&self) -> Json {
        Json::obj([("type", Json::str("value")), ("value", Json::Num(self.value))])
    }
}

/// `getIntValue`: the integer value of `x` if it has no fractional part (and fits an `Int`).
pub fn get_int_value(x: f64) -> Option<i64> {
    let t = x.trunc();
    if x - t == 0.0 && t.is_finite() {
        // properFraction to Int wraps; stay within range where that is exact.
        Some(t as i64)
    } else {
        None
    }
}

pub fn is_integer(x: f64) -> bool {
    get_int_value(x).is_some()
}

pub fn is_multiple(base: i64, x: f64) -> bool {
    get_int_value(x / base as f64).is_some()
}

pub fn is_integer_between(x: f64, low: i64, high: i64) -> bool {
    match get_int_value(x) {
        Some(i) => low <= i && i <= high,
        None => false,
    }
}

pub fn parse_double(s: &str) -> Option<f64> {
    if s.starts_with('.') {
        atto_double(&format!("0{}", s))
    } else {
        atto_double(s)
    }
}

/// `decimalsToDouble`.
pub fn decimals_to_double(x: f64) -> f64 {
    let mut m = 1.0f64;
    for _ in 0..10 {
        if x - m < 0.0 {
            return x / m;
        }
        m *= 10.0;
    }
    0.0
}

/// `diffIntegerDigits`.
pub fn diff_integer_digits(a: f64, b: f64) -> i64 {
    fn digits_of(x: f64) -> i64 {
        let mut n = x.abs().floor() as i64;
        let mut d = 0;
        while n != 0 {
            d += 1;
            n /= 10;
        }
        d
    }
    digits_of(a) - digits_of(b)
}

/// `double`.
pub fn double(x: f64) -> Option<Token> {
    Some(Token::Numeral(NumeralData {
        value: x,
        grain: None,
        multipliable: false,
        ok_for_any_time: true,
    }))
}

/// `integer`.
pub fn integer(x: i64) -> Option<Token> {
    double(x as f64)
}

pub fn as_numeral(t: &Token) -> Option<&NumeralData> {
    match t {
        Token::Numeral(n) => Some(n),
        _ => None,
    }
}

pub fn with_multipliable(t: &Token) -> Option<Token> {
    match t {
        Token::Numeral(n) => Some(Token::Numeral(NumeralData { multipliable: true, ..n.clone() })),
        _ => None,
    }
}

pub fn with_grain(g: i64, t: &Token) -> Option<Token> {
    match t {
        Token::Numeral(n) => Some(Token::Numeral(NumeralData { grain: Some(g), ..n.clone() })),
        _ => None,
    }
}

pub fn not_ok_for_any_time(t: &Token) -> Option<Token> {
    match t {
        Token::Numeral(n) => {
            Some(Token::Numeral(NumeralData { ok_for_any_time: false, ..n.clone() }))
        }
        _ => None,
    }
}

/// `multiply`.
pub fn multiply(a: &Token, b: &Token) -> Option<Token> {
    let (n1, n2) = (as_numeral(a)?, as_numeral(b)?);
    let (v1, v2) = (n1.value, n2.value);
    match n2.grain {
        None => double(v1 * v2),
        Some(g) => {
            if v2 > v1 {
                double(v1 * v2).and_then(|t| with_grain(g, &t))
            } else {
                None
            }
        }
    }
}

/// `divide`.
pub fn divide(a: &Token, b: &Token) -> Option<Token> {
    let (n1, n2) = (as_numeral(a)?, as_numeral(b)?);
    let x = n1.value / n2.value;
    if x.is_infinite() || x.is_nan() {
        None
    } else {
        double(x)
    }
}

/// `parseDecimal`.
pub fn parse_decimal(is_dot: bool, m: &str) -> Option<Token> {
    if is_dot {
        parse_double(m).and_then(double)
    } else {
        parse_double(&m.replace(',', ".")).and_then(double)
    }
}

/// Convenience: `parseInt` as used by `integer <$> parseInt`.
pub fn parse_int_token(m: &str) -> Option<Token> {
    parse_int(m).and_then(integer)
}

// ----- predicates ------------------------------------------------------------------------

/// `numberWith f pred`.
pub fn number_with(
    f: impl Fn(&NumeralData) -> f64 + Send + Sync + 'static,
    p: impl Fn(f64) -> bool + Send + Sync + 'static,
) -> PatternItem {
    pred(move |t| match t {
        Token::Numeral(n) => p(f(n)),
        _ => false,
    })
}

/// `numberBetween low up` (a `Predicate`, i.e. `Token -> Bool`).
pub fn number_between(low: f64, up: f64) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| match t {
        Token::Numeral(NumeralData { value, multipliable: false, .. }) => low <= *value && *value < up,
        _ => false,
    }
}

pub fn is_natural(t: &Token) -> bool {
    matches!(t, Token::Numeral(n) if is_integer(n.value) && n.value > 0.0)
}

pub fn is_positive(t: &Token) -> bool {
    matches!(t, Token::Numeral(n) if n.value >= 0.0)
}

pub fn is_multipliable(t: &Token) -> bool {
    matches!(t, Token::Numeral(n) if n.multipliable)
}

pub fn has_grain(t: &Token) -> bool {
    matches!(t, Token::Numeral(NumeralData { grain: Some(g), .. }) if *g > 1)
}

/// `oneOf`.
pub fn one_of(vs: Vec<f64>) -> PatternItem {
    pred(move |t| matches!(t, Token::Numeral(n) if vs.contains(&n.value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimals() {
        assert_eq!(decimals_to_double(77.0), 0.77);
        assert_eq!(decimals_to_double(5.0), 0.5);
        assert_eq!(decimals_to_double(0.0), 0.0 / 1.0);
    }

    #[test]
    fn int_value() {
        assert_eq!(get_int_value(3.0), Some(3));
        assert_eq!(get_int_value(3.5), None);
        assert_eq!(get_int_value(-2.0), Some(-2));
    }
}
