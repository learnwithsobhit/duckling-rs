//! `Duckling.Distance.GA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance::{with_unit, Unit};
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

fn tok(d: crate::dims::distance::DistanceData) -> Option<Token> {
    Some(Token::Distance(d))
}

fn dist_unit(name: &str, re: &str, unit: Unit) -> Rule {
    rule(name, vec![dimension(Dim::Distance), regex(re)], move |ts| match ts.first()? {
        Token::Distance(dd) => tok(with_unit(unit, dd.clone())),
        _ => None,
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleDistCentimeters
        dist_unit("<dist> centimeters", "(c\\.?m\\.?|g?ch?eintimh?(e|é)adai?r)", Unit::Centimetre),
        // ruleDistMAmbiguousMilesOrMeters
        dist_unit("<dist> m (ambiguous miles or meters)", "m", Unit::M),
        // ruleDistMeters
        dist_unit("<dist> meters", "mh?(e|é)adai?r", Unit::Metre),
        // ruleDistMiles
        dist_unit("<dist> miles", "mh?(í|i)lt?e", Unit::Mile),
        // ruleLatentDistKm
        dist_unit("<latent dist> km", "(k\\.?(m\\.?)?|g?ch?ilim(e|é)adai?r)", Unit::Kilometre),
        // ruleLatentDistOrlach
        dist_unit("<latent dist> orlach", "(''|([nth]-?)?orl(ach|aigh|a(í|i)|\\.))", Unit::Inch),
        // ruleLatentDistTroigh
        dist_unit("<latent dist> troigh", "('|d?th?roi[tg]he?|tr\\.?)", Unit::Foot),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
