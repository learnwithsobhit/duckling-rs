//! `Duckling.Volume.KO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::volume::{unit_only, Unit};
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let volumes: [(&str, &str, Unit); 4] = [
        ("<latent vol> ml", "ml|(밀|미)리리터", Unit::Millilitre),
        ("<vol> hectoliters", "(핵|헥)토리터", Unit::Hectolitre),
        ("<vol> liters", "(l|리터)", Unit::Litre),
        ("<latent vol> gallon", "gal(l?ons?)?|갤(런|론)", Unit::Gallon),
    ];
    volumes
        .into_iter()
        .map(|(name, re, u)| rule(name, vec![regex(re)], move |_| Some(Token::Volume(unit_only(u)))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
