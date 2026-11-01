//! `Duckling.Numeral.VI.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

static POWERS_OF_TEN: Lazy<HashMap<&'static str, (f64, i64)>> = Lazy::new(|| {
    [
        ("chục", (1e1, 1)),
        ("trăm", (1e2, 2)),
        ("nghìn", (1e3, 3)),
        ("ngàn", (1e3, 3)),
        ("triệu", (1e6, 6)),
        ("tỷ", (1e9, 9)),
        ("tỉ", (1e9, 9)),
    ]
    .into_iter()
    .collect()
});

fn rule_powers_of_ten() -> Rule {
    rule("powers of tens", vec![regex("(chục|trăm|nghìn|ngàn|triệu|t(ỷ|ỉ))")], |ts| {
        let g = groups(ts)?;
        let (value, grain) = *POWERS_OF_TEN.get(to_lower(g.first()?).as_str())?;
        double(value).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
    })
}

fn rule_numerals_prefix_with_m() -> Rule {
    rule(
        "numbers prefix with -, âm",
        vec![regex(r"\-|âm"), pred(is_positive)],
        |ts| match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * -1.0),
            _ => None,
        },
    )
}

fn rule_numerals2() -> Rule {
    rule(
        "numbers 25 35 45 55 65 75 85 95",
        vec![one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]), regex("lăm")],
        |ts| match ts.first()? {
            Token::Numeral(nd) => double(nd.value + 5.0),
            _ => None,
        },
    )
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule("decimal with thousands separator", vec![regex(r"(\d+(\.\d\d\d)+,\d+)")], |ts| {
        let g = groups(ts)?;
        parse_double(&g.first()?.replace('.', "")).and_then(double)
    })
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex(r"(\d*,\d+)")], |ts| {
        parse_decimal(false, groups(ts)?.first()?)
    })
}

fn rule_integer3() -> Rule {
    rule(
        "integer 21..99",
        vec![
            one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
            _ => None,
        },
    )
}

fn rule_numeral_dot() -> Rule {
    rule(
        "number dot 1 9",
        vec![dimension(Dim::Numeral), regex("chấm|phẩy"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(nd1), Token::Numeral(nd2)) => {
                double(nd1.value + decimals_to_double(nd2.value))
            }
            _ => None,
        },
    )
}

fn rule_intersect() -> Rule {
    rule(
        "intersect",
        vec![pred(has_grain), pred(|t| !is_multipliable(t) && is_positive(t))],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(a), Token::Numeral(b)) => {
                let g = a.grain?;
                if 10f64.powf(g as f64) > b.value {
                    double(a.value + b.value)
                } else {
                    None
                }
            }
            _ => None,
        },
    )
}

fn rule_multiply() -> Rule {
    rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
        multiply(ts.first()?, ts.get(1)?)
    })
}

fn rule_numerals_suffixes_kmg() -> Rule {
    rule(
        "numbers suffixes (K, M, G)",
        vec![dimension(Dim::Numeral), regex(r"([kmg])(?=[\W\$€]|$)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(nd), Token::RegexMatch(g)) => match to_lower(g.0.first()?).as_str() {
                "k" => double(nd.value * 1e3),
                "m" => double(nd.value * 1e6),
                "g" => double(nd.value * 1e9),
                _ => None,
            },
            _ => None,
        },
    )
}

static INTEGER_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("không", 0),
        ("một", 1),
        ("linh một", 1),
        ("lẻ một", 1),
        ("hai", 2),
        ("lẻ hai", 2),
        ("linh hai", 2),
        ("ba", 3),
        ("lẻ", 3),
        ("linh ba", 3),
        ("lẻ bốn", 4),
        ("linh bốn", 4),
        ("bốn", 4),
        ("năm", 5),
        ("lẻ năm", 5),
        ("linh năm", 5),
        ("linh sáu", 6),
        ("sáu", 6),
        ("lẻ sáu", 6),
        ("linh bảy", 7),
        ("lẻ bảy", 7),
        ("bảy", 7),
        ("lẻ tám", 8),
        ("linh tám", 8),
        ("tám", 8),
        ("lẻ chín", 9),
        ("chín", 9),
        ("linh chín", 9),
        ("linh mười", 10),
        ("mười", 10),
        ("lẻ mười", 10),
        ("mười một", 11),
        ("mười hai", 12),
        ("mười ba", 13),
        ("mười bốn", 14),
        ("mười lăm", 15),
        ("mười sáu", 16),
        ("mười bảy", 17),
        ("mười tám", 18),
        ("mười chín", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_integer() -> Rule {
    rule(
        "integer (0..19)",
        vec![regex("(không|một|linh một|lẻ một|hai|linh hai|lẻ hai|ba|linh ba|lẻ ba|bốn|linh bốn|lẻ bốn|năm|linh năm|lẻ năm|sáu|lẻ sáu|linh sáu|bảy|lẻ bảy|linh bảy|tám|linh tám|lẻ tám|chín|linh chín|lẻ chín|mười một|mười hai|mười ba|mười bốn|mười lăm|mười sáu|mười bảy|mười tám|mười chín|mười|linh mười)")],
        |ts| {
            let g = groups(ts)?;
            INTEGER_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("hai mươi", 20),
        ("ba mươi", 30),
        ("bốn mươi", 40),
        ("năm mươi", 50),
        ("sáu mươi", 60),
        ("bảy mươi", 70),
        ("tám mươi", 80),
        ("chín mươi", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_integer2() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(hai mươi|ba mươi|bốn mươi|năm mươi|sáu mươi|bảy mươi|tám mươi|chín mươi)")],
        |ts| {
            let g = groups(ts)?;
            TENS_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_numerals() -> Rule {
    rule(
        "numbers 21 31 41 51 61 71 81 91",
        vec![one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]), regex("mốt")],
        |ts| match ts.first()? {
            Token::Numeral(nd) => double(nd.value + 1.0),
            _ => None,
        },
    )
}

fn rule_t() -> Rule {
    rule("tá", vec![regex("tá")], |_| {
        integer(12).and_then(|t| with_grain(1, &t)).and_then(|t| with_multipliable(&t))
    })
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule("integer with thousands separator ,", vec![regex(r"(\d{1,3}(\.\d\d\d){1,5})")], |ts| {
        let fmt = groups(ts)?.first()?.replace('.', "");
        parse_double(&fmt).and_then(double)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_integer(),
        rule_integer2(),
        rule_integer3(),
        rule_integer_with_thousands_separator(),
        rule_intersect(),
        rule_multiply(),
        rule_numeral_dot(),
        rule_numerals(),
        rule_numerals2(),
        rule_numerals_prefix_with_m(),
        rule_numerals_suffixes_kmg(),
        rule_powers_of_ten(),
        rule_t(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
