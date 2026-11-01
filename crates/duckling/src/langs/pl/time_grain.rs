//! `Duckling.TimeGrain.PL.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "sekund(y|zie|(e|ę)|om|ami|ach|o|a)?|s", Grain::Second),
        ("minute (grain)", "minut(y|cie|(e|ę)|om|o|ami|ach|(a|ą))?|m", Grain::Minute),
        ("hour (grain)", "h|godzin(y|(e|ę)|ie|om|o|ami|ach|(a|ą))?", Grain::Hour),
        ("day (grain)", "dzie(n|ń|ni(a|ą))|dni(owi|ach|a|ą)?", Grain::Day),
        ("week (grain)", "tydzie(n|ń|)|tygod(ni(owi|u|a|em))|tygodn(iach|iami|iom|ie|i)|tyg\\.?", Grain::Week),
        ("month (grain)", "miesi(a|ą)c(owi|em|u|e|om|ami|ach|a)?", Grain::Month),
        ("quarter (grain)", "kwarta(l|ł)(u|owi|em|e|(o|ó)w|om|ach|ami|y)?", Grain::Quarter),
        ("year (grain)", "rok(u|owi|iem)?|lat(ami|ach|a|om)?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
