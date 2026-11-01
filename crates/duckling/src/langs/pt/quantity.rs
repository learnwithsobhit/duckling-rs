//! `Duckling.Quantity.PT.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::is_positive;
use crate::dims::quantity::{quantity, Unit};
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

/// `getValue opsMap match`.
fn get_value(m: &str, v: f64) -> f64 {
    match to_lower(m).as_str() {
        "miligrama" | "miligramas" | "mg" | "mgs" => v / 1000.0,
        "quilograma" | "quilogramas" | "quilo" | "quilos" | "kg" | "kgs" => v * 1000.0,
        _ => v,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let quantities: [(&str, &str, fn() -> Unit); 3] = [
        ("<quantity> copos", "(copos?)", || Unit::Cup),
        ("<quantity> gramas", "((((mili)|(quilo))?(grama)s?)|(quilos?)|((m|k)?g))", || Unit::Gram),
        ("<quantity> libras", "((lb|libra)s?)", || Unit::Pound),
    ];
    let mut v: Vec<Rule> = vec![rule(
        "<quantity> of product",
        vec![dimension(Dim::Quantity), regex("de (\\w+)")],
        |ts| {
            let q = match ts.first()? {
                Token::Quantity(q) => q,
                _ => return None,
            };
            let p = groups(&ts[1..])?.first()?.clone();
            let mut q = q.clone();
            q.aproduct = Some(to_lower(&p));
            Some(Token::Quantity(q))
        },
    )];
    for (name, re, unit) in quantities {
        v.push(rule(name, vec![pred(is_positive), regex(re)], move |ts| {
            let n = match ts.first()? {
                Token::Numeral(n) => n.value,
                _ => return None,
            };
            let m = match ts.get(1)? {
                Token::RegexMatch(g) => g.0.first()?.clone(),
                _ => return None,
            };
            Some(Token::Quantity(quantity(unit(), get_value(&m, n))))
        }));
    }
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
