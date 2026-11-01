//! `Duckling.Volume.CA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::volume::{is_unit_only, unit_only, volume, Unit, VolumeData};
use crate::types::{pred, regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let volumes: [(&str, &str, Unit); 4] = [
        ("<latent vol> ml", "m(l|ililitres?)", Unit::Millilitre),
        ("<vol> hectoliters", "(hectolitres?)", Unit::Hectolitre),
        ("<vol> liters", "l(itres?)?", Unit::Litre),
        ("<latent vol> gallon", "gal(ons|ó)", Unit::Gallon),
    ];
    let mut v: Vec<Rule> = volumes
        .into_iter()
        .map(|(name, re, u)| rule(name, vec![regex(re)], move |_| Some(Token::Volume(unit_only(u)))))
        .collect();
    let fractions: [(&str, &str, f64); 1] = [("half", "mig", 1.0 / 2.0)];
    for (name, re, f) in fractions {
        v.push(rule(name, vec![regex(re), pred(is_unit_only)], move |ts| match ts.get(1)? {
            Token::Volume(VolumeData { unit: Some(u), .. }) => Some(Token::Volume(volume(*u, f))),
            _ => None,
        }));
    }
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
