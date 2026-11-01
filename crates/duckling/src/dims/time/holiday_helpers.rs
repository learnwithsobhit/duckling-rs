//! `Duckling.Time.HolidayHelpers`.

use super::computed::easter_sunday;
use super::helpers::*;
use super::types::{TimeData, TimeIntervalType::Closed};
use crate::dims::time_grain::Grain as G;

/// Last Saturday of March unless it falls on Holy Saturday,
/// in which case it's the Saturday before.
pub fn compute_earth_hour() -> Option<TimeData> {
    let holy_saturday = cycle_nth_after(false, G::Day, -1, &easter_sunday());
    let tentative = pred_last_of(&day_of_week(6), &month(3));
    let alternative = cycle_nth_after(false, G::Day, -7, &tentative);
    let day = intersect_with_replacement(&holy_saturday, &tentative, &alternative)?;
    let start = intersect(&day, &hour_minute(true, 20, 30))?;
    interval(Closed, &start, &cycle_nth_after(false, G::Minute, 60, &start))
}

/// King's day is on April 27th unless it's on Sunday, in which case it's a day
/// earlier.
pub fn compute_kings_day() -> Option<TimeData> {
    let tentative = month_day(4, 27);
    let alternative = month_day(4, 26);
    let last_sunday_of_april = pred_last_of(&day_of_week(7), &month(4));
    intersect_with_replacement(&last_sunday_of_april, &tentative, &alternative)
}
