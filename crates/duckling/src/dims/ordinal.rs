//! `Duckling.Ordinal.Types` (rules are per language).

use crate::json::Json;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrdinalData {
    pub value: i64,
}

impl OrdinalData {
    pub fn resolve(&self) -> Json {
        Json::obj([("type", Json::str("value")), ("value", Json::Int(self.value))])
    }
}

pub fn is_between(x: i64, low: i64, high: i64) -> bool {
    low <= x && x <= high
}
