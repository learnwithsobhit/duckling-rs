//! `Duckling.Numeral.MN.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn rule_numerals_prefix_with_negative_or_minus() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex("-|хасах|сөрөг"), pred(is_positive)],
        |ts| match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * (-1.0)),
            _ => None,
        },
    )
}

fn rule_few() -> Rule {
    rule("few", vec![regex("хэдхэн")], |_| integer(3))
}

fn rule_ten() -> Rule {
    rule("ten", vec![regex("арав")], |_| integer(10).and_then(|t| with_grain(1, &t)))
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule(
        "decimal with thousands separator",
        vec![regex("(\\d+(\\.\\d\\d\\d)+\\,\\d+)")],
        |ts| {
            let g = groups(ts)?;
            let fmt = g.first()?.replace('.', "").replace(',', ".");
            parse_double(&fmt).and_then(double)
        },
    )
}

fn rule_integer3() -> Rule {
    rule(
        "integer ([2-9][1-9])",
        vec![regex(
            "(хорин|гучин|дөчин|тавин|жаран|далан|наян|ерэн) ?(нэг|хоёр|гурав|дөрөв|тав|зургаа|долоо|найм|ес)",
        )],
        |ts| {
            let g = groups(ts)?;
            let v1 = match to_lower(g.first()?).as_str() {
                "хорин" => 20,
                "гучин" => 30,
                "дөчин" => 40,
                "тавин" => 50,
                "жаран" => 60,
                "далан" => 70,
                "наян" => 80,
                "ерэн" => 90,
                _ => return None,
            };
            let v2 = match to_lower(g.get(1)?).as_str() {
                "нэг" => 1,
                "хоёр" => 2,
                "гурав" => 3,
                "дөрөв" => 4,
                "тав" => 5,
                "зургаа" => 6,
                "долоо" => 7,
                "найм" => 8,
                "ес" => 9,
                _ => return None,
            };
            integer(v1 + v2)
        },
    )
}

fn rule_numerals_und() -> Rule {
    rule(
        "numbers und",
        vec![
            one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(v1), Token::Numeral(v2)) => double(v1.value + v2.value),
            _ => None,
        },
    )
}

fn rule_multiply() -> Rule {
    rule(
        "compose by multiplication",
        vec![dimension(Dim::Numeral), pred(is_multipliable)],
        |ts| multiply(ts.first()?, ts.get(1)?),
    )
}

fn rule_intersect() -> Rule {
    rule(
        "intersect",
        vec![pred(has_grain), pred(|t| !is_multipliable(t) && is_positive(t))],
        |ts| match (ts.first()?, ts.get(1)?) {
            (
                Token::Numeral(NumeralData { value: val1, grain: Some(g), .. }),
                Token::Numeral(NumeralData { value: val2, .. }),
            ) if 10f64.powf(*g as f64) > *val2 => double(val1 + val2),
            _ => None,
        },
    )
}

fn rule_numerals_suffixes_kmg() -> Rule {
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
    )
}

fn rule_couple() -> Rule {
    rule("couple", vec![regex("хос")], |_| integer(2))
}

fn rule_powers_of_ten() -> Rule {
    rule("powers of tens", vec![regex("(зуу?|мянга?|сая?)")], |ts| {
        match to_lower(groups(ts)?.first()?).as_str() {
            "зуу" => double(1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t)),
            "зуун" => double(1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t)),
            "мянга" => {
                double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t))
            }
            "мянган" => {
                double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t))
            }
            "сая" => double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t)),
            "саяын" => {
                double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t))
            }
            _ => None,
        }
    })
}

static ZERO_NINETEEN_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("нуль", 0),
        ("тэг", 0),
        ("нойл", 0),
        ("нэг", 1),
        ("ганц", 1),
        ("хоёр", 2),
        ("гурав", 3),
        ("дөрөв", 4),
        ("тав", 5),
        ("зургаа", 6),
        ("долоо", 7),
        ("найм", 8),
        ("ес", 9),
        ("арван", 10),
        ("арав", 10),
        ("арван нэг", 11),
        ("арван хоёр", 12),
        ("арван гурав", 13),
        ("арван дөрөв", 14),
        ("арван тав", 15),
        ("арван зургаа", 16),
        ("арван долоо", 17),
        ("арван найм", 18),
        ("арван ес", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_zero_to_nineteen() -> Rule {
    rule(
        "integer (0..19)",
        vec![regex(
            "(нуль|тэг|нойл|нэг|ганц|хоёр|гурав|дөрөв|тав|зургаа|долоо|найм|ес|арван нэг|арван хоёр|арван гурав|арван дөрөв|арван тав|арван зургаа|арван долоо|арван найм|арван ес|арав|арван)",
        )],
        |ts| {
            let g = groups(ts)?;
            ZERO_NINETEEN_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("хорь", 20),
        ("гуч", 30),
        ("дөч", 40),
        ("тавь", 50),
        ("жар", 60),
        ("дал", 70),
        ("ная", 80),
        ("ер", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_integer2() -> Rule {
    rule("integer (20..90)", vec![regex("(хорь|гуч|дөч|тавь|жар|дал|ная|ер)")], |ts| {
        let g = groups(ts)?;
        TENS_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
    })
}

fn rule_numeral_dot_numeral() -> Rule {
    rule(
        "number dot number",
        vec![dimension(Dim::Numeral), regex("цэг"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(nd1), Token::Numeral(nd2)) => {
                double(nd1.value + decimals_to_double(nd2.value))
            }
            _ => None,
        },
    )
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex("(\\d*\\.\\d+)")], |ts| {
        parse_decimal(true, groups(ts)?.first()?)
    })
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule(
        "integer with thousands separator .",
        vec![regex("(\\d{1,3}(\\.\\d\\d\\d){1,5})")],
        |ts| {
            let g = groups(ts)?;
            parse_double(&g.first()?.replace('.', "")).and_then(double)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_couple(),
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_few(),
        rule_integer2(),
        rule_integer3(),
        rule_integer_with_thousands_separator(),
        rule_intersect(),
        rule_multiply(),
        rule_numeral_dot_numeral(),
        rule_numerals_prefix_with_negative_or_minus(),
        rule_numerals_suffixes_kmg(),
        rule_numerals_und(),
        rule_powers_of_ten(),
        rule_ten(),
        rule_zero_to_nineteen(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
