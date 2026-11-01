//! `Duckling.TimeGrain.DE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "sekunden?", Grain::Second),
        ("minute (grain)", "min(uten?)?", Grain::Minute),
        ("hour (grain)", "stunden?", Grain::Hour),
        ("day (grain)", "tage?n?", Grain::Day),
        ("week (grain)", "wochen?", Grain::Week),
        ("month (grain)", "monate?n?", Grain::Month),
        ("quarter (grain)", "quartale?", Grain::Quarter),
        ("year (grain)", "jahre?n?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
