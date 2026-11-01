//! `Duckling.TimeGrain.DA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "sekund(er)?", Grain::Second),
        ("minute (grain)", "minut(ter)?", Grain::Minute),
        ("hour (grain)", "t(imer?)?", Grain::Hour),
        ("day (grain)", "dag(e)?", Grain::Day),
        ("week (grain)", "uger?", Grain::Week),
        ("month (grain)", "måned(er)?", Grain::Month),
        ("quarter (grain)", "kvartal(er)?", Grain::Quarter),
        ("year (grain)", "år", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
