//! `Duckling.Time.Computed`: holidays given as precomputed day lists.

use super::computed_data::*;
use super::helpers::time_computed;
use super::types::{timedata, TimeData};
use crate::dims::time_grain::Grain;

fn computed_days(xs: &'static [(i64, u32, u32)]) -> TimeData {
    TimeData { time_pred: time_computed(xs), time_grain: Grain::Day, ..timedata() }
}

pub fn chanukah() -> TimeData {
    computed_days(CHANUKAH)
}

pub fn chinese_new_year() -> TimeData {
    computed_days(CHINESE_NEW_YEAR)
}

pub fn easter_sunday() -> TimeData {
    computed_days(EASTER_SUNDAY)
}

pub fn lag_ba_omer() -> TimeData {
    computed_days(LAG_BA_OMER)
}

pub fn orthodox_easter() -> TimeData {
    computed_days(ORTHODOX_EASTER)
}

pub fn passover() -> TimeData {
    computed_days(PASSOVER)
}

pub fn rosh_hashana() -> TimeData {
    computed_days(ROSH_HASHANA)
}

pub fn tisha_b_av() -> TimeData {
    computed_days(TISHA_B_AV)
}

pub fn yom_haatzmaut() -> TimeData {
    computed_days(YOM_HAATZMAUT)
}

pub fn eidal_adha() -> TimeData {
    computed_days(EIDAL_ADHA)
}

pub fn eidal_fitr() -> TimeData {
    computed_days(EIDAL_FITR)
}

pub fn rajab() -> TimeData {
    computed_days(RAJAB)
}

pub fn muharram() -> TimeData {
    computed_days(MUHARRAM)
}

pub fn ramadan() -> TimeData {
    computed_days(RAMADAN)
}

pub fn mawlid() -> TimeData {
    computed_days(MAWLID)
}

pub fn tu_bishvat() -> TimeData {
    computed_days(TU_BISHVAT)
}

pub fn dhanteras() -> TimeData {
    computed_days(DHANTERAS)
}

pub fn navaratri() -> TimeData {
    computed_days(NAVARATRI)
}

pub fn raksha_bandhan() -> TimeData {
    computed_days(RAKSHA_BANDHAN)
}

pub fn ganesh_chaturthi() -> TimeData {
    computed_days(GANESH_CHATURTHI)
}

pub fn mahavir_jayanti() -> TimeData {
    computed_days(MAHAVIR_JAYANTI)
}

pub fn thai_pongal() -> TimeData {
    computed_days(THAI_PONGAL)
}

pub fn maha_shiva_ratri() -> TimeData {
    computed_days(MAHA_SHIVA_RATRI)
}

pub fn thiru_onam() -> TimeData {
    computed_days(THIRU_ONAM)
}

pub fn ratha_yatra() -> TimeData {
    computed_days(RATHA_YATRA)
}

pub fn vasant_panchami() -> TimeData {
    computed_days(VASANT_PANCHAMI)
}

pub fn global_youth_service_day() -> TimeData {
    computed_days(GLOBAL_YOUTH_SERVICE_DAY)
}

pub fn guru_gobind_singh_jayanti() -> TimeData {
    computed_days(GURU_GOBIND_SINGH_JAYANTI)
}

pub fn karva_chauth() -> TimeData {
    computed_days(KARVA_CHAUTH)
}

pub fn krishna_janmashtami() -> TimeData {
    computed_days(KRISHNA_JANMASHTAMI)
}

pub fn vesak() -> TimeData {
    computed_days(VESAK)
}

pub fn vaisakhi() -> TimeData {
    computed_days(VAISAKHI)
}

pub fn purim() -> TimeData {
    computed_days(PURIM)
}

pub fn saraswati_jayanti() -> TimeData {
    computed_days(SARASWATI_JAYANTI)
}

pub fn rabindra_jayanti() -> TimeData {
    computed_days(RABINDRA_JAYANTI)
}

pub fn ravidass_jayanti() -> TimeData {
    computed_days(RAVIDASS_JAYANTI)
}

pub fn pargat_diwas() -> TimeData {
    computed_days(PARGAT_DIWAS)
}

pub fn ugadi() -> TimeData {
    computed_days(UGADI)
}

pub fn rama_navami() -> TimeData {
    computed_days(RAMA_NAVAMI)
}

pub fn super_tuesday() -> TimeData {
    computed_days(SUPER_TUESDAY)
}
