//! `Duckling.Distance.TR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance::{with_unit, Unit};
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

fn unit_rule(name: &str, re: &str, unit: Unit) -> Rule {
    rule(name, vec![dimension(Dim::Distance), regex(re)], move |ts| match ts.first()? {
        Token::Distance(dd) => Some(Token::Distance(with_unit(unit, dd.clone()))),
        _ => None,
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        unit_rule("<dist> centimeters", "cm|santim(etre)?", Unit::Centimetre),
        unit_rule("<dist> meters", "m(etre)?", Unit::Metre),
        unit_rule("<dist> miles", "mil", Unit::Mile),
        unit_rule("<latent dist> km", "k(ilo)?m?(etre)?", Unit::Kilometre),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
