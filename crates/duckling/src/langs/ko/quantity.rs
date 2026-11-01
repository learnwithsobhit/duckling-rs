//! `Duckling.Quantity.KO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::quantity::{quantity, QuantityData, Unit};
use crate::types::{dimension, groups, regex, rule, Dim, Rule, Token};

fn tok(d: QuantityData) -> Option<Token> {
    Some(Token::Quantity(d))
}

fn with_product(p: String, mut qd: QuantityData) -> QuantityData {
    qd.aproduct = Some(p);
    qd
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "<number> <units>",
            vec![dimension(Dim::Numeral), regex("(개|판|그(램|람)|근|파운(드|즈)|접1시|그릇|컵)")],
            |ts| {
                let v = match ts.first()? {
                    Token::Numeral(n) => n.value,
                    _ => return None,
                };
                let m = groups(&ts[1..])?.first()?.as_ref();
                match m {
                    "개" => tok(quantity(Unit::Unnamed, v)),
                    "판" => tok(quantity(Unit::Custom("판".to_string()), v)),
                    "근" => tok(quantity(Unit::Custom("근".to_string()), v)),
                    "그램" => tok(quantity(Unit::Gram, v)),
                    "그람" => tok(quantity(Unit::Gram, v)),
                    "파운드" => tok(quantity(Unit::Pound, v)),
                    "파운즈" => tok(quantity(Unit::Pound, v)),
                    "접1시" => tok(quantity(Unit::Dish, v)),
                    "그릇" => tok(quantity(Unit::Bowl, v)),
                    "컵" => tok(quantity(Unit::Cup, v)),
                    _ => None,
                }
            },
        ),
        rule(
            "<quantity> of product",
            vec![dimension(Dim::Quantity), regex("의 (삼겹살|콜라)")],
            |ts| {
                let qd = match ts.first()? {
                    Token::Quantity(q) => q.clone(),
                    _ => return None,
                };
                let p = groups(&ts[1..])?.first()?.to_string();
                tok(with_product(p, qd))
            },
        ),
        rule(
            "<quantity> of product",
            vec![regex("(삼겹살|콜라)"), dimension(Dim::Quantity)],
            |ts| {
                let p = groups(ts)?.first()?.to_string();
                let qd = match ts.get(1)? {
                    Token::Quantity(q) => q.clone(),
                    _ => return None,
                };
                tok(with_product(p, qd))
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
