//! `Duckling.Distance.CS.Rules`.

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
        // ruleDistCentimeters
        unit_rule("<dist> centimeters", "cm|centimetr(y|ů)?", Unit::Centimetre),
        // ruleDistMeters
        unit_rule("<dist> meters", "metr(y|ů)?|m", Unit::Metre),
        // ruleDistMiles
        unit_rule("<dist> miles", "mil", Unit::Mile),
        // ruleLatentDistKm
        unit_rule("<latent dist> km", "km|kilometr(y|ů)?", Unit::Kilometre),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
