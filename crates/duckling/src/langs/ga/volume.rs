//! `Duckling.Volume.GA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::is_positive;
use crate::dims::volume::{unit_only, value_only, Unit};
use crate::types::{pred, regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let volumes: [(&str, &str, Unit); 4] = [
        ("<latent vol> ml", "m(l\\.?|h?illil(í|i)t(ea|i)r)", Unit::Millilitre),
        ("<vol> hectoliters", "heictil(í|i)t(ea|i)r", Unit::Hectolitre),
        ("<vol> liters", "(l(í|i)t(ea|i)r|l\\.?)", Unit::Litre),
        ("<latent vol> gallon", "n?gh?al(ú|u)i?n", Unit::Gallon),
    ];
    let mut v = vec![
        // ruleKiloliter
        rule(
            "kiloliter",
            vec![pred(is_positive), regex("(kl\\.?|g?ch?illil(í|i)t(ea|i)r)")],
            |ts| match ts.first()? {
                Token::Numeral(n) => {
                    let mut vd = value_only(10.0 * n.value);
                    vd.unit = Some(Unit::Hectolitre);
                    Some(Token::Volume(vd))
                }
                _ => None,
            },
        ),
    ];
    v.extend(volumes.into_iter().map(|(name, re, u)| {
        rule(name, vec![regex(re)], move |_| Some(Token::Volume(unit_only(u))))
    }));
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
