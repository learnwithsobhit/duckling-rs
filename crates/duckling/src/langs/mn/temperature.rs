//! `Duckling.Temperature.MN.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{
    is_simple_temperature, is_value_only, unit_only, with_max, with_min, with_unit, TemperatureData,
    TemperatureUnit,
};
use crate::types::{pred, regex, rule, Rule, Token};

fn temp(t: &Token) -> Option<&TemperatureData> {
    match t {
        Token::Temperature(d) => Some(d),
        _ => None,
    }
}

fn unit_rule(name: &str, allow_degree: bool, re: &str, unit: TemperatureUnit) -> Rule {
    rule(name, vec![pred(is_value_only(allow_degree)), regex(re)], move |ts| match ts.first()? {
        Token::Temperature(td) => Some(Token::Temperature(with_unit(unit, td.clone()))),
        _ => None,
    })
}

fn rule_temperature_below_zero() -> Rule {
    rule(
        "<temp> below zero",
        vec![pred(is_value_only(true)), regex("тэгээс доош")],
        |ts| {
            let td = temp(ts.first()?)?;
            let v = td.value?;
            let mut neg = td.clone();
            neg.value = Some(-v);
            match td.unit {
                None => Some(Token::Temperature(with_unit(TemperatureUnit::Degree, neg))),
                _ => Some(Token::Temperature(neg)),
            }
        },
    )
}

fn rule_interval_max() -> Rule {
    rule(
        "under/less/lower/no more than <temp>",
        vec![regex("доогуур|(бага|ихгүй|их биш)"), pred(is_simple_temperature)],
        |ts| {
            let d = temp(ts.get(1)?)?;
            let (to, u) = (d.value?, d.unit?);
            Some(Token::Temperature(with_max(to, unit_only(u))))
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "over/above/at least/more than <temp>",
        vec![regex("дээгүүр|их|багадаа"), pred(is_simple_temperature)],
        |ts| {
            let d = temp(ts.get(1)?)?;
            let (from, u) = (d.value?, d.unit?);
            Some(Token::Temperature(with_min(from, unit_only(u))))
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        unit_rule("<latent temp> градус", false, "градус|°|хэм", TemperatureUnit::Degree),
        unit_rule("<temp> Celsius", true, "c(el[cs]?(ius)?)?\\.?", TemperatureUnit::Celsius),
        unit_rule(
            "<temp> Fahrenheit",
            true,
            "((f(ah?rh?eh?n(h?eit)?)?\\.?)|фарангейт)",
            TemperatureUnit::Fahrenheit,
        ),
        rule_temperature_below_zero(),
        unit_rule("<temp> °C", true, "c", TemperatureUnit::Celsius),
        rule_interval_min(),
        rule_interval_max(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
