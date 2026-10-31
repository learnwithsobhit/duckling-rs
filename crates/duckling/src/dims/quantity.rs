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
