//! `Duckling.Temperature.ES.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, with_unit, TemperatureUnit};
use crate::types::{pred, regex, rule, Rule, Token};

fn with_u(u: TemperatureUnit, t: &Token) -> Option<Token> {
    match t {
        Token::Temperature(td) => Some(Token::Temperature(with_unit(u, td.clone()))),
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "<latent temp> temp",
            vec![pred(is_value_only(false)), regex("(grados?)|°")],
            |ts| with_u(TemperatureUnit::Degree, ts.first()?),
        ),
        rule(
            "<latent temp> temp bajo cero",
            vec![pred(is_value_only(true)), regex("bajo cero")],
            |ts| match ts.first()? {
                Token::Temperature(td) => {
                    let v = td.value?;
                    let mut neg = td.clone();
                    neg.value = Some(-v);
                    match td.unit {
                        None => Some(Token::Temperature(with_unit(TemperatureUnit::Degree, neg))),
                        _ => Some(Token::Temperature(neg)),
                    }
                }
                _ => None,
            },
        ),
        rule(
            "<temp> Celsius",
            vec![pred(is_value_only(true)), regex("(cent(i|í)grados?|c(el[cs]?(ius)?)?\\.?)")],
            |ts| with_u(TemperatureUnit::Celsius, ts.first()?),
        ),
        rule(
            "<temp> Fahrenheit",
            vec![pred(is_value_only(true)), regex("f(ah?reh?n(h?eit)?)?\\.?")],
            |ts| with_u(TemperatureUnit::Fahrenheit, ts.first()?),
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
