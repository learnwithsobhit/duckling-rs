//! `Duckling.Numeral.UK.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

static TWENTY_NINETY_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("двадцять", 20),
        ("тридцять", 30),
        ("сорок", 40),
        ("п‘ятдесят", 50),
        ("шістдесят", 60),
        ("сімдесят", 70),
        ("дев‘яносто", 90),
        ("вісімдесят", 80),
    ]
    .into_iter()
    .collect()
});

fn rule_integer5() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(двадцять|тридцять|сорок|п‘ятдесят|шістдесят|сімдесят|вісімдесят|дев‘яносто)")],
        |ts| {
            let g = groups(ts)?;
            TWENTY_NINETY_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule("decimal with thousands separator", vec![regex("(\\d+(,\\d\\d\\d)+\\.\\d+)")], |ts| {
        let g = groups(ts)?;
        parse_double(&g.first()?.replace(',', "")).and_then(double)
    })
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex("(\\d*\\.\\d+)")], |ts| {
        parse_decimal(true, groups(ts)?.first()?)
    })
}

fn rule_integer3() -> Rule {
    rule("integer 2", vec![regex("(два|дві|двоє|пара|пару|парочку|парочка)")], |_| integer(2))
}

static HUNDREDS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("сто", 100),
        ("двісті", 200),
        ("триста", 300),
        ("чотириста", 400),
        ("п‘ятсот", 500),
        ("шістсот", 600),
        ("сімсот", 700),
        ("вісімсот", 800),
        ("дев‘ятсот", 900),
    ]
    .into_iter()
    .collect()
});

fn rule_integer6() -> Rule {
    rule(
        "integer (100..900)",
        vec![regex("(сто|двісті|триста|чотириста|п‘ятсот|шістсот|сімсот|вісімсот|дев‘ятсот)")],
        |ts| {
            let g = groups(ts)?;
            HUNDREDS_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_numerals_prefix_with_minus() -> Rule {
    rule("numbers prefix with -, minus", vec![regex("-|мінус\\s?"), dimension(Dim::Numeral)], |ts| {
        match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * (-1.0)),
            _ => None,
        }
    })
}

fn rule_numerals_suffixes_kmg() -> Rule {
    rule(
        "numbers suffixes (K, M, G)",
        vec![dimension(Dim::Numeral), regex("((к|м|г)|(К|М|Г))(?=[\\W\\$€]|$)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(nd), Token::RegexMatch(g)) => match to_lower(g.0.first()?).as_str() {
                "к" => double(nd.value * 1e3),
                "К" => double(nd.value * 1e3),
                "м" => double(nd.value * 1e6),
                "М" => double(nd.value * 1e6),
                "г" => double(nd.value * 1e9),
                "Г" => double(nd.value * 1e9),
                _ => None,
            },
            _ => None,
        },
    )
}

fn rule_integer7() -> Rule {
    rule(
        "integer 21..99",
        vec![one_of(vec![70.0, 20.0, 60.0, 50.0, 40.0, 90.0, 30.0, 80.0]), pred(number_between(1.0, 10.0))],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(v1), Token::Numeral(v2)) => double(v1.value + v2.value),
            _ => None,
        },
    )
}

fn rule_integer8() -> Rule {
    rule(
        "integer 101..999",
        vec![
            one_of(vec![300.0, 600.0, 500.0, 100.0, 800.0, 200.0, 900.0, 700.0, 400.0]),
            pred(number_between(1.0, 100.0)),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(v1), Token::Numeral(v2)) => double(v1.value + v2.value),
            _ => None,
        },
    )
}

fn rule_integer() -> Rule {
    rule("integer 0", vec![regex("(нуль)")], |_| integer(0))
}

static THREE_NINETEEN_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("три", 3),
        ("чотири", 4),
        ("п‘ять", 5),
        ("шість", 6),
        ("сім", 7),
        ("вісім", 8),
        ("дев‘ять", 9),
        ("десять", 10),
        ("одинадцять", 11),
        ("дванадцять", 12),
        ("тринадцять", 13),
        ("чотирнадцять", 14),
        ("п‘ятнадцять", 15),
        ("шістнадцять", 16),
        ("сімнадцять", 17),
        ("вісімнадцять", 18),
        ("дев‘ятнадцять", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_integer4() -> Rule {
    rule(
        "integer (3..19)",
        vec![regex(
            "(три|чотирнадцять|чотири|п‘ятнадцять|п‘ять|шістнадцять|шість|сімнадцять|сім|вісімнадцять|вісім|дев‘ятнадцять|дев‘ять|десять|одинадцять|дванадцять|тринадцять)",
        )],
        |ts| {
            let g = groups(ts)?;
            THREE_NINETEEN_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_integer2() -> Rule {
    rule("integer 1", vec![regex("(один|одна|одну|одне|одного)")], |_| integer(1))
}

fn rule_numeral_dot_numeral() -> Rule {
    rule(
        "number dot number",
        vec![dimension(Dim::Numeral), regex("крапка"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(nd1), Token::Numeral(nd2)) => {
                double(nd1.value + decimals_to_double(nd2.value))
            }
            _ => None,
        },
    )
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule("integer with thousands separator ,", vec![regex("(\\d{1,3}(,\\d\\d\\d){1,5})")], |ts| {
        let g = groups(ts)?;
        parse_double(&g.first()?.replace(',', "")).and_then(double)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_integer(),
        rule_integer2(),
        rule_integer3(),
        rule_integer4(),
        rule_integer5(),
        rule_integer6(),
        rule_integer7(),
        rule_integer8(),
        rule_integer_with_thousands_separator(),
        rule_numeral_dot_numeral(),
        rule_numerals_prefix_with_minus(),
        rule_numerals_suffixes_kmg(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
