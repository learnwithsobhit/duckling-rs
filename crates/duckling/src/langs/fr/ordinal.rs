//! `Duckling.Ordinal.FR.Rules`.

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
        ("première", 1), ("premiere", 1), ("premier", 1), ("deuxième", 2), ("deuxieme", 2),
        ("second", 2), ("seconde", 2), ("troisième", 3), ("troisieme", 3), ("quatrieme", 4),
        ("quatrième", 4), ("cinquieme", 5), ("cinquième", 5), ("sixième", 6), ("sixieme", 6),
        ("septieme", 7), ("septième", 7), ("huitième", 8), ("huitieme", 8), ("neuvieme", 9),
        ("neuvième", 9), ("dixième", 10), ("dixieme", 10), ("onzième", 11), ("onzieme", 11),
        ("douzieme", 12), ("douzième", 12), ("treizieme", 13), ("treizième", 13),
        ("quatorzième", 14), ("quatorzieme", 14), ("quinzième", 15), ("quinzieme", 15),
        ("seizieme", 16), ("seizième", 16),
    ]
    .into_iter()
    .collect()
});

fn rule_ordinals_premierseizieme() -> Rule {
    rule(
        "ordinals (premier..seizieme)",
        vec![regex("(premi(ere?|ère)|(deux|trois|quatr|cinqu|six|sept|huit|neuv|dix|onz|douz|treiz|quatorz|quinz|seiz)i(e|è)me|seconde?)")],
        |ts| {
            let g = groups(ts)?;
            ORDINALS_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| ordinal(*v))
        },
    )
}

fn rule_ordinal_digits() -> Rule {
    rule("ordinal (digits)", vec![regex("0*(\\d+) ?(ere?|ère|ème|eme|e)")], |ts| {
        parse_int(groups(ts)?.first()?).and_then(ordinal)
    })
}

static RULES: Lazy<Vec<Rule>> =
    Lazy::new(|| vec![rule_ordinal_digits(), rule_ordinals_premierseizieme()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
