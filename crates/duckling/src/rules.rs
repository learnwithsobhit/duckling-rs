//! Rule selection (`Duckling.Rules`, `Duckling.Rules.Common`, `Duckling.Rules.<LANG>`).

use std::collections::{BTreeSet, HashSet};

use crate::dims;
use crate::locale::{Lang, Locale, Region};
use crate::types::{Dim, Rule};

/// Dimensions specific to a language (`Duckling.Dimensions.<LANG>.allDimensions`).
pub fn lang_dimensions(lang: Lang) -> Vec<Dim> {
    match lang {
        Lang::EN => vec![
            Dim::Distance,
            Dim::Duration,
            Dim::Numeral,
            Dim::Ordinal,
            Dim::Quantity,
            Dim::Temperature,
            Dim::Time,
            Dim::Volume,
        ],
        _ => Vec::new(),
    }
}

/// `explicitDimensions`: targets augmented with all dependent dimensions.
pub fn explicit_dimensions(targets: &HashSet<Dim>) -> BTreeSet<Dim> {
    let mut out: BTreeSet<Dim> = targets.iter().copied().collect();
    for d in targets {
        out.extend(d.dependents().iter().copied());
    }
    out
}

/// `Duckling.Rules.Common.rules`.
fn common_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::CreditCardNumber => dims::credit_card::rules().iter().collect(),
        Dim::Email => dims::email::rules().iter().collect(),
        Dim::Numeral => dims::numeral::rules::rules().iter().collect(),
        Dim::PhoneNumber => dims::phone_number::rules().iter().collect(),
        Dim::Url => dims::url::rules().iter().collect(),
        _ => Vec::new(),
    }
}

fn en_lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::Email => dims::email::en_rules().iter().collect(),
        Dim::Numeral => dims::numeral::en::rules().iter().collect(),
        _ => Vec::new(),
    }
}

fn lang_rules(lang: Lang, dim: Dim) -> Vec<&'static Rule> {
    match lang {
        Lang::EN => en_lang_rules(dim),
        _ => Vec::new(),
    }
}

fn locale_rules(_lang: Lang, _region: Region, _dim: Dim) -> Vec<&'static Rule> {
    Vec::new()
}

fn default_rules(lang: Lang, dim: Dim) -> Vec<&'static Rule> {
    lang_rules(lang, dim)
}

fn rules_for_dim(locale: Locale, dim: Dim) -> Vec<&'static Rule> {
    let mut v = common_rules(dim);
    match locale.region {
        Some(region) => {
            v.extend(lang_rules(locale.lang, dim));
            v.extend(locale_rules(locale.lang, region, dim));
        }
        None => v.extend(default_rules(locale.lang, dim)),
    }
    v
}

/// `allRules`.
pub fn all_rules(locale: Locale) -> Vec<&'static Rule> {
    let dims: HashSet<Dim> = crate::api::all_dimensions(locale.lang).into_iter().collect();
    explicit_dimensions(&dims).into_iter().flat_map(|d| rules_for_dim(locale, d)).collect()
}

/// `rulesFor`.
pub fn rules_for(locale: Locale, targets: &HashSet<Dim>) -> Vec<&'static Rule> {
    if targets.is_empty() {
        return all_rules(locale);
    }
    explicit_dimensions(targets).into_iter().flat_map(|d| rules_for_dim(locale, d)).collect()
}
