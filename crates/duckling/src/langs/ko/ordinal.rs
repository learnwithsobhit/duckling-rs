//! `Duckling.Ordinal.KO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule(
        "ordinals (첫번째)",
        vec![dimension(Dim::Numeral), regex("번째|째(번)?")],
        |ts| match ts.first()? {
            Token::Numeral(n) => ordinal(n.value.floor() as i64),
            _ => None,
        },
    )]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
