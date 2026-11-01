//! ES: port of `Duckling.*.ES.*` rules (`Duckling.Rules.ES`).

use crate::locale::Region;
use crate::types::{Dim, Rule};

pub mod amount_of_money;
pub mod distance;
pub mod duration;
pub mod numeral;
pub mod ordinal;
pub mod quantity;
pub mod temperature;
pub mod time_grain;
pub mod volume;

/// Dimensions whose rules for this language (all locales) are fully ported and fixture-verified.
pub const PORTED: &[Dim] = &[
    Dim::Numeral,
    Dim::Ordinal,
    Dim::TimeGrain,
    Dim::Duration,
    Dim::Distance,
    Dim::Temperature,
    Dim::Volume,
    Dim::Quantity,
    Dim::AmountOfMoney,
];

/// `langRules`.
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::AmountOfMoney => amount_of_money::rules().iter().collect(),
        Dim::Distance => distance::rules().iter().collect(),
        Dim::Duration => duration::rules().iter().collect(),
        Dim::Numeral => numeral::rules().iter().collect(),
        Dim::Ordinal => ordinal::rules().iter().collect(),
        Dim::Quantity => quantity::rules().iter().collect(),
        Dim::Temperature => temperature::rules().iter().collect(),
        Dim::TimeGrain => time_grain::rules().iter().collect(),
        Dim::Volume => volume::rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `localeRules`.
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::Numeral => match region {
            Region::MX => numeral::mx_locale_rules().iter().collect(),
            Region::AR | Region::CL | Region::CO | Region::ES | Region::PE | Region::VE => {
                numeral::comma_locale_rules().iter().collect()
            }
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

/// `defaultRules` (used when the locale has no region).
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::Numeral => {
            // `NumeralES.rulesBackwardCompatible ++ langRules dim`
            let mut v: Vec<&'static Rule> = numeral::comma_locale_rules().iter().collect();
            v.extend(lang_rules(dim));
            v
        }
        _ => lang_rules(dim),
    }
}
