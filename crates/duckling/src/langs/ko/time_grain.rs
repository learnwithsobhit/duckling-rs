//! `Duckling.TimeGrain.KO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "초", Grain::Second),
        ("minute (grain)", "분", Grain::Minute),
        ("hour (grain)", "시(간)?", Grain::Hour),
        ("day (grain)", "날|일(간|동안)?", Grain::Day),
        ("week (grain)", "주(간|동안|일)?", Grain::Week),
        ("month (grain)", "(달)(간|동안)?", Grain::Month),
        ("quarter (grain)", "분기(간|동안)?", Grain::Quarter),
        ("year (grain)", "해|연간|년(간|동안)?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
