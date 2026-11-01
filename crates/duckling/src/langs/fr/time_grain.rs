//! `Duckling.TimeGrain.FR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("seconde (grain)", "sec(onde)?s?", Grain::Second),
        ("minute (grain)", "min(ute)?s?", Grain::Minute),
        ("heure (grain)", "heures?", Grain::Hour),
        ("jour (grain)", "jour(n(e|é)e?)?s?", Grain::Day),
        ("semaine (grain)", "semaines?", Grain::Week),
        ("mois (grain)", "mois", Grain::Month),
        ("trimestre (grain)", "trimestres?", Grain::Quarter),
        ("année (grain)", "an(n(e|é)e?)?s?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
