//! Registry of what has been ported so far. The parity test only checks fixtures whose
//! language/dimension combination is listed here, and reports how much is covered.
//! Each language lists its fully-ported dimensions in `langs::<xx>::PORTED`.

use crate::locale::Locale;
use crate::types::Dim;

/// True when `dim` is fully ported for `locale` (all rules, including regional variants).
pub fn is_ported(locale: Locale, dim: Dim) -> bool {
    crate::langs::is_ported(locale.lang, dim)
}
