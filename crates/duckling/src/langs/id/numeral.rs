//! `Duckling.Numeral.ID.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, group, groups, pred, regex, rule, Dim, Rule, Token};

static INTEGER_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("kosong", 0),
        ("nol", 0),
        ("satu", 1),
        ("dua", 2),
        ("tiga", 3),
        ("empat", 4),
        ("lima", 5),
        ("enam", 6),
        ("tujuh", 7),
        ("delapan", 8),
        ("sembilan", 9),
        ("sebelas", 11),
    ]
    .into_iter()
    .collect()
});

fn rule_teen() -> Rule {
    rule("teen", vec![pred(number_between(2.0, 10.0)), regex("belas")], |ts| match ts.first()? {
        Token::Numeral(nd) => double(nd.value + 10.0),
        _ => None,
    })
}

fn rule_numeral_comma_numeral() -> Rule {
    rule(
        "number comma number",
        vec![dimension(Dim::Numeral), regex("koma"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(nd1), Token::Numeral(nd2)) => {
                double(nd1.value + decimals_to_double(nd2.value))
            }
            _ => None,
        },
    )
}

fn rule_numerals_prefix_with_negative_or_minus() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex("-|minus|negatif"), pred(is_positive)],
        |ts| match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * -1.0),
            _ => None,
        },
    )
}

fn rule_ten() -> Rule {
    rule("ten", vec![regex("(se)?puluh")], |_| integer(10).and_then(|t| with_grain(1, &t)))
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule("decimal with thousands separator", vec![regex(r"(\d+(\.\d\d\d)+,\d+)")], |ts| {
        let g = groups(ts)?;
        let fmt = g.first()?.replace('.', "").replace(',', ".");
        parse_double(&fmt).and_then(double)
    })
}

fn rule_multiply() -> Rule {
    rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
        multiply(ts.first()?, ts.get(1)?)
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
            one_of(vec![70.0, 20.0, 60.0, 50.0, 40.0, 90.0, 30.0, 80.0]),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(v1), Token::Numeral(v2)) => double(v1.value + v2.value),
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

fn rule_somefewcouple() -> Rule {
    rule("some/few/couple", vec![regex("beberapa")], |_| integer(3))
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

fn rule_powers_of_ten() -> Rule {
    rule("powers of tens", vec![regex("(se)?(ratus|ribu|juta)")], |ts| {
        match to_lower(group(ts, 1)?.as_ref()).as_str() {
            "ratus" => double(1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t)),
            "ribu" => double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t)),
            "juta" => double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t)),
            _ => None,
        }
    })
}

fn rule_dozen() -> Rule {
    rule("dozen", vec![regex("(se)?lusin")], |_| {
        integer(12).and_then(|t| with_grain(1, &t)).and_then(|t| with_multipliable(&t))
    })
}

fn rule_integer() -> Rule {
    rule(
        "integer (0..9 11)",
        vec![regex("(kosong|nol|satu|dua|tiga|empat|lima|enam|tujuh|delapan|sembilan|sebelas)")],
        |ts| {
            let g = groups(ts)?;
            INTEGER_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_integer2() -> Rule {
    rule(
        "integer 20..90",
        vec![pred(number_between(2.0, 10.0)), number_with(|n| n.value, |v| v == 10.0)],
        |ts| match (ts.first()?, ts.get(1)?) {
            (
                Token::Numeral(v1),
                Token::Numeral(NumeralData { value: v2, grain: Some(g), .. }),
            ) => double(v1.value * v2).and_then(|t| with_grain(*g, &t)),
            _ => None,
        },
    )
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule("integer with thousands separator .", vec![regex(r"(\d{1,3}(\.\d\d\d){1,5})")], |ts| {
        let fmt = groups(ts)?.first()?.replace('.', "");
        parse_double(&fmt).and_then(double)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_dozen(),
        rule_integer(),
        rule_integer2(),
        rule_integer3(),
        rule_integer_with_thousands_separator(),
        rule_intersect(),
        rule_multiply(),
        rule_numeral_comma_numeral(),
        rule_numerals_prefix_with_negative_or_minus(),
        rule_numerals_suffixes_kmg(),
        rule_powers_of_ten(),
        rule_somefewcouple(),
        rule_teen(),
        rule_ten(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
