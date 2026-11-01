//! SW: port of `Duckling.*.SW.*` rules (`Duckling.Rules.SW`).
//!
//! `defaultRules = langRules`; no locale-specific rules.
//! Dimensions/SW lists only Numeral.

use crate::locale::Region;
use crate::types::{Dim, Rule};

mod numeral;

/// Dimensions whose rules for this language (all locales) are fully ported and fixture-verified.
pub const PORTED: &[Dim] = &[Dim::Numeral];

/// `langRules`.
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::Numeral => numeral::rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `localeRules` (SW has none).
#[allow(unused_variables)]
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    Vec::new()
}

/// `defaultRules` (`defaultRules = langRules`).
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    lang_rules(dim)
}
