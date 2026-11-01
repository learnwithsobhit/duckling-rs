//! `Duckling.{Distance,Temperature,Volume,AmountOfMoney,Email}.IT.Rules`.

use once_cell::sync::Lazy;

use crate::dims::amount_of_money::{
    currency_only, is_cents, is_currency_only, is_money_with_value, is_simple_amount_of_money,
    is_without_cents, with_cents, with_interval, with_max, with_min, with_value, AmountOfMoneyData,
    Currency,
};
use crate::dims::distance::{self, DistanceData};
use crate::dims::email::EmailData;
use crate::dims::numeral::{is_natural, is_positive};
use crate::dims::temperature::{self, is_value_only, TemperatureData, TemperatureUnit};
use crate::dims::util::to_lower;
use crate::dims::volume::{self, is_unit_only, VolumeData};
use crate::types::{dimension, groups, pred, regex, rule, Dim, PatternItem, Rule, Token};

// ----- Distance --------------------------------------------------------------------------

fn dist_unit_rule(name: &str, re: &str, u: distance::Unit) -> Rule {
    rule(name, vec![dimension(Dim::Distance), regex(re)], move |ts| match ts.first()? {
        Token::Distance(dd) => Some(Token::Distance(distance::with_unit(u, dd.clone()))),
        _ => None,
    })
}

static DISTANCE_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    use distance::Unit;
    vec![
        dist_unit_rule("<dist> centimeters", "cm|centimetro?i?", Unit::Centimetre),
        dist_unit_rule("<dist> meters", "metro?i?|m", Unit::Metre),
        dist_unit_rule("<dist> miles", "miglio?a?", Unit::Mile),
        dist_unit_rule("<latent dist> km", "km|(k|ch)(ilo)?m?(etr)o?i?", Unit::Kilometre),
    ]
});

pub fn distance_rules() -> &'static [Rule] {
    let _: Option<DistanceData> = None;
    &DISTANCE_RULES
}

// ----- Temperature -----------------------------------------------------------------------

fn temp_unit_rule(name: &str, allow_degree: bool, re: &str, u: TemperatureUnit) -> Rule {
    rule(name, vec![pred(is_value_only(allow_degree)), regex(re)], move |ts| match ts.first()? {
        Token::Temperature(td) => Some(Token::Temperature(temperature::with_unit(u, td.clone()))),
        _ => None,
    })
}

static TEMPERATURE_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        temp_unit_rule("<latent temp> degrees", false, "(grad[io]?\\.?)|°", TemperatureUnit::Degree),
        temp_unit_rule(
            "<temp> Celcius",
            true,
            "(c((el[cs]?(ius)?)|(entigrad[io]))?\\.?)",
            TemperatureUnit::Celsius,
        ),
        temp_unit_rule(
            "<temp> Fahrenheit",
            true,
            "f(ah?rh?eh?n(h?eit)?)?\\.?",
            TemperatureUnit::Fahrenheit,
        ),
    ]
});

pub fn temperature_rules() -> &'static [Rule] {
    let _: Option<TemperatureData> = None;
    &TEMPERATURE_RULES
}

// ----- Volume ----------------------------------------------------------------------------

static VOLUME_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    use volume::Unit;
    let volumes: [(&str, &str, Unit); 4] = [
        ("<latent vol> ml", "m(l|illilitr(i|o))", Unit::Millilitre),
        ("<vol> hectoliters", "ettolitr(i|o)", Unit::Hectolitre),
        ("<vol> liters", "l(itr(i|o))?", Unit::Litre),
        ("<latent vol> gallon", "gal(lon(e|i))?", Unit::Gallon),
    ];
    let mut v: Vec<Rule> = Vec::new();
    for (name, re, u) in volumes {
        v.push(rule(name, vec![regex(re)], move |_| Some(Token::Volume(volume::unit_only(u)))));
    }
    let fractions: [(&str, &str, f64); 1] = [("half", "mezzo", 1.0 / 2.0)];
    for (name, re, f) in fractions {
        let pattern: Vec<PatternItem> = vec![regex(re), pred(is_unit_only)];
        v.push(rule(name, pattern, move |ts| match ts.get(1)? {
            Token::Volume(VolumeData { unit: Some(u), .. }) => {
                Some(Token::Volume(volume::volume(*u, f)))
            }
            _ => None,
        }));
    }
    v
});

pub fn volume_rules() -> &'static [Rule] {
    &VOLUME_RULES
}

// ----- Email -----------------------------------------------------------------------------

static EMAIL_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule(
        "email spelled out",
        vec![regex("([\\w\\._+-]+) chiocciola ([\\w_-]+(\\.[\\w_-]+)+)")],
        |ts| {
            let g = groups(ts)?;
            let value = format!("{}@{}", g.first()?, g.get(1)?);
            Some(Token::Email(EmailData { value }))
        },
    )]
});

pub fn email_rules() -> &'static [Rule] {
    &EMAIL_RULES
}

// ----- AmountOfMoney ---------------------------------------------------------------------

fn aom(t: &Token) -> Option<&AmountOfMoneyData> {
    match t {
        Token::AmountOfMoney(d) => Some(d),
        _ => None,
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

fn intersect_cents(name: &str, pattern: Vec<PatternItem>, cents_idx: usize) -> Rule {
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

static AMOUNT_OF_MONEY_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleUnitAmount
        rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
            let c = aom(ts.first()?)?.currency;
            let v = numeral_value(ts.get(1)?)?;
            tok(with_value(v, currency_only(c)))
        }),
        // ruleCent
        rule("cent", vec![regex("cent(esim)i?o?")], |_| tok(currency_only(Currency::Cent))),
        // ruleIntersect
        intersect_cents("intersect", vec![pred(is_without_cents), pred(is_natural)], 1),
        // ruleIntersectAndNumeral
        intersect_cents(
            "intersect (and number)",
            vec![pred(is_without_cents), regex("e"), pred(is_natural)],
            2,
        ),
        // ruleIntersectAndXCents
        intersect_cents(
            "intersect (and X cents)",
            vec![pred(is_without_cents), regex("e"), pred(is_cents)],
            2,
        ),
        // ruleIntersectXCents
        intersect_cents(
            "intersect (X cents)",
            vec![pred(is_without_cents), pred(is_cents)],
            1,
        ),
        // ruleIntervalBetween
        rule(
            "between|from <amount-of-money> to|and <amount-of-money>",
            vec![
                regex("tra"),
                pred(is_simple_amount_of_money),
                regex("e"),
                pred(is_simple_amount_of_money),
            ],
            |ts| {
                let a = aom(ts.get(1)?)?;
                let b = aom(ts.get(3)?)?;
                let (from, to) = (a.value?, b.value?);
                if a.currency == b.currency && from < to {
                    tok(with_interval(from, to, currency_only(a.currency)))
                } else {
                    None
                }
            },
        ),
        // ruleIntervalBetweenNumeral
        rule(
            "between|from <numeral> to|and <amount-of-money>",
            vec![regex("tra"), pred(is_positive), regex("e"), pred(is_simple_amount_of_money)],
            |ts| {
                let from = numeral_value(ts.get(1)?)?;
                let d = aom(ts.get(3)?)?;
                let to = d.value?;
                if from < to {
                    tok(with_interval(from, to, currency_only(d.currency)))
                } else {
                    None
                }
            },
        ),
        // ruleIntervalDash
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
        ),
        // ruleIntervalMax
        rule(
            "under/less/lower/no more than <amount-of-money>",
            vec![regex("(meno|non più) di"), pred(is_simple_amount_of_money)],
            |ts| {
                let d = aom(ts.get(1)?)?;
                tok(with_max(d.value?, currency_only(d.currency)))
            },
        ),
        // ruleIntervalMin
        rule(
            "over/above/at least/more than <amount-of-money>",
            vec![regex("più di|almeno"), pred(is_simple_amount_of_money)],
            |ts| {
                let d = aom(ts.get(1)?)?;
                tok(with_min(d.value?, currency_only(d.currency)))
            },
        ),
        // ruleIntervalNumeralDash
        rule(
            "<numeral> - <amount-of-money>",
            vec![pred(is_natural), regex("-"), pred(is_simple_amount_of_money)],
            |ts| {
                let from = numeral_value(ts.first()?)?;
                let d = aom(ts.get(2)?)?;
                let to = d.value?;
                if from < to {
                    tok(with_interval(from, to, currency_only(d.currency)))
                } else {
                    None
                }
            },
        ),
        // ruleCurrencies
        rule("£, $", vec![regex("(dollari|sterline)")], |ts| {
            match to_lower(groups(ts)?.first()?).as_str() {
                "dollari" => tok(currency_only(Currency::Dollar)),
                "sterline" => tok(currency_only(Currency::Pound)),
                _ => None,
            }
        }),
        // rulePrecision
        rule(
            "precision",
            vec![regex("esattamente|quasi|più o meno|circa"), pred(is_money_with_value)],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
    ]
});

pub fn amount_of_money_rules() -> &'static [Rule] {
    &AMOUNT_OF_MONEY_RULES
}
