//! `Duckling.TimeGrain.JA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "秒(毎|間)?|びょう", Grain::Second),
        ("minute (grain)", "分(毎|間)?|ふん", Grain::Minute),
        ("hour (grain)", "時(毎|間)?|じ", Grain::Hour),
        ("day (grain)", "曜?日(毎|間)?|ようび|にち|ひ", Grain::Day),
        ("week (grain)", "週(毎|間)?|しゅう", Grain::Week),
        ("month (grain)", "月(毎|間)?|げつ|つき", Grain::Month),
        ("quarter (grain)", "(四|4)半期|しはんき|QTR", Grain::Quarter),
        ("year (grain)", "年(毎|間)?|ねん|とし", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
