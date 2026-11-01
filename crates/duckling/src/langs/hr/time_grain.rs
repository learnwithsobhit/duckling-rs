//! `Duckling.TimeGrain.HR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "sek(und)?(a|e|u)?", Grain::Second),
        ("minute (grain)", "min(ut)?(a|e|u)?", Grain::Minute),
        ("hour (grain)", "h|sat(i|a|e)?", Grain::Hour),
        ("day (grain)", "dan(i|a|e)?", Grain::Day),
        ("week (grain)", "tjeda?n(a|e|u|i)?", Grain::Week),
        ("month (grain)", "mjesec(a|e|u|i)?", Grain::Month),
        ("quarter (grain)", "kvartalu?|tromjese(c|č)j(e|u)", Grain::Quarter),
        ("year (grain)", "godin(a|e|u)", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
