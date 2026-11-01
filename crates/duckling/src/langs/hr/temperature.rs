//! `Duckling.Temperature.HR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, with_unit, TemperatureUnit};
use crate::types::{pred, regex, rule, Rule, Token};

fn unit_rule(name: &str, allow_degree: bool, re: &str, unit: TemperatureUnit) -> Rule {
    rule(name, vec![pred(is_value_only(allow_degree)), regex(re)], move |ts| match ts.first()? {
        Token::Temperature(td) => Some(Token::Temperature(with_unit(unit, td.clone()))),
        _ => None,
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        unit_rule(
            "<latent temp> stupnjevi",
            false,
            r"deg\.?|stupa?nj((ev)?a)?|°",
            TemperatureUnit::Degree,
        ),
        unit_rule("<temp> Celzij", true, r"c(elz?(ija?)?)?\.?", TemperatureUnit::Celsius),
        unit_rule(
            "<temp> Fahrenheit",
            true,
            r"f(ah?rh?eh?n(h?eit)?)?\.?",
            TemperatureUnit::Fahrenheit,
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
