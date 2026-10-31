//! Registry of what has been ported so far. The parity test only checks fixtures whose
//! language/dimension combination is listed here, and reports how much is covered.

use crate::locale::{Lang, Locale};
use crate::types::Dim;

/// True when `dim` is fully ported for `locale` (all rules, including regional variants).
pub fn is_ported(locale: Locale, dim: Dim) -> bool {
    match (locale.lang, dim) {
        // Common rules plus EN's "email spelled out" rule. Other languages add their own rules
        // (e.g. AR digits, DE/FR "at"), so they are enabled per language as those are ported.
        (Lang::EN, Dim::Email | Dim::Url | Dim::PhoneNumber | Dim::CreditCardNumber) => true,
        (
            Lang::EN,
            Dim::Numeral
            | Dim::Ordinal
            | Dim::TimeGrain
            | Dim::Duration
            | Dim::Distance
            | Dim::Temperature
            | Dim::Volume
            | Dim::Quantity
            | Dim::AmountOfMoney,
        ) => true,
        _ => false,
    }
}
