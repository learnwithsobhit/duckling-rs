//! `Duckling.Temperature.CA.Rules`.

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
            vec![pred(is_value_only(false)), regex("(gra(d|u)s?|°)")],
            |ts| with_u(TemperatureUnit::Degree, ts.first()?),
        ),
        rule(
            "<latent temp> temp bajo zero",
            vec![pred(is_value_only(true)), regex("sota zero")],
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
            vec![pred(is_value_only(true)), regex("(cent(i|í)gra(d|u)s?|(C|c)(el[cs]?(ius)?)?\\.?)")],
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
