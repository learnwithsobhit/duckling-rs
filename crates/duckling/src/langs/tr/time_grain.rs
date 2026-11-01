//! `Duckling.TimeGrain.TR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("saniye (grain)", "sa?n(iye)?(nin)?", Grain::Second),
        ("dakika (grain)", "da?k(ika)?(nın)?", Grain::Minute),
        ("saat (grain)", "sa(at)?(in)?", Grain::Hour),
        ("gün (grain)", "gün(ün)?", Grain::Day),
        ("hafta (grain)", "hafta(nın)?", Grain::Week),
        ("ay (grain)", "ay(ın)?", Grain::Month),
        ("çeyrek yıl (grain)", "çeyrek yıl(ın)?", Grain::Quarter),
        ("yıl (grain)", "yıl(ın)?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
