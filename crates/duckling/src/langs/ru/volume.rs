//! `Duckling.Volume.RU.Rules`.

use once_cell::sync::Lazy;

use crate::dims::volume::{is_unit_only, unit_only, volume, Unit, VolumeData};
use crate::types::{pred, regex, rule, Rule, Token};

fn tok(d: VolumeData) -> Option<Token> {
    Some(Token::Volume(d))
}

fn rule_half_liter() -> Rule {
    rule("half liter", vec![regex("поллитра")], |_| tok(volume(Unit::Litre, 0.5)))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let volumes: [(&str, &str, Unit); 4] = [
        ("<latent vol> ml", "мл|миллилитр(а|ов)?", Unit::Millilitre),
        ("<vol> hectoliters", "гл|гектолитр(а|ов)?", Unit::Hectolitre),
        ("<vol> liters", "л(итр(а|ов)?)?", Unit::Litre),
        ("<latent vol> gallon", "галлон(а|ов)?", Unit::Gallon),
    ];
    let mut v: Vec<Rule> = vec![rule_half_liter()];
    v.extend(volumes.into_iter().map(|(name, re, u)| {
        rule(name, vec![regex(re)], move |_| tok(unit_only(u)))
    }));
    let fractions: [(&str, &str, f64); 1] = [("half", "пол[-\\s]?", 1.0 / 2.0)];
    for (name, re, f) in fractions {
        v.push(rule(name, vec![regex(re), pred(is_unit_only)], move |ts| match ts.get(1)? {
            Token::Volume(VolumeData { unit: Some(u), .. }) => tok(volume(*u, f)),
            _ => None,
        }));
    }
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
