//! `Duckling.Numeral.CS.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{groups, regex, rule, Rule};

static NUMERAL_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("nula", 0),
        ("jeden", 1),
        ("jedna", 1),
        ("jedno", 1),
        ("dva", 2),
        ("dvĕ", 2),
        ("tři", 3),
        ("čtyři", 4),
        ("pĕt", 5),
        ("šest", 6),
        ("sedm", 7),
        ("osm", 8),
        ("devĕt", 9),
        ("deset", 10),
    ]
    .into_iter()
    .collect()
});

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleNumeral
        rule(
            "number (0..10)",
            vec![regex(
                "(nula|jed(en|n[ao])|dv(a|ĕ)|t(ř)i|(č)ty(ř)i|p(ĕ)t|(š)est|sedm|osm|dev(ĕ)t|deset)",
            )],
            |ts| {
                NUMERAL_MAP
                    .get(to_lower(groups(ts)?.first()?).as_str())
                    .and_then(|v| integer(*v))
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
