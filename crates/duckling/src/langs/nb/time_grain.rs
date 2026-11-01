//! `Duckling.TimeGrain.NB.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "sek(und(er)?)?", Grain::Second),
        ("minute (grain)", "min(utt(er)?)?", Grain::Minute),
        ("hour (grain)", "t(ime(r)?)?", Grain::Hour),
        ("day (grain)", "dag(er)?", Grain::Day),
        ("week (grain)", "uke(r|n)?", Grain::Week),
        ("month (grain)", "måned(er)?", Grain::Month),
        ("quarter (grain)", "kvart(al|er)(et)?", Grain::Quarter),
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
