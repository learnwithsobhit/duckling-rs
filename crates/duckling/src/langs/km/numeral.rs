//! `Duckling.Numeral.KM.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

static NUMERAL_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("០", 0),
        ("១", 1),
        ("២", 2),
        ("៣", 3),
        ("៤", 4),
        ("៥", 5),
        ("៦", 6),
        ("៧", 7),
        ("៨", 8),
        ("៩", 9),
        ("សូន្យ", 0),
        ("មួយ", 1),
        ("ពីរ", 2),
        ("បី", 3),
        ("បួន", 4),
        ("ប្រាំ", 5),
        ("ប្រាំមួយ", 6),
        ("ប្រាំពីរ", 7),
        ("ប្រាំបី", 8),
        ("ប្រាំបួន", 9),
    ]
    .into_iter()
    .collect()
});

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ដប់", 10),
        ("ម្ភៃ", 20),
        ("សាម", 30),
        ("សែ", 40),
        ("ហា", 50),
        ("ហុក", 60),
        ("ចិត", 70),
        ("ប៉ែត", 80),
        ("កៅ", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral() -> Rule {
    rule(
        "integer (0..9)",
        vec![regex("(០|១|២|៣|៤|៥|៦|៧|៨|៩|ប្រាំបួន|ប្រាំបី|ប្រាំពីរ|ប្រាំមួយ|ប្រាំ|បួន|បី|ពីរ|មួយ|សូន្យ)")],
        |ts| {
            let g = groups(ts)?;
            NUMERAL_MAP.get(g.first()?.as_ref()).and_then(|v| integer(*v))
        },
    )
}

fn rule_tens() -> Rule {
    rule("integer (10..90)", vec![regex("(កៅ|ប៉ែត|ចិត|ហុក|ហា|សែ|សាម|ម្ភៃ|ដប់)")], |ts| {
        let g = groups(ts)?;
        TENS_MAP.get(g.first()?.as_ref()).and_then(|v| integer(*v))
    })
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer (11..99)",
        vec![
            one_of(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
            one_of(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(v1), Token::Numeral(v2)) => double(v1.value + v2.value),
            _ => None,
        },
    )
}

fn rule_powers_of_ten() -> Rule {
    rule("powers of tens", vec![regex("(រយ|ពាន់|ម៉ឺន|សែន|លាន)")], |ts| {
        let g = groups(ts)?;
        match g.first()?.as_ref() {
            "រយ" => double(1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t)),
            "ពាន់" => double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t)),
            "ម៉ឺន" => double(1e4).and_then(|t| with_grain(4, &t)).and_then(|t| with_multipliable(&t)),
            "សែន" => double(1e5).and_then(|t| with_grain(5, &t)).and_then(|t| with_multipliable(&t)),
            "លាន" => double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t)),
            _ => None,
        }
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
            (Token::Numeral(n1), Token::Numeral(n2)) => {
                let g = n1.grain?;
                if 10f64.powf(g as f64) > n2.value {
                    double(n1.value + n2.value)
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
        rule_numeral(),
        rule_tens(),
        rule_composite_tens(),
        rule_powers_of_ten(),
        rule_sum(),
        rule_multiply(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
