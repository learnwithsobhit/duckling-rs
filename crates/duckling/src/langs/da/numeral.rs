//! `Duckling.Numeral.DA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, group, pred, regex, rule, Dim, Rule, Token};

const ZERO_NINETEEN: &[(&str, i64)] = &[
    ("ingen", 0),
    ("nul", 0),
    ("intet", 0),
    ("en", 1),
    ("et", 1),
    ("én", 1),
    ("ét", 1),
    ("to", 2),
    ("tre", 3),
    ("fire", 4),
    ("fem", 5),
    ("seks", 6),
    ("syv", 7),
    ("otte", 8),
    ("ni", 9),
    ("ti", 10),
    ("elleve", 11),
    ("tolv", 12),
    ("tretten", 13),
    ("fjorten", 14),
    ("femten", 15),
    ("seksten", 16),
    ("sytten", 17),
    ("atten", 18),
    ("nitten", 19),
];

fn lookup(table: &[(&str, i64)], k: &str) -> Option<i64> {
    table.iter().find(|(n, _)| *n == k).map(|(_, v)| *v)
}

fn nv(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleAPair
        rule("a pair", vec![regex("et par")], |_| integer(2).and_then(|t| with_grain(1, &t))),
        // ruleDecimalNumeral
        rule("decimal number", vec![regex("(\\d*,\\d+)")], |ts| {
            let m = group(ts, 0)?;
            parse_decimal(false, &m)
        }),
        // ruleDecimalWithThousandsSeparator
        rule(
            "decimal with thousands separator",
            vec![regex("(\\d+(\\.\\d\\d\\d)+\\,\\d+)")],
            |ts| {
                let m = group(ts, 0)?;
                parse_double(&m.replace('.', "")).and_then(double)
            },
        ),
        // ruleDozen
        rule("dozen", vec![regex("dusin")], |_| {
            integer(12).and_then(|t| with_grain(1, &t)).and_then(|t| with_multipliable(&t))
        }),
        // ruleFew
        rule("few", vec![regex("(nogle )?få")], |_| integer(3)),
        // ruleInteger
        rule(
            "integer (0..19)",
            vec![regex(
                "(intet|ingen|nul|en|et|én|ét|to|tretten|tre|fire|femten|fem|seksten|seks|syv|otte|nitten|ni|ti|elleve|tolv|fjorten|sytten|atten)",
            )],
            |ts| {
                let m = group(ts, 0)?;
                lookup(ZERO_NINETEEN, &to_lower(&m)).and_then(integer)
            },
        ),
        // ruleInteger2
        rule(
            "integer (20..90)",
            vec![regex("(tyve|tredive|fyrre|halvtreds|tres|halvfjerds|firs|halvfems)")],
            |ts| {
                let m = to_lower(&group(ts, 0)?);
                match m.as_str() {
                    "tyve" => integer(20),
                    "tredive" => integer(30),
                    "fyrre" => integer(40),
                    "halvtreds" => integer(50),
                    "tres" => integer(60),
                    "halvfjerds" => integer(70),
                    "firs" => integer(80),
                    "halvfems" => integer(90),
                    _ => None,
                }
            },
        ),
        // ruleInteger3
        rule(
            "integer 21..99",
            vec![one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]), pred(number_between(1.0, 10.0))],
            |ts| double(nv(ts.first()?)? + nv(ts.get(1)?)?),
        ),
        // ruleIntegerWithThousandsSeparator
        rule(
            "integer with thousands separator .",
            vec![regex("(\\d{1,3}(\\.\\d\\d\\d){1,5})")],
            |ts| {
                let m = group(ts, 0)?;
                parse_double(&m.replace('.', "")).and_then(double)
            },
        ),
        // ruleIntersect
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
        ),
        // ruleMultiply
        rule(
            "compose by multiplication",
            vec![dimension(Dim::Numeral), pred(is_multipliable)],
            |ts| multiply(ts.first()?, ts.get(1)?),
        ),
        // ruleNumeralDotNumeral
        rule(
            "number dot number",
            vec![dimension(Dim::Numeral), regex("komma"), pred(|t| !has_grain(t))],
            |ts| double(nv(ts.first()?)? + decimals_to_double(nv(ts.get(2)?)?)),
        ),
        // ruleNumeralsOg
        rule(
            "numbers og",
            vec![
                pred(number_between(1.0, 10.0)),
                regex("og"),
                one_of(vec![70.0, 20.0, 60.0, 50.0, 40.0, 90.0, 30.0, 80.0]),
            ],
            |ts| double(nv(ts.first()?)? + nv(ts.get(2)?)?),
        ),
        // ruleNumeralsPrefixWithNegativeOrMinus
        rule(
            "numbers prefix with -, negative or minus",
            vec![regex("-|minus|negativ"), pred(is_positive)],
            |ts| double(nv(ts.get(1)?)? * (-1.0)),
        ),
        // ruleNumeralsSuffixesKMG
        rule(
            "numbers suffixes (K, M, G)",
            vec![dimension(Dim::Numeral), regex("([kmg])(?=[\\W\\$€]|$)")],
            |ts| {
                let v = nv(ts.first()?)?;
                let m = match ts.get(1)? {
                    Token::RegexMatch(g) => g.0.first()?.clone(),
                    _ => return None,
                };
                match to_lower(&m).as_str() {
                    "k" => double(v * 1e3),
                    "m" => double(v * 1e6),
                    "g" => double(v * 1e9),
                    _ => None,
                }
            },
        ),
        // rulePowersOfTen
        rule(
            "powers of tens",
            vec![regex("(hundrede?|tohundrede|tusinde?|totusinde|million(er)?)")],
            |ts| {
                let m = to_lower(&group(ts, 0)?);
                match m.as_str() {
                    "hundred" => double(1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t)),
                    "hundrede" => double(1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t)),
                    "tohundrede" => {
                        double(2.0 * 1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t))
                    }
                    "tusind" => double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t)),
                    "tusinde" => double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t)),
                    "totusinde" => {
                        double(2.0 * 1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t))
                    }
                    "million" => double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t)),
                    "millioner" => double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t)),
                    _ => None,
                }
            },
        ),
        // ruleSingle
        rule("single", vec![regex("enkelt")], |_| integer(1).and_then(|t| with_grain(1, &t))),
        // ruleTen
        rule("ten", vec![regex("ti")], |_| integer(10).and_then(|t| with_grain(1, &t))),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
