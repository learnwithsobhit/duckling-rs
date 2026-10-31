//! `Duckling.Temperature.Types` (rules are per language).

use std::hash::{Hash, Hasher};

use crate::dims::interval::{self, Direction};
use crate::dims::util::hash_opt_f64;
use crate::json::Json;
use crate::types::{Context, Options};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TemperatureUnit {
    Degree,
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    pub fn json(self) -> Json {
        Json::str(match self {
            TemperatureUnit::Degree => "degree",
            TemperatureUnit::Celsius => "celsius",
            TemperatureUnit::Fahrenheit => "fahrenheit",
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TemperatureData {
    pub unit: Option<TemperatureUnit>,
    pub value: Option<f64>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl Eq for TemperatureData {}

impl Hash for TemperatureData {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.unit.hash(h);
        hash_opt_f64(self.value, h);
        hash_opt_f64(self.min_value, h);
        hash_opt_f64(self.max_value, h);
    }
}

fn single(u: TemperatureUnit, v: f64) -> Json {
    Json::obj([("value", Json::Num(v)), ("unit", u.json())])
}

impl TemperatureData {
    pub fn resolve(&self, _ctx: &Context, _opts: &Options) -> Option<(Json, bool)> {
        let u = self.unit?;
        if let Some(v) = self.value {
            return Some((interval::simple(single(u, v)), false));
        }
        // Note: unlike other dimensions, matching ignores `value` for the interval cases.
        match (self.min_value, self.max_value) {
            (Some(from), Some(to)) => Some((interval::between(single(u, from), single(u, to)), false)),
            (Some(v), None) => Some((interval::open(Direction::Above, single(u, v)), false)),
            (None, Some(v)) => Some((interval::open(Direction::Under, single(u, v)), false)),
            (None, None) => None,
        }
    }
}

/// `unitsAreCompatible`.
pub fn units_are_compatible(u1: Option<TemperatureUnit>, u2: TemperatureUnit) -> bool {
    match u1 {
        Some(u1) => u1 == u2,
        None => true,
    }
}

// ----- helpers + rules (`Duckling.Temperature.Helpers`, `.Rules`, `.EN.Rules`) ---------------

use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};
use once_cell::sync::Lazy;

fn tok(d: TemperatureData) -> Option<Token> {
    Some(Token::Temperature(d))
}

pub fn value_only(v: f64) -> TemperatureData {
    TemperatureData { unit: None, value: Some(v), min_value: None, max_value: None }
}

pub fn unit_only(u: TemperatureUnit) -> TemperatureData {
    TemperatureData { unit: Some(u), value: None, min_value: None, max_value: None }
}

pub fn with_unit(u: TemperatureUnit, mut td: TemperatureData) -> TemperatureData {
    td.unit = Some(u);
    td
}

pub fn with_interval(from: f64, to: f64, mut td: TemperatureData) -> TemperatureData {
    td.value = None;
    td.min_value = Some(from);
    td.max_value = Some(to);
    td
}

pub fn with_max(v: f64, mut td: TemperatureData) -> TemperatureData {
    td.value = None;
    td.min_value = None;
    td.max_value = Some(v);
    td
}

pub fn with_min(v: f64, mut td: TemperatureData) -> TemperatureData {
    td.value = None;
    td.min_value = Some(v);
    td.max_value = None;
    td
}

pub fn is_value_only(allow_degree: bool) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| match t {
        Token::Temperature(TemperatureData { unit, value: Some(_), min_value: None, max_value: None }) => {
            unit.is_none() || (allow_degree && *unit == Some(TemperatureUnit::Degree))
        }
        _ => false,
    }
}

pub fn is_simple_temperature(t: &Token) -> bool {
    matches!(t, Token::Temperature(TemperatureData { value: Some(_), .. }))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule("number as temp", vec![dimension(Dim::Numeral)], |ts| match ts.first()? {
        Token::Numeral(n) => tok(value_only(n.value)),
        _ => None,
    })]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}

fn temp(t: &Token) -> Option<&TemperatureData> {
    match t {
        Token::Temperature(d) => Some(d),
        _ => None,
    }
}

/// `(unit, value)` of a simple temperature.
fn uv(t: &Token) -> Option<(Option<TemperatureUnit>, f64)> {
    let d = temp(t)?;
    Some((d.unit, d.value?))
}

fn interval_rule(name: &str, pattern: Vec<crate::types::PatternItem>, second: usize) -> Rule {
    let first = second - 2;
    rule(name, pattern, move |ts| {
        let (u1, from) = uv(ts.get(first)?)?;
        let (u2, to) = uv(ts.get(second)?)?;
        let u2 = u2?;
        if from < to && units_are_compatible(u1, u2) {
            tok(with_interval(from, to, unit_only(u2)))
        } else {
            None
        }
    })
}

static EN_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "<latent temp> degrees",
            vec![pred(is_value_only(false)), regex("(deg(ree?)?s?\\.?)|\u{b0}")],
            |ts| temp(ts.first()?).and_then(|td| tok(with_unit(TemperatureUnit::Degree, td.clone()))),
        ),
        rule(
            "<temp> Celsius",
            vec![pred(is_value_only(true)), regex("c(el[cs]?(ius)?)?\\.?")],
            |ts| temp(ts.first()?).and_then(|td| tok(with_unit(TemperatureUnit::Celsius, td.clone()))),
        ),
        rule(
            "<temp> Fahrenheit",
            vec![pred(is_value_only(true)), regex("f(ah?rh?eh?n(h?eit)?)?\\.?")],
            |ts| temp(ts.first()?).and_then(|td| tok(with_unit(TemperatureUnit::Fahrenheit, td.clone()))),
        ),
        rule(
            "<temp> below zero",
            vec![pred(is_value_only(true)), regex("below zero")],
            |ts| {
                let td = temp(ts.first()?)?;
                let v = td.value?;
                let mut neg = td.clone();
                neg.value = Some(-v);
                match td.unit {
                    None => tok(with_unit(TemperatureUnit::Degree, neg)),
                    _ => tok(neg),
                }
            },
        ),
        interval_rule(
            "between|from <temp> and|to <temp>",
            vec![regex("between|from"), pred(is_simple_temperature), regex("to|and"), pred(is_simple_temperature)],
            3,
        ),
        interval_rule(
            "<temp> - <temp>",
            vec![pred(is_simple_temperature), regex("-"), pred(is_simple_temperature)],
            2,
        ),
        rule(
            "over/above/at least/more than <temp>",
            vec![regex("over|above|at least|more than"), pred(is_simple_temperature)],
            |ts| {
                let d = temp(ts.get(1)?)?;
                let (from, u) = (d.value?, d.unit?);
                tok(with_min(from, unit_only(u)))
            },
        ),
        rule(
            "under/less/lower/no more than <temp>",
            vec![regex("under|(less|lower|not? more) than"), pred(is_simple_temperature)],
            |ts| {
                let d = temp(ts.get(1)?)?;
                let (to, u) = (d.value?, d.unit?);
                tok(with_max(to, unit_only(u)))
            },
        ),
    ]
});

pub fn en_rules() -> &'static [Rule] {
    &EN_RULES
}
