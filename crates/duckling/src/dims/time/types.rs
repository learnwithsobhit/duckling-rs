//! `Duckling.Time.Types` (placeholder until the Time dimension is ported).

use std::hash::{Hash, Hasher};

use crate::json::Json;
use crate::types::{Context, Options};

#[derive(Clone, Debug)]
pub struct TimeData {
    pub latent: bool,
    pub time_grain: crate::dims::time_grain::Grain,
}

impl PartialEq for TimeData {
    fn eq(&self, other: &TimeData) -> bool {
        self.latent == other.latent
    }
}
impl Eq for TimeData {}
impl Hash for TimeData {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.latent.hash(h)
    }
}

impl TimeData {
    pub fn resolve(&self, _ctx: &Context, _opts: &Options) -> Option<(Json, bool)> {
        None
    }
}
