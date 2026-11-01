//! `Duckling.Temperature.HI.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, with_unit, TemperatureUnit};
use crate::dims::util::to_lower;
use crate::types::{pred, regex, rule, Rule, Token};

fn rule_latent_temp_degrees() -> Rule {
    rule(
        "<latent temp> degrees",
        vec![pred(is_value_only(false)), regex("डिग्री|°")],
        |ts| match ts.first()? {
            Token::Temperature(td) => Some(Token::Temperature(with_unit(
                TemperatureUnit::Degree,
                td.clone(),
            ))),
            _ => None,
        },
    )
}

fn rule_temp_specific() -> Rule {
    rule(
        "<temp> Fahrenheit|Celsius",
        vec![pred(is_value_only(true)), regex("(सेल्सीयस|फारेनहाइट)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Temperature(td), Token::RegexMatch(g)) => match to_lower(g.0.first()?).as_str()
            {
                "सेल्सीयस" => Some(Token::Temperature(with_unit(
                    TemperatureUnit::Celsius,
                    td.clone(),
                ))),
                "फारेनहाइट" => Some(Token::Temperature(with_unit(
                    TemperatureUnit::Fahrenheit,
                    td.clone(),
                ))),
                _ => None,
            },
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> =
    Lazy::new(|| vec![rule_latent_temp_degrees(), rule_temp_specific()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
