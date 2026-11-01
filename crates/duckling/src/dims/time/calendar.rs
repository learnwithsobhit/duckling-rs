//! Proleptic Gregorian calendar helpers matching the parts of `Data.Time.Calendar` Duckling uses.
//! Days are Modified Julian Day numbers (`i64`).

use crate::time_zone::{TimeZoneSeries, UtcTime, PSECS_PER_DAY};

pub const PSECS_PER_SEC: i64 = 1_000_000_000_000;

pub fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0) && ((y % 400 == 0) || (y % 100 != 0))
}

/// `gregorianMonthLength`.
pub fn month_length(y: i64, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if is_leap_year(y) {
                29
            } else {
                28
            }
        }
    }
}

const UNIX_EPOCH_MJD: i64 = 40587;

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `toGregorian`.
pub fn to_gregorian(mjd: i64) -> (i64, u32, u32) {
    civil_from_days(mjd - UNIX_EPOCH_MJD)
}

/// `fromGregorian` (clips month and day into range).
pub fn from_gregorian(y: i64, m: i64, d: i64) -> i64 {
    let m = m.clamp(1, 12) as u32;
    let d = d.clamp(1, month_length(y, m) as i64) as u32;
    days_from_civil(y, m, d) + UNIX_EPOCH_MJD
}

/// `fromGregorianValid`.
pub fn from_gregorian_valid(y: i64, m: i64, d: i64) -> Option<i64> {
    if !(1..=12).contains(&m) {
        return None;
    }
    if d < 1 || d > month_length(y, m as u32) as i64 {
        return None;
    }
    Some(days_from_civil(y, m as u32, d as u32) + UNIX_EPOCH_MJD)
}

/// `addGregorianMonthsClip`.
pub fn add_months_clip(mjd: i64, n: i64) -> i64 {
    let (y, m, d) = to_gregorian(mjd);
    let total = m as i64 + n;
    let y2 = y + (total - 1).div_euclid(12);
    let m2 = (total - 1).rem_euclid(12) + 1;
    from_gregorian(y2, m2, d as i64)
}

/// `addGregorianYearsClip`.
pub fn add_years_clip(mjd: i64, n: i64) -> i64 {
    add_months_clip(mjd, n * 12)
}

/// ISO day of week: Monday = 1 .. Sunday = 7.
pub fn day_of_week(mjd: i64) -> u32 {
    ((mjd + 2).rem_euclid(7) + 1) as u32
}

fn week_one_monday(iso_year: i64) -> i64 {
    let jan4 = from_gregorian(iso_year, 1, 4);
    jan4 - (day_of_week(jan4) as i64 - 1)
}

/// `toWeekDate`: (ISO year, ISO week, ISO day of week).
pub fn to_week_date(mjd: i64) -> (i64, u32, u32) {
    let (y, _, _) = to_gregorian(mjd);
    let mut iso_year = y + 1;
    while week_one_monday(iso_year) > mjd {
        iso_year -= 1;
    }
    let week = (mjd - week_one_monday(iso_year)) / 7 + 1;
    (iso_year, week as u32, day_of_week(mjd))
}

/// `fromWeekDate` (week and day are clipped).
pub fn from_week_date(iso_year: i64, week: i64, dow: i64) -> i64 {
    let weeks_in_year = {
        let next = week_one_monday(iso_year + 1);
        (next - week_one_monday(iso_year)) / 7
    };
    let w = week.clamp(1, weeks_in_year);
    let d = dow.clamp(1, 7);
    week_one_monday(iso_year) + (w - 1) * 7 + (d - 1)
}

// ----- UTCTime arithmetic ----------------------------------------------------------------

/// `addUTCTime` with a whole number of seconds.
pub fn add_seconds(t: UtcTime, secs: i128) -> UtcTime {
    let total = t.psecs as i128 + secs * PSECS_PER_SEC as i128;
    let days = total.div_euclid(PSECS_PER_DAY as i128);
    let ps = total.rem_euclid(PSECS_PER_DAY as i128);
    UtcTime { mjd: (t.mjd as i128 + days) as i64, psecs: ps as i64 }
}

/// Inverse of [`crate::service::make_reftime`]: Duckling stores wall clock as fake UTC
/// (`wall = utc + offset(utc)`). TZif transitions are UTC, so `zone_at(wall)` is the wrong
/// lookup near DST changes. Iterate until `offset(utc)` is consistent with the stored wall.
pub fn utc_from_wall(series: &TimeZoneSeries, wall: UtcTime) -> UtcTime {
    let mut off = series.zone_at(wall).offset_minutes as i128;
    for _ in 0..4 {
        let utc = add_seconds(wall, -off * 60);
        let next = series.zone_at(utc).offset_minutes as i128;
        if next == off {
            return utc;
        }
        off = next;
    }
    add_seconds(wall, -off * 60)
}

/// `diffUTCTime x y` in picoseconds.
pub fn diff_psecs(x: UtcTime, y: UtcTime) -> i128 {
    (x.mjd as i128 - y.mjd as i128) * PSECS_PER_DAY as i128 + (x.psecs as i128 - y.psecs as i128)
}

/// `Time.timeToTimeOfDay`: (hours, minutes, picoseconds within the minute).
pub fn time_of_day(psecs: i64) -> (i64, i64, i64) {
    let total_secs = psecs / PSECS_PER_SEC;
    let frac = psecs % PSECS_PER_SEC;
    let h = total_secs / 3600;
    let m = (total_secs / 60) % 60;
    let s = total_secs % 60;
    (h, m, s * PSECS_PER_SEC + frac)
}

pub fn time_of_day_to_psecs(h: i64, m: i64, s_ps: i64) -> i64 {
    (h * 3600 + m * 60) * PSECS_PER_SEC + s_ps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gregorian_roundtrip() {
        for mjd in [-100_000, 0, 40587, 51544, 60000, 80000] {
            let (y, m, d) = to_gregorian(mjd);
            assert_eq!(from_gregorian(y, m as i64, d as i64), mjd);
        }
        assert_eq!(to_gregorian(40587), (1970, 1, 1));
        assert_eq!(day_of_week(40587), 4); // Thursday
    }

    #[test]
    fn months_clip() {
        let jan31 = from_gregorian(2013, 1, 31);
        assert_eq!(to_gregorian(add_months_clip(jan31, 1)), (2013, 2, 28));
        assert_eq!(to_gregorian(add_months_clip(jan31, -2)), (2012, 11, 30));
        assert_eq!(to_gregorian(add_years_clip(from_gregorian(2012, 2, 29), 1)), (2013, 2, 28));
    }

    #[test]
    fn iso_weeks() {
        assert_eq!(to_week_date(from_gregorian(2012, 12, 31)), (2013, 1, 1));
        assert_eq!(to_week_date(from_gregorian(2016, 1, 3)), (2015, 53, 7));
        assert_eq!(to_week_date(from_gregorian(2013, 2, 12)), (2013, 7, 2));
        assert_eq!(from_week_date(2013, 7, 1), from_gregorian(2013, 2, 11));
    }
}
