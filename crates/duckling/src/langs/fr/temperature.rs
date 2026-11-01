//! `Duckling.Temperature.FR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, with_unit, TemperatureData, TemperatureUnit};
use crate::types::{pred, regex, rule, Rule, Token};

fn unit_rule(name: &str, allow_degree: bool, re: &str, unit: TemperatureUnit) -> Rule {
    rule(name, vec![pred(is_value_only(allow_degree)), regex(re)], move |ts| match ts.first()? {
        Token::Temperature(td) => Some(Token::Temperature(with_unit(unit, td.clone()))),
        _ => None,
    })
}

fn rule_en_dessous_de_zero() -> Rule {
    rule(
        "<latent temp> en dessous de zero",
        vec![pred(is_value_only(true)), regex("en dessous de (0|z(é|e)ro)")],
        |ts| match ts.first()? {
            Token::Temperature(td @ TemperatureData { value: Some(v), .. }) => {
                let mut neg = td.clone();
                neg.value = Some(-*v);
                match td.unit {
                    None => Some(Token::Temperature(with_unit(TemperatureUnit::Degree, neg))),
                    _ => Some(Token::Temperature(neg)),
                }
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        unit_rule("<latent temp> degrees", false, "(deg(r(é|e|è))?s?\\.?)|°", TemperatureUnit::Degree),
        rule_en_dessous_de_zero(),
        unit_rule("<temp> Celsius", true, "c(el[cs]?(ius)?)?\\.?", TemperatureUnit::Celsius),
        unit_rule(
            "<temp> Fahrenheit",
            true,
            "f(ah?reh?n(h?eit)?)?\\.?",
            TemperatureUnit::Fahrenheit,
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
