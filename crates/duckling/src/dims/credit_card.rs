//! `Duckling.CreditCardNumber.*`.

use once_cell::sync::Lazy;

use crate::json::Json;
use crate::types::{group, regex, rule, Rule, Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Issuer {
    Visa,
    Amex,
    Discover,
    Mastercard,
    DinerClub,
    Other,
}

impl Issuer {
    fn json(self) -> Json {
        Json::str(
            match self {
                Issuer::Visa => "visa",
                Issuer::Amex => "amex",
                Issuer::Discover => "discover",
                Issuer::Mastercard => "mastercard",
                Issuer::DinerClub => "dinerclub",
                Issuer::Other => "other",
            }
            .to_string(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CreditCardNumberData {
    pub number: String,
    pub issuer: Issuer,
}

impl CreditCardNumberData {
    pub fn resolve(&self) -> Json {
        Json::obj([("value", Json::str(self.number.clone())), ("issuer", self.issuer.json())])
    }
}

pub const MIN_NUMBER_DIGITS: usize = 8;
pub const MAX_NUMBER_DIGITS: usize = 19;

fn with_dashes_alt(without: &str, with: &str) -> String {
    format!("({}|{})", without, with)
}

pub fn visa_regex() -> String {
    with_dashes_alt("4[0-9]{15}", "4[0-9]{3}-[0-9]{4}-[0-9]{4}-[0-9]{4}")
}
pub fn amex_regex() -> String {
    with_dashes_alt("3[47][0-9]{13}", "3[47][0-9]{2}-[0-9]{6}-[0-9]{5}")
}
pub fn discover_regex() -> String {
    with_dashes_alt(
        "6(?:011|[45][0-9]{2})[0-9]{12}",
        "6(?:011|[45][0-9]{2})-[0-9]{4}-[0-9]{4}-[0-9]{4}",
    )
}
pub fn mastercard_regex() -> String {
    with_dashes_alt("5[1-5][0-9]{14}", "5[1-5][0-9]{2}-[0-9]{4}-[0-9]{4}-[0-9]{4}")
}
pub fn diner_club_regex() -> String {
    with_dashes_alt(
        "3(?:0[0-5]|[68][0-9])[0-9]{11}",
        "3(?:0[0-5]|[68][0-9])[0-9]-[0-9]{6}-[0-9]{4}",
    )
}
pub fn other_regex() -> String {
    format!(
        "((?!{})(?!{})(?!{})(?!{})(?!{})\\d{{{},{}}})",
        visa_regex(),
        amex_regex(),
        discover_regex(),
        mastercard_regex(),
        diner_club_regex(),
        MIN_NUMBER_DIGITS,
        MAX_NUMBER_DIGITS
    )
}

/// Luhn check, mirroring `isValidCreditCardNumber` (including the length bounds).
pub fn is_valid_credit_card_number(cc: &str) -> bool {
    let n = cc.chars().count();
    if n < MIN_NUMBER_DIGITS || n > MAX_NUMBER_DIGITS {
        return false;
    }
    if !cc.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    // foldr from the right: the rightmost digit has e = 0, alternating.
    let mut checksum = 0u32;
    let mut e = 0u32;
    for c in cc.chars().rev() {
        let val = c.to_digit(10).unwrap();
        let mut d = val << e;
        if d > 9 {
            d -= 9;
        }
        checksum += d;
        e = 1 - e;
    }
    checksum % 10 == 0
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let cards: Vec<(&str, String, Issuer)> = vec![
        ("visa credit card number", visa_regex(), Issuer::Visa),
        ("amex card number", amex_regex(), Issuer::Amex),
        ("discover card number", discover_regex(), Issuer::Discover),
        ("mastercard card number", mastercard_regex(), Issuer::Mastercard),
        ("diner club card number", diner_club_regex(), Issuer::DinerClub),
        ("credit card number", other_regex(), Issuer::Other),
    ];
    cards
        .into_iter()
        .map(|(name, re, issuer)| {
            rule(name, vec![regex(&re)], move |ts| {
                let g = group(ts, 0)?;
                let cc: String = g.chars().filter(|c| c.is_ascii_digit()).collect();
                if is_valid_credit_card_number(&cc) {
                    Some(Token::CreditCardNumber(CreditCardNumberData { number: cc, issuer }))
                } else {
                    None
                }
            })
        })
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
