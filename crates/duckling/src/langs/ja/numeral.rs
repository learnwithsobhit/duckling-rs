//! `Duckling.Numeral.JA.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn num_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

fn rule_integer5() -> Rule {
    rule("integer (100)", vec![regex("百")], |_| integer(100))
}

fn rule_numerals_prefix_with_negative_or_minus() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex(r"-|マイナス\s?|負\s?"), pred(is_positive)],
        |ts| double(num_value(ts.get(1)?)? * -1.0),
    )
}

fn rule_integer10() -> Rule {
    rule(
        "integer (1000..1999)",
        vec![regex("千"), pred(number_between(1.0, 1000.0))],
        |ts| double(num_value(ts.get(1)?)? + 1000.0),
    )
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule(
        "decimal with thousands separator",
        vec![regex(r"(\d+(,\d\d\d)+\.\d+)")],
        |ts| {
            let m = groups(ts)?.first()?;
            parse_double(&m.replace(',', "")).and_then(double)
        },
    )
}

fn rule_integer15() -> Rule {
    rule(
        "integer (20000..90000)",
        vec![pred(number_between(2.0, 10.0)), regex("万")],
        |ts| double(num_value(ts.first()?)? * 10000.0),
    )
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex(r"(\d*\.\d+)")], |ts| {
        parse_decimal(true, groups(ts)?.first()?)
    })
}

fn rule_numeral() -> Rule {
    rule("<number>个", vec![dimension(Dim::Numeral), regex("个")], |ts| Some(ts[0].clone()))
}

fn rule_integer3() -> Rule {
    rule(
        "integer (20..90)",
        vec![pred(number_between(2.0, 10.0)), regex("十")],
        |ts| double(num_value(ts.first()?)? * 10.0),
    )
}

fn rule_integer13() -> Rule {
    rule("integer (10000)", vec![regex("万")], |_| integer(10000))
}

fn rule_integer6() -> Rule {
    rule(
        "integer (100..199)",
        vec![regex("百"), pred(number_between(1.0, 100.0))],
        |ts| double(num_value(ts.get(1)?)? + 100.0),
    )
}

fn rule_integer12() -> Rule {
    rule(
        "integer 2001..9999",
        vec![
            one_of(vec![3000.0, 9000.0, 7000.0, 8000.0, 2000.0, 4000.0, 6000.0, 5000.0]),
            pred(number_between(1.0, 1000.0)),
        ],
        |ts| double(num_value(ts.first()?)? + num_value(ts.get(1)?)?),
    )
}

fn rule_numerals_suffixes_kmg() -> Rule {
    rule(
        "numbers suffixes (K, M, G, 千, 万)",
        vec![dimension(Dim::Numeral), regex("(k|m|g|千|万)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(nd), Token::RegexMatch(g)) => match to_lower(g.0.first()?).as_str() {
                "k" => double(nd.value * 1e3),
                "千" => double(nd.value * 1e3),
                "万" => double(nd.value * 1e4),
                "m" => double(nd.value * 1e6),
                "g" => double(nd.value * 1e9),
                _ => None,
            },
            _ => None,
        },
    )
}

fn rule_integer7() -> Rule {
    rule(
        "integer (200..900)",
        vec![pred(number_between(2.0, 10.0)), regex("百")],
        |ts| double(num_value(ts.first()?)? * 100.0),
    )
}

fn rule_integer14() -> Rule {
    rule(
        "integer (10000..19999)",
        vec![regex("万"), pred(number_between(1.0, 10000.0))],
        |ts| double(num_value(ts.get(1)?)? + 10000.0),
    )
}

fn rule_integer8() -> Rule {
    rule(
        "integer 201..999",
        vec![
            one_of(vec![300.0, 600.0, 500.0, 800.0, 200.0, 900.0, 700.0, 400.0]),
            pred(number_between(1.0, 100.0)),
        ],
        |ts| double(num_value(ts.first()?)? + num_value(ts.get(1)?)?),
    )
}

fn rule_integer16() -> Rule {
    rule(
        "integer 20001..99999",
        vec![
            one_of(vec![20000.0, 40000.0, 80000.0, 60000.0, 30000.0, 70000.0, 90000.0, 50000.0]),
            pred(number_between(1.0, 10000.0)),
        ],
        |ts| double(num_value(ts.first()?)? + num_value(ts.get(1)?)?),
    )
}

fn rule_integer9() -> Rule {
    rule("integer (1000)", vec![regex("千")], |_| integer(1000))
}

static INTEGER_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("零", 0),
        ("ゼロ", 0),
        ("一", 1),
        ("二", 2),
        ("三", 3),
        ("四", 4),
        ("五", 5),
        ("六", 6),
        ("七", 7),
        ("八", 8),
        ("九", 9),
        ("十", 10),
    ]
    .into_iter()
    .collect()
});

fn rule_integer() -> Rule {
    rule(
        "integer (0..10)",
        vec![regex("(ゼロ|零|一|二|三|四|五|六|七|八|九|十)")],
        |ts| {
            let m = to_lower(groups(ts)?.first()?);
            INTEGER_MAP.get(m.as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_integer4() -> Rule {
    rule(
        "integer 21..99",
        vec![
            one_of(vec![70.0, 20.0, 60.0, 50.0, 40.0, 90.0, 30.0, 80.0]),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| double(num_value(ts.first()?)? + num_value(ts.get(1)?)?),
    )
}

fn rule_integer2() -> Rule {
    rule(
        "integer (11..19)",
        vec![regex("十"), pred(number_between(1.0, 10.0))],
        |ts| double(num_value(ts.get(1)?)? + 10.0),
    )
}

fn rule_integer11() -> Rule {
    rule(
        "integer (2000..9000)",
        vec![pred(number_between(2.0, 10.0)), regex("千")],
        |ts| double(num_value(ts.first()?)? * 1000.0),
    )
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule(
        "integer with thousands separator ,",
        vec![regex(r"(\d{1,3}(,\d\d\d){1,5})")],
        |ts| {
            let m = groups(ts)?.first()?;
            parse_double(&m.replace(',', "")).and_then(double)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_integer(),
        rule_integer10(),
        rule_integer11(),
        rule_integer12(),
        rule_integer13(),
        rule_integer14(),
        rule_integer15(),
        rule_integer16(),
        rule_integer2(),
        rule_integer3(),
        rule_integer4(),
        rule_integer5(),
        rule_integer6(),
        rule_integer7(),
        rule_integer8(),
        rule_integer9(),
        rule_integer_with_thousands_separator(),
        rule_numeral(),
        rule_numerals_prefix_with_negative_or_minus(),
        rule_numerals_suffixes_kmg(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
