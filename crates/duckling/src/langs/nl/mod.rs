//! NL: port of `Duckling.*.NL.*` rules (`Duckling.Rules.NL`).

use crate::locale::Region;
use crate::types::{Dim, Rule};

mod amount_of_money;
mod duration;
mod numeral;
mod ordinal;
mod time;
mod units;

/// Dimensions whose rules for this language (all locales) are fully ported and fixture-verified.
pub const PORTED: &[Dim] = &[
    Dim::Numeral,
    Dim::Ordinal,
    Dim::TimeGrain,
    Dim::Duration,
    Dim::Distance,
    Dim::Volume,
    Dim::Quantity,
    Dim::Time,
    Dim::AmountOfMoney,
    Dim::Email,
    Dim::Url,
    Dim::PhoneNumber,
    Dim::CreditCardNumber,
];

/// `langRules`.
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::AmountOfMoney => amount_of_money::rules().iter().collect(),
        Dim::Distance => units::distance_rules().iter().collect(),
        Dim::Duration => duration::rules().iter().collect(),
        Dim::Numeral => numeral::rules().iter().collect(),
        Dim::Ordinal => ordinal::ordinal_rules().iter().collect(),
        Dim::Quantity => units::quantity_rules().iter().collect(),
        Dim::Time => time::rules().iter().collect(),
        Dim::TimeGrain => ordinal::time_grain_rules().iter().collect(),
        Dim::Volume => units::volume_rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `localeRules`.
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    match (region, dim) {
        (Region::BE, Dim::Time) => time::be_rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `defaultRules` (used when the locale has no region).
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::Time => {
            let mut v: Vec<&'static Rule> = time::rules_backward_compatible().iter().collect();
            v.extend(lang_rules(dim));
            v
        }
        _ => lang_rules(dim),
    }
}
