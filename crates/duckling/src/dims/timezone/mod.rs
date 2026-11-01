//! Opt-in location timezone dimension. Not part of upstream Duckling.

pub mod cities;
pub mod en;

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::dims::time::calendar::utc_from_wall;
use crate::json::Json;
use crate::time_zone::{TimeZone, TimeZoneSeries};
use crate::types::{Context, Options};

pub use cities::{rank_of, City, CITIES};

/// A city timezone token. Bare location tokens do not resolve; composed phrase tokens do.
#[derive(Clone, Debug)]
pub struct TimezoneData {
    pub city: &'static City,
    pub composed: bool,
}

impl PartialEq for TimezoneData {
    fn eq(&self, o: &TimezoneData) -> bool {
        self.city.iana == o.city.iana && self.city.label == o.city.label && self.composed == o.composed
    }
}
impl Eq for TimezoneData {}

impl Hash for TimezoneData {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.city.iana.hash(h);
        self.city.label.hash(h);
        self.composed.hash(h);
    }
}

impl TimezoneData {
    pub fn resolve(&self, ctx: &Context, _opts: &Options) -> Option<(Json, bool)> {
        if !self.composed {
            return None;
        }
        let off = offset_minutes(ctx, self.city);
        Some((
            Json::obj([
                ("timezone", Json::str(self.city.iana)),
                ("city", Json::str(self.city.label)),
                ("offset", Json::str(format_offset(off))),
            ]),
            false,
        ))
    }
}

pub fn format_offset(min: i32) -> String {
    let sign = if min < 0 { '-' } else { '+' };
    let t = min.unsigned_abs();
    format!("{sign}{:02}:{:02}", t / 60, t % 60)
}

pub fn series_for(ctx: &Context, city: &City) -> TimeZoneSeries {
    ctx.time_zones
        .as_ref()
        .and_then(|m| m.get(city.iana).cloned())
        .unwrap_or_else(|| TimeZoneSeries {
            default_zone: TimeZone {
                offset_minutes: city.fallback_offset_min,
                summer_only: false,
                name: city.iana.to_string(),
            },
            transitions: Vec::new(),
        })
}

/// City UTC offset at the request instant (real UTC, so TZif DST is correct).
pub fn offset_minutes(ctx: &Context, city: &City) -> i32 {
    let real = utc_from_wall(&ctx.reference_time.series, ctx.reference_time.utc);
    series_for(ctx, city).zone_at(real).offset_minutes
}

pub fn zones_used_by_cities(tzs: &std::collections::HashMap<String, TimeZoneSeries>) -> Arc<std::collections::HashMap<String, TimeZoneSeries>> {
    let mut m = std::collections::HashMap::new();
    for c in CITIES {
        if let Some(s) = tzs.get(c.iana) {
            m.insert(c.iana.to_string(), s.clone());
        }
    }
    Arc::new(m)
}
