//! `Duckling.Temperature.KO.Rules`.

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
            "<latent temp> degrees",
            vec![pred(is_value_only(false)), regex("도|°")],
            |ts| with_u(TemperatureUnit::Degree, ts.first()?),
        ),
        rule("섭씨 <temp>", vec![regex("섭씨"), pred(is_value_only(true))], |ts| {
            with_u(TemperatureUnit::Celsius, ts.get(1)?)
        }),
        rule("화씨 <temp>", vec![regex("화씨"), pred(is_value_only(true))], |ts| {
            with_u(TemperatureUnit::Fahrenheit, ts.get(1)?)
        }),
        rule("<temp> °C", vec![pred(is_value_only(true)), regex("c")], |ts| {
            with_u(TemperatureUnit::Celsius, ts.first()?)
        }),
        rule("<temp> °F", vec![pred(is_value_only(true)), regex("f")], |ts| {
            with_u(TemperatureUnit::Fahrenheit, ts.first()?)
        }),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
