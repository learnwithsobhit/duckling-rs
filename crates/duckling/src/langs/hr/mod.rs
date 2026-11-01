//! HR: port of `Duckling.*.HR.*` rules.
//!
//! Status: not ported yet. Fill in `lang_rules` (Duckling.Rules.HR.langRules), `locale_rules`
//! (localeRules), `default_rules` (defaultRules) and list fully-ported dimensions in `PORTED`.

use crate::locale::Region;
use crate::types::{Dim, Rule};

/// Dimensions whose rules for this language (all locales) are fully ported and fixture-verified.
pub const PORTED: &[Dim] = &[];

/// `langRules`.
#[allow(unused_variables)]
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    Vec::new()
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
