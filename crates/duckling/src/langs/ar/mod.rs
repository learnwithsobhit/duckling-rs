//! AR: port of `Duckling.*.AR.*` rules (`Duckling.Rules.AR`).

use crate::locale::Region;
use crate::types::{Dim, Rule};

mod amount_of_money;
mod duration;
mod numeral;
mod ordinal;
mod phone_number;
mod quantity;
mod temperature;
mod time;
mod time_grain;
mod volume;

/// Dimensions whose rules for this language (all locales) are fully ported and fixture-verified.
pub const PORTED: &[Dim] = &[
    Dim::Numeral,
    Dim::Ordinal,
    Dim::TimeGrain,
    Dim::Duration,
    Dim::Quantity,
    Dim::Temperature,
    Dim::Volume,
    Dim::AmountOfMoney,
    Dim::Time,
    Dim::Email,
    Dim::Url,
    Dim::PhoneNumber,
    Dim::CreditCardNumber,
];

/// `langRules`.
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::AmountOfMoney => amount_of_money::rules().iter().collect(),
        Dim::Duration => duration::rules().iter().collect(),
        Dim::Numeral => numeral::rules().iter().collect(),
        Dim::Ordinal => ordinal::rules().iter().collect(),
        Dim::PhoneNumber => phone_number::rules().iter().collect(),
        Dim::Quantity => quantity::rules().iter().collect(),
        Dim::Temperature => temperature::rules().iter().collect(),
        Dim::Time => time::rules().iter().collect(),
        Dim::TimeGrain => time_grain::rules().iter().collect(),
        Dim::Volume => volume::rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `localeRules`.
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    match (region, dim) {
        (Region::EG, Dim::Numeral) => numeral::eg_rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `defaultRules` (used when the locale has no region).
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    lang_rules(dim)
}
