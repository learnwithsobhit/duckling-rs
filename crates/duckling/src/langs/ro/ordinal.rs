//! `Duckling.Ordinal.RO.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

static ORDINAL_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("doi", 2),
        ("trei", 3),
        ("patru", 4),
        ("cinci", 5),
        ("sase", 6),
        ("șase", 6),
        ("sapte", 7),
        ("șapte", 7),
        ("opt", 8),
        ("noua", 9),
        ("nouă", 9),
    ]
    .into_iter()
    .collect()
});

fn rule_primul_doua_patra_noua() -> Rule {
    rule("special ordinals", vec![regex("(prim(a|ul)|a (patra|[dn]oua))")], |ts| {
        match to_lower(groups(ts)?.first()?).as_str() {
            "prima" | "primul" => ordinal(1),
            "a doua" => ordinal(2),
            "a patra" => ordinal(4),
            "a noua" => ordinal(9),
            _ => None,
        }
    })
}

fn rule_ordinal_digits() -> Rule {
    rule("ordinals (digits)", vec![regex(r"al?\s0*(\d+)[ -]?(le)?a")], |ts| {
        parse_int(groups(ts)?.first()?).and_then(ordinal)
    })
}

fn rule_spelled_out_ordinals() -> Rule {
    rule(
        "spelled out ordinals",
        vec![regex(r"al?\s(doi|trei|patru|cinci|(s|ș)a(s|pt)e|opt|nou(a|ă))[ -]?(le)?a")],
        |ts| ORDINAL_MAP.get(to_lower(groups(ts)?.first()?).as_str()).copied().and_then(ordinal),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule_ordinal_digits(), rule_primul_doua_patra_noua(), rule_spelled_out_ordinals()]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
