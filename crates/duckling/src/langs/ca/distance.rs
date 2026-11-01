//! `Duckling.Distance.CA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance::{with_unit, Unit};
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let units: [(&str, &str, Unit); 4] = [
        ("<dist> centimeters", "(cm|cent(í|i)m(etres?))", Unit::Centimetre),
        ("<dist> meters", "m(etres?)?", Unit::Metre),
        ("<dist> miles", "milles?", Unit::Mile),
        ("<latent dist> km", "(k|qu)(il(o|ò|ó))?m?(etre)?s?", Unit::Kilometre),
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
