//! `Duckling.Distance.MN.Rules`.

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
        unit_rule("<latent dist> km", "км|километр", Unit::Kilometre),
        unit_rule("<latent dist> feet", "('|фут)", Unit::Foot),
        unit_rule("<latent dist> inch", "(\"|''|дюйм|инч)", Unit::Inch),
        unit_rule("<latent dist> yard", "яард", Unit::Yard),
        unit_rule("<dist> meters", "м(етр(ийн|ээр)?)?", Unit::Metre),
        unit_rule("<dist> centimeters", "см|сантиметр", Unit::Centimetre),
        unit_rule("<dist> millimeters", "мм|миллиметр", Unit::Millimetre),
        unit_rule("<dist> miles", "мил(и|ь)", Unit::Mile),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
