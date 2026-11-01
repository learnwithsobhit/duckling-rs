//! `Duckling.Ordinal.TR.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

const ORDINALS: &[&str] = &[
    "birinci",
    "ikinci",
    "üçüncü",
    "dördüncü",
    "beşinci",
    "altıncı",
    "yedinci",
    "sekizinci",
    "dokuzuncu",
    "onuncu",
    "on birinci",
    "on ikinci",
    "on üçüncü",
    "on dördüncü",
    "on beşinci",
    "on altıncı",
    "on yedinci",
    "on sekizinci",
    "on dokuzuncu",
    "yirminci",
    "yirmi birinci",
    "yirmi ikinci",
    "yirmi üçüncü",
    "yirmi dördüncü",
    "yirmi beşinci",
    "yirmi altıncı",
    "yirmi yedinci",
    "yirmi sekizinci",
    "yirmi dokuzuncu",
    "otuzuncu",
    "otuz birinci",
];

static ORDINALS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    ORDINALS.iter().enumerate().map(|(i, s)| (*s, (i as i64) + 1)).collect()
});

fn rule_ordinals_firstst() -> Rule {
    let re = format!("({})", ORDINALS.join("|"));
    rule("ordinals (first..31st)", vec![regex(&re)], move |ts| {
        let g = groups(ts)?;
        ORDINALS_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| ordinal(*v))
    })
}

fn rule_ordinal_digits() -> Rule {
    rule(
        "ordinal (digits)",
        vec![regex("0*(\\d+) ?('?)(inci|nci|ıncı|ncı|uncu|ncu|üncü|ncü|.)")],
        |ts| parse_int(groups(ts)?.first()?).and_then(ordinal),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![rule_ordinal_digits(), rule_ordinals_firstst()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
