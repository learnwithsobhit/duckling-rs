//! `Duckling.Numeral.RU.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("двадцать", 20),
        ("двадцати", 20),
        ("тридцать", 30),
        ("тридцати", 30),
        ("сорок", 40),
        ("сорока", 40),
        ("пятьдесят", 50),
        ("пятидесяти", 50),
        ("шестьдесят", 60),
        ("шестидесяти", 60),
        ("шестидесят", 60),
        ("семьдесят", 70),
        ("семидесяти", 70),
        ("семидесят", 70),
        ("восемьдесят", 80),
        ("восьмидесяти", 80),
        ("восьмидесят", 80),
        ("восемьдесяти", 80),
        ("девяносто", 90),
        ("девяноста", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_integer5() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex(
            "(двадцат(ь|и)|тридцат(ь|и)|сорока?|пят(ь|и)десяти?|шест(ь|и)десяти?|сем(ь|и)десяти?|вос(е|ь)м(ь|и)десяти?|девяност(о|а))",
        )],
        |ts| {
            let g = groups(ts)?;
            TENS_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
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
    rule("integer 2", vec![regex("(дв(а|е|ух|ое)|пар(а|у|очк(у|а)))")], |_| integer(2))
}

fn rule_decimal_one_and_a_half() -> Rule {
    rule("decimal one and a half", vec![regex("(полтора|полторы|полутора)")], |_| double(1.5))
}

fn rule_integer_and_a_half() -> Rule {
    rule("<integer> and a half", vec![pred(is_natural), regex("с половиной")], |ts| {
        match ts.first()? {
            Token::Numeral(n) => double(n.value + 0.5),
            _ => None,
        }
    })
}

static HUNDREDS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("сто", 100),
        ("двести", 200),
        ("триста", 300),
        ("четыреста", 400),
        ("пятьсот", 500),
        ("шестьсот", 600),
        ("семьсот", 700),
        ("восемьсот", 800),
        ("девятьсот", 900),
    ]
    .into_iter()
    .collect()
});

fn rule_integer6() -> Rule {
    rule(
        "integer (100..900)",
        vec![regex("(сто|двести|триста|четыреста|пятьсот|шестьсот|семьсот|восемьсот|девятьсот)")],
        |ts| {
            let g = groups(ts)?;
            HUNDREDS_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_numerals_prefix_with_minus() -> Rule {
    rule("numbers prefix with -, minus", vec![regex("-|минус"), pred(is_positive)], |ts| {
        match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * -1.0),
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
                "к" | "К" => double(nd.value * 1e3),
                "м" | "М" => double(nd.value * 1e6),
                "г" | "Г" => double(nd.value * 1e9),
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
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
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
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
            _ => None,
        },
    )
}

fn rule_integer() -> Rule {
    rule("integer 0", vec![regex("(ноль|нуля|нисколько)")], |_| integer(0))
}

static THREE_TO_NINETEEN_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("три", 3),
        ("четыре", 4),
        ("пять", 5),
        ("шесть", 6),
        ("семь", 7),
        ("восемь", 8),
        ("девять", 9),
        ("десять", 10),
        ("одиннадцать", 11),
        ("двенадцать", 12),
        ("тринадцать", 13),
        ("четырнадцать", 14),
        ("пятнадцать", 15),
        ("шестнадцать", 16),
        ("семнадцать", 17),
        ("восемнадцать", 18),
        ("девятнадцать", 19),
    ]
    .into_iter()
    .collect()
});

static THREE_TO_NINETEEN_MAP_GENITIVE: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("трех", 3),
        ("четырех", 4),
        ("пяти", 5),
        ("шести", 6),
        ("семи", 7),
        ("восьми", 8),
        ("девяти", 9),
        ("десяти", 10),
        ("одиннадцати", 11),
        ("двенадцати", 12),
        ("тринадцати", 13),
        ("четырнадцати", 14),
        ("пятнадцати", 15),
        ("шестнадцати", 16),
        ("семнадцати", 17),
        ("восемнадцати", 18),
        ("девятнадцати", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_integer4() -> Rule {
    rule(
        "integer (3..19)",
        vec![regex(
            "(три|четырнадцать|четыре|пятнадцать|пять|шестнадцать|шесть|семнадцать|семь|восемнадцать|восемь|девятнадцать|девять|десять|одиннадцать|двенадцать|тринадцать)",
        )],
        |ts| {
            let g = groups(ts)?;
            THREE_TO_NINETEEN_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_integer4_genitive() -> Rule {
    rule(
        "integer (3..19)",
        vec![regex(
            "(трех|четырнадцати|четырех|пятнадцати|пяти|шестнадцати|шести|семнадцати|семи|восемнадцати|восьми|девятнадцати|девяти|десяти|одиннадцати|двенадцати|тринадцати)",
        )],
        |ts| {
            let g = groups(ts)?;
            THREE_TO_NINETEEN_MAP_GENITIVE.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_integer2() -> Rule {
    rule("integer 1", vec![regex("(один|одна|одну)")], |_| integer(1))
}

fn rule_numeral_dot_numeral() -> Rule {
    rule(
        "number dot number",
        vec![dimension(Dim::Numeral), regex("точка"), pred(|t| !has_grain(t))],
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
        rule_integer4_genitive(),
        rule_integer5(),
        rule_integer6(),
        rule_integer7(),
        rule_integer8(),
        rule_integer_and_a_half(),
        rule_decimal_one_and_a_half(),
        rule_integer_with_thousands_separator(),
        rule_numeral_dot_numeral(),
        rule_numerals_prefix_with_minus(),
        rule_numerals_suffixes_kmg(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
