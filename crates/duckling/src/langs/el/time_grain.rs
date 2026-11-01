//! `Duckling.TimeGrain.EL.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    // Names and regexes copied from `Duckling.TimeGrain.EL.Rules` (`grains`).
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain) ", "δε[υύ]τερ([οό]λ[εέ]πτ)?(ου?|α|ων)", Grain::Second),
        ("minute (grain)", "λεπτ(o|όν?|ού|ά|ών)", Grain::Minute),
        ("hour (grain)", "[ωώ](ρ(ας?|ες|ών))?", Grain::Hour),
        ("day (grain)", "η?μέρ(ας?|ες|ών)", Grain::Day),
        ("week (grain)", "ε?βδομάδ(ας?ν?|ες|ων)", Grain::Week),
        ("month (grain)", "μήν(ας?|ες|ών)", Grain::Month),
        ("quarter (grain)", "τρ[ιί]μ[ηή]ν(ου?|α|ων)", Grain::Quarter),
        (
            "year (grain)",
            "έτ(ου?ς|η|ών)|χρ[οό]ν(ο[ιςυ]?|ι([αά]|ές)|ι?ών)",
            Grain::Year,
        ),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
