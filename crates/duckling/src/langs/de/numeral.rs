//! `Duckling.Numeral.DE.Rules`.

use once_cell::sync::Lazy;

use super::num_parser::parse_numeral;
use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, group, groups, pred, regex, rule, Dim, Rule, Token};

fn rule_numerals_prefix_with_negative_or_minus() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex("-|minus|negativ"), pred(is_positive)],
        |ts| match ts.get(1)? {
            Token::Numeral(n) => double(n.value * -1.0),
            _ => None,
        },
    )
}

fn rule_few() -> Rule {
    rule("few", vec![regex("mehrere")], |_| integer(3))
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule(
        "decimal with thousands separator",
        vec![regex(r"(\d+(\.\d\d\d)+\,\d+)")],
        |ts| {
            let m = group(ts, 0)?;
            let fmt = m.replace('.', "").replace(',', ".");
            parse_double(&fmt).and_then(double)
        },
    )
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex(r"(\d*,\d+)")], |ts| {
        let m = group(ts, 0)?;
        parse_decimal(false, &m)
    })
}

fn rule_numerals_und() -> Rule {
    rule(
        "numbers und",
        vec![
            pred(number_between(1.0, 10.0)),
            regex("und"),
            one_of((2..=9).map(|i| (i * 10) as f64).collect()),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
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

fn rule_numerals_suffixes_kmg() -> Rule {
    rule(
        "numbers suffixes (K, M, G)",
        vec![dimension(Dim::Numeral), regex(r"([kmg])(?=[\W\$€]|$)")],
        |ts| match ts.first()? {
            Token::Numeral(n) => {
                let m = match ts.get(1)? {
                    Token::RegexMatch(g) => g.0.first()?.clone(),
                    _ => return None,
                };
                match to_lower(&m).as_str() {
                    "k" => double(n.value * 1e3),
                    "m" => double(n.value * 1e6),
                    "g" => double(n.value * 1e9),
                    _ => None,
                }
            }
            _ => None,
        },
    )
}

fn rule_couple() -> Rule {
    rule("couple", vec![regex("(ein )?paar")], |_| integer(2))
}

fn rule_dozen() -> Rule {
    rule("dozen", vec![regex("dutzend")], |_| {
        integer(12).and_then(|t| with_grain(1, &t)).and_then(|t| with_multipliable(&t))
    })
}

fn rule_powers_of_ten() -> Rule {
    rule(
        "powers of tens",
        vec![regex("(hunderte?|tausende?|million(en)?|milliarde(n)?|billion(en)?|billiarde(n)?)")],
        |ts| {
            let g = groups(ts)?;
            let (v, grain) = match to_lower(g.first()?).as_str() {
                "hundert" | "hunderte" => (1e2, 2),
                "tausend" | "tausende" => (1e3, 3),
                "million" | "millionen" => (1e6, 6),
                "milliarde" | "milliarden" => (1e9, 9),
                "billion" | "billionen" => (1e12, 12),
                "billiarde" | "billiarden" => (1e15, 15),
                _ => return None,
            };
            double(v).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
        },
    )
}

fn rule_zero() -> Rule {
    rule("integer 0", vec![regex("(keine(m|n|r|s)?|keins?|null|nichts)")], |_| integer(0))
}

fn rule_numeral_dot_numeral() -> Rule {
    rule(
        "number dot number",
        vec![dimension(Dim::Numeral), regex("komma"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + decimals_to_double(b.value)),
            _ => None,
        },
    )
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule(
        "integer with thousands separator .",
        vec![regex(r"(\d{1,3}(\.\d\d\d){1,5})")],
        |ts| {
            let m = group(ts, 0)?;
            parse_double(&m.replace('.', "")).and_then(double)
        },
    )
}

fn rule_all_numeral_words() -> Rule {
    rule(
        "simple and complex numerals written as one word",
        vec![regex("(ein|zwei|drei|vier|fünf|sech|sieb|acht|neun|zehn|elf|zwölf|hundert|tausend)?([^\\s]+)?(eine[m|n|r|s]?|eins?|zwei|drei|vier|fünf|sechs|sieben|acht|neun|zehn|elf|zwölf|[s|ß|z]ig|hundert|tausend)")],
        |ts| {
            let g = groups(ts)?;
            let s: String = g.iter().map(|m| to_lower(m)).collect();
            parse_numeral(&s).and_then(integer)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_couple(),
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_dozen(),
        rule_few(),
        rule_integer_with_thousands_separator(),
        rule_intersect(),
        rule_multiply(),
        rule_numeral_dot_numeral(),
        rule_numerals_prefix_with_negative_or_minus(),
        rule_numerals_suffixes_kmg(),
        rule_numerals_und(),
        rule_powers_of_ten(),
        rule_zero(),
        rule_all_numeral_words(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
