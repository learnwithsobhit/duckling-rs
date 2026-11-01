//! `Duckling.TimeGrain.KA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    // Names and regexes copied from `Duckling.TimeGrain.KA.Rules` (`grains`).
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain)", "წამ(ით|ი|ში|ს)?", Grain::Second),
        ("minute (grain)", "წუთ(ით|ი|ში|ს)?", Grain::Minute),
        ("hour (grain)", "საათ(ით|ი|ში|ს)?", Grain::Hour),
        ("day (grain)", "დღით|დღის|დღე?(ში)?ს?", Grain::Day),
        ("week (grain)", "კვირ(აში)?(ას|ით|ის|ა)?", Grain::Week),
        ("month (grain)", "თვ(ეში|ით|ის|ეს|ს|ე)?", Grain::Month),
        ("quarter (grain)", "კვარტა?ლ(ით|ი|ში|ს)?", Grain::Quarter),
        (
            "year (grain)",
            "წლით|წელიწადი?(ით|ი|ში|ის|ს)?|წე?ლი?(ით|ის|ში|ს|იდან)?",
            Grain::Year,
        ),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
