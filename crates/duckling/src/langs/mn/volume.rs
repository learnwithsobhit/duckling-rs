//! `Duckling.Volume.MN.Rules`.

use once_cell::sync::Lazy;

use crate::dims::volume::{unit_only, Unit};
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let volumes: [(&str, &str, Unit); 4] = [
        ("<latent vol> ml", "мл|миллилитр", Unit::Millilitre),
        ("<vol> hectoliters", "гл|гектолитр", Unit::Hectolitre),
        ("<vol> liters", "л(итр(ийн|ээр)?)?", Unit::Litre),
        ("<latent vol> gallon", "галлон", Unit::Gallon),
    ];
    volumes
        .into_iter()
        .map(|(name, re, u)| rule(name, vec![regex(re)], move |_| Some(Token::Volume(unit_only(u)))))
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
