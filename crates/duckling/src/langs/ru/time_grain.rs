//! `Duckling.TimeGrain.RU.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain) ", "сек(унд(а|у|ы)?)?", Grain::Second),
        ("minute (grain)", "мин(ут(а|у|ы)?)?", Grain::Minute),
        ("hour (grain)", "ч(ас(а|ов)?)?", Grain::Hour),
        ("day (grain)", "день|дня|дней", Grain::Day),
        ("week (grain)", "недел(ь|я|и|ю|е)?", Grain::Week),
        ("month (grain)", "месяц(а|ев|е)?", Grain::Month),
        ("quarter (grain)", "квартал(а|е)?", Grain::Quarter),
        ("year (grain)", "год(а|у)?|лет", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
