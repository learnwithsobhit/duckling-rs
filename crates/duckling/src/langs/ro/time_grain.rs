//! `Duckling.TimeGrain.RO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("secunde (grain)", "sec(und(a|e|ă))?", Grain::Second),
        ("minute (grain)", "min(ut(e|ul)?)?", Grain::Minute),
        ("ore (grain)", "h|or(a|e(le)?|ă)", Grain::Hour),
        ("zile (grain)", "zi(le(le)?|u(a|ă))?", Grain::Day),
        ("saptamani (grain)", "sapt(a|ă)m(a|â)n(ile|a|ă|i)", Grain::Week),
        ("luni (grain)", "lun(i(le)?|a|ă)", Grain::Month),
        ("trimestru (grain)", "trimestr(e(le)?|ul?)", Grain::Quarter),
        ("ani (grain)", "an(ul|ii?)?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
