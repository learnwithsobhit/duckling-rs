//! `Duckling.Ordinal.EL.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, pred, regex, rule, Rule, Token};

static ORDINALS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("πρώτ", 1),
        ("δεύτερ", 2),
        ("δευτέρ", 2),
        ("τρίτ", 3),
        ("τέταρτ", 4),
        ("τετάρτ", 4),
        ("πέμπτ", 5),
        ("έκτ", 6),
        ("έβδομ", 7),
        ("εβδόμ", 7),
        ("όγδο", 8),
        ("ογδό", 8),
        ("ένατ", 9),
        ("ενάτ", 9),
        ("δέκατ", 10),
        ("δεκάτ", 10),
        ("ενδέκατ", 11),
        ("ενδεκάτ", 11),
        ("δωδέκατ", 12),
        ("δωδεκάτ", 12),
        ("εικοστ", 20),
        ("τριακοστ", 30),
        ("τεσσαρακοστ", 40),
        ("πεντηκοστ", 50),
        ("εξηκοστ", 60),
        ("εβδομηκοστ", 70),
        ("ογδοηκοστ", 80),
        ("ενενηκοστ", 90),
    ]
    .into_iter()
    .collect()
});

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

/// `Duckling.Ordinal.Helpers.oneOf`.
fn one_of(vs: Vec<i64>) -> crate::types::PatternItem {
    pred(move |t| matches!(t, Token::Ordinal(o) if vs.contains(&o.value)))
}

fn rule_ordinals() -> Rule {
    rule(
        "ordinals (1st..12th, 20th, 30th..90th)",
        vec![regex(
            "(πρώτ|δε[υύ]τ[εέ]ρ|τρίτ|τ[εέ]τ[αά]ρτ|πέμπτ|έκτ|[εέ]βδ[οό]μ(ηκοστ)?|[οό]γδ[οό](ηκοστ)?|[εέ]ν[αά]τ|δ[εέ]κ[αά]τ|εν[δτ][εέ]κ[αά]τ|δωδ[εέ]κ[αά]τ|εικοστ|τριακοστ|τεσσαρακοστ|πεντηκοστ|εξηκοστ|ενενηκοστ)([οό][υύιί]?ς?|[ηή]ς?|[εέ]ς|ων)",
        )],
        |ts| {
            let m = to_lower(groups(ts)?.first()?);
            ORDINALS.get(m.as_str()).copied().and_then(ordinal)
        },
    )
}

fn rule_composite_ordinals() -> Rule {
    rule(
        "ordinals (composite: 11th..19th, 21st..29th, ..., 91st..99th)",
        vec![one_of((10..=90).collect()), one_of((1..=9).collect())],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Ordinal(t), Token::Ordinal(u)) => ordinal(t.value + u.value),
            _ => None,
        },
    )
}

fn rule_ordinal_digits() -> Rule {
    rule(
        "ordinal (digits)",
        vec![regex("0*(\\d+) ?(ο[ςυι]?|ης?|ες)")],
        |ts| parse_int(groups(ts)?.first()?).and_then(ordinal),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule_ordinals(), rule_composite_ordinals(), rule_ordinal_digits()]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
