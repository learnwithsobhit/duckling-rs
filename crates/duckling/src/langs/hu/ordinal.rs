//! `Duckling.Ordinal.HU.Rules`.

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
        ("első", 1),
        ("második", 2),
        ("harmadik", 3),
        ("negyedik", 4),
        ("ötödik", 5),
        ("hatodik", 6),
        ("hetedik", 7),
        ("nyolcadik", 8),
        ("kilencedik", 9),
        ("tizedik", 10),
        ("huszadik", 20),
        ("harmincadik", 30),
        ("negyvenedik", 40),
        ("ötvenedik", 50),
        ("hatvanadik", 60),
        ("hetvenedik", 70),
        ("nyolcvanadik", 80),
        ("kilencvenedik", 90),
    ]
    .into_iter()
    .collect()
});

static ORDINALS_MAP2: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("egyedik", 1),
        ("kettedik", 2),
        ("harmadik", 3),
        ("negyedik", 4),
        ("ötödik", 5),
        ("hatodik", 6),
        ("hetedik", 7),
        ("nyolcadik", 8),
        ("kilencedik", 9),
    ]
    .into_iter()
    .collect()
});

static CARDINALS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("tizen", 10),
        ("huszon", 20),
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

fn rule_ordinals() -> Rule {
    rule(
        "ordinals (first..twentieth,thirtieth,...)",
        vec![regex(
            "(első|második|harmadik|negyedik|ötödik|hatodik|hetedik|nyolcadik|kilencedik|tizedik|huszadik|harmincadik|negyvenedik|ötvenedik|hatvanadik|hetvenedik|nyolcvanadik|kilencvenedik)",
        )],
        |ts| {
            let g = groups(ts)?;
            ORDINALS_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| ordinal(*v))
        },
    )
}

fn rule_composite_ordinals() -> Rule {
    rule(
        "ordinals (composite, e.g., eighty-seven)",
        vec![regex(
            "(tizen|huszon|harminc|negyven|ötven|hatvan|hetven|nyolcvan|kilencven)\\-?(egyedik|kettedik|harmadik|negyedik|ötödik|hatodik|hetedik|nyolcadik|kilencedik)",
        )],
        |ts| {
            let g = groups(ts)?;
            let tt = CARDINALS_MAP.get(to_lower(g.first()?).as_str())?;
            let uu = ORDINALS_MAP2.get(to_lower(g.get(1)?).as_str())?;
            ordinal(tt + uu)
        },
    )
}

fn rule_ordinal_digits() -> Rule {
    rule("ordinal (digits)", vec![regex("0*(\\d+)\\.")], |ts| {
        parse_int(groups(ts)?.first()?).and_then(ordinal)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule_ordinals(), rule_composite_ordinals(), rule_ordinal_digits()]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
