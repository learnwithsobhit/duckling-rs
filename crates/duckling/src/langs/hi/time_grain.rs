//! `Duckling.TimeGrain.HI.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "सेकंड", Grain::Second),
        ("minute (grain)", "मिनट", Grain::Minute),
        ("hour (grain)", "घंटा", Grain::Hour),
        ("day (grain)", "(दिन|दिवस)", Grain::Day),
        ("week (grain)", "(सप्ताह|हफ़्ता)", Grain::Week),
        ("month (grain)", "महीना", Grain::Month),
        ("quarter (grain)", "चौथाई", Grain::Quarter),
        ("year (grain)", "(साल|वर्ष)", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
