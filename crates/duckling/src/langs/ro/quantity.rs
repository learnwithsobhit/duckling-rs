//! `Duckling.Quantity.RO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::is_positive;
use crate::dims::quantity::{quantity, Unit};
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("<number> <units>", vec![pred(is_positive), regex("(de )?livr(a|e|ă)")], |ts| {
            match ts.first()? {
                Token::Numeral(n) => Some(Token::Quantity(quantity(Unit::Pound, n.value))),
                _ => None,
            }
        }),
        rule(
            "<quantity> of product",
            vec![dimension(Dim::Quantity), regex("de (carne|can[aă]|zah[aă]r|mamaliga)")],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Quantity(qd), Token::RegexMatch(g)) => {
                    let mut q = qd.clone();
                    q.aproduct = Some(g.0.first()?.to_string());
                    Some(Token::Quantity(q))
                }
                _ => None,
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
