//! `Duckling.TimeGrain.MN.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain) ", "сек(унд)?", Grain::Second),
        ("minute (grain)", "мин(ут)?", Grain::Minute),
        ("hour (grain)", "ц(аг)?", Grain::Hour),
        ("day (grain)", "өдөр?", Grain::Day),
        ("week (grain)", "долоо хоног?", Grain::Week),
        ("month (grain)", "сар?", Grain::Month),
        ("quarter (grain)", "улирал?", Grain::Quarter),
        ("year (grain)", "жил?|жил", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
