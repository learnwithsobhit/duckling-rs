//! `Duckling.Temperature.KM.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, with_unit, TemperatureUnit};
use crate::types::{pred, regex, rule, Rule, Token};

fn rule_latent_temp_degrees() -> Rule {
    rule(
        "<latent temp> degrees",
        vec![pred(is_value_only(false)), regex("ដឺក្រេ|°")],
        |ts| match ts.first()? {
            Token::Temperature(td) => {
                Some(Token::Temperature(with_unit(TemperatureUnit::Degree, td.clone())))
            }
            _ => None,
        },
    )
}

fn rule_temp_celcius() -> Rule {
    rule(
        "<temp> Celcius",
        vec![pred(is_value_only(true)), regex("អង្សា(សេ)?")],
        |ts| match ts.first()? {
            Token::Temperature(td) => {
                Some(Token::Temperature(with_unit(TemperatureUnit::Celsius, td.clone())))
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![rule_temp_celcius(), rule_latent_temp_degrees()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
