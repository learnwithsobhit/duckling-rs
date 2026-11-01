//! `Duckling.Numeral.FR.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn rule_numerals4() -> Rule {
    rule(
        "numbers 81",
        vec![number_with(|n| n.value, |v| v == 80.0), number_with(|n| n.value, |v| v == 1.0)],
        |_| integer(81),
    )
}

fn rule_numerals_prefix_with_negative_or_minus() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex("-|moins"), dimension(Dim::Numeral)],
        |ts| match ts.get(1)? {
            Token::Numeral(n) => double(n.value * -1.0),
            _ => None,
        },
    )
}

fn sum2(ts: &[&Token]) -> Option<Token> {
    match (ts.first()?, ts.get(2)?) {
        (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
        _ => None,
    }
}

fn rule_numerals2() -> Rule {
    rule(
        "numbers 22..29 32..39 .. 52..59",
        vec![
            one_of(vec![20.0, 50.0, 40.0, 30.0]),
            regex(r"[\s\-]+"),
            pred(number_between(2.0, 10.0)),
        ],
        sum2,
    )
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule(
        "decimal with thousands separator",
        vec![regex(r"(\d+(([\. ])\d\d\d)+,\d+)")],
        |ts| {
            let g = groups(ts)?;
            let m = g.first()?;
            let sep = g.get(2)?;
            let fmt = m.replace(&**sep, "").replace(',', ".");
            parse_double(&fmt).and_then(double)
        },
    )
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex(r"(\d*,\d+)")], |ts| {
        let g = groups(ts)?;
        parse_decimal(false, g.first()?)
    })
}

static NUMERAL2_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [("vingt", 20), ("trente", 30), ("quarante", 40), ("cinquante", 50), ("soixante", 60)]
        .into_iter()
        .collect()
});

fn rule_numeral2() -> Rule {
    rule("number (20..60)", vec![regex("(vingt|trente|quarante|cinquante|soixante)")], |ts| {
        let g = groups(ts)?;
        NUMERAL2_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
    })
}

fn rule_numerals5() -> Rule {
    rule(
        "numbers 62..69 .. 92..99",
        vec![one_of(vec![60.0, 80.0]), regex(r"[\s\-]+"), pred(number_between(2.0, 20.0))],
        sum2,
    )
}

static NUMERAL_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("zero", 0), ("zéro", 0), ("un", 1), ("une", 1), ("deux", 2), ("trois", 3),
        ("quatre", 4), ("cinq", 5), ("six", 6), ("sept", 7), ("huit", 8), ("neuf", 9),
        ("dix", 10), ("onze", 11), ("douze", 12), ("treize", 13), ("quatorze", 14),
        ("quinze", 15), ("seize", 16),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral() -> Rule {
    rule(
        "number (0..16)",
        vec![regex("(z(e|é)ro|une?|deux|trois|quatre|cinq|six|sept|huit|neuf|dix|onze|douze|treize|quatorze|quinze|seize)")],
        |ts| {
            let g = groups(ts)?;
            NUMERAL_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_numeral3() -> Rule {
    rule(
        "number (17..19)",
        vec![
            number_with(|n| n.value, |v| v == 10.0),
            regex(r"[\s\-]+"),
            pred(number_between(7.0, 10.0)),
        ],
        |ts| match ts.get(2)? {
            Token::Numeral(n) => double(10.0 + n.value),
            _ => None,
        },
    )
}

fn rule_numerals3() -> Rule {
    rule(
        "numbers 61 71",
        vec![
            number_with(|n| n.value, |v| v == 60.0),
            regex("-?et-?"),
            one_of(vec![1.0, 11.0]),
        ],
        sum2,
    )
}

fn rule_numerals_suffixes_kmg() -> Rule {
    rule(
        "numbers suffixes (K, M, G)",
        vec![dimension(Dim::Numeral), regex(r"([kmg])(?=[\W$€¢£]|$)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(nd), Token::RegexMatch(g)) => {
                match to_lower(g.0.first()?).as_str() {
                    "k" => double(nd.value * 1e3),
                    "m" => double(nd.value * 1e6),
                    "g" => double(nd.value * 1e9),
                    _ => None,
                }
            }
            _ => None,
        },
    )
}

fn rule_numeral4() -> Rule {
    rule("number 80", vec![regex("quatre"), regex("vingts?")], |_| integer(80))
}

fn rule_numerals() -> Rule {
    rule(
        "numbers 21 31 41 51",
        vec![
            one_of(vec![20.0, 50.0, 40.0, 30.0]),
            regex("-?et-?"),
            number_with(|n| n.value, |v| v == 1.0),
        ],
        sum2,
    )
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule(
        "integer with thousands separator .",
        vec![regex(r"(\d{1,3}(([\. ])\d\d\d){1,5})")],
        |ts| {
            let g = groups(ts)?;
            let m = g.first()?;
            let sep = g.get(2)?;
            parse_double(&m.replace(&**sep, "")).and_then(double)
        },
    )
}

fn rule_powers_of_ten() -> Rule {
    rule("powers of tens", vec![regex("(cent|mille|millions?|milliards?)")], |ts| {
        let g = groups(ts)?;
        let (v, grain) = match to_lower(g.first()?).as_str() {
            "cent" => (1e2, 2),
            "mille" => (1e3, 3),
            "million" | "millions" => (1e6, 6),
            "milliard" | "milliards" => (1e9, 9),
            _ => return None,
        };
        double(v).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
    })
}

fn rule_sum() -> Rule {
    rule(
        "intersect 2 numbers",
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

fn rule_multiply() -> Rule {
    rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
        multiply(ts.first()?, ts.get(1)?)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_integer_with_thousands_separator(),
        rule_numeral(),
        rule_numeral2(),
        rule_numeral3(),
        rule_numeral4(),
        rule_numerals(),
        rule_numerals2(),
        rule_numerals3(),
        rule_numerals4(),
        rule_numerals5(),
        rule_numerals_prefix_with_negative_or_minus(),
        rule_numerals_suffixes_kmg(),
        rule_powers_of_ten(),
        rule_sum(),
        rule_multiply(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
