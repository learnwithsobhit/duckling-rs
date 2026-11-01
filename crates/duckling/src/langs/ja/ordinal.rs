//! `Duckling.Ordinal.JA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::types::{dimension, rule, regex, Dim, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

fn rule_ordinal_digits() -> Rule {
    rule(
        "ordinal (digits)",
        vec![regex("第"), dimension(Dim::Numeral)],
        |ts| match ts.get(1)? {
            Token::Numeral(n) => ordinal(n.value.floor() as i64),
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![rule_ordinal_digits()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
