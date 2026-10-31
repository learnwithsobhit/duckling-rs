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
