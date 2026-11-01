//! `Duckling.Numeral.SW.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{groups, pred, regex, rule, Rule, Token};

/// `ruleNumeralMap`.
static RULE_NUMERAL_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("sufuri", 0),
        ("zero", 0),
        ("moja", 1),
        ("mbili", 2),
        ("tatu", 3),
        ("nne", 4),
        ("tano", 5),
        ("sita", 6),
        ("saba", 7),
        ("nane", 8),
        ("tisa", 9),
        ("kumi", 10),
    ]
    .into_iter()
    .collect()
});

/// `tensMap`.
static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ishirini", 20),
        ("thelathini", 30),
        ("arubaini", 40),
        ("arobaini", 40),
        ("hamsini", 50),
        ("sitini", 60),
        ("sabini", 70),
        ("themanini", 80),
        ("tisini", 90),
    ]
    .into_iter()
    .collect()
});

fn lookup_int(map: &HashMap<&'static str, i64>, ts: &[&Token]) -> Option<Token> {
    map.get(to_lower(groups(ts)?.first()?).as_str()).and_then(|v| integer(*v))
}

fn rule_numeral() -> Rule {
    rule(
        "number (0..10)",
        vec![regex("(sufuri|zero|moja|mbili|tatu|nne|tano|sita|saba|nane|tisa|kumi)")],
        |ts| lookup_int(&RULE_NUMERAL_MAP, ts),
    )
}

fn rule_tens() -> Rule {
    rule(
        "integer (20,30..90)",
        vec![regex(
            "(ishirini|thelathini|arubaini|arobaini|hamsini|sitini|sabini|themanini|tisini)",
        )],
        |ts| lookup_int(&TENS_MAP, ts),
    )
}

fn rule_composite_tens() -> Rule {
    // Upstream prod matches tokens[0], tokens[1] as Numeral (skips the regex incorrectly).
    rule(
        "integer 11..19 21..29 .. 91..99",
        vec![
            one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
            regex("-?na-?"),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(v1), Token::Numeral(v2)) => double(v1.value + v2.value),
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![rule_numeral(), rule_tens(), rule_composite_tens()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
