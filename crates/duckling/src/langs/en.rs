//! EN: `Duckling.Rules.EN`.

use crate::dims;
use crate::locale::Region;
use crate::types::{Dim, Rule};

/// Everything in English is ported (all dimensions, all locales).
pub const PORTED: &[Dim] = &[
    Dim::Email,
    Dim::Url,
    Dim::PhoneNumber,
    Dim::CreditCardNumber,
    Dim::Numeral,
    Dim::Ordinal,
    Dim::TimeGrain,
    Dim::Duration,
    Dim::Distance,
    Dim::Temperature,
    Dim::Volume,
    Dim::Quantity,
    Dim::AmountOfMoney,
    Dim::Time,
];

/// `langRules`.
pub fn lang_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::AmountOfMoney => dims::amount_of_money::en_rules().iter().collect(),
        Dim::Distance => dims::distance::en_rules().iter().collect(),
        Dim::Duration => dims::duration::en_rules().iter().collect(),
        Dim::Email => dims::email::en_rules().iter().collect(),
        Dim::Numeral => dims::numeral::en::rules().iter().collect(),
        Dim::Ordinal => dims::ordinal::en_rules().iter().collect(),
        Dim::Quantity => dims::quantity::en_rules().iter().collect(),
        Dim::Temperature => dims::temperature::en_rules().iter().collect(),
        Dim::Time => dims::time::en::rules().iter().collect(),
        Dim::TimeGrain => dims::time_grain::en_rules().iter().collect(),
        Dim::Timezone => dims::timezone::en::rules().iter().collect(),
        Dim::Volume => dims::volume::en_rules().iter().collect(),
        _ => Vec::new(),
    }
}

/// `localeRules`.
pub fn locale_rules(region: Region, dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::AmountOfMoney => dims::amount_of_money::en_region_rules(region).iter().collect(),
        Dim::Time => dims::time::locales::en_rules(region).iter().collect(),
        _ => Vec::new(),
    }
}

/// `defaultRules`.
pub fn default_rules(dim: Dim) -> Vec<&'static Rule> {
    match dim {
        Dim::Time => {
            let mut v: Vec<&'static Rule> = dims::time::locales::backward_compatible().iter().collect();
            v.extend(lang_rules(dim));
            v
        }
        _ => lang_rules(dim),
    }
}
