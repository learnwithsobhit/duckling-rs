//! `Duckling.Numeral.KN.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::*;
use crate::dims::util::parse_int;
use crate::types::{groups, regex, rule, Rule};

/// `ankiMap` / `ankiToArab`.
fn anki_to_arab(c: char) -> char {
    match c {
        '೦' => '0',
        '೧' => '1',
        '೨' => '2',
        '೩' => '3',
        '೪' => '4',
        '೫' => '5',
        '೬' => '6',
        '೭' => '7',
        '೮' => '8',
        '೯' => '9',
        _ => c,
    }
}

fn rule_anki() -> Rule {
    rule(
        "anki forms",
        vec![regex("([೦೧೨೩೪೫೬೭೮೯]{1,10})")],
        |ts| {
            let mapped: String = groups(ts)?.first()?.chars().map(anki_to_arab).collect();
            parse_int(&mapped).and_then(integer)
        },
    )
}

/// `constructRule` for `ruleNumeralMap` entries.
fn construct_rule(s: &'static str, i: i64) -> Rule {
    rule(&format!("number: {s}"), vec![regex(s)], move |_| integer(i))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut out = vec![rule_anki()];
    out.extend(
        [
            ("ಸೊನ್ನೆ", 0),
            ("ಒಂದು", 1),
            ("ಎರಡು", 2),
            ("ಮೂರು", 3),
            ("ನಾಲ್ಕು", 4),
            ("ಐದು", 5),
            ("ಆರು", 6),
            ("ಏಳು", 7),
            ("ಎಂಟು", 8),
            ("ಒಂಬತ್ತು", 9),
        ]
        .into_iter()
        .map(|(s, i)| construct_rule(s, i)),
    );
    out
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
