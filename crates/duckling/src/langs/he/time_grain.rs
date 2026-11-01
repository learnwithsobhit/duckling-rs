//! `Duckling.TimeGrain.HE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "שניות|שנייה", Grain::Second),
        ("minute (grain)", "דקה|דקות", Grain::Minute),
        ("hour (grain)", "שעות|שעה", Grain::Hour),
        ("day (grain)", "ימים|יום", Grain::Day),
        ("week (grain)", "שבוע|שבועות", Grain::Week),
        ("month (grain)", "חודש|חודשים", Grain::Month),
        ("quarter (grain)", "רבע", Grain::Quarter),
        ("year (grain)", "שנה", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
