//! `Duckling.TimeGrain.FI.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    // Keep names identical to Haskell, including the trailing space on seconds.
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain) ", "sekuntia?|s", Grain::Second),
        ("minute (grain)", "minuuttia?|min", Grain::Minute),
        ("hour (grain)", "tuntia?|h", Grain::Hour),
        ("day (grain)", "päivä(ä)?|vuorokau(si|tta)|vrk|d", Grain::Day),
        ("week (grain)", "viikkoa?|vko?", Grain::Week),
        ("month (grain)", "kuukau(si|tta)|kk", Grain::Month),
        ("quarter (grain)", "kvartaali(a)?|neljännesvuo(si|tta)", Grain::Quarter),
        ("year (grain)", "vuo(si|tta)|v\\.?|a", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
