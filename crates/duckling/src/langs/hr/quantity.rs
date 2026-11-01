//! `Duckling.Quantity.HR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::quantity::{quantity, Unit};
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, regex, rule, Dim, Rule, Token};

fn rule_number_units() -> Rule {
    rule(
        "<number> <units>",
        vec![dimension(Dim::Numeral), regex(r"k(il(o|e|a))?(g(rama?)?)?")],
        |ts| match ts.first()? {
            Token::Numeral(n) => Some(Token::Quantity(quantity(Unit::Gram, 1000.0 * n.value))),
            _ => None,
        },
    )
}

fn rule_quantity_product() -> Rule {
    rule(
        "<quantity> product",
        vec![dimension(Dim::Quantity), regex("(mes(o|a)|soli?)")],
        |ts| match ts.first()? {
            Token::Quantity(q) => {
                let m = to_lower(groups(&ts[1..])?.first()?);
                let product = match m.as_str() {
                    "meso" | "mesa" => "meso",
                    "sol" | "soli" => "sol",
                    _ => return None,
                };
                let mut q = q.clone();
                q.aproduct = Some(product.to_string());
                Some(Token::Quantity(q))
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![rule_number_units(), rule_quantity_product()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
