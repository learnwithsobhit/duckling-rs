//! `Duckling.Temperature.JA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, with_unit, TemperatureUnit};
use crate::types::{pred, regex, rule, Rule, Token};

fn with_u(u: TemperatureUnit, t: &Token) -> Option<Token> {
    match t {
        Token::Temperature(td) => Some(Token::Temperature(with_unit(u, td.clone()))),
        _ => None,
    }
}

fn rule_latent_temp_degrees() -> Rule {
    rule(
        "<latent temp> degrees",
        vec![pred(is_value_only(false)), regex("度|°")],
        |ts| with_u(TemperatureUnit::Degree, ts.first()?),
    )
}

fn rule_temp_celcius() -> Rule {
    rule(
        "<temp> Celcius",
        vec![pred(is_value_only(true)), regex("摂氏(°|度)|(°)C")],
        |ts| with_u(TemperatureUnit::Celsius, ts.first()?),
    )
}

fn rule_celcius_temp() -> Rule {
    rule(
        "Celcius <temp>",
        vec![regex("摂氏"), pred(is_value_only(true)), regex("度|°")],
        |ts| with_u(TemperatureUnit::Celsius, ts.get(1)?),
    )
}

fn rule_temp_fahrenheit() -> Rule {
    rule(
        "<temp> Fahrenheit",
        vec![pred(is_value_only(true)), regex("華氏(°|度)|(°)F")],
        |ts| with_u(TemperatureUnit::Fahrenheit, ts.first()?),
    )
}

fn rule_fahrenheit_temp() -> Rule {
    rule(
        "Fahrenheit <temp>",
        vec![regex("華氏"), pred(is_value_only(true)), regex("度|°")],
        |ts| with_u(TemperatureUnit::Fahrenheit, ts.get(1)?),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_celcius_temp(),
        rule_fahrenheit_temp(),
        rule_latent_temp_degrees(),
        rule_temp_celcius(),
        rule_temp_fahrenheit(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
