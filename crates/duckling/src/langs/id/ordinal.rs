//! `Duckling.Ordinal.ID.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

fn rule_ordinals() -> Rule {
    rule(
        "ordinals",
        vec![regex("(pertama|kedua|ketiga|keempat|kelima|keenam|ketujuh|kedelapan|kesembilan|kesepuluh)")],
        |ts| match to_lower(groups(ts)?.first()?).as_str() {
            "pertama" => ordinal(1),
            "kedua" => ordinal(2),
            "ketiga" => ordinal(3),
            "keempat" => ordinal(4),
            "kelima" => ordinal(5),
            "keenam" => ordinal(6),
            "ketujuh" => ordinal(7),
            "kedelapan" => ordinal(8),
            "kesembilan" => ordinal(9),
            "kesepuluh" => ordinal(10),
            _ => None,
        },
    )
}

fn rule_ordinals_digits() -> Rule {
    rule("ordinals (digits)", vec![regex(r"ke-0*(\d+)")], |ts| {
        parse_int(groups(ts)?.first()?).and_then(ordinal)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![rule_ordinals(), rule_ordinals_digits()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
