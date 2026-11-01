//! `Duckling.Email.DE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::email::EmailData;
use crate::types::{groups, regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule(
        "email spelled out",
        vec![regex(
            r"([\w_+-]+(?:(?: punkt |\.)[\w_+-]+){0,10})(?: at |@)([a-zA-Z]+(?:(?:\.| punkt )[\w_-]+){1,10})",
        )],
        |ts| {
            let g = groups(ts)?;
            let replace_dots = |s: &str| s.replace(" punkt ", ".");
            let value = format!("{}@{}", replace_dots(g.first()?), replace_dots(g.get(1)?));
            Some(Token::Email(EmailData { value }))
        },
    )]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
