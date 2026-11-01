//! `Duckling.Ordinal.HI.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

static ORDINALS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("शून्य", 0),
        ("प्रथम", 1),
        ("पहला", 1),
        ("पहली", 1),
        ("पहले", 1),
        ("द्वितीय", 2),
        ("दूसरा", 2),
        ("दूसरी", 2),
        ("दूसरे", 2),
        ("तृतीय", 3),
        ("तीसरा", 3),
        ("तीसरी", 3),
        ("तीसरे", 3),
        ("चौथा", 4),
        ("चौथी", 4),
        ("चौथे", 4),
        ("छठा", 6),
        ("छठी", 6),
        ("छठे", 6),
    ]
    .into_iter()
    .collect()
});

static CARDINALS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("पाँच", 5),
        ("सात", 7),
        ("आठ", 8),
        ("नौ", 9),
        ("दस", 10),
        ("ग्यारह", 11),
        ("बारह", 12),
        ("तेरह", 13),
        ("चौदह", 14),
        ("पन्द्रह", 15),
        ("सोलह", 16),
        ("सत्रह", 17),
        ("अठारह", 18),
        ("उन्नीस", 19),
        ("बीस", 20),
        ("इक्कीस", 21),
        ("बाईस", 22),
        ("तेईस", 23),
        ("चौबीस", 24),
        ("पच्चीस", 25),
        ("छब्बीस", 26),
        ("सत्ताईस", 27),
        ("अट्ठाईस", 28),
        ("उनतीस", 29),
        ("तीस", 30),
        ("चालीस", 40),
        ("पचास", 50),
        ("साठ", 60),
        ("सत्तर", 70),
        ("अस्सी", 80),
        ("नब्बे", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_ordinals() -> Rule {
    rule(
        "ordinals (first..fourth, sixth)",
        vec![regex("(शून्य|प्रथम|पहला|पहली|पहले|द्वितीय|दूसरा|दूसरी|दूसरे|तृतीय|तीसरा|तीसरी|तीसरे|चौथा|चौथी|चौथे|छठा|छठी|छठे)")],
        |ts| ORDINALS_MAP.get(to_lower(groups(ts)?.first()?).as_str()).copied().and_then(ordinal),
    )
}

fn rule_other_ordinals() -> Rule {
    rule(
        "ordinals (fifth, seventh ...)",
        vec![regex("(पाँच|सात|आठ|नौ|दस|ग्यारह|बारह|तेरह|चौदह|पन्द्रह|सोलह|सत्रह|अठारह|उन्नीस|बीस|इक्कीस|बाईस|तेईस|चौबीस|पच्चीस|छब्बीस|सत्ताईस|अट्ठाईस|उनतीस|तीस|चालीस|पचास|साठ|सत्तर|अस्सी|नब्बे)(वा|वी|वे)")],
        |ts| CARDINALS_MAP.get(to_lower(groups(ts)?.first()?).as_str()).copied().and_then(ordinal),
    )
}

fn rule_ordinal_digits() -> Rule {
    rule(
        "ordinal (digits)",
        vec![regex("0*(\\d+) ?(वा|वी|वे)")],
        |ts| parse_int(groups(ts)?.first()?).and_then(ordinal),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_ordinals(),
        rule_other_ordinals(),
        rule_ordinal_digits(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
