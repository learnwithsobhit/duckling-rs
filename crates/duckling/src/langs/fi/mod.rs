//! FI: port of `Duckling.*.FI.*` rules (`Duckling.Rules.FI`).
//!
//! `defaultRules = langRules`; no locale-specific rules.
//! Dimensions/FI lists only Numeral; TimeGrain still has lang rules.

use crate::locale::Region;
use crate::types::{Dim, Rule};

mod numeral;
mod time_grain;

/// Dimensions whose rules for this language (all locales) are fully ported and fixture-verified.
pub const PORTED: &[Dim] = &[Dim::Numeral];

/// `langRules`.
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::Numeral => numeral::rules().iter().collect(),
        Dim::TimeGrain => time_grain::rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `localeRules` (FI has none).
#[allow(unused_variables)]
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    Vec::new()
}

/// `defaultRules` (`defaultRules = langRules`).
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    lang_rules(dim)
}
