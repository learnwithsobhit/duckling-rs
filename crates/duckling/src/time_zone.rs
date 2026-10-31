//! Reference time and time zone series (the parts of `Data.Time` and `timezone-series`
//! that Duckling relies on).

/// Haskell's `UTCTime`: a Modified Julian Day plus picoseconds since midnight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcTime {
    pub mjd: i64,
    pub psecs: i64,
}

pub const PSECS_PER_DAY: i64 = 86_400 * 1_000_000_000_000;
/// MJD of 1970-01-01.
pub const UNIX_EPOCH_MJD: i64 = 40587;

impl UtcTime {
    pub fn new(mjd: i64, psecs: i64) -> UtcTime {
        UtcTime { mjd, psecs }
    }

    /// From milliseconds since the Unix epoch.
    pub fn from_unix_millis(ms: i64) -> UtcTime {
        let secs = ms.div_euclid(1000);
        let rem_ms = ms.rem_euclid(1000);
        let days = secs.div_euclid(86_400);
        let sod = secs.rem_euclid(86_400);
        UtcTime { mjd: UNIX_EPOCH_MJD + days, psecs: sod * 1_000_000_000_000 + rem_ms * 1_000_000_000 }
    }

    pub fn to_unix_millis(self) -> i64 {
        (self.mjd - UNIX_EPOCH_MJD) * 86_400_000 + self.psecs / 1_000_000_000
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TimeZone {
    pub offset_minutes: i32,
    pub summer_only: bool,
    pub name: String,
}

impl TimeZone {
    pub fn fixed(offset_minutes: i32) -> TimeZone {
        TimeZone { offset_minutes, summer_only: false, name: String::new() }
    }
}

/// `TimeZoneSeries`: an initial zone plus transitions, latest first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeZoneSeries {
    pub default_zone: TimeZone,
    pub transitions: Vec<(UtcTime, TimeZone)>,
}

impl TimeZoneSeries {
    pub fn fixed(offset_minutes: i32) -> TimeZoneSeries {
        TimeZoneSeries { default_zone: TimeZone::fixed(offset_minutes), transitions: Vec::new() }
    }

    /// `timeZoneFromSeries`: the zone in force at `t`.
    pub fn zone_at(&self, t: UtcTime) -> &TimeZone {
        self.transitions
            .iter()
            .find(|(at, _)| *at <= t)
            .map(|(_, z)| z)
            .unwrap_or(&self.default_zone)
    }
}

/// `DucklingTime`: an instant plus the zone series used to interpret it locally.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DucklingTime {
    pub utc: UtcTime,
    pub series: TimeZoneSeries,
}
