//! `Duckling.TimeGrain.ZH.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 7] = [
        ("second (grain)", "秒(钟|鐘)?", Grain::Second),
        ("minute (grain)", "分(钟|鐘)?", Grain::Minute),
        ("hour (grain)", "小时|小時|個鐘|鐘(\u{982d})?", Grain::Hour),
        ("day (grain)", "天|日", Grain::Day),
        ("week (grain)", "周|週|礼拜|禮拜|星期", Grain::Week),
        ("month (grain)", "月", Grain::Month),
        ("year (grain)", "年", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
