//! `Duckling.Numeral.NL.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, group, groups, pred, regex, rule, Rule, Token};

const ZERO_NINETEEN: &[(&str, i64)] = &[
    ("niks", 0),
    ("nul", 0),
    ("geen", 0),
    ("één", 1),
    ("een", 1),
    ("twee", 2),
    ("drie", 3),
    ("vier", 4),
    ("vijf", 5),
    ("zes", 6),
    ("zeven", 7),
    ("acht", 8),
    ("negen", 9),
    ("tien", 10),
    ("elf", 11),
    ("twaalf", 12),
    ("dertien", 13),
    ("veertien", 14),
    ("vijftien", 15),
    ("zestien", 16),
    ("zeventien", 17),
    ("achttien", 18),
    ("negentien", 19),
];

const DOZENS: &[(&str, i64)] = &[
    ("twintig", 20),
    ("dertig", 30),
    ("veertig", 40),
    ("vijftig", 50),
    ("zestig", 60),
    ("zeventig", 70),
    ("tachtig", 80),
    ("negentig", 90),
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
        rule("couple", vec![regex("(een )?paar")], |_| integer(2)),
        // ruleDecimalNumeral
        rule("decimal number", vec![regex("(\\d*,\\d+)")], |ts| {
            let m = group(ts, 0)?;
            parse_decimal(false, &m)
        }),
        // ruleDecimalWithThousandsSeparator
        rule(
            "decimal with thousands separator",
            vec![regex("(\\d+(\\.\\d\\d\\d)+(,\\d+)?)")],
            |ts| {
                let m = group(ts, 0)?;
                let fmt = m.replace('.', "").replace(',', ".");
                parse_double(&fmt).and_then(double)
            },
        ),
        // ruleDozen
        rule("dozen", vec![regex("dozijn")], |_| integer(12).and_then(|t| with_grain(1, &t))),
        // ruleGross
        rule("gros", vec![regex("gros")], |_| integer(144).and_then(|t| with_grain(1, &t))),
        // ruleFew
        rule("few", vec![regex("meerdere")], |_| integer(3)),
        // ruleInteger
        rule(
            "integer (0..19)",
            vec![regex("(geen|nul|niks|een|één|twee|drie|vier|vijftien|vijf|zestien|zes|zeventien|zeven|achttien|acht|negentien|negen|tien|elf|twaalf|dertien|veertien)")],
            |ts| {
                let m = group(ts, 0)?;
                lookup(ZERO_NINETEEN, &to_lower(&m)).and_then(integer)
            },
        ),
        // ruleInteger2
        rule(
            "integer (20..90)",
            vec![regex("(twintig|dertig|veertig|vijftig|zestig|zeventig|tachtig|negentig)")],
            |ts| {
                let m = group(ts, 0)?;
                lookup(DOZENS, &to_lower(&m)).and_then(integer)
            },
        ),
        // ruleInteger3
        rule(
            "integer ([2-9][1-9])",
            vec![regex("(een|twee|drie|vier|vijf|zes|zeven|acht|negen)(?:e|ë)n(twintig|dertig|veertig|vijftig|zestig|zeventig|tachtig|negentig)")],
            |ts| {
                let g = groups(ts)?;
                let v1 = lookup(ZERO_NINETEEN, &to_lower(g.first()?))?;
                let v2 = lookup(DOZENS, &to_lower(g.get(1)?))?;
                integer(v1 + v2)
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
            vec![dimension(crate::types::Dim::Numeral), pred(is_multipliable)],
            |ts| multiply(ts.first()?, ts.get(1)?),
        ),
        // ruleNumeralsEn
        rule(
            "numbers en",
            vec![
                pred(number_between(1.0, 10.0)),
                regex("-?en-?"),
                one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
            ],
            |ts| double(nv(ts.first()?)? + nv(ts.get(2)?)?),
        ),
        // ruleNumeralsPrefixWithNegativeOrMinus
        rule(
            "numbers prefix with -, negative or minus",
            vec![regex("-|min|minus|negatief"), pred(is_positive)],
            |ts| double(nv(ts.get(1)?)? * (-1.0)),
        ),
        // ruleNumeralsSuffixesKMG
        rule(
            "numbers suffixes (K, M, G)",
            vec![dimension(crate::types::Dim::Numeral), regex("([kmg])(?=[\\W\\$€]|$)")],
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
        rule("powers of tens", vec![regex("(honderd|duizend|miljoen)")], |ts| {
            let m = group(ts, 0)?;
            let (v, g) = match to_lower(&m).as_str() {
                "honderd" => (1e2, 2),
                "duizend" => (1e3, 3),
                "miljoen" => (1e6, 6),
                _ => return None,
            };
            double(v).and_then(|t| with_grain(g, &t)).and_then(|t| with_multipliable(&t))
        }),
        // ruleTen
        rule("ten", vec![regex("tien")], |_| integer(10).and_then(|t| with_grain(1, &t))),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
