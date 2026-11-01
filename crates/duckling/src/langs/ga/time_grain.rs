//! `Duckling.TimeGrain.GA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("soicind (grain)", "t?sh?oicind(í|i)?", Grain::Second),
        ("nóiméad (grain)", "n[óo]im(é|e)[ai]da?", Grain::Minute),
        ("uair (grain)", "([thn]-?)?uair(e|eanta)?", Grain::Hour),
        ("lá (grain)", "l(ae(thanta)?|(á|a))", Grain::Day),
        ("seachtain (grain)", "t?sh?eachtain(e|(í|i))?", Grain::Week),
        ("mí (grain)", "mh?(í|i)(sa|nna)", Grain::Month),
        ("ráithe (grain)", "r(á|a)ith(e|(í|i))", Grain::Quarter),
        ("bliain (grain)", "m?bh?lia(in|na|nta)", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
