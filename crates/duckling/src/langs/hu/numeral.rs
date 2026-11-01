//! `Duckling.Numeral.HU.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::integer;
use crate::dims::util::to_lower;
use crate::types::{groups, regex, rule, Rule, Token};

static NUMERAL_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("nulla", 0),
        ("zéró", 0),
        ("egy", 1),
        ("kettő", 2),
        ("három", 3),
        ("négy", 4),
        ("öt", 5),
        ("hat", 6),
        ("hét", 7),
        ("nyolc", 8),
        ("kilenc", 9),
        ("tíz", 10),
    ]
    .into_iter()
    .collect()
});

static ELEVEN_TO_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("tizenegy", 11),
        ("tizenkettő", 12),
        ("tizenhárom", 13),
        ("tizennégy", 14),
        ("tizenöt", 15),
        ("tizenhat", 16),
        ("tizenhét", 17),
        ("tizennyolc", 18),
        ("tizenkilenc", 19),
    ]
    .into_iter()
    .collect()
});

static TWENTYONE_TO_TWENTYNINE: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("huszonegy", 21),
        ("huszonkettő", 22),
        ("huszonhárom", 23),
        ("huszonnégy", 24),
        ("huszonöt", 25),
        ("huszonhat", 26),
        ("huszonhét", 27),
        ("huszonnyolc", 28),
        ("huszonkilenc", 29),
    ]
    .into_iter()
    .collect()
});

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("húsz", 20),
        ("harminc", 30),
        ("negyven", 40),
        ("ötven", 50),
        ("hatvan", 60),
        ("hetven", 70),
        ("nyolcvan", 80),
        ("kilencven", 90),
    ]
    .into_iter()
    .collect()
});

fn lookup_int(map: &HashMap<&'static str, i64>, ts: &[&Token]) -> Option<Token> {
    let g = groups(ts)?;
    map.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
}

fn rule_numeral() -> Rule {
    rule(
        "number (0..10)",
        vec![regex("(nulla|zéró|egy|kettő|három|négy|öt|hat|hét|nyolc|kilenc|tíz)")],
        |ts| lookup_int(&NUMERAL_MAP, ts),
    )
}

fn rule_eleven_to_nineteen() -> Rule {
    rule(
        "number (11..19)",
        vec![regex(
            "(tizenegy|tizenkettő|tizenhárom|tizennégy|tizenöt|tizenhat|tizenhét|tizennyolc|tizenkilenc)",
        )],
        |ts| lookup_int(&ELEVEN_TO_NINETEEN, ts),
    )
}

fn rule_twentyone_to_twentynine() -> Rule {
    rule(
        "number (21..29)",
        vec![regex(
            "(huszonegy|huszonkettő|huszonhárom|huszonnégy|huszonöt|huszonhat|huszonhét|huszonnyolc|huszonkilenc)",
        )],
        |ts| lookup_int(&TWENTYONE_TO_TWENTYNINE, ts),
    )
}

fn rule_tens() -> Rule {
    rule(
        "integer (20,30..90)",
        vec![regex("(húsz|harminc|negyven|ötven|hatvan|hetven|nyolcvan|kilencven)")],
        |ts| lookup_int(&TENS_MAP, ts),
    )
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer ([3-9][1-9])",
        vec![regex(
            "(harminc|negyven|ötven|hatvan|hetven|nyolcvan|kilencven)(egy|kettő|három|négy|öt|hat|hét|nyolc|kilenc)",
        )],
        |ts| {
            let g = groups(ts)?;
            let v1 = TENS_MAP.get(to_lower(g.first()?).as_str())?;
            let v2 = NUMERAL_MAP.get(to_lower(g.get(1)?).as_str())?;
            integer(v1 + v2)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_numeral(),
        rule_eleven_to_nineteen(),
        rule_twentyone_to_twentynine(),
        rule_tens(),
        rule_composite_tens(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
