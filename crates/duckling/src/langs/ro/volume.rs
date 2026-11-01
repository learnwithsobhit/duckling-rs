//! `Duckling.Volume.RO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::volume::{is_unit_only, unit_only, volume, Unit, VolumeData};
use crate::types::{pred, regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let volumes: [(&str, &str, Unit); 4] = [
        ("<latent vol> ml", "(de )?m(ililitr[ui]|l)", Unit::Millilitre),
        ("<vol> hectoliters", "(de )?hectolitr[ui]", Unit::Hectolitre),
        ("<vol> liters", "(de )?l(itr[ui])?", Unit::Litre),
        ("<latent vol> gallon", "(de )?gal(oane|on)?", Unit::Gallon),
    ];
    let mut v: Vec<Rule> = Vec::new();
    for (name, re, u) in volumes {
        v.push(rule(name, vec![regex(re)], move |_| Some(Token::Volume(unit_only(u)))));
    }
    let fractions: [(&str, &str, f64); 1] = [("half", "jum(a|ă)tate", 1.0 / 2.0)];
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
