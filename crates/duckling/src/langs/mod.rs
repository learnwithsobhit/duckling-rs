//! Per-language rule sets. Each language module exposes `PORTED`, `lang_rules`, `locale_rules`, `default_rules`.
//! A language module is compiled only with its `lang-xx` Cargo feature.

#[cfg(feature = "lang-af")]
pub mod af;
#[cfg(feature = "lang-ar")]
pub mod ar;
#[cfg(feature = "lang-bg")]
pub mod bg;
#[cfg(feature = "lang-bn")]
pub mod bn;
#[cfg(feature = "lang-ca")]
pub mod ca;
#[cfg(feature = "lang-cs")]
pub mod cs;
#[cfg(feature = "lang-da")]
pub mod da;
#[cfg(feature = "lang-de")]
pub mod de;
#[cfg(feature = "lang-el")]
pub mod el;
#[cfg(feature = "lang-en")]
pub mod en;
#[cfg(feature = "lang-es")]
pub mod es;
#[cfg(feature = "lang-et")]
pub mod et;
#[cfg(feature = "lang-fa")]
pub mod fa;
#[cfg(feature = "lang-fi")]
pub mod fi;
#[cfg(feature = "lang-fr")]
pub mod fr;
#[cfg(feature = "lang-ga")]
pub mod ga;
#[cfg(feature = "lang-he")]
pub mod he;
#[cfg(feature = "lang-hi")]
pub mod hi;
#[cfg(feature = "lang-hr")]
pub mod hr;
#[cfg(feature = "lang-hu")]
pub mod hu;
#[cfg(feature = "lang-id")]
pub mod id;
#[cfg(feature = "lang-is")]
pub mod is;
#[cfg(feature = "lang-it")]
pub mod it;
#[cfg(feature = "lang-ja")]
pub mod ja;
#[cfg(feature = "lang-ka")]
pub mod ka;
#[cfg(feature = "lang-km")]
pub mod km;
#[cfg(feature = "lang-kn")]
pub mod kn;
#[cfg(feature = "lang-ko")]
pub mod ko;
#[cfg(feature = "lang-lo")]
pub mod lo;
#[cfg(feature = "lang-ml")]
pub mod ml;
#[cfg(feature = "lang-mn")]
pub mod mn;
#[cfg(feature = "lang-my")]
pub mod my;
#[cfg(feature = "lang-nb")]
pub mod nb;
#[cfg(feature = "lang-ne")]
pub mod ne;
#[cfg(feature = "lang-nl")]
pub mod nl;
#[cfg(feature = "lang-pl")]
pub mod pl;
#[cfg(feature = "lang-pt")]
pub mod pt;
#[cfg(feature = "lang-ro")]
pub mod ro;
#[cfg(feature = "lang-ru")]
pub mod ru;
#[cfg(feature = "lang-sk")]
pub mod sk;
#[cfg(feature = "lang-sv")]
pub mod sv;
#[cfg(feature = "lang-sw")]
pub mod sw;
#[cfg(feature = "lang-ta")]
pub mod ta;
#[cfg(feature = "lang-te")]
pub mod te;
#[cfg(feature = "lang-th")]
pub mod th;
#[cfg(feature = "lang-tr")]
pub mod tr;
#[cfg(feature = "lang-uk")]
pub mod uk;
#[cfg(feature = "lang-vi")]
pub mod vi;
#[cfg(feature = "lang-zh")]
pub mod zh;

use crate::locale::{Lang, Region};
use crate::types::{Dim, Rule};

pub fn lang_rules(lang: Lang, dim: Dim) -> Vec<&'static Rule> {
    match lang {
        #[cfg(feature = "lang-af")]
        Lang::AF => af::lang_rules(dim),
        #[cfg(feature = "lang-ar")]
        Lang::AR => ar::lang_rules(dim),
        #[cfg(feature = "lang-bg")]
        Lang::BG => bg::lang_rules(dim),
        #[cfg(feature = "lang-bn")]
        Lang::BN => bn::lang_rules(dim),
        #[cfg(feature = "lang-ca")]
        Lang::CA => ca::lang_rules(dim),
        #[cfg(feature = "lang-cs")]
        Lang::CS => cs::lang_rules(dim),
        #[cfg(feature = "lang-da")]
        Lang::DA => da::lang_rules(dim),
        #[cfg(feature = "lang-de")]
        Lang::DE => de::lang_rules(dim),
        #[cfg(feature = "lang-el")]
        Lang::EL => el::lang_rules(dim),
        #[cfg(feature = "lang-en")]
        Lang::EN => en::lang_rules(dim),
        #[cfg(feature = "lang-es")]
        Lang::ES => es::lang_rules(dim),
        #[cfg(feature = "lang-et")]
        Lang::ET => et::lang_rules(dim),
        #[cfg(feature = "lang-fa")]
        Lang::FA => fa::lang_rules(dim),
        #[cfg(feature = "lang-fi")]
        Lang::FI => fi::lang_rules(dim),
        #[cfg(feature = "lang-fr")]
        Lang::FR => fr::lang_rules(dim),
        #[cfg(feature = "lang-ga")]
        Lang::GA => ga::lang_rules(dim),
        #[cfg(feature = "lang-he")]
        Lang::HE => he::lang_rules(dim),
        #[cfg(feature = "lang-hi")]
        Lang::HI => hi::lang_rules(dim),
        #[cfg(feature = "lang-hr")]
        Lang::HR => hr::lang_rules(dim),
        #[cfg(feature = "lang-hu")]
        Lang::HU => hu::lang_rules(dim),
        #[cfg(feature = "lang-id")]
        Lang::ID => id::lang_rules(dim),
        #[cfg(feature = "lang-is")]
        Lang::IS => is::lang_rules(dim),
        #[cfg(feature = "lang-it")]
        Lang::IT => it::lang_rules(dim),
        #[cfg(feature = "lang-ja")]
        Lang::JA => ja::lang_rules(dim),
        #[cfg(feature = "lang-ka")]
        Lang::KA => ka::lang_rules(dim),
        #[cfg(feature = "lang-km")]
        Lang::KM => km::lang_rules(dim),
        #[cfg(feature = "lang-kn")]
        Lang::KN => kn::lang_rules(dim),
        #[cfg(feature = "lang-ko")]
        Lang::KO => ko::lang_rules(dim),
        #[cfg(feature = "lang-lo")]
        Lang::LO => lo::lang_rules(dim),
        #[cfg(feature = "lang-ml")]
        Lang::ML => ml::lang_rules(dim),
        #[cfg(feature = "lang-mn")]
        Lang::MN => mn::lang_rules(dim),
        #[cfg(feature = "lang-my")]
        Lang::MY => my::lang_rules(dim),
        #[cfg(feature = "lang-nb")]
        Lang::NB => nb::lang_rules(dim),
        #[cfg(feature = "lang-ne")]
        Lang::NE => ne::lang_rules(dim),
        #[cfg(feature = "lang-nl")]
        Lang::NL => nl::lang_rules(dim),
        #[cfg(feature = "lang-pl")]
        Lang::PL => pl::lang_rules(dim),
        #[cfg(feature = "lang-pt")]
        Lang::PT => pt::lang_rules(dim),
        #[cfg(feature = "lang-ro")]
        Lang::RO => ro::lang_rules(dim),
        #[cfg(feature = "lang-ru")]
        Lang::RU => ru::lang_rules(dim),
        #[cfg(feature = "lang-sk")]
        Lang::SK => sk::lang_rules(dim),
        #[cfg(feature = "lang-sv")]
        Lang::SV => sv::lang_rules(dim),
        #[cfg(feature = "lang-sw")]
        Lang::SW => sw::lang_rules(dim),
        #[cfg(feature = "lang-ta")]
        Lang::TA => ta::lang_rules(dim),
        #[cfg(feature = "lang-te")]
        Lang::TE => te::lang_rules(dim),
        #[cfg(feature = "lang-th")]
        Lang::TH => th::lang_rules(dim),
        #[cfg(feature = "lang-tr")]
        Lang::TR => tr::lang_rules(dim),
        #[cfg(feature = "lang-uk")]
        Lang::UK => uk::lang_rules(dim),
        #[cfg(feature = "lang-vi")]
        Lang::VI => vi::lang_rules(dim),
        #[cfg(feature = "lang-zh")]
        Lang::ZH => zh::lang_rules(dim),
        #[allow(unreachable_patterns)]
        _ => Vec::new(),
    }
}

pub fn locale_rules(lang: Lang, region: Region, dim: Dim) -> Vec<&'static Rule> {
    match lang {
        #[cfg(feature = "lang-af")]
        Lang::AF => af::locale_rules(region, dim),
        #[cfg(feature = "lang-ar")]
        Lang::AR => ar::locale_rules(region, dim),
        #[cfg(feature = "lang-bg")]
        Lang::BG => bg::locale_rules(region, dim),
        #[cfg(feature = "lang-bn")]
        Lang::BN => bn::locale_rules(region, dim),
        #[cfg(feature = "lang-ca")]
        Lang::CA => ca::locale_rules(region, dim),
        #[cfg(feature = "lang-cs")]
        Lang::CS => cs::locale_rules(region, dim),
        #[cfg(feature = "lang-da")]
        Lang::DA => da::locale_rules(region, dim),
        #[cfg(feature = "lang-de")]
        Lang::DE => de::locale_rules(region, dim),
        #[cfg(feature = "lang-el")]
        Lang::EL => el::locale_rules(region, dim),
        #[cfg(feature = "lang-en")]
        Lang::EN => en::locale_rules(region, dim),
        #[cfg(feature = "lang-es")]
        Lang::ES => es::locale_rules(region, dim),
        #[cfg(feature = "lang-et")]
        Lang::ET => et::locale_rules(region, dim),
        #[cfg(feature = "lang-fa")]
        Lang::FA => fa::locale_rules(region, dim),
        #[cfg(feature = "lang-fi")]
        Lang::FI => fi::locale_rules(region, dim),
        #[cfg(feature = "lang-fr")]
        Lang::FR => fr::locale_rules(region, dim),
        #[cfg(feature = "lang-ga")]
        Lang::GA => ga::locale_rules(region, dim),
        #[cfg(feature = "lang-he")]
        Lang::HE => he::locale_rules(region, dim),
        #[cfg(feature = "lang-hi")]
        Lang::HI => hi::locale_rules(region, dim),
        #[cfg(feature = "lang-hr")]
        Lang::HR => hr::locale_rules(region, dim),
        #[cfg(feature = "lang-hu")]
        Lang::HU => hu::locale_rules(region, dim),
        #[cfg(feature = "lang-id")]
        Lang::ID => id::locale_rules(region, dim),
        #[cfg(feature = "lang-is")]
        Lang::IS => is::locale_rules(region, dim),
        #[cfg(feature = "lang-it")]
        Lang::IT => it::locale_rules(region, dim),
        #[cfg(feature = "lang-ja")]
        Lang::JA => ja::locale_rules(region, dim),
        #[cfg(feature = "lang-ka")]
        Lang::KA => ka::locale_rules(region, dim),
        #[cfg(feature = "lang-km")]
        Lang::KM => km::locale_rules(region, dim),
        #[cfg(feature = "lang-kn")]
        Lang::KN => kn::locale_rules(region, dim),
        #[cfg(feature = "lang-ko")]
        Lang::KO => ko::locale_rules(region, dim),
        #[cfg(feature = "lang-lo")]
        Lang::LO => lo::locale_rules(region, dim),
        #[cfg(feature = "lang-ml")]
        Lang::ML => ml::locale_rules(region, dim),
        #[cfg(feature = "lang-mn")]
        Lang::MN => mn::locale_rules(region, dim),
        #[cfg(feature = "lang-my")]
        Lang::MY => my::locale_rules(region, dim),
        #[cfg(feature = "lang-nb")]
        Lang::NB => nb::locale_rules(region, dim),
        #[cfg(feature = "lang-ne")]
        Lang::NE => ne::locale_rules(region, dim),
        #[cfg(feature = "lang-nl")]
        Lang::NL => nl::locale_rules(region, dim),
        #[cfg(feature = "lang-pl")]
        Lang::PL => pl::locale_rules(region, dim),
        #[cfg(feature = "lang-pt")]
        Lang::PT => pt::locale_rules(region, dim),
        #[cfg(feature = "lang-ro")]
        Lang::RO => ro::locale_rules(region, dim),
        #[cfg(feature = "lang-ru")]
        Lang::RU => ru::locale_rules(region, dim),
        #[cfg(feature = "lang-sk")]
        Lang::SK => sk::locale_rules(region, dim),
        #[cfg(feature = "lang-sv")]
        Lang::SV => sv::locale_rules(region, dim),
        #[cfg(feature = "lang-sw")]
        Lang::SW => sw::locale_rules(region, dim),
        #[cfg(feature = "lang-ta")]
        Lang::TA => ta::locale_rules(region, dim),
        #[cfg(feature = "lang-te")]
        Lang::TE => te::locale_rules(region, dim),
        #[cfg(feature = "lang-th")]
        Lang::TH => th::locale_rules(region, dim),
        #[cfg(feature = "lang-tr")]
        Lang::TR => tr::locale_rules(region, dim),
        #[cfg(feature = "lang-uk")]
        Lang::UK => uk::locale_rules(region, dim),
        #[cfg(feature = "lang-vi")]
        Lang::VI => vi::locale_rules(region, dim),
        #[cfg(feature = "lang-zh")]
        Lang::ZH => zh::locale_rules(region, dim),
        #[allow(unreachable_patterns)]
        _ => Vec::new(),
    }
}

pub fn default_rules(lang: Lang, dim: Dim) -> Vec<&'static Rule> {
    match lang {
        #[cfg(feature = "lang-af")]
        Lang::AF => af::default_rules(dim),
        #[cfg(feature = "lang-ar")]
        Lang::AR => ar::default_rules(dim),
        #[cfg(feature = "lang-bg")]
        Lang::BG => bg::default_rules(dim),
        #[cfg(feature = "lang-bn")]
        Lang::BN => bn::default_rules(dim),
        #[cfg(feature = "lang-ca")]
        Lang::CA => ca::default_rules(dim),
        #[cfg(feature = "lang-cs")]
        Lang::CS => cs::default_rules(dim),
        #[cfg(feature = "lang-da")]
        Lang::DA => da::default_rules(dim),
        #[cfg(feature = "lang-de")]
        Lang::DE => de::default_rules(dim),
        #[cfg(feature = "lang-el")]
        Lang::EL => el::default_rules(dim),
        #[cfg(feature = "lang-en")]
        Lang::EN => en::default_rules(dim),
        #[cfg(feature = "lang-es")]
        Lang::ES => es::default_rules(dim),
        #[cfg(feature = "lang-et")]
        Lang::ET => et::default_rules(dim),
        #[cfg(feature = "lang-fa")]
        Lang::FA => fa::default_rules(dim),
        #[cfg(feature = "lang-fi")]
        Lang::FI => fi::default_rules(dim),
        #[cfg(feature = "lang-fr")]
        Lang::FR => fr::default_rules(dim),
        #[cfg(feature = "lang-ga")]
        Lang::GA => ga::default_rules(dim),
        #[cfg(feature = "lang-he")]
        Lang::HE => he::default_rules(dim),
        #[cfg(feature = "lang-hi")]
        Lang::HI => hi::default_rules(dim),
        #[cfg(feature = "lang-hr")]
        Lang::HR => hr::default_rules(dim),
        #[cfg(feature = "lang-hu")]
        Lang::HU => hu::default_rules(dim),
        #[cfg(feature = "lang-id")]
        Lang::ID => id::default_rules(dim),
        #[cfg(feature = "lang-is")]
        Lang::IS => is::default_rules(dim),
        #[cfg(feature = "lang-it")]
        Lang::IT => it::default_rules(dim),
        #[cfg(feature = "lang-ja")]
        Lang::JA => ja::default_rules(dim),
        #[cfg(feature = "lang-ka")]
        Lang::KA => ka::default_rules(dim),
        #[cfg(feature = "lang-km")]
        Lang::KM => km::default_rules(dim),
        #[cfg(feature = "lang-kn")]
        Lang::KN => kn::default_rules(dim),
        #[cfg(feature = "lang-ko")]
        Lang::KO => ko::default_rules(dim),
        #[cfg(feature = "lang-lo")]
        Lang::LO => lo::default_rules(dim),
        #[cfg(feature = "lang-ml")]
        Lang::ML => ml::default_rules(dim),
        #[cfg(feature = "lang-mn")]
        Lang::MN => mn::default_rules(dim),
        #[cfg(feature = "lang-my")]
        Lang::MY => my::default_rules(dim),
        #[cfg(feature = "lang-nb")]
        Lang::NB => nb::default_rules(dim),
        #[cfg(feature = "lang-ne")]
        Lang::NE => ne::default_rules(dim),
        #[cfg(feature = "lang-nl")]
        Lang::NL => nl::default_rules(dim),
        #[cfg(feature = "lang-pl")]
        Lang::PL => pl::default_rules(dim),
        #[cfg(feature = "lang-pt")]
        Lang::PT => pt::default_rules(dim),
        #[cfg(feature = "lang-ro")]
        Lang::RO => ro::default_rules(dim),
        #[cfg(feature = "lang-ru")]
        Lang::RU => ru::default_rules(dim),
        #[cfg(feature = "lang-sk")]
        Lang::SK => sk::default_rules(dim),
        #[cfg(feature = "lang-sv")]
        Lang::SV => sv::default_rules(dim),
        #[cfg(feature = "lang-sw")]
        Lang::SW => sw::default_rules(dim),
        #[cfg(feature = "lang-ta")]
        Lang::TA => ta::default_rules(dim),
        #[cfg(feature = "lang-te")]
        Lang::TE => te::default_rules(dim),
        #[cfg(feature = "lang-th")]
        Lang::TH => th::default_rules(dim),
        #[cfg(feature = "lang-tr")]
        Lang::TR => tr::default_rules(dim),
        #[cfg(feature = "lang-uk")]
        Lang::UK => uk::default_rules(dim),
        #[cfg(feature = "lang-vi")]
        Lang::VI => vi::default_rules(dim),
        #[cfg(feature = "lang-zh")]
        Lang::ZH => zh::default_rules(dim),
        #[allow(unreachable_patterns)]
        _ => Vec::new(),
    }
}

/// True when `dim` is fully ported (all locales) for `lang`.
pub fn is_ported(lang: Lang, dim: Dim) -> bool {
    match lang {
        #[cfg(feature = "lang-af")]
        Lang::AF => af::PORTED.contains(&dim),
        #[cfg(feature = "lang-ar")]
        Lang::AR => ar::PORTED.contains(&dim),
        #[cfg(feature = "lang-bg")]
        Lang::BG => bg::PORTED.contains(&dim),
        #[cfg(feature = "lang-bn")]
        Lang::BN => bn::PORTED.contains(&dim),
        #[cfg(feature = "lang-ca")]
        Lang::CA => ca::PORTED.contains(&dim),
        #[cfg(feature = "lang-cs")]
        Lang::CS => cs::PORTED.contains(&dim),
        #[cfg(feature = "lang-da")]
        Lang::DA => da::PORTED.contains(&dim),
        #[cfg(feature = "lang-de")]
        Lang::DE => de::PORTED.contains(&dim),
        #[cfg(feature = "lang-el")]
        Lang::EL => el::PORTED.contains(&dim),
        #[cfg(feature = "lang-en")]
        Lang::EN => en::PORTED.contains(&dim),
        #[cfg(feature = "lang-es")]
        Lang::ES => es::PORTED.contains(&dim),
        #[cfg(feature = "lang-et")]
        Lang::ET => et::PORTED.contains(&dim),
        #[cfg(feature = "lang-fa")]
        Lang::FA => fa::PORTED.contains(&dim),
        #[cfg(feature = "lang-fi")]
        Lang::FI => fi::PORTED.contains(&dim),
        #[cfg(feature = "lang-fr")]
        Lang::FR => fr::PORTED.contains(&dim),
        #[cfg(feature = "lang-ga")]
        Lang::GA => ga::PORTED.contains(&dim),
        #[cfg(feature = "lang-he")]
        Lang::HE => he::PORTED.contains(&dim),
        #[cfg(feature = "lang-hi")]
        Lang::HI => hi::PORTED.contains(&dim),
        #[cfg(feature = "lang-hr")]
        Lang::HR => hr::PORTED.contains(&dim),
        #[cfg(feature = "lang-hu")]
        Lang::HU => hu::PORTED.contains(&dim),
        #[cfg(feature = "lang-id")]
        Lang::ID => id::PORTED.contains(&dim),
        #[cfg(feature = "lang-is")]
        Lang::IS => is::PORTED.contains(&dim),
        #[cfg(feature = "lang-it")]
        Lang::IT => it::PORTED.contains(&dim),
        #[cfg(feature = "lang-ja")]
        Lang::JA => ja::PORTED.contains(&dim),
        #[cfg(feature = "lang-ka")]
        Lang::KA => ka::PORTED.contains(&dim),
        #[cfg(feature = "lang-km")]
        Lang::KM => km::PORTED.contains(&dim),
        #[cfg(feature = "lang-kn")]
        Lang::KN => kn::PORTED.contains(&dim),
        #[cfg(feature = "lang-ko")]
        Lang::KO => ko::PORTED.contains(&dim),
        #[cfg(feature = "lang-lo")]
        Lang::LO => lo::PORTED.contains(&dim),
        #[cfg(feature = "lang-ml")]
        Lang::ML => ml::PORTED.contains(&dim),
        #[cfg(feature = "lang-mn")]
        Lang::MN => mn::PORTED.contains(&dim),
        #[cfg(feature = "lang-my")]
        Lang::MY => my::PORTED.contains(&dim),
        #[cfg(feature = "lang-nb")]
        Lang::NB => nb::PORTED.contains(&dim),
        #[cfg(feature = "lang-ne")]
        Lang::NE => ne::PORTED.contains(&dim),
        #[cfg(feature = "lang-nl")]
        Lang::NL => nl::PORTED.contains(&dim),
        #[cfg(feature = "lang-pl")]
        Lang::PL => pl::PORTED.contains(&dim),
        #[cfg(feature = "lang-pt")]
        Lang::PT => pt::PORTED.contains(&dim),
        #[cfg(feature = "lang-ro")]
        Lang::RO => ro::PORTED.contains(&dim),
        #[cfg(feature = "lang-ru")]
        Lang::RU => ru::PORTED.contains(&dim),
        #[cfg(feature = "lang-sk")]
        Lang::SK => sk::PORTED.contains(&dim),
        #[cfg(feature = "lang-sv")]
        Lang::SV => sv::PORTED.contains(&dim),
        #[cfg(feature = "lang-sw")]
        Lang::SW => sw::PORTED.contains(&dim),
        #[cfg(feature = "lang-ta")]
        Lang::TA => ta::PORTED.contains(&dim),
        #[cfg(feature = "lang-te")]
        Lang::TE => te::PORTED.contains(&dim),
        #[cfg(feature = "lang-th")]
        Lang::TH => th::PORTED.contains(&dim),
        #[cfg(feature = "lang-tr")]
        Lang::TR => tr::PORTED.contains(&dim),
        #[cfg(feature = "lang-uk")]
        Lang::UK => uk::PORTED.contains(&dim),
        #[cfg(feature = "lang-vi")]
        Lang::VI => vi::PORTED.contains(&dim),
        #[cfg(feature = "lang-zh")]
        Lang::ZH => zh::PORTED.contains(&dim),
        #[allow(unreachable_patterns)]
        _ => false,
    }
}
