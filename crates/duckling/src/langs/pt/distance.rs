//! `Duckling.Distance.PT.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance::{with_unit, Unit};
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

fn unit_rule(name: &str, re: &str, u: Unit) -> Rule {
    rule(name, vec![dimension(Dim::Distance), regex(re)], move |ts| match ts.first()? {
        Token::Distance(dd) => Some(Token::Distance(with_unit(u, dd.clone()))),
        _ => None,
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        unit_rule("<dist> centimeters", "(cm|cent(í|i)m(etros?))", Unit::Centimetre),
        unit_rule("<dist> meters", "m(etros?)?", Unit::Metre),
        unit_rule("<dist> miles", "milhas?", Unit::Mile),
        unit_rule("<latent dist> km", "k(il(ó|o))?m?(etro)?s?", Unit::Kilometre),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
