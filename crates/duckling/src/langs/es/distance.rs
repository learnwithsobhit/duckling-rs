//! `Duckling.Distance.ES.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance::{with_unit, Unit};
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let units: [(&str, &str, Unit); 4] = [
        ("<dist> centimeters", "(cm|cent(í|i)m(etros?))", Unit::Centimetre),
        ("<dist> meters", "m(etros?)?", Unit::Metre),
        ("<dist> miles", "miles?", Unit::Mile),
        ("<latent dist> km", "k(il(ó|o))?m?(etro)?s?", Unit::Kilometre),
    ];
    units
        .into_iter()
        .map(|(name, re, u)| {
            rule(name, vec![dimension(Dim::Distance), regex(re)], move |ts| match ts.first()? {
                Token::Distance(dd) => Some(Token::Distance(with_unit(u, dd.clone()))),
                _ => None,
            })
        })
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
