//! `Duckling.Numeral.SK.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn not_multipliable_positive(t: &Token) -> bool {
    !is_multipliable(t) && is_positive(t)
}

fn intersect_sum(ts: &[&Token]) -> Option<Token> {
    match (ts.first()?, ts.get(1)?) {
        (Token::Numeral(a), Token::Numeral(b)) => {
            let g = a.grain?;
            if 10f64.powf(g as f64) > b.value {
                double(a.value + b.value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn intersect_sum_and(ts: &[&Token]) -> Option<Token> {
    match (ts.first()?, ts.get(2)?) {
        (Token::Numeral(a), Token::Numeral(b)) => {
            let g = a.grain?;
            if 10f64.powf(g as f64) > b.value {
                double(a.value + b.value)
            } else {
                None
            }
        }
        _ => None,
    }
}

static ZERO_TO_NINETEEN_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("nula", 0),
        ("jeden", 1),
        ("jedna", 1),
        ("jedno", 1),
        ("dva", 2),
        ("dve", 2),
        ("tri", 3),
        ("štyri", 4),
        ("päť", 5),
        ("šesť", 6),
        ("sedem", 7),
        ("osem", 8),
        ("deväť", 9),
        ("desať", 10),
        ("jedenásť", 11),
        ("dvanásť", 12),
        ("trinásť", 13),
        ("štrnásť", 14),
        ("pätnásť", 15),
        ("šestnásť", 16),
        ("sedemnásť", 17),
        ("osemnásť", 18),
        ("devätnásť", 19),
    ]
    .into_iter()
    .collect()
});

static DOZEN_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("dvadsať", 20),
        ("tridsať", 30),
        ("štyridsať", 40),
        ("päťdesiat", 50),
        ("šesťdesiat", 60),
        ("sedemdesiat", 70),
        ("osemdesiat", 80),
        ("devätdesiat", 90),
    ]
    .into_iter()
    .collect()
});

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleCouple
        rule("couple, a pair", vec![regex("páry?")], |_| integer(2)),
        // ruleDecimalNumeral
        rule("decimal number", vec![regex(r"(\d*,\d+)")], |ts| {
            parse_decimal(false, groups(ts)?.first()?)
        }),
        // ruleDecimalWithThousandsSeparator
        rule("decimal with thousands separator", vec![regex(r"(\d+(\.\d\d\d)+\,\d+)")], |ts| {
            let fmt = groups(ts)?.first()?.replace('.', "").replace(',', ".");
            parse_double(&fmt).and_then(double)
        }),
        // ruleDozen
        rule("dozen", vec![regex("tucet")], |_| {
            integer(12).and_then(|t| with_grain(1, &t)).and_then(|t| with_multipliable(&t))
        }),
        // ruleFew
        rule("few", vec![regex("(zo)?pár")], |_| integer(3)),
        // ruleInteger
        // e.g. jedenásť must be before jeden, otherwise jeden will always shadow jedenásť
        rule(
            "integer (0..19)",
            vec![regex(
                "(nula|jed(enásť|en|na|no)|dv(anásť|a|e)|trinásť|tri|štrnásť|štyri|pätnásť|päť|šestnásť|šesť|sedemnásť|sedem|osemnásť|osem|devätnásť|deväť|desať)",
            )],
            |ts| {
                ZERO_TO_NINETEEN_MAP
                    .get(to_lower(groups(ts)?.first()?).as_str())
                    .and_then(|v| integer(*v))
            },
        ),
        // ruleInteger2
        rule(
            "integer (20..90)",
            vec![regex("((dva|tri|štyri)dsať|(päť|šesť|sedem|osem|devät)desiat)")],
            |ts| DOZEN_MAP.get(to_lower(groups(ts)?.first()?).as_str()).and_then(|v| integer(*v)),
        ),
        // ruleIntegerCompositeTens
        rule(
            "integer 21..99",
            vec![one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]), pred(number_between(1.0, 10.0))],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(tens), Token::Numeral(units)) => double(tens.value + units.value),
                _ => None,
            },
        ),
        // ruleIntegerWithThousandsSeparator
        rule("integer with thousands separator .", vec![regex(r"(\d{1,3}(\.\d\d\d){1,5})")], |ts| {
            parse_double(&groups(ts)?.first()?.replace('.', "")).and_then(double)
        }),
        // ruleSum
        rule("intersect", vec![pred(has_grain), pred(not_multipliable_positive)], intersect_sum),
        // ruleIntersectWithAnd
        rule(
            "intersect (with and)",
            vec![pred(has_grain), regex("a"), pred(not_multipliable_positive)],
            intersect_sum_and,
        ),
        // ruleMultiply
        rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
            multiply(ts.first()?, ts.get(1)?)
        }),
        // ruleNumeralDotNumeral
        rule(
            "number dot number",
            vec![dimension(Dim::Numeral), regex("celá|celých|celé"), pred(|t| !has_grain(t))],
            |ts| match (ts.first()?, ts.get(2)?) {
                (Token::Numeral(nd1), Token::Numeral(nd2)) => {
                    double(nd1.value + decimals_to_double(nd2.value))
                }
                _ => None,
            },
        ),
        // ruleNumeralsPrefixWithNegativeOrMinus
        rule(
            "numbers prefix with -, negative or minus",
            vec![regex("-|mínus|záporné"), pred(is_positive)],
            |ts| match ts.get(1)? {
                Token::Numeral(nd) => double(nd.value * -1.0),
                _ => None,
            },
        ),
        // ruleNumeralsSuffixesKMG
        rule(
            "numbers suffixes (K, M, G)",
            vec![dimension(Dim::Numeral), regex("([kmg])(?=[\\W\\$€]|$)")],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(nd), Token::RegexMatch(g)) => match to_lower(g.0.first()?).as_str() {
                    "k" => double(nd.value * 1e3),
                    "m" => double(nd.value * 1e6),
                    "g" => double(nd.value * 1e9),
                    _ => None,
                },
                _ => None,
            },
        ),
        // rulePowersOfTen
        rule("powers of tens", vec![regex("(sto(vky)?|tisíce?|milióny?)")], |ts| {
            match to_lower(groups(ts)?.first()?).as_str() {
                "sto" | "stovky" => {
                    double(1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t))
                }
                "tisíc" | "tisíce" => {
                    double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t))
                }
                "milión" | "milióny" => {
                    double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t))
                }
                _ => None,
            }
        }),
        // ruleSingle
        rule("single", vec![regex("jed(en|no|na)")], |_| integer(1).and_then(|t| with_grain(1, &t))),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
