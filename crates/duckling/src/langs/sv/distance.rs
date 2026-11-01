//! `Duckling.Distance.SV.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance::{with_unit, DistanceData, Unit};
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

fn tok(d: DistanceData) -> Option<Token> {
    Some(Token::Distance(d))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleDistCentimeters
        rule(
            "<dist> centimeters",
            vec![dimension(Dim::Distance), regex("cm|centimeter")],
            |ts| match ts.first()? {
                Token::Distance(dd) => tok(with_unit(Unit::Centimetre, dd.clone())),
                _ => None,
            },
        ),
        // ruleDistMeters
        rule("<dist> meter", vec![dimension(Dim::Distance), regex("m(eter)?")], |ts| {
            match ts.first()? {
                Token::Distance(dd) => tok(with_unit(Unit::Metre, dd.clone())),
                _ => None,
            }
        }),
        // ruleDistMils — Swedish mil = 10 km
        rule("<dist> mils", vec![dimension(Dim::Distance), regex("mils?")], |ts| {
            match ts.first()? {
                Token::Distance(dd @ DistanceData { value: Some(x), .. }) => {
                    let mut out = with_unit(Unit::Kilometre, dd.clone());
                    out.value = Some(10.0 * *x);
                    tok(out)
                }
                _ => None,
            }
        }),
        // ruleLatentDistKm
        rule(
            "<latent dist> km",
            vec![dimension(Dim::Distance), regex("k(ilo)?m?(eter)?")],
            |ts| match ts.first()? {
                Token::Distance(dd) => tok(with_unit(Unit::Kilometre, dd.clone())),
                _ => None,
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
