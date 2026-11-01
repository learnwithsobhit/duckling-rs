//! `Duckling.Numeral.CA.Rules`.

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

static ZERO_TO_FIFTEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("zero", 0),
        ("u", 1),
        ("un", 1),
        ("una", 1),
        ("dos", 2),
        ("dues", 2),
        ("tres", 3),
        ("quatre", 4),
        ("cinc", 5),
        ("sis", 6),
        ("set", 7),
        ("vuit", 8),
        ("nou", 9),
        ("deu", 10),
        ("onze", 11),
        ("dotze", 12),
        ("tretze", 13),
        ("catorze", 14),
        ("quinze", 15),
    ]
    .into_iter()
    .collect()
});

fn rule_zero_to_fifteen() -> Rule {
    rule(
        "number (0..15)",
        vec![regex(
            "(zero|u(na|n)?|d(o|ue)s|tres|quatre|cinc|sis|set|vuit|nou|deu|onze|dotze|tretze|catorze|quinze)",
        )],
        |ts| {
            let g = groups(ts)?;
            ZERO_TO_FIFTEEN.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_numerals_prefix_with_negative_or_minus() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex("-|menys"), pred(is_positive)],
        |ts| double(num_value(ts.get(1)?)? * -1.0),
    )
}

static TENS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("vint", 20),
        ("trenta", 30),
        ("quaranta", 40),
        ("cinquanta", 50),
        ("seixanta", 60),
        ("setanta", 70),
        ("vuitanta", 80),
        ("noranta", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_tens() -> Rule {
    rule(
        "number (20..90)",
        vec![regex("(vint|(tre|quara|cinqua|seixa|seta|vuita|nora)nta)")],
        |ts| {
            let g = groups(ts)?;
            TENS.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

static SIXTEEN_TO_TWENTY_NINE: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("setze", 16),
        ("disset", 17),
        ("dèsset", 17),
        ("devuit", 18),
        ("divuit", 18),
        ("dihuit", 18),
        ("dinou", 19),
        ("dènou", 19),
        ("denou", 19),
        ("vint-i-u", 21),
        ("vint-i-una", 21),
        ("vint-i-dos", 22),
        ("vint-i-tres", 23),
        ("vint-i-quatre", 24),
        ("vint-i-cinc", 25),
        ("vint-i-sis", 26),
        ("vint-i-set", 27),
        ("vint-i-vuit", 28),
        ("vint-i-nou", 29),
    ]
    .into_iter()
    .collect()
});

fn rule_lower_tens_with_ones() -> Rule {
    rule(
        "number (16..19 21..29)",
        vec![regex(
            "(setze|d(i|e|è)sset|d(e|i)(v|h)uit|d(i|e|è)nou|vint-i-u(na)?|vint-i-dos|vint-i-tres|vint-i-quatre|vint-i-cinc|vint-i-sis|vint-i-set|vint-i-vuit|vint-i-nou)",
        )],
        |ts| {
            let g = groups(ts)?;
            SIXTEEN_TO_TWENTY_NINE.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_higher_tens_with_ones() -> Rule {
    rule(
        "number (31..39 41..49 51..59 61..69 71..79 81..89 91..99)",
        vec![one_of(vec![30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]), regex("-"), pred(number_between(1.0, 9.0))],
        |ts| double(num_value(ts.first()?)? + num_value(ts.get(2)?)?),
    )
}

fn rule_numerals_suffixes_kmg() -> Rule {
    rule(
        "numbers suffixes (K, M, G)",
        vec![dimension(Dim::Numeral), regex("([kmg])(?=[\\W\\$€]|$)")],
        |ts| {
            let v = num_value(ts.first()?)?;
            match ts.get(1)? {
                Token::RegexMatch(g) => match to_lower(g.0.first()?).as_str() {
                    "k" => double(v * 1e3),
                    "m" => double(v * 1e6),
                    "g" => double(v * 1e9),
                    _ => None,
                },
                _ => None,
            }
        },
    )
}

static ONE_HUNDRED_TO_THOUSAND: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("cent", 100),
        ("cents", 100),
        ("dos-cents", 200),
        ("tres-cents", 300),
        ("quatre-cents", 400),
        ("cinc-cents", 500),
        ("sis-cents", 600),
        ("set-cents", 700),
        ("vuit-cents", 800),
        ("nou-cents", 900),
        ("mil", 1000),
    ]
    .into_iter()
    .collect()
});

fn rule_twenties() -> Rule {
    rule(
        "number (21..29)",
        vec![one_of(vec![20.0]), regex("(-i-| i )"), pred(number_between(1.0, 10.0))],
        |ts| double(num_value(ts.first()?)? + num_value(ts.get(2)?)?),
    )
}

fn rule_hundreds() -> Rule {
    rule(
        "number 100..1000 ",
        vec![regex(
            "(cent(s)?|dos-cents|tres-cents|quatre-cents|cinc-cents|sis-cents|set-cents|vuit-cents|nou-cents|mil)",
        )],
        |ts| {
            let g = groups(ts)?;
            ONE_HUNDRED_TO_THOUSAND.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

/// Upstream `ruleNumerals` binds `v2` to the `== 100` token (third item), not the 0..99 remainder.
fn rule_numerals() -> Rule {
    rule(
        "numbers 200..999",
        vec![
            pred(number_between(2.0, 10.0)),
            regex("-"),
            number_with(|n| n.value, |v| v == 100.0),
            pred(number_between(0.0, 100.0)),
        ],
        |ts| double(100.0 * num_value(ts.first()?)? + num_value(ts.get(2)?)?),
    )
}

fn rule_numeral_dot_numeral() -> Rule {
    rule(
        "number dot number",
        vec![dimension(Dim::Numeral), regex("coma"), pred(|t| !has_grain(t))],
        |ts| double(num_value(ts.first()?)? + decimals_to_double(num_value(ts.get(2)?)?)),
    )
}

fn rule_below_ten_with_two_digits() -> Rule {
    rule(
        "integer (0-9) with two digits",
        vec![regex("zero|0"), pred(number_between(1.0, 10.0))],
        |ts| double(num_value(ts.get(1)?)?),
    )
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule(
        "decimal with thousands separator .",
        vec![regex("(\\d+(\\.\\d\\d\\d)+,\\d+)")],
        |ts| {
            let g = groups(ts)?;
            let fmt = g.first()?.replace('.', "").replace(',', ".");
            parse_double(&fmt).and_then(double)
        },
    )
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number ,", vec![regex("(\\d*,\\d+)")], |ts| {
        let g = groups(ts)?;
        parse_decimal(false, g.first()?)
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
        rule_below_ten_with_two_digits(),
        rule_zero_to_fifteen(),
        rule_tens(),
        rule_twenties(),
        rule_lower_tens_with_ones(),
        rule_higher_tens_with_ones(),
        rule_hundreds(),
        rule_numeral_dot_numeral(),
        rule_numerals(),
        rule_numerals_prefix_with_negative_or_minus(),
        rule_numerals_suffixes_kmg(),
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_integer_with_thousands_separator(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
