//! `Duckling.Distance.Types` (rules are per language).

use std::hash::{Hash, Hasher};

use crate::dims::interval::{self, Direction};
use crate::dims::util::hash_opt_f64;
use crate::json::Json;
use crate::types::{Context, Options};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Unit {
    Centimetre,
    Foot,
    Inch,
    Kilometre,
    /// Ambiguous between Mile and Metre.
    M,
    Metre,
    Mile,
    Millimetre,
    Yard,
}

impl Unit {
    pub fn json(self) -> Json {
        Json::str(match self {
            Unit::Centimetre => "centimetre",
            Unit::Foot => "foot",
            Unit::Inch => "inch",
            Unit::Kilometre => "kilometre",
            Unit::M => "m",
            Unit::Metre => "metre",
            Unit::Mile => "mile",
            Unit::Millimetre => "millimetre",
            Unit::Yard => "yard",
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DistanceData {
    pub unit: Option<Unit>,
    pub value: Option<f64>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl Eq for DistanceData {}

impl Hash for DistanceData {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.unit.hash(h);
        hash_opt_f64(self.value, h);
        hash_opt_f64(self.min_value, h);
        hash_opt_f64(self.max_value, h);
    }
}

fn single(u: Unit, v: f64) -> Json {
    Json::obj([("value", Json::Num(v)), ("unit", u.json())])
}

impl DistanceData {
    pub fn resolve(&self, _ctx: &Context, _opts: &Options) -> Option<(Json, bool)> {
        let u = self.unit?;
        match (self.value, self.min_value, self.max_value) {
            (Some(v), _, _) => Some((interval::simple(single(u, v)), false)),
            (None, Some(from), Some(to)) => Some((interval::between(single(u, from), single(u, to)), false)),
            (None, Some(v), None) => Some((interval::open(Direction::Above, single(u, v)), false)),
            (None, None, Some(v)) => Some((interval::open(Direction::Under, single(u, v)), false)),
            (None, None, None) => None,
        }
    }
}
