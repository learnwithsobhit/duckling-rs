//! `Duckling.Volume.AR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::volume::{unit_only, volume, Unit};
use crate::types::{regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = vec![
        rule("half liter", vec![regex("نصف? لي?تي?ر")], |_| {
            Some(Token::Volume(volume(Unit::Litre, 0.5)))
        }),
        rule("quarter liter", vec![regex("ربع لي?تي?ر")], |_| {
            Some(Token::Volume(volume(Unit::Litre, 0.25)))
        }),
        rule("two liters", vec![regex("لي?تي?ر(ان|ين)")], |_| {
            Some(Token::Volume(volume(Unit::Litre, 2.0)))
        }),
        rule("liter and half", vec![regex("لي?تي?ر و ?نصف?")], |_| {
            Some(Token::Volume(volume(Unit::Litre, 1.5)))
        }),
        rule("liter and quarter", vec![regex("لي?تي?ر و ?ربع")], |_| {
            Some(Token::Volume(volume(Unit::Litre, 1.25)))
        }),
    ];
    let volumes: [(&str, &str, Unit); 4] = [
        ("<latent vol> ml", "مي?لي?( ?لي?تي?ر)?", Unit::Millilitre),
        ("<vol> hectoliters", "(هي?كتو ?لي?تر)", Unit::Hectolitre),
        ("<vol> liters", "لي?تي?ر(ات)?", Unit::Litre),
        ("<latent vol> gallon", "[جغق]الون(ين|ان|ات)?", Unit::Gallon),
    ];
    v.extend(volumes.into_iter().map(|(name, re, u)| {
        rule(name, vec![regex(re)], move |_| Some(Token::Volume(unit_only(u))))
    }));
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
