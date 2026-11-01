//! `Duckling.TimeGrain.ES.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("segundo (grain)", "seg(undo)?s?", Grain::Second),
        ("minutos (grain)", "min(uto)?s?", Grain::Minute),
        ("hora (grain)", "h(ora)?s?", Grain::Hour),
        ("dia (grain)", "d(í|i)as?", Grain::Day),
        ("semana (grain)", "semanas?", Grain::Week),
        ("mes (grain)", "mes(es)?", Grain::Month),
        ("trimestre (grain)", "trimestres?", Grain::Quarter),
        ("año (grain)", "a(n|ñ)os?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
