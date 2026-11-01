//! ID: port of `Duckling.*.ID.*` rules (`Duckling.Rules.ID`).

use crate::locale::Region;
use crate::types::{Dim, Rule};

mod amount_of_money;
mod numeral;
mod ordinal;

/// Dimensions whose rules for this language (all locales) are fully ported and fixture-verified.
pub const PORTED: &[Dim] = &[Dim::Numeral, Dim::Ordinal, Dim::AmountOfMoney];

/// `langRules`.
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::AmountOfMoney => amount_of_money::rules().iter().collect(),
        Dim::Numeral => numeral::rules().iter().collect(),
        Dim::Ordinal => ordinal::rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `localeRules` (ID has none).
#[allow(unused_variables)]
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    Vec::new()
}

/// `defaultRules` (used when the locale has no region).
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    lang_rules(dim)
}
