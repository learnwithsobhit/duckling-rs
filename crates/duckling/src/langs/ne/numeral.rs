//! `Duckling.Numeral.NE.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{groups, regex, rule, Rule};

static ZERO_TO_NINETEEN_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("शुन्य", 0),
        ("सुन्ना", 0),
        ("एक", 1),
        ("दुई", 2),
        ("तीन", 3),
        ("चार", 4),
        ("पाँच", 5),
        ("छ", 6),
        ("सात", 7),
        ("आठ", 8),
        ("नौ", 9),
        ("दश", 10),
        ("एघार", 11),
        ("बाह्र", 12),
        ("तेह्र", 13),
        ("चौध", 14),
        ("पन्ध्र", 15),
        ("सोह्र", 16),
        ("सत्र", 17),
        ("अठार", 18),
        ("उन्नाइस", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_to_nineteen() -> Rule {
    rule(
        "integer (0..19)",
        vec![regex(
            "(शुन्य|सुन्ना|एक|दुई|तीन|चार|पाँच|छ|सात|आठ|नौ|दश|एघार|बाह्र|तेह्र|चौध|पन्ध्र|सोह्र|सत्र|अठार|उन्नाइस)",
        )],
        |ts| {
            ZERO_TO_NINETEEN_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(integer)
        },
    )
}

static TWENTYONE_TO_TWENTYNINE_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("एक्काइस", 21),
        ("बाइस", 22),
        ("तेइस", 23),
        ("चौबिस", 24),
        ("पच्चिस", 25),
        ("छब्बिस", 26),
        ("सत्ताइस", 27),
        ("अट्ठाइस", 28),
        ("उनन्तिस", 29),
    ]
    .into_iter()
    .collect()
});

fn rule_twentyone_to_twentynine() -> Rule {
    rule(
        "number (21..29)",
        vec![regex("(एक्काइस|बाइस|तेइस|चौबिस|पच्चिस|छब्बिस|सत्ताइस|अट्ठाइस|उनन्तिस)")],
        |ts| {
            TWENTYONE_TO_TWENTYNINE_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(integer)
        },
    )
}

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("बिस", 20),
        ("तिस", 30),
        ("चालिस", 40),
        ("पचास", 50),
        ("साठी", 60),
        ("सत्तरी", 70),
        ("असी", 80),
        ("नब्बे", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_tens() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(बिस|तिस|चालिस|पचास|साठी|सत्तरी|असी|नब्बे)")],
        |ts| {
            TENS_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(integer)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule_to_nineteen(), rule_twentyone_to_twentynine(), rule_tens()]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
