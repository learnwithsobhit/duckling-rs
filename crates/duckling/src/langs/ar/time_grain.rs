//! `Duckling.TimeGrain.AR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain) ", "(ثاني(ة|ه)?|ثواني|لحظ(ة|ه|ات))", Grain::Second),
        ("minute (grain)", "دق(يق(ة|ه)|ائق)", Grain::Minute),
        ("hour (grain)", "ساع(ة|ه|ات)", Grain::Hour),
        ("day (grain)", "يوم|(ا|أ)يام", Grain::Day),
        ("week (grain)", "(ا|أ|إ)س(بوع|ابيع)", Grain::Week),
        ("month (grain)", "شهر|(ا|أ|إ)شهر", Grain::Month),
        ("quarter (grain)", "(ربع(ين|ان)|[أا]رباع)(سنة|عام)?", Grain::Quarter),
        ("year (grain)", "سن(ة|ين)|عام", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
