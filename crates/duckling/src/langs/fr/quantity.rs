//! `Duckling.Quantity.FR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::quantity::{quantity, Unit};
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, regex, rule, Dim, Rule, Token};

fn rule_numeral_units() -> Rule {
    rule(
        "<number> <units>",
        vec![dimension(Dim::Numeral), regex("(tasses?|cuill?(e|è)res? (a|à) soupe?)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(n), Token::RegexMatch(g)) => {
                let u = match to_lower(g.0.first()?).as_str() {
                    "tasse" | "tasses" => Unit::Cup,
                    _ => Unit::Tablespoon,
                };
                Some(Token::Quantity(quantity(u, n.value)))
            }
            _ => None,
        },
    )
}

fn rule_quantity_of_product() -> Rule {
    rule(
        "<quantity> of product",
        vec![dimension(Dim::Quantity), regex("de (caf(e|é)|sucre)")],
        |ts| match ts.first()? {
            Token::Quantity(q) => {
                let m = groups(&ts[1..])?.first()?.to_string();
                let mut q = q.clone();
                q.aproduct = Some(m);
                Some(Token::Quantity(q))
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![rule_numeral_units(), rule_quantity_of_product()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
