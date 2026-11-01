//! `Duckling.TimeGrain.CA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("segon (grain)", "seg(on)?s?", Grain::Second),
        ("minuts (grain)", "min(ut)?s?", Grain::Minute),
        ("hora (grain)", "h(or(a|es))?", Grain::Hour),
        ("dia (grain)", "d(í|i)(a|es)", Grain::Day),
        ("setmana (grain)", "setman(a|es)", Grain::Week),
        ("mes (grain)", "mes(os)?", Grain::Month),
        ("trimestre (grain)", "trimestres?", Grain::Quarter),
        ("any (grain)", "anys?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
