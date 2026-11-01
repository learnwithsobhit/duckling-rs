//! `Duckling.Distance.RU.Rules`.

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
        unit_rule("<latent dist> km", "км|километр(а|ов)?", Unit::Kilometre),
        unit_rule("<latent dist> feet", "('|фут(а|ов)?)", Unit::Foot),
        unit_rule("<latent dist> inch", "(\"|''|дюйм(а|ов)?)", Unit::Inch),
        unit_rule("<latent dist> yard", "ярд(а|ов)?", Unit::Yard),
        unit_rule("<dist> meters", "м(етр(а|ов)?)?", Unit::Metre),
        unit_rule("<dist> centimeters", "см|сантиметр(а|ов)?", Unit::Centimetre),
        unit_rule("<dist> millimeters", "мм|миллиметр(а|ов)?", Unit::Millimetre),
        unit_rule("<dist> miles", "мил(я|и|ь)", Unit::Mile),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
