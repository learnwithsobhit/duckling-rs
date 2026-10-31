//! `Duckling.Email.*` (common rules + EN).

use once_cell::sync::Lazy;

use crate::json::Json;
use crate::types::{group, groups, regex, rule, Rule, Token};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EmailData {
    pub value: String,
}

impl EmailData {
    pub fn resolve(&self) -> Json {
        Json::obj([("value", Json::str(self.value.clone())), ("type", Json::str("value"))])
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule("email", vec![regex(r"([\w\._+-]+@[\w_-]+(\.[\w_-]+)+)")], |ts| {
        let x = group(ts, 0)?;
        Some(Token::Email(EmailData { value: x.to_string() }))
    })]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}

static EN_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule(
        "email spelled out",
        vec![regex(
            r"([\w_+-]+(?:(?: dot |\.)[\w_+-]+){0,10})(?: at |@)([a-zA-Z]+(?:(?:\.| dot )[\w_-]+){1,10})",
        )],
        |ts| {
            let g = groups(ts)?;
            let replace_dots = |s: &str| s.replace(" dot ", ".");
            let value = format!("{}@{}", replace_dots(g.first()?), replace_dots(g.get(1)?));
            Some(Token::Email(EmailData { value }))
        },
    )]
});

pub fn en_rules() -> &'static [Rule] {
    &EN_RULES
}
