//! KM: port of `Duckling.*.KM.*` rules (`Duckling.Rules.KM`).

use crate::locale::Region;
use crate::types::{Dim, Rule};

mod distance;
mod numeral;
mod ordinal;
mod quantity;
mod temperature;
mod volume;

/// Dimensions whose rules for this language (all locales) are fully ported and fixture-verified.
pub const PORTED: &[Dim] = &[
    Dim::Numeral,
    Dim::Ordinal,
    Dim::Distance,
    Dim::Quantity,
    Dim::Temperature,
    Dim::Volume,
];

/// `langRules`.
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::Distance => distance::rules().iter().collect(),
        Dim::Numeral => numeral::rules().iter().collect(),
        Dim::Ordinal => ordinal::rules().iter().collect(),
        Dim::Quantity => quantity::rules().iter().collect(),
        Dim::Temperature => temperature::rules().iter().collect(),
        Dim::Volume => volume::rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `localeRules` (KM has none).
#[allow(unused_variables)]
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    Vec::new()
}

/// `defaultRules` (used when the locale has no region).
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    lang_rules(dim)
}
