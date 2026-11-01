//! `Duckling.TimeGrain.UK.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "секунд(а|и|і|у)?|сек", Grain::Second),
        ("minute (grain)", "хвилин(а|и|і|у)?|хв", Grain::Minute),
        ("hour (grain)", "годин(а|и|і|у)?", Grain::Hour),
        ("day (grain)", "день|дн(і|я)в?", Grain::Day),
        ("week (grain)", "тиждень|тижн(я|і)в?", Grain::Week),
        ("month (grain)", "місяц(ь|я|і)в?", Grain::Month),
        ("quarter (grain)", "квартал(и|і)?в?", Grain::Quarter),
        ("year (grain)", "рік|ро(к|ц)(и|і|у)?в?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
