//! `Duckling.Quantity.Types` (rules are per language).

use std::hash::{Hash, Hasher};

use crate::dims::interval::{self, Direction};
use crate::dims::util::hash_opt_f64;
use crate::json::Json;
use crate::types::{Context, Options};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Unit {
    Bowl,
    Cup,
    Custom(String),
    Dish,
    Gram,
    Ounce,
    Pint,
    Pound,
    Quart,
    Tablespoon,
    Teaspoon,
    Unnamed,
}

impl Unit {
    pub fn json(&self) -> Json {
        use crate::dims::util::to_lower;
        Json::str(match self {
            Unit::Custom(x) => to_lower(x),
            Unit::Bowl => "bowl".to_string(),
            Unit::Cup => "cup".to_string(),
            Unit::Dish => "dish".to_string(),
            Unit::Gram => "gram".to_string(),
            Unit::Ounce => "ounce".to_string(),
            Unit::Pint => "pint".to_string(),
            Unit::Pound => "pound".to_string(),
            Unit::Quart => "quart".to_string(),
            Unit::Tablespoon => "tablespoon".to_string(),
            Unit::Teaspoon => "teaspoon".to_string(),
            Unit::Unnamed => "unnamed".to_string(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuantityData {
    pub unit: Option<Unit>,
    pub value: Option<f64>,
    pub aproduct: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub latent: bool,
}

impl Eq for QuantityData {}

impl Hash for QuantityData {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.unit.hash(h);
        hash_opt_f64(self.value, h);
        self.aproduct.hash(h);
        hash_opt_f64(self.min_value, h);
        hash_opt_f64(self.max_value, h);
        self.latent.hash(h);
    }
}

fn single(u: &Unit, v: f64, p: &Option<String>) -> Json {
    let mut pairs = vec![("value", Json::Num(v)), ("unit", u.json())];
    if let Some(p) = p {
        pairs.push(("product", Json::str(p.clone())));
    }
    Json::obj(pairs)
}

impl QuantityData {
    pub fn resolve(&self, _ctx: &Context, opts: &Options) -> Option<(Json, bool)> {
        if !opts.with_latent && self.latent {
            return None;
        }
        let p = &self.aproduct;
        if let Some(v) = self.value {
            let u = self.unit.clone().unwrap_or(Unit::Unnamed);
            return Some((interval::simple(single(&u, v, p)), self.latent));
        }
        let u = self.unit.as_ref()?;
        match (self.min_value, self.max_value) {
            (Some(from), Some(to)) => Some((interval::between(single(u, from, p), single(u, to, p)), self.latent)),
            (Some(v), None) => Some((interval::open(Direction::Above, single(u, v, p)), self.latent)),
            (None, Some(v)) => Some((interval::open(Direction::Under, single(u, v, p)), self.latent)),
            (None, None) => None,
        }
    }
}

// ----- helpers + EN rules (`Duckling.Quantity.Helpers`, `.EN.Rules`) ----------------------

use crate::dims::numeral::is_positive;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};
use once_cell::sync::Lazy;

fn tok(d: QuantityData) -> Option<Token> {
    Some(Token::Quantity(d))
}

pub fn quantity(u: Unit, v: f64) -> QuantityData {
    QuantityData { unit: Some(u), value: Some(v), aproduct: None, min_value: None, max_value: None, latent: false }
}

pub fn unit_only(u: Unit) -> QuantityData {
    QuantityData { unit: Some(u), value: None, aproduct: None, min_value: None, max_value: None, latent: false }
}

pub fn value_only(v: f64) -> QuantityData {
    QuantityData { unit: None, value: Some(v), aproduct: None, min_value: None, max_value: None, latent: false }
}

pub fn with_interval(from: f64, to: f64, mut q: QuantityData) -> QuantityData {
    q.min_value = Some(from);
    q.max_value = Some(to);
    q
}

pub fn with_min(from: f64, mut q: QuantityData) -> QuantityData {
    q.min_value = Some(from);
    q
}

pub fn with_max(to: f64, mut q: QuantityData) -> QuantityData {
    q.max_value = Some(to);
    q
}

pub fn is_simple_quantity(t: &Token) -> bool {
    matches!(t, Token::Quantity(QuantityData { unit: Some(_), value: Some(_), .. }))
}

/// `(value, unit)` of a simple quantity without a product.
fn qv(t: &Token) -> Option<(f64, Unit)> {
    match t {
        Token::Quantity(QuantityData { value: Some(v), unit: Some(u), aproduct: None, .. }) => {
            Some((*v, u.clone()))
        }
        _ => None,
    }
}

fn numeral_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

/// `getValue opsMap match`.
fn get_value(m: &str, v: f64) -> f64 {
    match to_lower(m).as_str() {
        "milligram" | "milligrams" | "mg" | "mgs" | "m.g" | "m.gs" | "m.g." | "m.g.s" => v / 1000.0,
        "kilogram" | "kilograms" | "kg" | "kgs" | "k.g" | "k.gs" | "k.g." | "k.g.s" => v * 1000.0,
        _ => v,
    }
}

const QUANTITIES: [(&str, &str, fn() -> Unit); 4] = [
    ("<quantity> cups", "(cups?)", || Unit::Cup),
    (
        "<quantity> grams",
        "(((m(illi)?[.]?)|(k(ilo)?)[.]?)?g(ram)?s?[.]?)[.]?",
        || Unit::Gram,
    ),
    ("<quantity> lb", "((lb|pound)s?)", || Unit::Pound),
    ("<quantity> oz", "((ounces?)|oz)", || Unit::Ounce),
];

static EN_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v: Vec<Rule> = vec![
        rule("<quantity> of product", vec![dimension(Dim::Quantity), regex("of (\\w+)")], |ts| {
            let q = match ts.first()? {
                Token::Quantity(q) => q,
                _ => return None,
            };
            let p = groups(&ts[1..])?.first()?.clone();
            let mut q = q.clone();
            q.aproduct = Some(to_lower(&p));
            tok(q)
        }),
        rule(
            "over/above/exceeding/beyond/at least/more than <quantity>",
            vec![
                regex("over|above|exceeding|beyond|at least|(more|larger|bigger|heavier) than"),
                pred(is_simple_quantity),
            ],
            |ts| {
                let (from, u) = qv(ts.get(1)?)?;
                tok(with_min(from, unit_only(u)))
            },
        ),
        rule(
            "under/below/less/lower/at most/no more than <dist>",
            vec![regex("under|below|at most|(less|lower|not? more) than"), pred(is_simple_quantity)],
            |ts| {
                let (to, u) = qv(ts.get(1)?)?;
                tok(with_max(to, unit_only(u)))
            },
        ),
        rule(
            "between|from <numeral> and|to <quantity>",
            vec![regex("between|from"), pred(is_positive), regex("to|and"), pred(is_simple_quantity)],
            |ts| {
                let from = numeral_value(ts.get(1)?)?;
                let (to, u) = qv(ts.get(3)?)?;
                if from < to { tok(with_interval(from, to, unit_only(u))) } else { None }
            },
        ),
        rule(
            "between|from <quantity> to|and <quantity>",
            vec![regex("between|from"), pred(is_simple_quantity), regex("and|to"), pred(is_simple_quantity)],
            |ts| {
                let (from, u1) = qv(ts.get(1)?)?;
                let (to, u2) = qv(ts.get(3)?)?;
                if from < to && u1 == u2 { tok(with_interval(from, to, unit_only(u1))) } else { None }
            },
        ),
        rule(
            "<numeral> - <quantity>",
            vec![pred(is_positive), regex("\\-"), pred(is_simple_quantity)],
            |ts| {
                let from = numeral_value(ts.first()?)?;
                let (to, u) = qv(ts.get(2)?)?;
                if from < to { tok(with_interval(from, to, unit_only(u))) } else { None }
            },
        ),
        rule(
            "<quantity> - <quantity>",
            vec![pred(is_simple_quantity), regex("\\-"), pred(is_simple_quantity)],
            |ts| {
                let (from, u1) = qv(ts.first()?)?;
                let (to, u2) = qv(ts.get(2)?)?;
                if from < to && u1 == u2 { tok(with_interval(from, to, unit_only(u1))) } else { None }
            },
        ),
        rule(
            "about|exactly <quantity>",
            vec![
                regex("\\~|exactly|precisely|about|approx(\\.|imately)?|close to|near( to)?|around|almost"),
                dimension(Dim::Quantity),
            ],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        rule("<quantity> (latent)", vec![pred(is_positive)], |ts| {
            let v = numeral_value(ts.first()?)?;
            let mut q = value_only(v);
            q.latent = true;
            tok(q)
        }),
    ];
    for (name, re, unit) in QUANTITIES {
        v.push(rule(name, vec![pred(is_positive), regex(re)], move |ts| {
            let n = numeral_value(ts.first()?)?;
            let m = match ts.get(1)? {
                Token::RegexMatch(g) => g.0.first()?.clone(),
                _ => return None,
            };
            tok(quantity(unit(), get_value(&m, n)))
        }));
    }
    for (name, re, unit) in QUANTITIES {
        v.push(rule(name, vec![regex(&format!("an? {}", re))], move |ts| {
            let m = match ts.first()? {
                Token::RegexMatch(g) => g.0.first()?.clone(),
                _ => return None,
            };
            tok(quantity(unit(), get_value(&m, 1.0)))
        }));
    }
    v
});

pub fn en_rules() -> &'static [Rule] {
    &EN_RULES
}
