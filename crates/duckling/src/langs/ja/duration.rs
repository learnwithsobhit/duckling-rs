//! `Duckling.Duration.JA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::duration;
use crate::dims::numeral::number_with;
use crate::dims::time_grain::Grain;
use crate::types::{regex, rule, Rule, Token};

fn rule_duration_month_with_counter() -> Rule {
    rule(
        "<integer> counter months",
        vec![number_with(|n| n.value, |v| v >= 1.0), regex("(ケ|ヶ|カ|ヵ|か|箇)(月|げつ|つき)")],
        |ts| match ts.first()? {
            Token::Numeral(n) => Some(Token::Duration(duration(Grain::Month, n.value.floor() as i64))),
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![rule_duration_month_with_counter()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
