//! `Duckling.PhoneNumber.PT.Rules`.

use once_cell::sync::Lazy;

use crate::dims::phone_number::PhoneNumberData;
use crate::dims::util::parse_integer;
use crate::types::{groups, regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let pattern = String::new()
        + r"(?:\(?\+(\d{1,2})\)?[\s-\.]*)?" // area code
        + r"((?=[-\d()\s\.]{6,16}(?:\s*ramal\.?\s*(?:\d{1,20}))?(?:[^\d]+|$))(?:[\d(]{1,20}(?:[-)\s\.]*\d{1,20}){0,20}){1,20})" // nums
        + r"(?:\s*ramal\.?\s*(\d{1,20}))?"; // extension
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
