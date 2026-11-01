//! `Duckling.PhoneNumber.AR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::phone_number::PhoneNumberData;
use super::numeral::{parse_arabic_int_as_text_pub, parse_arabic_integer_from_text_i128_pub};
use crate::types::{groups, regex, rule, Rule, Token};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    // Haskell `\1632-\1641` is U+0660–U+0669 (٠-٩); `\1601\1585\1593\1610` is فرعي.
    let pattern = String::new()
        + r"(?:\(?\+([٠-٩]{1,2})\)?[\s-\.]*)?"
        + r"((?=[-٠-٩()\s\.]{6,16}(?:\s*(?:e?xt?|فرعي)?\.?\s*(?:[٠-٩]{1,20}))?(?:[^٠-٩]+|$))(?:[٠-٩(]{1,20}(?:[-)\s\.]*[٠-٩]{1,20}){0,20}){1,20})"
        + r"(?:\s*(?:e?xt?|فرعي)\.?\s*([٠-٩]{1,20}))?";
    vec![rule("phone number", vec![regex(&pattern)], |ts| {
        let g = groups(ts)?;
        let (code, nums, ext) = (g.first()?, g.get(1)?, g.get(2)?);
        let is_ws = |c: char| ['.', ' ', '-', '\t', '(', ')'].contains(&c);
        let cleaned: String = nums.chars().filter(|c| !is_ws(*c)).collect();
        Some(Token::PhoneNumber(PhoneNumberData {
            prefix: parse_arabic_integer_from_text_i128_pub(code),
            number: parse_arabic_int_as_text_pub(&cleaned),
            extension: parse_arabic_integer_from_text_i128_pub(ext),
        }))
    })]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
