//! IS: port of `Duckling.*.IS.*` rules (`Duckling.Rules.IS`).
//!
//! `defaultRules = langRules`. `localeRules` is empty.

use crate::locale::Region;
use crate::types::{Dim, Rule};

mod email;
mod numeral;

/// Dimensions whose rules for this language (all locales) are fully ported and fixture-verified.
pub const PORTED: &[Dim] = &[Dim::Numeral, Dim::Email];

/// `langRules`.
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::Email => email::rules().iter().collect(),
        Dim::Numeral => numeral::rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `localeRules` (IS has none).
#[allow(unused_variables)]
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    Vec::new()
}

/// `defaultRules` (`defaultRules = langRules`).
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    lang_rules(dim)
}
