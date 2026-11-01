//! IT: port of `Duckling.*.IT.*` rules (`Duckling.Rules.IT`).

mod holidays;
mod measures;
mod numeral;
mod simple;
mod time;

use crate::locale::Region;
use crate::types::{Dim, Rule};

/// Dimensions whose rules for this language (all locales) are fully ported and fixture-verified.
pub const PORTED: &[Dim] = &[
    Dim::Email,
    Dim::Numeral,
    Dim::Ordinal,
    Dim::TimeGrain,
    Dim::Duration,
    Dim::Distance,
    Dim::Temperature,
    Dim::Volume,
    Dim::AmountOfMoney,
    Dim::Time,
    Dim::Url,
    Dim::PhoneNumber,
    Dim::CreditCardNumber,
];

/// `langRules`.
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::AmountOfMoney => measures::amount_of_money_rules().iter().collect(),
        Dim::Distance => measures::distance_rules().iter().collect(),
        Dim::Duration => simple::duration_rules().iter().collect(),
        Dim::Email => measures::email_rules().iter().collect(),
        Dim::Numeral => numeral::rules().iter().collect(),
        Dim::Ordinal => simple::ordinal_rules().iter().collect(),
        Dim::Temperature => measures::temperature_rules().iter().collect(),
        Dim::Time => time::rules().iter().collect(),
        Dim::TimeGrain => simple::time_grain_rules().iter().collect(),
        Dim::Volume => measures::volume_rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `localeRules`.
#[allow(unused_variables)]
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    Vec::new()
}

/// `defaultRules` (used when the locale has no region).
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    lang_rules(dim)
}
