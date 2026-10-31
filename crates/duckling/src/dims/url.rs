//! `Duckling.Url.*`.

use once_cell::sync::Lazy;

use crate::dims::util::to_lower;
use crate::json::Json;
use crate::types::{groups, regex, rule, Rule, Token};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UrlData {
    pub value: String,
    pub domain: String,
}

impl UrlData {
    pub fn resolve(&self) -> Json {
        Json::obj([
            ("value", Json::str(self.value.clone())),
            ("domain", Json::str(self.domain.clone())),
            ("type", Json::str("value")),
        ])
    }
}

fn url(value: &str, domain: &str) -> Option<Token> {
    Some(Token::Url(UrlData { value: value.to_string(), domain: to_lower(domain) }))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "url",
            vec![regex(r"((([a-zA-Z]+)://)?(w{2,3}[0-9]*\.)?(([\w_-]+\.)+[a-z]{2,4})(:(\d+))?(/[^?\s#]*)?(\?[^\s#]+)?(#[\-,*=&a-z0-9]+)?)")],
            |ts| {
                let g = groups(ts)?;
                // (m : _ : _protocol : _ : domain : ...)
                url(g.first()?, g.get(4)?)
            },
        ),
        rule(
            "localhost",
            vec![regex(r"((([a-zA-Z]+)://)?localhost(:(\d+))?(/[^?\s#]*)?(\?[^\s#]+)?)")],
            |ts| {
                let g = groups(ts)?;
                url(g.first()?, "localhost")
            },
        ),
        rule(
            "local url",
            vec![regex(r"(([a-zA-Z]+)://([\w_-]+)(:(\d+))?(/[^?\s#]*)?(\?[^\s#]+)?)")],
            |ts| {
                let g = groups(ts)?;
                // (m : _protocol : domain : ...)
                url(g.first()?, g.get(2)?)
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
