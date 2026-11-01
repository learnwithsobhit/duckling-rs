//! `Duckling.Temperature.TR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, with_unit, TemperatureData, TemperatureUnit};
use crate::types::{pred, regex, rule, Rule, Token};

fn unit_rule(name: &str, allow_degree: bool, re: &str, unit: TemperatureUnit) -> Rule {
    rule(name, vec![pred(is_value_only(allow_degree)), regex(re)], move |ts| match ts.first()? {
        Token::Temperature(td) => Some(Token::Temperature(with_unit(unit, td.clone()))),
        _ => None,
    })
}

fn negate_temp(td: &TemperatureData) -> Option<Token> {
    let v = td.value?;
    let mut neg = td.clone();
    neg.value = Some(-v);
    match td.unit {
        None => Some(Token::Temperature(with_unit(TemperatureUnit::Degree, neg))),
        _ => Some(Token::Temperature(neg)),
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        unit_rule("<latent temp> degrees", false, "derece|°", TemperatureUnit::Degree),
        unit_rule("<temp> Celsius", true, "c|santigrat", TemperatureUnit::Celsius),
        unit_rule("<temp> Fahrenheit", true, "f(ahrenh?ayt)?", TemperatureUnit::Fahrenheit),
        rule(
            "below zero <temp>",
            vec![regex("sıfırın altında"), pred(is_value_only(true))],
            |ts| match ts.get(1)? {
                Token::Temperature(td) => negate_temp(td),
                _ => None,
            },
        ),
        rule(
            "<temp> below zero",
            vec![pred(is_value_only(true)), regex("sıfırın altında")],
            |ts| match ts.first()? {
                Token::Temperature(td) => negate_temp(td),
                _ => None,
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
