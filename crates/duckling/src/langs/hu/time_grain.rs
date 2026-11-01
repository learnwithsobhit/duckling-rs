//! `Duckling.TimeGrain.HU.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    // Keep names identical to Haskell, including the trailing space on seconds.
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain) ", "másodperc(ek)?|mp", Grain::Second),
        ("minute (grain)", "perc(ek)?", Grain::Minute),
        ("hour (grain)", "óra|órák", Grain::Hour),
        ("day (grain)", "nap(ok)?", Grain::Day),
        ("week (grain)", "hét|hetek", Grain::Week),
        ("month (grain)", "hónap|hónapok", Grain::Month),
        ("quarter (grain)", "negyed\\s?év(ek)?", Grain::Quarter),
        ("year (grain)", "év(ek)?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
