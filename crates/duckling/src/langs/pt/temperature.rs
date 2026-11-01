//! `Duckling.Temperature.PT.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, with_unit, TemperatureData, TemperatureUnit};
use crate::types::{pred, regex, rule, Rule, Token};

fn unit_rule(name: &str, flag: bool, re: &str, u: TemperatureUnit) -> Rule {
    rule(name, vec![pred(is_value_only(flag)), regex(re)], move |ts| match ts.first()? {
        Token::Temperature(td) => Some(Token::Temperature(with_unit(u, td.clone()))),
        _ => None,
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        unit_rule("<latent temp> temp", false, "(graus?)|°", TemperatureUnit::Degree),
        rule(
            "<latent temp> temp abaixo de zero",
            vec![pred(is_value_only(true)), regex("((graus?)|°)?( abaixo (de)? zero)")],
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
        ),
        unit_rule(
            "<temp> Celsius",
            true,
            "(cent(i|í)grados?|c(el[cs]?(ius)?)?\\.?)",
            TemperatureUnit::Celsius,
        ),
        unit_rule("<temp> Fahrenheit", true, "f(ah?reh?n(h?eit)?)?\\.?", TemperatureUnit::Fahrenheit),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
