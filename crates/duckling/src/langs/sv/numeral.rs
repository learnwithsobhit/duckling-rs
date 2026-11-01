//! `Duckling.Numeral.SV.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, group, pred, regex, rule, Dim, Rule, Token};

const ZERO_NINETEEN: &[(&str, i64)] = &[
    ("inget", 0),
    ("ingen", 0),
    ("noll", 0),
    ("en", 1),
    ("ett", 1),
    ("två", 2),
    ("tre", 3),
    ("fyra", 4),
    ("fem", 5),
    ("sex", 6),
    ("sju", 7),
    ("åtta", 8),
    ("nio", 9),
    ("tio", 10),
    ("elva", 11),
    ("tolv", 12),
    ("tretton", 13),
    ("fjorton", 14),
    ("femton", 15),
    ("sexton", 16),
    ("sjutton", 17),
    ("arton", 18),
    ("nitton", 19),
];

const DOZENS: &[(&str, i64)] = &[
    ("tjugo", 20),
    ("trettio", 30),
    ("fyrtio", 40),
    ("femtio", 50),
    ("sextio", 60),
    ("sjuttio", 70),
    ("åttio", 80),
    ("nittio", 90),
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
        // ruleCouple
        rule("couple, a pair", vec![regex("ett par")], |_| integer(2)),
        // ruleDecimalNumeral
        rule("decimal number", vec![regex("(\\d*,\\d+)")], |ts| {
            parse_decimal(false, &group(ts, 0)?)
        }),
        // ruleDecimalWithThousandsSeparator
        rule(
            "decimal with thousands separator",
            vec![regex("(\\d+(\\.\\d\\d\\d)+\\,\\d+)")],
            |ts| {
                let m = group(ts, 0)?;
                let fmt = m.replace('.', "").replace(',', ".");
                parse_double(&fmt).and_then(double)
            },
        ),
        // ruleDozen
        rule("dozen", vec![regex("dussin")], |_| {
            integer(12).and_then(|t| with_grain(1, &t)).and_then(|t| with_multipliable(&t))
        }),
        // ruleFew
        rule("few", vec![regex("(några )?få")], |_| integer(3)),
        // ruleInteger
        rule(
            "integer (0..19)",
            vec![regex("(inget|ingen|noll|en|ett|två|tretton|tre|fyra|femton|fem|sexton|sex|sjutton|sju|åtta|nio|tio|elva|tolv|fjorton|arton|nitton)")],
            |ts| lookup(ZERO_NINETEEN, &to_lower(&group(ts, 0)?)).and_then(integer),
        ),
        // ruleInteger2
        rule(
            "integer (20..90)",
            vec![regex("(tjugo|trettio|fyrtio|femtio|sextio|sjuttio|åttio|nittio)")],
            |ts| lookup(DOZENS, &to_lower(&group(ts, 0)?)).and_then(integer),
        ),
        // ruleInteger3
        rule(
            "integer 21..99",
            vec![one_of(vec![70.0, 20.0, 60.0, 50.0, 40.0, 90.0, 30.0, 80.0]), pred(number_between(1.0, 10.0))],
            |ts| double(nv(ts.first()?)? + nv(ts.get(1)?)?),
        ),
        // ruleIntegerWithThousandsSeparator
        rule(
            "integer with thousands separator .",
            vec![regex("(\\d{1,3}(\\.\\d\\d\\d){1,5})")],
            |ts| {
                let fmt = group(ts, 0)?.replace('.', "");
                parse_double(&fmt).and_then(double)
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
        // ruleIntersectWithAnd
        rule(
            "intersect (with and)",
            vec![pred(has_grain), regex("och"), pred(|t| !is_multipliable(t) && is_positive(t))],
            |ts| match (ts.first()?, ts.get(2)?) {
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
            |ts| match (ts.first()?, ts.get(2)?) {
                (Token::Numeral(a), Token::Numeral(b)) => double(a.value + decimals_to_double(b.value)),
                _ => None,
            },
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
        rule("powers of tens", vec![regex("(hundra?|tusen?|miljon(er)?)")], |ts| {
            let (v, g) = match to_lower(&group(ts, 0)?).as_str() {
                "hundr" => (1e2, 2),
                "hundra" => (1e2, 2),
                "tuse" => (1e3, 3),
                "tusen" => (1e3, 3),
                "miljon" => (1e6, 6),
                "miljoner" => (1e6, 6),
                _ => return None,
            };
            double(v).and_then(|t| with_grain(g, &t)).and_then(|t| with_multipliable(&t))
        }),
        // ruleSingle
        rule("single", vec![regex("enkel")], |_| integer(1).and_then(|t| with_grain(1, &t))),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
