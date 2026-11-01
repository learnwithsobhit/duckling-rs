//! `Duckling.TimeGrain.VI.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "(giây|s|sec)", Grain::Second),
        ("minute (grain)", "(phút|m|min)", Grain::Minute),
        ("hour (grain)", "(giờ|h|tiếng)", Grain::Hour),
        ("day (grain)", "ngày", Grain::Day),
        ("week (grain)", "tuần", Grain::Week),
        ("month (grain)", "tháng", Grain::Month),
        ("quarter (grain)", "quý", Grain::Quarter),
        ("year (grain)", "năm", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
