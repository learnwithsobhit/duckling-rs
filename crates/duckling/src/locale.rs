//! Languages, regions and locales (port of `Duckling.Locale` / `Duckling.Region`).

use std::fmt;

macro_rules! string_enum {
    ($(#[$m:meta])* $name:ident { $($var:ident),* $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum $name { $($var),* }
        impl $name {
            /// All values, in Haskell `[minBound..maxBound]` order.
            pub const ALL: &'static [$name] = &[$($name::$var),*];
            pub fn as_str(self) -> &'static str {
                match self { $($name::$var => stringify!($var)),* }
            }
            /// Parses the upper-case Haskell `Show`/`Read` name.
            pub fn parse(s: &str) -> Option<$name> {
                match s { $(stringify!($var) => Some($name::$var),)* _ => None }
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
        }
    };
}

string_enum! {
    /// ISO 639-1 language. Declaration order matches Haskell's derived `Enum`.
    Lang {
        AF, AR, BG, BN, CA, CS, DA, DE, EL, EN, ES, ET, FA, FI, FR, GA, HE, HI, HR, HU,
        ID, IS, IT, JA, KA, KN, KM, KO, LO, ML, MN, MY, NB, NE, NL, PL, PT, RO, RU, SK,
        SV, SW, TA, TE, TH, TR, UK, VI, ZH
    }
}

string_enum! {
    /// ISO 3166-1 alpha-2 country code.
    Region {
        AR, AU, BE, BZ, CA, CL, CN, CO, EG, ES, GB, HK, IE, IN, JM, MN, MX, MO, NL, NZ,
        PE, PH, TT, TW, US, VE, ZA
    }
}

/// A language plus an optional region (e.g. `EN_US`; `EN_XX` when no region).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Locale {
    pub lang: Lang,
    pub region: Option<Region>,
}

/// Regions for which a language has dedicated locale rules (`allLocales`).
pub fn locales_of(lang: Lang) -> &'static [Region] {
    use Region::*;
    match lang {
        Lang::AR => &[EG],
        Lang::EN => &[AU, BZ, CA, GB, IN, IE, JM, NZ, PH, ZA, TT, US],
        Lang::ES => &[AR, CL, CO, ES, MX, PE, VE],
        Lang::NL => &[BE, NL],
        Lang::ZH => &[CN, HK, MO, TW],
        _ => &[],
    }
}

impl Locale {
    /// `makeLocale`: unknown regions for the language fall back to no region.
    pub fn new(lang: Lang, region: Option<Region>) -> Locale {
        match region {
            Some(r) if locales_of(lang).contains(&r) => Locale { lang, region: Some(r) },
            _ => Locale { lang, region: None },
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.region {
            None => write!(f, "{}_XX", self.lang),
            Some(r) => write!(f, "{}_{}", self.lang, r),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_locale_falls_back() {
        assert_eq!(Locale::new(Lang::EN, Some(Region::US)).to_string(), "EN_US");
        assert_eq!(Locale::new(Lang::EN, Some(Region::MX)).to_string(), "EN_XX");
        assert_eq!(Locale::new(Lang::FR, Some(Region::US)).to_string(), "FR_XX");
    }

    #[test]
    fn lang_order() {
        assert_eq!(Lang::ALL.len(), 49);
        assert_eq!(Lang::ALL[0], Lang::AF);
        assert_eq!(Lang::parse("ZH"), Some(Lang::ZH));
    }
}
