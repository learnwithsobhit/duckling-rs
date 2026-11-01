//! `Duckling.Numeral.FA.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

/// `numeralToStringMap` / `parseIntAsText`.
fn parse_int_as_text(s: &str) -> String {
    s.chars()
        .filter_map(|c| match c {
            '۰' => Some('0'),
            '۱' => Some('1'),
            '۲' => Some('2'),
            '۳' => Some('3'),
            '۴' => Some('4'),
            '۵' => Some('5'),
            '۶' => Some('6'),
            '۷' => Some('7'),
            '۸' => Some('8'),
            '۹' => Some('9'),
            _ => None,
        })
        .collect()
}

/// `parseIntegerFromText`.
fn parse_integer_from_text(s: &str) -> Option<i64> {
    parse_int(&parse_int_as_text(s))
}

/// `zeroNineteenMap`.
static ZERO_NINETEEN_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("صفر", 0),
        ("یک", 1),
        ("دو", 2),
        ("سه", 3),
        ("چهار", 4),
        ("پنج", 5),
        ("شش", 6),
        ("شیش", 6),
        ("هفت", 7),
        ("هشت", 8),
        ("نه", 9),
        ("ده", 10),
        ("یازده", 11),
        ("دوازده", 12),
        ("سیزده", 13),
        ("چهارده", 14),
        ("پانزده", 15),
        ("پونزده", 15),
        ("شانزده", 16),
        ("شونزده", 16),
        ("هفده", 17),
        ("هیفده", 17),
        ("هجده", 18),
        ("هیجده", 18),
        ("نوزده", 19),
    ]
    .into_iter()
    .collect()
});

/// `tensMap`.
static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("بیست", 20),
        ("سی", 30),
        ("چهل", 40),
        ("پنجاه", 50),
        ("شصت", 60),
        ("هفتاد", 70),
        ("هشتاد", 80),
        ("نود", 90),
        ("صد", 100),
        ("دویست", 200),
        ("سیصد", 300),
        ("سی صد", 300),
        ("چهارصد", 400),
        ("چهار صد", 400),
        ("پانصد", 500),
        ("پونصد", 500),
        ("شیشصد", 600),
        ("شیش صد", 600),
        ("ششصد", 600),
        ("شش صد", 600),
        ("هفتصد", 700),
        ("هفت صد", 700),
        ("هشتصد", 800),
        ("هشت صد", 800),
        ("نهصد", 900),
        ("نه صد", 900),
    ]
    .into_iter()
    .collect()
});

fn nv(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

fn grain_of(t: &Token) -> Option<i64> {
    match t {
        Token::Numeral(n) => n.grain,
        _ => None,
    }
}

fn add_if_grain(ts: &[&Token], a: usize, b: usize) -> Option<Token> {
    let val1 = nv(ts.get(a)?)?;
    let val2 = nv(ts.get(b)?)?;
    let g = grain_of(ts.get(a)?)?;
    if 10f64.powf(g as f64) > val2 {
        double(val1 + val2)
    } else {
        None
    }
}

fn rule_to_nineteen() -> Rule {
    rule(
        "integer (0..19)",
        vec![regex(
            "(صفر|یک|سه|چهارده|چهار|پنج|شی?ش|هفت|هشت|نه|یازده|دوازده|سیزده|پ(ا|و)نزده|ش(ا|و)نزده|هی?فده|هی?جده|نوزده|ده|دو)",
        )],
        |ts| {
            let x = to_lower(groups(ts)?.first()?);
            ZERO_NINETEEN_MAP.get(x.as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_tens() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex(
            "(دویست|(سی|چهار|پان|پون|شی?ش|هفت|هشت|نه)? ?صد|بیست|سی|چهل|پنجاه|شصت|هفتاد|هشتاد|نود)",
        )],
        |ts| TENS_MAP.get(to_lower(groups(ts)?.first()?).as_str()).and_then(|v| integer(*v)),
    )
}

fn rule_powers_of_ten() -> Rule {
    rule("powers of tens", vec![regex("(هزار|میلیون|ملیون|میلیارد)")], |ts| {
        match to_lower(groups(ts)?.first()?).as_str() {
            "هزار" => double(1e3).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t)),
            "میلیون" => {
                double(1e6).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t))
            }
            "ملیون" => {
                double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t))
            }
            "میلیارد" => {
                double(1e9).and_then(|t| with_grain(9, &t)).and_then(|t| with_multipliable(&t))
            }
            _ => None,
        }
    })
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer 21..99",
        vec![
            one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
            regex("و"),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| double(nv(ts.first()?)? + nv(ts.get(2)?)?),
    )
}

fn rule_composite_hundred() -> Rule {
    rule(
        "integer 21..99",
        vec![
            one_of(vec![100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0]),
            regex("و"),
            pred(number_between(1.0, 100.0)),
        ],
        |ts| double(nv(ts.first()?)? + nv(ts.get(2)?)?),
    )
}

fn rule_sum() -> Rule {
    rule(
        "intersect 2 numbers",
        vec![
            pred(|t| has_grain(t) && is_positive(t)),
            pred(|t| !is_multipliable(t) && is_positive(t)),
        ],
        |ts| add_if_grain(ts, 0, 1),
    )
}

fn rule_sum_and() -> Rule {
    rule(
        "intersect 2 numbers (with and)",
        vec![
            pred(|t| has_grain(t) && is_positive(t)),
            regex("و"),
            pred(|t| !is_multipliable(t) && is_positive(t)),
        ],
        |ts| add_if_grain(ts, 0, 2),
    )
}

fn rule_multiply() -> Rule {
    rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
        multiply(ts.first()?, ts.get(1)?)
    })
}

fn rule_integer_numeric() -> Rule {
    rule("Persian integer numeric", vec![regex("([۰-۹]{1,18})")], |ts| {
        parse_integer_from_text(groups(ts)?.first()?).and_then(integer)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_integer_numeric(),
        rule_to_nineteen(),
        rule_tens(),
        rule_powers_of_ten(),
        rule_composite_tens(),
        rule_composite_hundred(),
        rule_sum(),
        rule_sum_and(),
        rule_multiply(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
