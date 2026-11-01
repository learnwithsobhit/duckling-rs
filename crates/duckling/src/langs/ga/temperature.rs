//! `Duckling.Temperature.GA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, with_unit, TemperatureData, TemperatureUnit};
use crate::types::{pred, regex, rule, Rule, Token};

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
        // ruleLatentTempCim
        rule(
            "<latent temp> céim",
            vec![pred(is_value_only(false)), regex("g?ch?(é|e)im(e(anna)?)?|°")],
            |ts| match ts.first()? {
                Token::Temperature(td) => {
                    Some(Token::Temperature(with_unit(TemperatureUnit::Degree, td.clone())))
                }
                _ => None,
            },
        ),
        // ruleLatentTempFaoiBhunNid
        rule(
            "<latent temp> faoi bhun náid",
            vec![pred(is_value_only(true)), regex("faoi bhun (0|n(a|á)id)")],
            |ts| match ts.first()? {
                Token::Temperature(td) => negate_temp(td),
                _ => None,
            },
        ),
        // ruleTempCelsius
        rule(
            "<temp> Celsius",
            vec![pred(is_value_only(true)), regex("ceinteagr(á|a)d|c(el[cs]?(ius)?)?\\.?")],
            |ts| match ts.first()? {
                Token::Temperature(td) => {
                    Some(Token::Temperature(with_unit(TemperatureUnit::Celsius, td.clone())))
                }
                _ => None,
            },
        ),
        // ruleTempFahrenheit
        rule(
            "<temp> Fahrenheit",
            vec![pred(is_value_only(true)), regex("f(ah?reh?n(h?eit)?)?\\.?")],
            |ts| match ts.first()? {
                Token::Temperature(td) => {
                    Some(Token::Temperature(with_unit(TemperatureUnit::Fahrenheit, td.clone())))
                }
                _ => None,
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
