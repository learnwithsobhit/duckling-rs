//! `Duckling.Distance.NL.Rules`, `Duckling.Quantity.NL.Rules` and `Duckling.Volume.NL.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance as dist;
use crate::dims::numeral::is_positive;
use crate::dims::quantity as qty;
use crate::dims::util::to_lower;
use crate::dims::volume as vol;
use crate::types::{dimension, group, pred, regex, rule, Dim, Rule, Token};

// ----- Distance ---------------------------------------------------------------------------

static DISTANCE_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let units: [(&str, &str, dist::Unit); 4] = [
        ("<dist> centimeters", "(c(enti)?m(eter)?s?)", dist::Unit::Centimetre),
        ("<dist> meters", "m(eter)?", dist::Unit::Metre),
        ("<dist> miles", "mijl?", dist::Unit::Mile),
        ("<latent dist> km", "k(ilo)?m?(eter)?s?", dist::Unit::Kilometre),
    ];
    units
        .into_iter()
        .map(|(name, re, u)| {
            rule(name, vec![dimension(Dim::Distance), regex(re)], move |ts| match ts.first()? {
                Token::Distance(dd) => Some(Token::Distance(dist::with_unit(u, dd.clone()))),
                _ => None,
            })
        })
        .collect()
});

pub fn distance_rules() -> &'static [Rule] {
    &DISTANCE_RULES
}

// ----- Quantity ---------------------------------------------------------------------------

fn numeral_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

/// `getValue opsMap match`.
fn get_value(m: &str, v: f64) -> f64 {
    match to_lower(m).as_str() {
        "milligram" | "mg" => v / 1000.0,
        "kilo" | "kilogram" | "kg" => v * 1000.0,
        "pond" => v * 500.0,
        "ons" => v * 100.0,
        _ => v,
    }
}

/// `(value, unit)` of a simple quantity without a product.
fn qv(t: &Token) -> Option<(f64, qty::Unit)> {
    match t {
        Token::Quantity(qty::QuantityData { value: Some(v), unit: Some(u), aproduct: None, .. }) => {
            Some((*v, u.clone()))
        }
        _ => None,
    }
}

fn qty_tok(d: qty::QuantityData) -> Option<Token> {
    Some(Token::Quantity(d))
}

fn quantities() -> Vec<(&'static str, &'static str, qty::Unit)> {
    vec![
        ("<quantity> kopje", "(kopjes?)", qty::Unit::Cup),
        ("<quantity> grams", "(g((r)?(am)?)?)", qty::Unit::Gram),
        ("<quantity> milligrams", "((m(illi)?)(g(ram)?))", qty::Unit::Gram),
        ("<quantity> kilograms", "((k(ilo)?)(g(ram)?)?)", qty::Unit::Gram),
        ("<quantity> pond", "(pond(je(s)?)?)", qty::Unit::Gram),
        ("<quantity> ons", "(ons(je(s)?)?)", qty::Unit::Gram),
    ]
}

static QUANTITY_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v: Vec<Rule> = vec![
        // ruleQuantityOfProduct
        rule("<quantity> product", vec![dimension(Dim::Quantity), regex("(\\w+)")], |ts| {
            let q = match ts.first()? {
                Token::Quantity(q) => q,
                _ => return None,
            };
            let p = match ts.get(1)? {
                Token::RegexMatch(g) => g.0.first()?.clone(),
                _ => return None,
            };
            let mut q = q.clone();
            q.aproduct = Some(to_lower(&p));
            qty_tok(q)
        }),
        // ruleIntervalMin
        rule(
            "meer dan/minstens/op zijn minst <quantity>",
            vec![regex("meer dan|minstens|minimaal|op zijn minst|minder dan"), pred(qty::is_simple_quantity)],
            |ts| {
                let (from, u) = qv(ts.get(1)?)?;
                qty_tok(qty::with_min(from, qty::unit_only(u)))
            },
        ),
        // ruleIntervalMax
        rule(
            "minder dan/hoogstens/op zijn hoogst/maximaal/hooguit <quantity>",
            vec![regex("minder dan|hoogstens|hooguit|maximaal|op zijn hoogst"), pred(qty::is_simple_quantity)],
            |ts| {
                let (to, u) = qv(ts.get(1)?)?;
                qty_tok(qty::with_max(to, qty::unit_only(u)))
            },
        ),
        // ruleIntervalBetweenNumeral
        rule(
            "tussen|van <numeral> en|tot <quantity>",
            vec![regex("tussen|van"), pred(is_positive), regex("tot|en"), pred(qty::is_simple_quantity)],
            |ts| {
                let from = numeral_value(ts.get(1)?)?;
                let (to, u) = qv(ts.get(3)?)?;
                if from < to {
                    qty_tok(qty::with_interval(from, to, qty::unit_only(u)))
                } else {
                    None
                }
            },
        ),
        // ruleIntervalBetween
        rule(
            "rond|tussen|van <quantity> tot|en <quantity>",
            vec![
                regex("tussen|van"),
                pred(qty::is_simple_quantity),
                regex("en|tot"),
                pred(qty::is_simple_quantity),
            ],
            |ts| {
                let (from, u1) = qv(ts.get(1)?)?;
                let (to, u2) = qv(ts.get(3)?)?;
                if from < to && u1 == u2 {
                    qty_tok(qty::with_interval(from, to, qty::unit_only(u1)))
                } else {
                    None
                }
            },
        ),
        // ruleIntervalNumeralDash
        rule(
            "<numeral> - <quantity>",
            vec![pred(is_positive), regex("\\-"), pred(qty::is_simple_quantity)],
            |ts| {
                let from = numeral_value(ts.first()?)?;
                let (to, u) = qv(ts.get(2)?)?;
                if from < to {
                    qty_tok(qty::with_interval(from, to, qty::unit_only(u)))
                } else {
                    None
                }
            },
        ),
        // ruleIntervalDash
        rule(
            "<quantity> - <quantity>",
            vec![pred(qty::is_simple_quantity), regex("\\-"), pred(qty::is_simple_quantity)],
            |ts| {
                let (from, u1) = qv(ts.first()?)?;
                let (to, u2) = qv(ts.get(2)?)?;
                if from < to && u1 == u2 {
                    qty_tok(qty::with_interval(from, to, qty::unit_only(u1)))
                } else {
                    None
                }
            },
        ),
        // rulePrecision
        rule(
            "ongeveer|plm|plusminus <quantity>",
            vec![regex("\\~|precies|exact|ongeveer|bijna|ongeveer"), dimension(Dim::Quantity)],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
    ];
    // ruleNumeralQuantities
    for (name, re, unit) in quantities() {
        v.push(rule(name, vec![pred(is_positive), regex(re)], move |ts| {
            let n = numeral_value(ts.first()?)?;
            let m = match ts.get(1)? {
                Token::RegexMatch(g) => g.0.first()?.clone(),
                _ => return None,
            };
            qty_tok(qty::quantity(unit.clone(), get_value(&m, n)))
        }));
    }
    // ruleAQuantity
    for (name, re, unit) in quantities() {
        v.push(rule(name, vec![regex(&format!("een? {}", re))], move |ts| {
            let m = group(ts, 0)?;
            qty_tok(qty::quantity(unit.clone(), get_value(&m, 1.0)))
        }));
    }
    v
});

pub fn quantity_rules() -> &'static [Rule] {
    &QUANTITY_RULES
}

// ----- Volume -----------------------------------------------------------------------------

static VOLUME_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let volumes: [(&str, &str, vol::Unit); 3] = [
        ("<latent vol> ml", "m(illi)?l(iter)?", vol::Unit::Millilitre),
        ("<vol> hl", "h(ecto)?l(iter)?", vol::Unit::Hectolitre),
        ("<vol> liters", "l(iters?)?", vol::Unit::Litre),
    ];
    let mut v: Vec<Rule> = volumes
        .into_iter()
        .map(|(name, re, u)| {
            rule(name, vec![regex(re)], move |_| Some(Token::Volume(vol::unit_only(u))))
        })
        .collect();
    v.push(rule("half", vec![regex("halve"), pred(vol::is_unit_only)], |ts| match ts.get(1)? {
        Token::Volume(vol::VolumeData { unit: Some(u), .. }) => Some(Token::Volume(vol::volume(*u, 1.0 / 2.0))),
        _ => None,
    }));
    v
});

pub fn volume_rules() -> &'static [Rule] {
    &VOLUME_RULES
}
