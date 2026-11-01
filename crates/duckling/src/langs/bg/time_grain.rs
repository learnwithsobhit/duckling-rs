//! `Duckling.TimeGrain.BG.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain) ", "сек(унд(а|и))?", Grain::Second),
        ("minute (grain)", "мин(ут(а|и))?", Grain::Minute),
        ("hour (grain)", "ч(ас(ове(те)?|а|ът)?)?", Grain::Hour),
        ("day (grain)", "ден(а|я(т)?)?|дни(те)?", Grain::Day),
        ("week (grain)", "седмица(та)?|седмици(те)?", Grain::Week),
        ("month (grain)", "месец(и(те)?|а|ът)?", Grain::Month),
        ("quarter (grain)", "тримесечи(я|е(то|та)?)", Grain::Quarter),
        ("year (grain)", "г(од(ин(а(та)?|и(те)?))?)?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
