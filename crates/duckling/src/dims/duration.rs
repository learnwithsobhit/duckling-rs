//! `Duckling.Duration.Types` (rules are per language).

use crate::dims::time_grain::Grain;
use crate::json::Json;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DurationData {
    pub value: i64,
    pub grain: Grain,
}

impl DurationData {
    pub fn resolve(&self) -> Json {
        Json::obj([
            ("type", Json::str("value")),
            ("value", Json::Int(self.value)),
            ("unit", self.grain.json()),
            (self.grain.show(), Json::Int(self.value)),
            (
                "normalized",
                Json::obj([
                    ("unit", Json::str("second")),
                    ("value", Json::Int(self.grain.in_seconds_int(self.value))),
                ]),
            ),
        ])
    }

    /// `withGrain`: convert to `g`, rounding to the nearest integer (banker's rounding, like `round`).
    pub fn with_grain(&self, g: Grain) -> DurationData {
        if g == self.grain {
            return self.clone();
        }
        let v = self.grain.in_seconds(self.value as f64) / g.in_seconds(1.0);
        DurationData { value: hs_round(v), grain: g }
    }

    /// Semigroup `<>`: sum in the finer grain.
    pub fn combine(&self, other: &DurationData) -> DurationData {
        let g = self.grain.min(other.grain);
        let a = self.with_grain(g);
        let b = other.with_grain(g);
        DurationData { value: a.value + b.value, grain: g }
    }
}

/// Haskell's `round :: Double -> Int` (round half to even).
pub fn hs_round(x: f64) -> i64 {
    let r = x.round();
    if (x - x.trunc()).abs() == 0.5 {
        // tie: choose the even neighbour
        let t = x.trunc();
        if (t as i64) % 2 == 0 {
            t as i64
        } else {
            r as i64
        }
    } else {
        r as i64
    }
}
