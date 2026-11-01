//! Holiday definitions that upstream writes with `do` blocks
//! (`Duckling.Time.EN.AU.Rules`).

use super::helpers::*;
use super::types::{TimeData, TimeIntervalType::Open};
use crate::dims::time_grain::Grain as G;

fn ekka_start() -> Option<TimeData> {
    let tentative = nth_dow_of_month(1, 5, 8);
    let alternative = nth_dow_of_month(2, 5, 8);
    let forbidden = interval(Open, &month_day(8, 1), &month_day(8, 4))?;
    intersect_with_replacement(&forbidden, &tentative, &alternative)
}

/// "Royal Queensland Show Day": the Wednesday of the show.
pub fn royal_queensland_show_day() -> Option<TimeData> {
    ekka_start().map(|start| cycle_nth_after(false, G::Day, 5, &start))
}

/// "Royal Queensland Show": the nine days of the show.
pub fn royal_queensland_show() -> Option<TimeData> {
    let start = ekka_start()?;
    interval(Open, &start, &cycle_nth_after(false, G::Day, 9, &start))
}
