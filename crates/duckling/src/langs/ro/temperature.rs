//! `Duckling.Temperature.RO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, with_unit, TemperatureUnit};
use crate::types::{pred, regex, rule, Rule, Token};

fn unit_rule(name: &str, flag: bool, re: &str, u: TemperatureUnit) -> Rule {
    rule(name, vec![pred(is_value_only(flag)), regex(re)], move |ts| match ts.first()? {
        Token::Temperature(td) => Some(Token::Temperature(with_unit(u, td.clone()))),
        _ => None,
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        unit_rule("<latent temp> grade", false, "(de )?(grade)|°", TemperatureUnit::Degree),
        unit_rule("<temp> Celcius", true, "(de )?c(el[cs]?(ius)?)?\\.?", TemperatureUnit::Celsius),
        unit_rule(
            "<temp> Fahrenheit",
            true,
            "(de )?f(ah?rh?eh?n(h?eit)?)?\\.?",
            TemperatureUnit::Fahrenheit,
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
