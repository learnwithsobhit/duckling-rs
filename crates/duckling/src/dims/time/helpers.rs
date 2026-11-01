//! `Duckling.Time.Helpers`: time predicates, `TimeData` constructors and rule helpers.

use std::sync::Arc;

use super::calendar::from_gregorian_valid;
use super::tz_data::TIMEZONES;
use super::types::*;
use crate::dims::duration::DurationData;
use crate::dims::numeral::is_integer_between;
use crate::dims::time_grain::Grain;
use crate::time_zone::{TimeZone, TimeZoneSeries, UtcTime};
use crate::types::{regex, rule, Rule, Token};

pub type TimePredicate = super::types::Predicate;

pub fn get_int_value(t: &Token) -> Option<i64> {
    match t {
        Token::Numeral(n) => crate::dims::numeral::get_int_value(n.value),
        Token::Ordinal(o) => Some(o.value),
        _ => None,
    }
}

pub fn time_neg_period(d: &DurationData) -> DurationData {
    DurationData { value: -d.value, grain: d.grain }
}

pub fn time_shift_period(n: i64, d: &DurationData) -> DurationData {
    DurationData { value: d.value + n, grain: d.grain }
}

// ----- time predicates -------------------------------------------------------------------------------

/// `timeComputed`: a fixed, sorted list of days.
pub fn time_computed(days: &'static [(i64, u32, u32)]) -> TimePredicate {
    let xs: Vec<TimeObject> = days
        .iter()
        .filter_map(|&(y, m, d)| to_time_object_m(y, m as i64, d as i64))
        .collect();
    let xs = Arc::new(xs);
    Predicate::Series(series_fn(move |t, _| {
        let tt = *t;
        let mut start: Vec<TimeObject> = xs.iter().copied().take_while(|x| time_before(x, &tt)).collect();
        let n = start.len();
        let end: Vec<TimeObject> = xs[n..].to_vec();
        start.reverse();
        (Seq::from_vec(start), Seq::from_vec(end))
    }))
}

pub fn time_cycle(grain: Grain) -> TimePredicate {
    Predicate::Series(series_fn(move |t, _| time_sequence(grain, 1, time_round(t, grain))))
}

pub fn time_second(n: i64) -> TimePredicate {
    mk_second_predicate(n)
}
pub fn time_minute(n: i64) -> TimePredicate {
    mk_minute_predicate(n)
}
pub fn time_hour(is12h: bool, n: i64) -> TimePredicate {
    mk_hour_predicate(is12h, n)
}
pub fn time_day_of_week(n: i64) -> TimePredicate {
    mk_day_of_the_week_predicate(n)
}
pub fn time_day_of_month(n: i64) -> TimePredicate {
    mk_day_of_the_month_predicate(n)
}
pub fn time_month(n: i64) -> TimePredicate {
    mk_month_predicate(n)
}
pub fn time_year(n: i64) -> TimePredicate {
    mk_year_predicate(n)
}

/// `takeN`: takes `n` cycles of `f`.
pub fn take_n(n: i64, not_immediate: bool, f: &TimePredicate) -> TimePredicate {
    let run = run_predicate(f);
    Predicate::Series(series_fn(move |t, ctx| {
        let base = ctx.ref_time;
        let (past, future) = run(&base, ctx);
        let fut = match future.head() {
            Some(ahead) if not_immediate && time_intersect(&ahead, &base).is_some() => future.skip(1),
            _ => future.clone(),
        };
        let slot = if n >= 0 {
            match fut.head() {
                Some(start) => fut
                    .get(n as usize)
                    .map(|end| time_interval(TimeIntervalType::Open, &start, &end)),
                None => None,
            }
        } else {
            match past.head() {
                Some(end) => past
                    .get((-n - 1) as usize)
                    .map(|start| time_interval(TimeIntervalType::Closed, &start, &end)),
                None => None,
            }
        };
        match slot {
            Some(nth) => {
                if time_starts_before_the_end_of(t, &nth) {
                    (Seq::empty(), Seq::from_vec(vec![nth]))
                } else {
                    (Seq::from_vec(vec![nth]), Seq::empty())
                }
            }
            None => (Seq::empty(), Seq::empty()),
        }
    }))
}

/// `takeNth`: -1 is the first element in the past, 0 is the first element in the future.
pub fn take_nth(n: i64, not_immediate: bool, f: &TimePredicate) -> TimePredicate {
    let run = run_predicate(f);
    Predicate::Series(series_fn(move |t, ctx| {
        let base = ctx.ref_time;
        let (past, future) = run(&base, ctx);
        let rest = if n >= 0 {
            match future.head() {
                Some(ahead) if not_immediate && time_intersect(&ahead, &base).is_some() => {
                    future.skip((n + 1) as usize)
                }
                _ => future.skip(n as usize),
            }
        } else {
            past.skip((-(n + 1)) as usize)
        };
        match rest.head() {
            None => (Seq::empty(), Seq::empty()),
            Some(nth) => {
                if time_starts_before_the_end_of(t, &nth) {
                    (Seq::empty(), Seq::from_vec(vec![nth]))
                } else {
                    (Seq::from_vec(vec![nth]), Seq::empty())
                }
            }
        }
    }))
}

/// `takeNthAfter`: like `takeNth`, but takes the nth cyclic predicate after `base_pred`.
pub fn take_nth_after(n: i64, not_immediate: bool, cyclic: &TimePredicate, base: &TimePredicate) -> TimePredicate {
    let run = run_predicate(cyclic);
    let f: Arc<dyn Fn(&TimeObject, &TimeContext) -> Option<TimeObject> + Send + Sync> = Arc::new(move |t, ctx| {
        let (past, future) = run(t, ctx);
        let rest = if n >= 0 {
            match future.head() {
                Some(ahead) if not_immediate && time_before(&ahead, t) => future.skip((n + 1) as usize),
                _ => future.skip(n as usize),
            }
        } else {
            past.skip((-(n + 1)) as usize)
        };
        rest.head()
    });
    Predicate::Series(time_seq_map(false, f, base.clone()))
}

/// `takeNthClosest`: n = 0 is the closest value, n = 1 the second closest, etc.
pub fn take_nth_closest(n: i64, cyclic: &TimePredicate, base: &TimePredicate) -> TimePredicate {
    let run = run_predicate(cyclic);
    let f: Arc<dyn Fn(&TimeObject, &TimeContext) -> Option<TimeObject> + Send + Sync> = Arc::new(move |t, ctx| {
        let (mut pa, mut fu) = run(t, ctx);
        let mut m = n.max(0);
        let mut res: Option<TimeObject> = None;
        loop {
            if m < 0 {
                return res;
            }
            let x = pa.head();
            let y = fu.head();
            let against = |o: &Option<TimeObject>| o.as_ref().map(|o| -diff_start_time(t, o));
            if against(&x) > against(&y) {
                pa = pa.skip(1);
                res = x;
            } else {
                fu = fu.skip(1);
                res = y;
            }
            m -= 1;
        }
    });
    Predicate::Series(time_seq_map(false, f, base.clone()))
}

/// `takeLastOf`: the last occurrence of `cyclic` within `base`.
pub fn take_last_of(cyclic: &TimePredicate, base: &TimePredicate) -> TimePredicate {
    let run = run_predicate(cyclic);
    let f: Arc<dyn Fn(&TimeObject, &TimeContext) -> Option<TimeObject> + Send + Sync> = Arc::new(move |t, ctx| {
        let (past, _) = run(&time_starting_at_the_end_of(t), ctx);
        past.head()
    });
    Predicate::Series(time_seq_map(false, f, base.clone()))
}

pub fn time_compose(a: &TimePredicate, b: &TimePredicate) -> TimePredicate {
    mk_intersect_predicate(a, b)
}

pub fn add_duration(d: &DurationData, t: &TimeObject) -> TimeObject {
    time_plus(t, d.grain, d.value)
}

pub fn merge_duration(pred1: &TimePredicate, dd: &DurationData) -> TimePredicate {
    let dd = dd.clone();
    let f: Arc<dyn Fn(&TimeObject, &TimeContext) -> Option<TimeObject> + Send + Sync> = Arc::new(move |x, _| {
        let g = x.grain.min(dd.grain);
        let t2 = if g == x.grain { *x } else { time_round(x, g) };
        Some(add_duration(&dd, &t2))
    });
    Predicate::Series(time_seq_map(false, f, pred1.clone()))
}

pub fn shift_duration(pred1: &TimePredicate, dd: &DurationData) -> TimePredicate {
    let dd = dd.clone();
    let f: Arc<dyn Fn(&TimeObject, &TimeContext) -> Option<TimeObject> + Send + Sync> =
        Arc::new(move |x, _| Some(add_duration(&dd, &time_round(x, dd.grain.lower()))));
    Predicate::Series(time_seq_map(false, f, pred1.clone()))
}

pub fn shift_timezone(provided: TimeZoneSeries, pred1: &TimePredicate) -> TimePredicate {
    let f: Arc<dyn Fn(&TimeObject, &TimeContext) -> Option<TimeObject> + Send + Sync> = Arc::new(move |x, ctx| {
        let ctx_offset = ctx.tz_series.zone_at(x.start).offset_minutes as i64;
        let provided_offset = provided.zone_at(x.start).offset_minutes as i64;
        // This forgets about `end`, but it's OK since we act on time-of-days.
        Some(time_plus(x, Grain::Minute, ctx_offset - provided_offset))
    });
    Predicate::Series(time_seq_map(false, f, pred1.clone()))
}

// ----- patterns ---------------------------------------------------------------------------------------


pub fn is_grain_finer_than(value: Grain) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::Time(td) if td.time_grain < value)
}

pub fn is_grain_coarser_than(value: Grain) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::Time(td) if td.time_grain > value)
}

pub fn is_grain_of_time(value: Grain) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::Time(td) if td.time_grain == value)
}

pub fn same_grain(a: &TimeData, b: &TimeData) -> bool {
    a.time_grain == b.time_grain
}

pub fn has_timezone(t: &Token) -> bool {
    matches!(t, Token::Time(td) if td.has_timezone)
}

pub fn has_no_timezone(t: &Token) -> bool {
    !has_timezone(t)
}

pub fn is_a_day_of_week(t: &Token) -> bool {
    matches!(t, Token::Time(td) if td.form == Some(Form::DayOfWeek))
}

pub fn is_a_time_of_day(t: &Token) -> bool {
    matches!(t, Token::Time(td) if matches!(td.form, Some(Form::TimeOfDay { .. })))
}

pub fn is_a_part_of_day(t: &Token) -> bool {
    matches!(t, Token::Time(td) if td.form == Some(Form::PartOfDay))
}

pub fn is_a_month(t: &Token) -> bool {
    matches!(t, Token::Time(td) if matches!(td.form, Some(Form::Month(_))))
}

pub fn is_an_hour_of_day(t: &Token) -> bool {
    matches!(t, Token::Time(td)
        if matches!(td.form, Some(Form::TimeOfDay { hours: Some(_), .. })) && td.time_grain > Grain::Minute)
}

pub fn is_midnight_or_noon(t: &Token) -> bool {
    matches!(t, Token::Time(td)
        if matches!(td.form, Some(Form::TimeOfDay { hours: Some(x), .. }) if x == 0 || x == 12))
}

pub fn is_not_latent(t: &Token) -> bool {
    matches!(t, Token::Time(td) if !td.latent)
}

pub fn has_no_direction(t: &Token) -> bool {
    matches!(t, Token::Time(td) if td.direction.is_none())
}

pub fn is_integer_between_tok(low: i64, high: i64) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::Numeral(n) if n.ok_for_any_time && is_integer_between(n.value, low, high))
}

pub fn is_ordinal_between(low: i64, high: i64) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::Ordinal(o) if low <= o.value && o.value <= high)
}

pub fn is_duration_greater_than(value: Grain) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::Duration(d) if d.grain > value)
}

pub fn is_dom_ordinal(t: &Token) -> bool {
    is_ordinal_between(1, 31)(t)
}

pub fn is_dom_integer(t: &Token) -> bool {
    is_integer_between_tok(1, 31)(t)
}

pub fn is_dom_value(t: &Token) -> bool {
    is_dom_ordinal(t) || is_dom_integer(t)
}

pub fn is_ok_with_this_next(t: &Token) -> bool {
    matches!(t, Token::Time(td) if td.ok_for_this_next)
}

// ----- production ---------------------------------------------------------------------------------------

fn first_some<T>(a: Option<T>, b: Option<T>) -> Option<T> {
    a.or(b)
}

/// `intersect`: pass the interval second.
pub fn intersect(td1: &TimeData, td2: &TimeData) -> Option<TimeData> {
    let r = intersect_prime(td1, td2);
    if is_empty_predicate(&r.time_pred) {
        None
    } else {
        Some(r)
    }
}

pub fn intersect_with_replacement(a: &TimeData, b: &TimeData, c: &TimeData) -> Option<TimeData> {
    if a.time_grain == b.time_grain && b.time_grain == c.time_grain {
        Some(TimeData {
            time_pred: mk_replace_intersect_predicate(&a.time_pred, &b.time_pred, &c.time_pred),
            time_grain: a.time_grain,
            direction: None,
            holiday: first_some(first_some(a.holiday.clone(), b.holiday.clone()), c.holiday.clone()),
            ..timedata()
        })
    } else {
        None
    }
}

/// `intersect'`.
pub fn intersect_prime(a: &TimeData, b: &TimeData) -> TimeData {
    if a.time_grain < b.time_grain {
        TimeData {
            time_pred: time_compose(&a.time_pred, &b.time_pred),
            time_grain: a.time_grain,
            direction: first_some(a.direction, b.direction),
            holiday: first_some(a.holiday.clone(), b.holiday.clone()),
            ..timedata()
        }
    } else {
        TimeData {
            time_pred: time_compose(&b.time_pred, &a.time_pred),
            time_grain: b.time_grain,
            direction: first_some(a.direction, b.direction),
            holiday: first_some(a.holiday.clone(), b.holiday.clone()),
            ..timedata()
        }
    }
}

pub fn now() -> TimeData {
    TimeData { time_grain: Grain::NoGrain, ..cycle_nth(Grain::Second, 0) }
}

pub fn today() -> TimeData {
    cycle_nth(Grain::Day, 0)
}

pub fn hour(is12h: bool, n: i64) -> TimeData {
    time_of_day(Some(n), is12h, TimeData { time_pred: time_hour(is12h, n), time_grain: Grain::Hour, ..timedata() })
}

pub fn minute(n: i64) -> TimeData {
    TimeData { time_pred: time_minute(n), time_grain: Grain::Minute, ..timedata() }
}

pub fn second(n: i64) -> TimeData {
    TimeData { time_pred: time_second(n), time_grain: Grain::Second, ..timedata() }
}

pub fn day_of_week(n: i64) -> TimeData {
    form(
        Form::DayOfWeek,
        TimeData { time_pred: time_day_of_week(n), time_grain: Grain::Day, not_immediate: true, ..timedata() },
    )
}

pub fn day_of_month(n: i64) -> TimeData {
    TimeData { time_pred: time_day_of_month(n), time_grain: Grain::Day, ..timedata() }
}

pub fn month(n: i64) -> TimeData {
    form(Form::Month(n), TimeData { time_pred: time_month(n), time_grain: Grain::Month, ..timedata() })
}

/// Converts 2-digits to a year between 1950 and 2050.
pub fn year(n: i64) -> TimeData {
    let y = if n <= 99 { (n + 50).rem_euclid(100) + 2000 - 50 } else { n };
    TimeData { time_pred: time_year(y), time_grain: Grain::Year, ..timedata() }
}

pub fn year_adbc(n: i64) -> TimeData {
    TimeData { time_pred: time_year(n), time_grain: Grain::Year, ..timedata() }
}

pub fn year_month(y: i64, m: i64) -> TimeData {
    intersect_prime(&year(y), &month(m))
}

pub fn year_month_day(y: i64, m: i64, d: i64) -> TimeData {
    intersect_prime(&year_month(y, m), &day_of_month(d))
}

pub fn month_day(m: i64, d: i64) -> TimeData {
    intersect_prime(&month(m), &day_of_month(d))
}

pub fn hour_minute(is12h: bool, h: i64, m: i64) -> TimeData {
    time_of_day(Some(h), is12h, intersect_prime(&hour(is12h, h), &minute(m)))
}

pub fn hour_minute_second(is12h: bool, h: i64, m: i64, s: i64) -> TimeData {
    time_of_day(Some(h), is12h, intersect_prime(&intersect_prime(&hour(is12h, h), &minute(m)), &second(s)))
}

pub fn season() -> TimeData {
    TimeData { time_pred: season_predicate(), time_grain: Grain::Day, ..timedata() }
}

/// Note that this function is not the counterpart of `weekend`: `weekend` returns an interval
/// while `weekday` returns a single day.
pub fn weekday() -> TimeData {
    TimeData { time_pred: weekday_predicate(), time_grain: Grain::Day, ..timedata() }
}

pub fn cycle_n(not_immediate: bool, grain: Grain, n: i64) -> TimeData {
    TimeData { time_pred: take_n(n, not_immediate, &time_cycle(grain)), time_grain: grain, ..timedata() }
}

pub fn cycle_nth(grain: Grain, n: i64) -> TimeData {
    TimeData { time_pred: take_nth(n, false, &time_cycle(grain)), time_grain: grain, ..timedata() }
}

pub fn cycle_nth_after(not_immediate: bool, grain: Grain, n: i64, base: &TimeData) -> TimeData {
    TimeData {
        time_pred: take_nth_after(n, not_immediate, &time_cycle(grain), &base.time_pred),
        time_grain: grain,
        ..timedata()
    }
}

pub fn cycle_last_of(grain: Grain, base: &TimeData) -> TimeData {
    TimeData { time_pred: take_last_of(&time_cycle(grain), &base.time_pred), time_grain: grain, ..timedata() }
}

/// Generalized version of `cycleLastOf` with custom predicate.
pub fn pred_last_of(cyclic: &TimeData, base: &TimeData) -> TimeData {
    TimeData {
        time_pred: take_last_of(&cyclic.time_pred, &base.time_pred),
        time_grain: cyclic.time_grain,
        ..timedata()
    }
}

/// Generalized version of `cycleNth` with custom predicate.
pub fn pred_nth(n: i64, not_immediate: bool, td: &TimeData) -> TimeData {
    TimeData {
        time_pred: take_nth(n, not_immediate, &td.time_pred),
        time_grain: td.time_grain,
        holiday: td.holiday.clone(),
        ..timedata()
    }
}

/// Generalized version of `cycleNthAfter` with custom predicate.
pub fn pred_nth_after(n: i64, td: &TimeData, base: &TimeData) -> TimeData {
    TimeData {
        time_pred: take_nth_after(n, true, &td.time_pred, &base.time_pred),
        time_grain: td.time_grain,
        holiday: td.holiday.clone(),
        ..timedata()
    }
}

/// Expresses "closest" predicates, such as "the second closest Monday to Oct 5th".
pub fn pred_nth_closest(n: i64, td: &TimeData, base: &TimeData) -> TimeData {
    TimeData {
        time_pred: take_nth_closest(n, &td.time_pred, &base.time_pred),
        time_grain: td.time_grain,
        holiday: td.holiday.clone(),
        ..timedata()
    }
}

/// Periodic events, e.g. "every 365 days"; `given` is a known occurrence.
fn pred_every_from(period_grain: Grain, period: i64, given: TimeObject) -> TimeData {
    TimeData {
        time_pred: periodic_predicate(period_grain, period, given),
        time_grain: given.grain,
        ..timedata()
    }
}

pub fn pred_every_n_days_from(period: i64, given: (i64, i64, i64)) -> Option<TimeData> {
    let date = to_time_object_m(given.0, given.1, given.2)?;
    Some(pred_every_from(Grain::Day, period, date))
}

pub fn to_time_object_m(y: i64, m: i64, d: i64) -> Option<TimeObject> {
    let day = from_gregorian_valid(y, m, d)?;
    Some(TimeObject { start: UtcTime { mjd: day, psecs: 0 }, grain: Grain::Day, end: None })
}

/// `interval'`.
pub fn interval_prime(ty: TimeIntervalType, a: &TimeData, b: &TimeData) -> TimeData {
    let ty2 = if a.time_grain == b.time_grain && a.time_grain == Grain::Day { TimeIntervalType::Closed } else { ty };
    TimeData {
        time_pred: mk_time_intervals_predicate(ty2, &a.time_pred, &b.time_pred),
        time_grain: a.time_grain.min(b.time_grain),
        ..timedata()
    }
}

pub fn interval(ty: TimeIntervalType, a: &TimeData, b: &TimeData) -> Option<TimeData> {
    let r = interval_prime(ty, a, b);
    if is_empty_predicate(&r.time_pred) {
        None
    } else {
        Some(r)
    }
}

pub fn mk_ok_for_this_next(mut td: TimeData) -> TimeData {
    td.ok_for_this_next = true;
    td
}

pub fn duration_ago(dd: &DurationData) -> TimeData {
    in_duration(&time_neg_period(dd))
}

pub fn duration_interval_ago(dd: &DurationData) -> TimeData {
    in_duration_interval(&time_neg_period(dd))
}

pub fn duration_after(dd: &DurationData, td: &TimeData) -> TimeData {
    TimeData {
        time_pred: if td.time_grain == Grain::NoGrain {
            shift_duration(&td.time_pred, dd)
        } else {
            merge_duration(&td.time_pred, dd)
        },
        time_grain: dd.grain,
        ..timedata()
    }
}

pub fn duration_before(dd: &DurationData, td: &TimeData) -> TimeData {
    duration_after(&time_neg_period(dd), td)
}

pub fn in_duration(dd: &DurationData) -> TimeData {
    let t = take_nth(0, false, &time_cycle(Grain::Second));
    TimeData { time_pred: shift_duration(&t, dd), time_grain: dd.grain, ..timedata() }
}

pub fn in_duration_interval(dd: &DurationData) -> TimeData {
    interval_prime(TimeIntervalType::Open, &in_duration(dd), &in_duration(&time_shift_period(1, dd)))
}

pub fn parse_timezone(input: &str) -> Option<TimeZone> {
    TIMEZONES
        .iter()
        .find(|(n, _, _)| *n == input)
        .map(|(n, off, summer)| TimeZone { offset_minutes: *off, summer_only: *summer, name: n.to_string() })
}

pub fn in_timezone(input: &str, td: &TimeData) -> Option<TimeData> {
    let tz = parse_timezone(input)?;
    Some(TimeData {
        time_pred: shift_timezone(TimeZoneSeries { default_zone: tz, transitions: Vec::new() }, &td.time_pred),
        has_timezone: true,
        ..td.clone()
    })
}

pub fn with_holiday(n: &str, mut td: TimeData) -> TimeData {
    td.holiday = Some(Arc::from(n));
    td
}

pub fn mk_latent(mut td: TimeData) -> TimeData {
    td.latent = true;
    td
}

pub fn not_latent(mut td: TimeData) -> TimeData {
    td.latent = false;
    td
}

pub fn form(f: Form, mut td: TimeData) -> TimeData {
    td.form = Some(f);
    td
}

pub fn part_of_day(td: TimeData) -> TimeData {
    form(Form::PartOfDay, td)
}

pub fn time_of_day(h: Option<i64>, is12h: bool, td: TimeData) -> TimeData {
    form(Form::TimeOfDay { hours: h, is12h }, td)
}

pub fn time_of_day_ampm(is_am: bool, tod: &TimeData) -> TimeData {
    let ampm = TimeData {
        time_pred: mk_ampm_predicate(if is_am { AmPm::Am } else { AmPm::Pm }),
        time_grain: Grain::Hour,
        ..timedata()
    };
    time_of_day(None, false, intersect_prime(tod, &ampm))
}

pub fn with_direction(dir: IntervalDirection, mut td: TimeData) -> TimeData {
    td.direction = Some(dir);
    td
}

pub fn long_we_before(monday: &TimeData) -> TimeData {
    let fri = cycle_nth_after(false, Grain::Day, -3, monday);
    let tue = cycle_nth_after(false, Grain::Day, 1, monday);
    let start = intersect_prime(&fri, &hour(false, 18));
    let end = intersect_prime(&tue, &hour(false, 0));
    interval_prime(TimeIntervalType::Open, &start, &end)
}

pub fn weekend() -> TimeData {
    let fri = intersect_prime(&day_of_week(5), &hour(false, 18));
    let mon = intersect_prime(&day_of_week(1), &hour(false, 0));
    interval_prime(TimeIntervalType::Open, &fri, &mon)
}

pub fn workweek() -> TimeData {
    let mon = intersect_prime(&day_of_week(1), &hour(false, 10));
    let fri = intersect_prime(&day_of_week(5), &hour(false, 18));
    interval_prime(TimeIntervalType::Open, &mon, &fri)
}

/// Zero-indexed weeks, Monday is 1. Use `pred_last_of` for the last day of week of the month.
pub fn nth_dow_of_month(n: i64, dow: i64, m: i64) -> TimeData {
    pred_nth_after(n - 1, &day_of_week(dow), &month(m))
}

pub fn intersect_dom(td: &TimeData, token: &Token) -> Option<TimeData> {
    let n = get_int_value(token)?;
    intersect(&day_of_month(n), td)
}

pub fn minutes_before(n: i64, td: &TimeData) -> Option<TimeData> {
    match td.form {
        Some(Form::TimeOfDay { hours: Some(0), is12h }) => Some(hour_minute(is12h, 23, 60 - n)),
        Some(Form::TimeOfDay { hours: Some(1), is12h: true }) => Some(hour_minute(true, 12, 60 - n)),
        Some(Form::TimeOfDay { hours: Some(h), is12h }) => Some(hour_minute(is12h, h - 1, 60 - n)),
        _ => None,
    }
}

pub fn minutes_after(n: i64, td: &TimeData) -> Option<TimeData> {
    match td.form {
        Some(Form::TimeOfDay { hours: Some(h), is12h }) => Some(hour_minute(is12h, h, n)),
        _ => None,
    }
}

/// `tt`: a time token.
pub fn tt(td: TimeData) -> Option<Token> {
    Some(Token::Time(td))
}

// ----- rule constructors -----------------------------------------------------------------------------------

pub fn mk_single_regex_rule(name: &str, pattern: &str, td: Option<TimeData>) -> Rule {
    rule(name, vec![regex(pattern)], move |_| td.clone().map(Token::Time))
}

pub fn mk_rule_instants(items: &[(&str, Grain, i64, &str)]) -> Vec<Rule> {
    items
        .iter()
        .map(|(name, grain, n, pat)| mk_single_regex_rule(name, pat, Some(cycle_nth(*grain, *n))))
        .collect()
}

pub fn mk_rule_days_of_week(days: &[(&str, &str)]) -> Vec<Rule> {
    days.iter()
        .zip(1..=7)
        .map(|((name, pat), i)| mk_single_regex_rule(name, pat, Some(mk_ok_for_this_next(day_of_week(i)))))
        .collect()
}

pub fn mk_rule_days_of_week_latent(days: &[(&str, &str)]) -> Vec<Rule> {
    days.iter()
        .zip(1..=7)
        .map(|((name, pat), i)| {
            mk_single_regex_rule(name, pat, Some(mk_latent(mk_ok_for_this_next(day_of_week(i)))))
        })
        .collect()
}

pub fn mk_rule_months(months: &[(&str, &str)]) -> Vec<Rule> {
    let v: Vec<(&str, &str, bool)> = months.iter().map(|(n, p)| (*n, *p, false)).collect();
    mk_rule_months_with_latent(&v)
}

pub fn mk_rule_months_with_latent(months: &[(&str, &str, bool)]) -> Vec<Rule> {
    months
        .iter()
        .zip(1..=12)
        .map(|((name, pat, latent), i)| {
            let td = mk_ok_for_this_next(month(i));
            mk_single_regex_rule(name, pat, Some(if *latent { mk_latent(td) } else { td }))
        })
        .collect()
}

pub fn mk_rule_seasons(items: &[(&str, &str, TimeData, TimeData)]) -> Vec<Rule> {
    items
        .iter()
        .map(|(name, pat, start, end)| {
            mk_single_regex_rule(name, pat, interval(TimeIntervalType::Open, start, end).map(mk_ok_for_this_next))
        })
        .collect()
}

pub fn mk_rule_holidays(items: &[(&str, &str, TimeData)]) -> Vec<Rule> {
    items
        .iter()
        .map(|(name, pat, td)| {
            mk_single_regex_rule(name, pat, Some(with_holiday(name, mk_ok_for_this_next(td.clone()))))
        })
        .collect()
}

pub fn mk_rule_holidays_maybe(items: &[(&str, &str, Option<TimeData>)]) -> Vec<Rule> {
    items
        .iter()
        .map(|(name, pat, td)| {
            mk_single_regex_rule(name, pat, td.clone().map(|td| with_holiday(name, mk_ok_for_this_next(td))))
        })
        .collect()
}
