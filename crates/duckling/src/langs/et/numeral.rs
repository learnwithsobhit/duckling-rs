//! `Duckling.Numeral.ET.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, group, groups, pred, regex, rule, Dim, Rule, Token};

const ZERO_NINETEEN: &[(&str, i64)] = &[
    ("null", 0),
    ("üks", 1),
    ("kaks", 2),
    ("kolm", 3),
    ("neli", 4),
    ("viis", 5),
    ("kuus", 6),
    ("seitse", 7),
    ("kaheksa", 8),
    ("üheksa", 9),
    ("kümme", 10),
    ("üksteist", 11),
    ("kaksteist", 12),
    ("kolmteist", 13),
    ("neliteist", 14),
    ("viisteist", 15),
    ("kuusteist", 16),
    ("seitseteist", 17),
    ("kaheksateist", 18),
    ("üheksateist", 19),
];

const TWENTY_NINETY: &[(&str, i64)] = &[
    ("kakskümmend", 20),
    ("kolmkümmend", 30),
    ("nelikümmend", 40),
    ("viiskümmend", 50),
    ("kuuskümmend", 60),
    ("seitsekümmend", 70),
    ("kaheksakümmend", 80),
    ("üheksakümmend", 90),
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
        // ruleACoupleOf
        rule("a couple of", vec![regex("paar")], |_| integer(2)),
        // ruleAFew
        rule("(a )?few", vec![regex("mõni")], |_| integer(3)),
        // ruleDecimalNumeral
        rule("decimal number", vec![regex("(\\d*\\.\\d+)")], |ts| {
            parse_decimal(true, &group(ts, 0)?)
        }),
        // ruleDecimalWithThousandsSeparator
        rule(
            "decimal with thousands separator",
            vec![regex("(\\d+(,\\d\\d\\d)+\\.\\d+)")],
            |ts| {
                let fmt = group(ts, 0)?.replace(',', "");
                parse_double(&fmt).and_then(double)
            },
        ),
        // ruleInteger
        rule(
            "integer (0..19)",
            vec![regex("(null|üksteist|üks|kaksteist|kaks|kolmteist|kolm|neliteist|neli|viisteist|viis|kuusteist|kuus|seitseteist|seitse|kaheksateist|kaheksa|üheksateist|üheksa|kümme)")],
            |ts| lookup(ZERO_NINETEEN, &to_lower(&group(ts, 0)?)).and_then(integer),
        ),
        // ruletwentyNinety
        rule(
            "integer (20..90)",
            vec![regex("((kaks|kolm|neli|viis|kuus|seitse|kaheksa|(ü)heksa)k(ü)mmend)")],
            |ts| lookup(TWENTY_NINETY, &to_lower(&group(ts, 0)?)).and_then(integer),
        ),
        // ruleInteger3
        rule(
            "integer 21..99",
            vec![one_of(vec![70.0, 20.0, 60.0, 50.0, 40.0, 90.0, 30.0, 80.0]), pred(number_between(1.0, 10.0))],
            |ts| double(nv(ts.first()?)? + nv(ts.get(1)?)?),
        ),
        // ruleInteger4
        rule(
            "integer (200..900)",
            vec![regex("(kakssada|kolmsada|nelisada|viissada|kuussada|seitsesada|kaheksasada|üheksasada)")],
            |ts| {
                let t = match to_lower(&group(ts, 0)?).as_str() {
                    "kakssada" => integer(200)?,
                    "kolmsada" => integer(300)?,
                    "nelisada" => integer(400)?,
                    "viissada" => integer(500)?,
                    "kuussada" => integer(600)?,
                    "seitsesada" => integer(700)?,
                    "kaheksasada" => integer(800)?,
                    "üheksasada" => integer(900)?,
                    _ => return None,
                };
                with_grain(2, &t).and_then(|t| with_multipliable(&t))
            },
        ),
        // ruleIntegerWithThousandsSeparator
        rule(
            "integer with thousands separator ,",
            vec![regex("(\\d{1,3}(([, ])\\d\\d\\d){1,5})")],
            |ts| {
                let g = groups(ts)?;
                let m = g.first()?;
                let sep = g.get(2)?;
                parse_double(&m.replace(sep.as_ref(), "")).and_then(double)
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
            vec![dimension(Dim::Numeral), regex("dot|point"), pred(|t| !has_grain(t))],
            |ts| match (ts.first()?, ts.get(2)?) {
                (Token::Numeral(a), Token::Numeral(b)) => double(a.value + decimals_to_double(b.value)),
                _ => None,
            },
        ),
        // ruleNumeralsPrefixWithNegativeOrMinus
        rule(
            "numbers prefix with -, negative or minus",
            vec![regex("-|miinus|negatiivne"), pred(is_positive)],
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
        rule("powers of tens", vec![regex("(sada|tuhat|miljoni?t?)")], |ts| {
            let (v, g) = match to_lower(&group(ts, 0)?).as_str() {
                "sada" => (1e2, 2),
                "tuhat" => (1e3, 3),
                _ => (1e6, 6),
            };
            double(v).and_then(|t| with_grain(g, &t)).and_then(|t| with_multipliable(&t))
        }),
        // ruleTen
        rule("ten", vec![regex("kümme")], |_| integer(10).and_then(|t| with_grain(1, &t))),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
