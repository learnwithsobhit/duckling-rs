//! `Duckling.TimeGrain.SV.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "sek(und(er(na)?)?)?", Grain::Second),
        ("minute (grain)", "min(ut(er(na)?)?)?", Grain::Minute),
        ("hour (grain)", "t(imm(e(n)?|ar(na)?)?)?", Grain::Hour),
        ("day (grain)", "dag(en|ar(na)?)?", Grain::Day),
        ("week (grain)", "veck(or(na)?|a(n)?)?", Grain::Week),
        ("month (grain)", "månad(er(na)?)?", Grain::Month),
        ("quarter (grain)", "kvart(al)(et)?", Grain::Quarter),
        ("year (grain)", "år(e[nt])?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
