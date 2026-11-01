//! `Duckling.Distance.KO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance::{distance, with_unit, Unit};
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

fn with_u(u: Unit, t: &Token) -> Option<Token> {
    match t {
        Token::Distance(dd) => Some(Token::Distance(with_unit(u, dd.clone()))),
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "<dist> centimeters",
            vec![dimension(Dim::Distance), regex("cm|센(티|치)((미|메)터)?")],
            |ts| with_u(Unit::Centimetre, ts.first()?),
        ),
        rule("<dist> meters", vec![dimension(Dim::Distance), regex("m|(미|메|매)터")], |ts| {
            with_u(Unit::Metre, ts.first()?)
        }),
        rule("<dist> miles", vec![dimension(Dim::Distance), regex("miles?|마일(즈)?")], |ts| {
            with_u(Unit::Mile, ts.first()?)
        }),
        // Upstream pattern is a regex, but the production matches a Numeral: never fires.
        rule("half", vec![regex("반")], |ts| match ts.first()? {
            Token::Numeral(n) => Some(Token::Distance(distance(n.value))),
            _ => None,
        }),
        rule(
            "<latent dist> feet",
            vec![dimension(Dim::Distance), regex("('|f(oo|ee)?ts?)|피트")],
            |ts| with_u(Unit::Foot, ts.first()?),
        ),
        rule(
            "<latent dist> feet and <latent dist> inch ",
            vec![
                dimension(Dim::Distance),
                regex("('|f(oo|ee)?ts?)|피트"),
                dimension(Dim::Distance),
                regex("(''|inch(es)?)|인치"),
            ],
            |ts| with_u(Unit::Foot, ts.first()?),
        ),
        rule(
            "<latent dist> inch",
            vec![dimension(Dim::Distance), regex("(''|inch(es)?)|인치")],
            |ts| with_u(Unit::Inch, ts.first()?),
        ),
        rule(
            "<latent dist> km",
            vec![dimension(Dim::Distance), regex("km|(킬|키)로((미|메)터)?")],
            |ts| with_u(Unit::Kilometre, ts.first()?),
        ),
        rule(
            "<latent dist> yard",
            vec![dimension(Dim::Distance), regex("y(ar)?ds?|야드")],
            |ts| with_u(Unit::Yard, ts.first()?),
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
