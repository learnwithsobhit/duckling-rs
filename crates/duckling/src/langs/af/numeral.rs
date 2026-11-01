//! `Duckling.Numeral.AF.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, group, pred, regex, rule, Dim, Rule, Token};

/// `zeroAndTenMap`.
static ZERO_AND_TEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    HashMap::from([
        ("nul", 0),
        ("geen", 0),
        ("niks", 0),
        ("zero", 0),
        ("tien", 10),
    ])
});

/// `oneToNineMap`.
static ONE_TO_NINE: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    HashMap::from([
        ("een", 1),
        ("twee", 2),
        ("drie", 3),
        ("vier", 4),
        ("vyf", 5),
        ("ses", 6),
        ("sewe", 7),
        ("agt", 8),
        ("ag", 8),
        ("nege", 9),
    ])
});

/// `elevenToNineteenMap`.
static ELEVEN_TO_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    HashMap::from([
        ("elf", 11),
        ("twaalf", 12),
        ("dertien", 13),
        ("veertien", 14),
        ("vyftien", 15),
        ("sestien", 16),
        ("sewentien", 17),
        ("agtien", 18),
        ("negentien", 19),
        ("neentien", 19),
    ])
});

/// `tensMap`.
static TENS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    HashMap::from([
        ("twin", 20),
        ("der", 30),
        ("veer", 40),
        ("vyf", 50),
        ("ses", 60),
        ("sewen", 70),
        ("tag", 80),
        ("negen", 90),
        ("neen", 90),
    ])
});

fn lookup_int(map: &HashMap<&'static str, i64>, s: &str) -> Option<i64> {
    map.get(s).copied()
}

fn nv(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

fn rule_numeral() -> Rule {
    rule(
        "number (0..10)",
        vec![regex("(nul|geen|niks|zero|tien|een|twee|drie|vier|vyf|ses|sewe|agt?|nege)")],
        |ts| {
            let x = to_lower(&group(ts, 0)?);
            lookup_int(&ONE_TO_NINE, &x)
                .or_else(|| lookup_int(&ZERO_AND_TEN, &x))
                .and_then(integer)
        },
    )
}

fn rule_eleven_to_nineteen() -> Rule {
    rule(
        "number (11..19)",
        vec![regex("(elf|twaalf|dertien|veertien|vyftien|sestien|sewentien|agtien|neg?entien)")],
        |ts| lookup_int(&ELEVEN_TO_NINETEEN, &to_lower(&group(ts, 0)?)).and_then(integer),
    )
}

fn rule_tens() -> Rule {
    rule(
        "integer (20,30..90)",
        vec![regex("(twin|der|veer|vyf|ses|sewen|tag|neg?en)tig")],
        |ts| lookup_int(&TENS, &to_lower(&group(ts, 0)?)).and_then(integer),
    )
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer ([3-9][1-9])",
        vec![
            pred(number_between(1.0, 10.0)),
            regex("en"),
            one_of((2..=9).map(|i| (i * 10) as f64).collect()),
        ],
        |ts| double(nv(ts.get(2)?)? + nv(ts.first()?)?),
    )
}

fn rule_decimals() -> Rule {
    rule("decimal number", vec![regex(r"(\d*,\d+)")], |ts| {
        parse_decimal(false, &group(ts, 0)?)
    })
}

fn rule_powers_of_ten() -> Rule {
    rule("powers of tens", vec![regex("(honderd|duisend)")], |ts| {
        let (v, grain) = match to_lower(&group(ts, 0)?).as_str() {
            "honderd" => (1e2, 2),
            "duisend" => (1e3, 3),
            _ => return None,
        };
        double(v).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
    })
}

fn rule_dozen() -> Rule {
    rule("a dozen of", vec![regex("dosyn")], |_| {
        integer(12).and_then(|t| with_multipliable(&t)).and_then(|t| not_ok_for_any_time(&t))
    })
}

fn rule_sum() -> Rule {
    rule(
        "intersect 2 numbers",
        vec![
            pred(|t| has_grain(t) && is_positive(t)),
            pred(|t| !is_multipliable(t) && is_positive(t)),
        ],
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
    rule(
        "compose by multiplication",
        vec![dimension(Dim::Numeral), pred(is_multipliable)],
        |ts| multiply(ts.first()?, ts.get(1)?),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_numeral(),
        rule_dozen(),
        rule_decimals(),
        rule_powers_of_ten(),
        rule_eleven_to_nineteen(),
        rule_tens(),
        rule_composite_tens(),
        rule_sum(),
        rule_multiply(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
