//! `Duckling.Ordinal.VI.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::types::{regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

fn rule_ordinals() -> Rule {
    rule("ordinals", vec![regex("(đầu tiên|thứ nhất|thứ 1)")], |_| ordinal(1))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![rule_ordinals()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
