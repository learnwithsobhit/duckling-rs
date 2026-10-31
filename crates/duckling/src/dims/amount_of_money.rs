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

// ----- helpers (`Duckling.AmountOfMoney.Helpers`) -----------------------------------------

use crate::dims::numeral::{get_int_value, is_integer, is_natural, is_positive};
use crate::dims::util::to_lower;
use crate::types::{groups, pred, regex, rule, Rule, Token};
use once_cell::sync::Lazy;

pub fn currency_only(c: Currency) -> AmountOfMoneyData {
    AmountOfMoneyData { currency: c, ..amount_of_money_data() }
}

pub fn value_only(x: f64) -> AmountOfMoneyData {
    AmountOfMoneyData { value: Some(x), ..amount_of_money_data() }
}

pub fn with_value(x: f64, mut fd: AmountOfMoneyData) -> AmountOfMoneyData {
    fd.value = Some(x);
    fd
}

pub fn with_cents(x: f64, mut fd: AmountOfMoneyData) -> AmountOfMoneyData {
    match fd.value {
        Some(v) => {
            fd.value = Some(v + x / 100.0);
            fd
        }
        None => AmountOfMoneyData { value: Some(x), currency: Currency::Cent, ..amount_of_money_data() },
    }
}

pub fn with_interval(from: f64, to: f64, mut fd: AmountOfMoneyData) -> AmountOfMoneyData {
    fd.min_value = Some(from);
    fd.max_value = Some(to);
    fd.value = None;
    fd
}

pub fn with_min(x: f64, mut fd: AmountOfMoneyData) -> AmountOfMoneyData {
    fd.min_value = Some(x);
    fd
}

pub fn with_max(x: f64, mut fd: AmountOfMoneyData) -> AmountOfMoneyData {
    fd.max_value = Some(x);
    fd
}

pub fn dollar_coin(name: &str) -> Option<f64> {
    match name {
        "nickel" | "nickels" => Some(0.05),
        "dime" | "dimes" => Some(0.1),
        "quarter" | "quarters" => Some(0.25),
        _ => None,
    }
}

fn aom(t: &Token) -> Option<&AmountOfMoneyData> {
    match t {
        Token::AmountOfMoney(d) => Some(d),
        _ => None,
    }
}

pub fn is_cents(t: &Token) -> bool {
    matches!(t, Token::AmountOfMoney(AmountOfMoneyData { value: Some(_), currency: Currency::Cent, .. }))
}

pub fn is_without_cents(t: &Token) -> bool {
    match t {
        Token::AmountOfMoney(AmountOfMoneyData { currency: Currency::Cent, .. }) => false,
        Token::AmountOfMoney(AmountOfMoneyData { value: Some(v), .. }) => is_integer(*v),
        _ => false,
    }
}

pub fn is_money_with_value(t: &Token) -> bool {
    matches!(t, Token::AmountOfMoney(d) if d.value.is_some() || d.min_value.is_some() || d.max_value.is_some())
}

pub fn is_currency_only(t: &Token) -> bool {
    matches!(
        t,
        Token::AmountOfMoney(AmountOfMoneyData { value: None, min_value: None, max_value: None, .. })
    )
}

pub fn is_dollar_coin(t: &Token) -> bool {
    match t {
        Token::AmountOfMoney(AmountOfMoneyData { value: Some(d), currency, .. }) => {
            [0.05, 0.1, 0.25].contains(d)
                && [
                    Currency::Dollar,
                    Currency::AUD,
                    Currency::CAD,
                    Currency::JMD,
                    Currency::NZD,
                    Currency::SGD,
                    Currency::TTD,
                    Currency::USD,
                ]
                .contains(currency)
        }
        _ => false,
    }
}

pub fn is_simple_amount_of_money(t: &Token) -> bool {
    matches!(
        t,
        Token::AmountOfMoney(AmountOfMoneyData { value: Some(_), min_value: None, max_value: None, .. })
    )
}

#[allow(dead_code)]
pub fn is_dime(t: &Token) -> bool {
    match t {
        Token::AmountOfMoney(AmountOfMoneyData { value: Some(d), currency: Currency::Cent, .. }) => {
            get_int_value(*d).map(|i| i.rem_euclid(10) == 0).unwrap_or(false)
        }
        _ => false,
    }
}

fn tok(d: AmountOfMoneyData) -> Option<Token> {
    Some(Token::AmountOfMoney(d))
}

fn numeral_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

// ----- common rules (`Duckling.AmountOfMoney.Rules`) -----------------------------------------

fn currency_of(s: &str) -> Option<Currency> {
    use Currency::*;
    Some(match s {
        "aed" => AED, "aud" => AUD, "bgn" => BGN, "brl" => BRL, "byn" => BYN, "cad" => CAD,
        "\u{a2}" => Cent, "c" => Cent, "chf" => CHF, "cny" => CNY, "czk" => CZK, "rmb" => CNY,
        "yuan" => CNY, "$" => Dollar, "dinar" => Dinar, "dinars" => Dinar, "dkk" => DKK,
        "dollar" => Dollar, "dollars" => Dollar, "egp" => EGP, "\u{20ac}" => EUR, "eur" => EUR,
        "euro" => EUR, "euros" => EUR, "eurs" => EUR, "\u{20ac}ur" => EUR, "\u{20ac}uro" => EUR,
        "\u{20ac}uros" => EUR, "\u{20ac}urs" => EUR, "gbp" => GBP, "gel" => GEL, "hkd" => HKD,
        "hrk" => HRK, "idr" => IDR, "ils" => ILS, "\u{20aa}" => ILS, "nis" => ILS, "inr" => INR,
        "iqd" => IQD, "rs" => INR, "rs." => INR, "rupee" => INR, "rupees" => INR, "jmd" => JMD,
        "jod" => JOD, "\u{a5}" => JPY, "jpy" => JPY, "lari" => GEL, "\u{20be}" => GEL, "yen" => JPY,
        "krw" => KRW, "kwd" => KWD, "lbp" => LBP, "mad" => MAD, "mnt" => MNT, "myr" => MYR,
        "rm" => MYR, "\u{20ae}" => MNT, "tugrik" => MNT, "tugriks" => MNT, "nok" => NOK,
        "nzd" => NZD, "pkr" => PKR, "pln" => PLN, "\u{a3}" => Pound, "pt" => PTS, "pta" => PTS,
        "ptas" => PTS, "pts" => PTS, "qar" => QAR, "\u{20bd}" => RUB, "rial" => Rial,
        "rials" => Rial, "riyal" => Riyal, "riyals" => Riyal, "ron" => RON, "rub" => RUB,
        "sar" => SAR, "sek" => SEK, "sgd" => SGD, "shekel" => ILS, "shekels" => ILS, "thb" => THB,
        "ttd" => TTD, "\u{20b4}" => UAH, "uah" => UAH, "usd" => USD, "us$" => USD, "vnd" => VND,
        "zar" => ZAR, "tl" => TRY, "lira" => TRY, "\u{20ba}" => TRY,
        _ => return None,
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "<amount> <unit>",
            vec![pred(is_positive), pred(is_currency_only)],
            |ts| {
                let v = numeral_value(ts.first()?)?;
                let c = aom(ts.get(1)?)?.currency;
                tok(with_value(v, currency_only(c)))
            },
        ),
        rule("<amount> (latent)", vec![pred(is_positive)], |ts| {
            let v = numeral_value(ts.first()?)?;
            let mut d = value_only(v);
            d.latent = true;
            tok(d)
        }),
        rule(
            "currencies",
            vec![regex("(aed|aud|bgn|brl|byn|\u{a2}|cad|chf|cny|c|\\$|dinars?|dkk|dollars?|egp|(e|\u{20ac})uro?s?|\u{20ac}|gbp|gel|\u{20be}|hkd|hrk|idr|ils|\u{20aa}|inr|iqd|jmd|jod|\u{a5}|jpy|lari|krw|kwd|lbp|mad|\u{20ae}|mnt|tugriks?|myr|rm|nis|nok|nzd|\u{a3}|pkr|pln|pta?s?|qar|\u{20bd}|rs\\.?|riy?als?|ron|rub|rupees?|sar|sek|sgb|shekels?|thb|ttd|\u{20b4}|uah|us(d|\\$)|vnd|yen|yuan|zar|tl|lira|\u{20ba})")],
            |ts| {
                let m = groups(ts)?.first()?;
                tok(currency_only(currency_of(&to_lower(m))?))
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}

// ----- EN rules (`Duckling.AmountOfMoney.EN.Rules`) ------------------------------------------

fn const_currency(name: &str, re: &str, c: Currency) -> Rule {
    rule(name, vec![regex(re)], move |_| tok(currency_only(c)))
}

fn interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> to|and <amount-of-money>",
        vec![regex("between|from"), pred(is_positive), regex("to|and"), pred(is_simple_amount_of_money)],
        |ts| {
            let from = numeral_value(ts.get(1)?)?;
            let d = aom(ts.get(3)?)?;
            let to = d.value?;
            if from < to { tok(with_interval(from, to, currency_only(d.currency))) } else { None }
        },
    )
}

fn interval_between() -> Rule {
    rule(
        "between|from <amount-of-money> to|and <amount-of-money>",
        vec![regex("between|from"), pred(is_simple_amount_of_money), regex("to|and"), pred(is_simple_amount_of_money)],
        |ts| {
            let a = aom(ts.get(1)?)?;
            let b = aom(ts.get(3)?)?;
            let (from, to) = (a.value?, b.value?);
            if from < to && a.currency == b.currency {
                tok(with_interval(from, to, currency_only(a.currency)))
            } else {
                None
            }
        },
    )
}

fn interval_numeral_dash() -> Rule {
    rule(
        "<numeral> - <amount-of-money>",
        vec![pred(is_natural), regex("-"), pred(is_simple_amount_of_money)],
        |ts| {
            let from = numeral_value(ts.first()?)?;
            let d = aom(ts.get(2)?)?;
            let to = d.value?;
            if from < to { tok(with_interval(from, to, currency_only(d.currency))) } else { None }
        },
    )
}

fn interval_dash() -> Rule {
    rule(
        "<amount-of-money> - <amount-of-money>",
        vec![pred(is_simple_amount_of_money), regex("-"), pred(is_simple_amount_of_money)],
        |ts| {
            let a = aom(ts.first()?)?;
            let b = aom(ts.get(2)?)?;
            let (from, to) = (a.value?, b.value?);
            if from < to && a.currency == b.currency {
                tok(with_interval(from, to, currency_only(a.currency)))
            } else {
                None
            }
        },
    )
}

fn interval_max() -> Rule {
    rule(
        "under/less/lower/no more than <amount-of-money>",
        vec![regex("under|at most|(less|lower|not? more) than"), pred(is_simple_amount_of_money)],
        |ts| {
            let d = aom(ts.get(1)?)?;
            tok(with_max(d.value?, currency_only(d.currency)))
        },
    )
}

fn interval_min() -> Rule {
    rule(
        "over/above/at least/more than <amount-of-money>",
        vec![regex("over|above|at least|(more|not? less) than"), pred(is_simple_amount_of_money)],
        |ts| {
            let d = aom(ts.get(1)?)?;
            tok(with_min(d.value?, currency_only(d.currency)))
        },
    )
}

fn intersect_cents(name: &str, pattern: Vec<crate::types::PatternItem>, cents_idx: usize) -> Rule {
    rule(name, pattern, move |ts| {
        let fd = aom(ts.first()?)?.clone();
        let c = match ts.get(cents_idx)? {
            Token::AmountOfMoney(d) => d.value?,
            Token::Numeral(n) => n.value,
            _ => return None,
        };
        tok(with_cents(c, fd))
    })
}

static EN_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
            let c = aom(ts.first()?)?.currency;
            let v = numeral_value(ts.get(1)?)?;
            tok(with_value(v, currency_only(c)))
        }),
        rule("a <currency>", vec![regex("an?"), pred(is_currency_only)], |ts| {
            let mut fd = aom(ts.get(1)?)?.clone();
            fd.value = Some(1.0);
            tok(fd)
        }),
        rule("a <amount-of-money>", vec![regex("an?"), pred(is_simple_amount_of_money)], |ts| {
            ts.get(1).map(|t| (*t).clone())
        }),
        const_currency_unnamed("bucks", "bucks?"),
        const_currency("cent", "cents?|penn(y|ies)|pence|sens?", Currency::Cent),
        rule("a <dollar coin>", vec![regex("an?"), pred(is_dollar_coin)], |ts| ts.get(1).map(|t| (*t).clone())),
        rule("X <dollar coins>", vec![pred(is_natural), pred(is_dollar_coin)], |ts| {
            let c = numeral_value(ts.first()?)?;
            let d = aom(ts.get(1)?)?;
            tok(with_value(c * d.value?, currency_only(d.currency)))
        }),
        rule("dinars", vec![regex("(kuwaiti) ?dinars?")], |ts| {
            match to_lower(groups(ts)?.first()?).as_str() {
                "kuwaiti" => tok(currency_only(Currency::KWD)),
                _ => None,
            }
        }),
        const_currency("dirham", "dirhams?", Currency::AED),
        const_currency("hryvnia", "hryvnia", Currency::UAH),
        intersect_cents(
            "intersect",
            vec![pred(is_without_cents), pred(is_natural)],
            1,
        ),
        intersect_cents(
            "intersect (and number)",
            vec![pred(is_without_cents), regex("and"), pred(is_natural)],
            2,
        ),
        intersect_cents(
            "intersect (and X cents)",
            vec![pred(is_without_cents), regex("and"), pred(is_cents)],
            2,
        ),
        intersect_cents(
            "intersect (X cents)",
            vec![pred(is_without_cents), pred(is_cents)],
            1,
        ),
        interval_between_numeral(),
        interval_between(),
        interval_max(),
        interval_min(),
        interval_numeral_dash(),
        interval_dash(),
        const_currency("kopiyka", "kopiy(ok|kas?)", Currency::Cent),
        rule("other pounds", vec![regex("(egyptian|lebanese) ?pounds?")], |ts| {
            match to_lower(groups(ts)?.first()?).as_str() {
                "egyptian" => tok(currency_only(Currency::EGP)),
                "lebanese" => tok(currency_only(Currency::LBP)),
                _ => None,
            }
        }),
        const_currency("livre \u{e9}gyptienne", "[lL].?[eE].?", Currency::EGP),
        const_currency("geneh", "[Gg][eiy]*n[eiy]*h(at)?( m[aiey]?sr[eiy]+a?)?", Currency::EGP),
        const_currency("\u{a3}", "pounds?", Currency::Pound),
        rule(
            "about|exactly <amount-of-money>",
            vec![
                regex("exactly|precisely|about|approx(\\.|imately)?|close to|near( to)?|around|almost"),
                pred(is_money_with_value),
            ],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        const_currency("ringgit", "(malaysian? )?ringgits?", Currency::MYR),
        rule("riyals", vec![regex("(qatari|saudi) ?riyals?")], |ts| {
            match to_lower(groups(ts)?.first()?).as_str() {
                "qatari" => tok(currency_only(Currency::QAR)),
                "saudi" => tok(currency_only(Currency::SAR)),
                _ => None,
            }
        }),
    ]
});

fn const_currency_unnamed(name: &str, re: &str) -> Rule {
    const_currency(name, re, Currency::Unnamed)
}

pub fn en_rules() -> &'static [Rule] {
    &EN_RULES
}

// ----- EN locale rules (`Duckling.AmountOfMoney.EN.<REGION>.Rules`) ---------------------------

/// `(currency of "a grand"/"<amount> grand", currency of dollar coins)` per region; only AU has
/// grand rules named for AUD, etc. Every regional module has the same three rules.
fn locale_rules(grand: Currency, coins: Currency) -> Vec<Rule> {
    vec![
        rule("a grand", vec![regex("a grand")], move |_| tok(with_value(1000.0, currency_only(grand)))),
        rule("<amount> grand", vec![pred(is_positive), regex("grand")], move |ts| {
            let v = numeral_value(ts.first()?)?;
            tok(with_value(1000.0 * v, currency_only(grand)))
        }),
        rule("dollar coin", vec![regex("(nickel|dime|quarter)s?")], move |ts| {
            let c = dollar_coin(&to_lower(groups(ts)?.first()?))?;
            tok(with_value(c, currency_only(coins)))
        }),
    ]
}

macro_rules! region_rules {
    ($name:ident, $grand:expr, $coins:expr) => {
        static $name: Lazy<Vec<Rule>> = Lazy::new(|| locale_rules($grand, $coins));
    };
}

region_rules!(AU_RULES, Currency::AUD, Currency::AUD);
region_rules!(BZ_RULES, Currency::Dollar, Currency::Dollar);
region_rules!(CA_RULES, Currency::CAD, Currency::CAD);
region_rules!(GB_RULES, Currency::GBP, Currency::Dollar);
region_rules!(IE_RULES, Currency::Dollar, Currency::Dollar);
region_rules!(IN_RULES, Currency::Dollar, Currency::Dollar);
region_rules!(JM_RULES, Currency::JMD, Currency::JMD);
region_rules!(NZ_RULES, Currency::NZD, Currency::NZD);
region_rules!(PH_RULES, Currency::Dollar, Currency::Dollar);
region_rules!(TT_RULES, Currency::TTD, Currency::TTD);
region_rules!(US_RULES, Currency::USD, Currency::USD);
region_rules!(ZA_RULES, Currency::Dollar, Currency::Dollar);

/// Rules of `Duckling.AmountOfMoney.EN.<REGION>.Rules`.
pub fn en_region_rules(region: crate::locale::Region) -> &'static [Rule] {
    use crate::locale::Region::*;
    match region {
        AU => &AU_RULES,
        BZ => &BZ_RULES,
        CA => &CA_RULES,
        GB => &GB_RULES,
        IE => &IE_RULES,
        IN => &IN_RULES,
        JM => &JM_RULES,
        NZ => &NZ_RULES,
        PH => &PH_RULES,
        TT => &TT_RULES,
        US => &US_RULES,
        ZA => &ZA_RULES,
        _ => &[],
    }
}
