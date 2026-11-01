//! `Duckling.Email.FR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::email::EmailData;
use crate::types::{groups, regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule(
        "email spelled out",
        vec![regex(r"([\w\._+-]+) arobase ([\w_-]+(\.[\w_-]+)+)")],
        |ts| {
            let g = groups(ts)?;
            Some(Token::Email(EmailData { value: format!("{}@{}", g.first()?, g.get(1)?) }))
        },
    )]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
