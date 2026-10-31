//! `Duckling.AmountOfMoney.Types` (rules are per language).

use std::hash::{Hash, Hasher};

use crate::dims::interval::{self, Direction};
use crate::dims::util::hash_opt_f64;
use crate::json::Json;
use crate::types::{Context, Options};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Currency {
    Cent, Dinar, Dirham, Dollar, Pound, Rial, Riyal, Unnamed, AED, AUD, BGN, BRL, BYN, CAD, CHF, CNY, CZK, DKK, EGP, EUR, GBP, HKD, HRK, IDR, ILS, INR, IQD, JMD, JOD, JPY, GEL, KRW, KWD, LBP, MAD, MNT, MYR, NOK, NZD, PKR, PLN, PTS, QAR, RON, RUB, SAR, SEK, SGD, THB, TTD, USD, VND, ZAR, UAH, TRY,
}

impl Currency {
    pub fn json(self) -> Json {
        Json::str(match self {
            Currency::Cent => "cent",
            Currency::Dinar => "dinar",
            Currency::Dirham => "dirham",
            Currency::Dollar => "$",
            Currency::Pound => "\u{a3}",
            Currency::Rial => "rial",
            Currency::Riyal => "riyal",
            Currency::Unnamed => "unknown",
            Currency::AED => "AED",
            Currency::AUD => "AUD",
            Currency::BGN => "BGN",
            Currency::BRL => "BRL",
            Currency::BYN => "BYN",
            Currency::CAD => "CAD",
            Currency::CHF => "CHF",
            Currency::CNY => "CNY",
            Currency::CZK => "CZK",
            Currency::DKK => "DKK",
            Currency::EGP => "EGP",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::HKD => "HKD",
            Currency::HRK => "HRK",
            Currency::IDR => "IDR",
            Currency::ILS => "ILS",
            Currency::INR => "INR",
            Currency::IQD => "IQD",
            Currency::JMD => "JMD",
            Currency::JOD => "JOD",
            Currency::JPY => "JPY",
            Currency::GEL => "GEL",
            Currency::KRW => "KRW",
            Currency::KWD => "KWD",
            Currency::LBP => "LBP",
            Currency::MAD => "MAD",
            Currency::MNT => "MNT",
            Currency::MYR => "MYR",
            Currency::NOK => "NOK",
            Currency::NZD => "NZD",
            Currency::PKR => "PKR",
            Currency::PLN => "PLN",
            Currency::PTS => "PTS",
            Currency::QAR => "QAR",
            Currency::RON => "RON",
            Currency::RUB => "RUB",
            Currency::SAR => "SAR",
            Currency::SEK => "SEK",
            Currency::SGD => "SGD",
            Currency::THB => "THB",
            Currency::TTD => "TTD",
            Currency::USD => "USD",
            Currency::VND => "VND",
            Currency::ZAR => "ZAR",
            Currency::UAH => "UAH",
            Currency::TRY => "TRY",
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AmountOfMoneyData {
    pub value: Option<f64>,
    pub currency: Currency,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub latent: bool,
}

impl Eq for AmountOfMoneyData {}

impl Hash for AmountOfMoneyData {
    fn hash<H: Hasher>(&self, h: &mut H) {
        hash_opt_f64(self.value, h);
        self.currency.hash(h);
        hash_opt_f64(self.min_value, h);
        hash_opt_f64(self.max_value, h);
        self.latent.hash(h);
    }
}

/// `amountOfMoneyData'`: an empty value with an unnamed currency.
pub fn amount_of_money_data() -> AmountOfMoneyData {
    AmountOfMoneyData { value: None, currency: Currency::Unnamed, min_value: None, max_value: None, latent: false }
}

fn single(c: Currency, v: f64) -> Json {
    Json::obj([("value", Json::Num(v)), ("unit", c.json())])
}

impl AmountOfMoneyData {
    pub fn resolve(&self, _ctx: &Context, opts: &Options) -> Option<(Json, bool)> {
        if !opts.with_latent && self.latent {
            return None;
        }
        let c = self.currency;
        match (self.value, self.min_value, self.max_value) {
            (None, None, None) => None,
            (Some(v), _, _) => Some((interval::simple(single(c, v)), self.latent)),
            (None, Some(from), Some(to)) => Some((interval::between(single(c, from), single(c, to)), self.latent)),
            (None, Some(v), None) => Some((interval::open(Direction::Above, single(c, v)), self.latent)),
            (None, None, Some(v)) => Some((interval::open(Direction::Under, single(c, v)), self.latent)),
        }
    }
}
