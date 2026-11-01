//! AF: port of `Duckling.*.AF.*` rules (`Duckling.Rules.AF`).
//!
//! `defaultRules = langRules`; no locale-specific rules.
//! Dimensions: Numeral only (`Duckling.Dimensions.AF.allDimensions`).

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

/// `localeRules` (AF has none).
#[allow(unused_variables)]
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    Vec::new()
}

/// `defaultRules` (used when the locale has no region).
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    lang_rules(dim)
}
