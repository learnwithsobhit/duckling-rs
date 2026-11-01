//! `Duckling.Temperature.AR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::temperature::{is_value_only, value_only, with_unit, TemperatureUnit};
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
            "<latent temp> degrees",
            vec![pred(is_value_only(false)), regex("(درج([ةه]|ات)( مئوي[ةه])?)|°")],
            |ts| with_u(TemperatureUnit::Degree, ts.first()?),
        ),
        rule("two degrees", vec![regex("درجت(ين|ان)")], |_| {
            Some(Token::Temperature(value_only(2.0)))
        }),
        rule(
            "<temp> Celsius",
            vec![pred(is_value_only(true)), regex("(درج([ةه]|ات) )?سي?لي?[سز]ي?وس")],
            |ts| with_u(TemperatureUnit::Celsius, ts.first()?),
        ),
        rule(
            "<temp> Fahrenheit",
            vec![pred(is_value_only(true)), regex("(درج([ةه]|ات) )?ف(ا|ي)?هرنها?يت")],
            |ts| with_u(TemperatureUnit::Fahrenheit, ts.first()?),
        ),
        rule("<temp> below zero", vec![pred(is_value_only(true)), regex("تحت الصفر")], |ts| {
            match ts.first()? {
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
            }
        }),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
