//! `Duckling.Temperature.ZH.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, with_unit, TemperatureUnit};
use crate::types::{pred, regex, rule, Rule, Token};

fn rule_latent_temp_degrees() -> Rule {
    rule(
        "<latent temp> degrees",
        vec![pred(is_value_only(false)), regex("度|°")],
        |ts| match ts.first()? {
            Token::Temperature(td) => Some(Token::Temperature(with_unit(TemperatureUnit::Degree, td.clone()))),
            _ => None,
        },
    )
}

fn rule_temp_celcius() -> Rule {
    rule(
        "<temp> Celcius",
        vec![pred(is_value_only(true)), regex("(摄|攝)氏(°|度)|(°)C")],
        |ts| match ts.first()? {
            Token::Temperature(td) => {
                Some(Token::Temperature(with_unit(TemperatureUnit::Celsius, td.clone())))
            }
            _ => None,
        },
    )
}

fn rule_celcius_temp() -> Rule {
    rule(
        "Celcius <temp>",
        vec![regex("(摄|攝)氏"), pred(is_value_only(true)), regex("度|°")],
        |ts| match ts.get(1)? {
            Token::Temperature(td) => {
                Some(Token::Temperature(with_unit(TemperatureUnit::Celsius, td.clone())))
            }
            _ => None,
        },
    )
}

fn rule_temp_fahrenheit() -> Rule {
    rule(
        "<temp> Fahrenheit",
        vec![pred(is_value_only(true)), regex("(华|華)氏(°|度)|(°)F")],
        |ts| match ts.first()? {
            Token::Temperature(td) => {
                Some(Token::Temperature(with_unit(TemperatureUnit::Fahrenheit, td.clone())))
            }
            _ => None,
        },
    )
}

fn rule_fahrenheit_temp() -> Rule {
    rule(
        "Fahrenheit <temp>",
        vec![regex("(华|華)氏"), pred(is_value_only(true)), regex("度|°")],
        |ts| match ts.get(1)? {
            Token::Temperature(td) => {
                Some(Token::Temperature(with_unit(TemperatureUnit::Fahrenheit, td.clone())))
            }
            _ => None,
        },
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
