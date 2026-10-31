//! `Duckling.PhoneNumber.*`.

use once_cell::sync::Lazy;

use crate::dims::util::parse_integer;
use crate::json::Json;
use crate::types::{groups, regex, rule, Rule, Token};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhoneNumberData {
    pub prefix: Option<i128>,
    pub number: String,
    pub extension: Option<i128>,
}

impl PhoneNumberData {
    pub fn resolve(&self) -> Json {
        let p = match self.prefix {
            Some(p) => format!("(+{}) ", p),
            None => String::new(),
        };
        let e = match self.extension {
            Some(e) => format!(" ext {}", e),
            None => String::new(),
        };
        Json::obj([
            ("value", Json::str(format!("{}{}{}", p, self.number, e))),
            ("type", Json::str("value")),
        ])
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let pattern = String::new()
        + r"(?:\(?\+(\d{1,2})\)?[\s-\.]*)?" // area code
        + r"((?=[-\d()\s\.]{6,16}(?:\s*e?xt?\.?\s*(?:\d{1,20}))?(?:[^\d]+|$))(?:[\d(]{1,20}(?:[-)\s\.]*\d{1,20}){0,20}){1,20})" // nums
        + r"(?:\s*e?xt?\.?\s*(\d{1,20}))?"; // extension
    vec![rule("phone number", vec![regex(&pattern)], |ts| {
        let g = groups(ts)?;
        let (code, nums, ext) = (g.first()?, g.get(1)?, g.get(2)?);
        let is_ws = |c: char| ['.', ' ', '-', '\t', '(', ')'].contains(&c);
        Some(Token::PhoneNumber(PhoneNumberData {
            prefix: parse_integer(code),
            number: nums.chars().filter(|c| !is_ws(*c)).collect(),
            extension: parse_integer(ext),
        }))
    })]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
