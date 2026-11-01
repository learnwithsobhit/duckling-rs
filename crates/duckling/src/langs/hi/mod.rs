//! HI: port of `Duckling.*.HI.*` rules (`Duckling.Rules.HI`).

use crate::locale::Region;
use crate::types::{Dim, Rule};

mod duration;
mod numeral;
mod ordinal;
mod temperature;
mod time_grain;

/// Dimensions whose rules for this language (all locales) are fully ported and fixture-verified.
pub const PORTED: &[Dim] = &[Dim::Duration, Dim::Numeral, Dim::Ordinal, Dim::Temperature];

/// `langRules`.
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::Duration => duration::rules().iter().collect(),
        Dim::Numeral => numeral::rules().iter().collect(),
        Dim::Ordinal => ordinal::rules().iter().collect(),
        Dim::Temperature => temperature::rules().iter().collect(),
        Dim::TimeGrain => time_grain::rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `localeRules` (HI has none).
#[allow(unused_variables)]
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    Vec::new()
}

/// `defaultRules` (used when the locale has no region).
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    lang_rules(dim)
}
