//! `Duckling.Time.Types`: time objects, series predicates, `TimeData` and its resolution.
//!
//! The Haskell implementation is built on lazy, memoised, possibly infinite lists. [`Seq`] mirrors
//! that: it is a cheap-to-clone, memoised lazy list of [`TimeObject`]s.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use super::calendar::*;
use crate::dims::time_grain::Grain;
use crate::json::Json;
use crate::time_zone::{TimeZoneSeries, UtcTime};
use crate::types::{Context, Options};

// ----- lazy lists ----------------------------------------------------------------------------

struct Shared {
    buf: RefCell<Vec<TimeObject>>,
    src: RefCell<Option<Box<dyn Iterator<Item = TimeObject>>>>,
}

/// A memoised lazy list (a suffix view into shared storage).
#[derive(Clone)]
pub struct Seq {
    shared: Rc<Shared>,
    pos: usize,
}

impl Seq {
    pub fn new(it: impl Iterator<Item = TimeObject> + 'static) -> Seq {
        Seq {
            shared: Rc::new(Shared { buf: RefCell::new(Vec::new()), src: RefCell::new(Some(Box::new(it))) }),
            pos: 0,
        }
    }

    pub fn empty() -> Seq {
        Seq::from_vec(Vec::new())
    }

    pub fn from_vec(v: Vec<TimeObject>) -> Seq {
        Seq { shared: Rc::new(Shared { buf: RefCell::new(v), src: RefCell::new(None) }), pos: 0 }
    }

    /// Element `i` of this list, forcing as much of it as needed.
    pub fn get(&self, i: usize) -> Option<TimeObject> {
        let idx = self.pos + i;
        loop {
            if let Some(x) = self.shared.buf.borrow().get(idx) {
                return Some(*x);
            }
            let mut src = self.shared.src.borrow_mut();
            match src.as_mut() {
                None => return None,
                Some(it) => match it.next() {
                    Some(x) => self.shared.buf.borrow_mut().push(x),
                    None => {
                        *src = None;
                        return None;
                    }
                },
            }
        }
    }

    pub fn head(&self) -> Option<TimeObject> {
        self.get(0)
    }

    /// `drop n` (lazy).
    pub fn skip(&self, n: usize) -> Seq {
        Seq { shared: self.shared.clone(), pos: self.pos + n }
    }

    pub fn iter(&self) -> SeqIter {
        SeqIter { seq: self.clone(), i: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.get(0).is_none()
    }

    /// `span p xs`: the longest prefix satisfying `p` and the remainder (forced up to the first
    /// failing element only).
    pub fn span(&self, p: impl Fn(&TimeObject) -> bool) -> (Vec<TimeObject>, Seq) {
        let mut prefix = Vec::new();
        let mut i = 0;
        while let Some(x) = self.get(i) {
            if p(&x) {
                prefix.push(x);
                i += 1;
            } else {
                break;
            }
        }
        (prefix, self.skip(i))
    }

    /// `takeWhile`, lazily.
    pub fn take_while(&self, p: impl Fn(&TimeObject) -> bool + 'static) -> Seq {
        Seq::new(self.iter().take_while(move |x| p(x)))
    }

    /// `take n`, lazily.
    pub fn take(&self, n: usize) -> Seq {
        Seq::new(self.iter().take(n))
    }

    /// `prefix ++ self`.
    pub fn prepend(&self, prefix: Vec<TimeObject>) -> Seq {
        Seq::new(prefix.into_iter().chain(self.iter()))
    }

    pub fn to_vec(&self, n: usize) -> Vec<TimeObject> {
        self.iter().take(n).collect()
    }
}

pub struct SeqIter {
    seq: Seq,
    i: usize,
}

impl Iterator for SeqIter {
    type Item = TimeObject;
    fn next(&mut self) -> Option<TimeObject> {
        let r = self.seq.get(self.i);
        if r.is_some() {
            self.i += 1;
        }
        r
    }
}

// ----- time objects ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeObject {
    pub start: UtcTime,
    pub grain: Grain,
    pub end: Option<UtcTime>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeIntervalType {
    Open,
    Closed,
}

#[derive(Clone)]
pub struct TimeContext {
    pub ref_time: TimeObject,
    pub tz_series: Arc<TimeZoneSeries>,
    pub max_time: TimeObject,
    pub min_time: TimeObject,
}

fn update_day(t: UtcTime, f: impl Fn(i64) -> i64) -> UtcTime {
    UtcTime { mjd: f(t.mjd), psecs: t.psecs }
}

/// `TG.add`.
pub fn grain_add(t: UtcTime, g: Grain, n: i64) -> UtcTime {
    let n128 = n as i128;
    match g {
        Grain::NoGrain | Grain::Second => add_seconds(t, n128),
        Grain::Minute => add_seconds(t, 60 * n128),
        Grain::Hour => add_seconds(t, 3600 * n128),
        Grain::Day => update_day(t, |d| d + n),
        Grain::Week => update_day(t, |d| d + 7 * n),
        Grain::Month => update_day(t, |d| add_months_clip(d, n)),
        Grain::Quarter => update_day(t, |d| add_months_clip(d, 3 * n)),
        Grain::Year => update_day(t, |d| add_years_clip(d, n)),
    }
}

pub fn time_end(t: &TimeObject) -> UtcTime {
    t.end.unwrap_or_else(|| grain_add(t.start, t.grain, 1))
}

pub fn time_plus(t: &TimeObject, g: Grain, n: i64) -> TimeObject {
    TimeObject { start: grain_add(t.start, g, n), grain: t.grain.min(g), end: None }
}

pub fn time_plus_end(t: &TimeObject, g: Grain, n: i64) -> TimeObject {
    TimeObject {
        start: grain_add(t.start, g, n),
        grain: t.grain.min(g),
        end: t.end.map(|e| grain_add(e, g, n)),
    }
}

pub fn time_starting_at_the_end_of(t: &TimeObject) -> TimeObject {
    TimeObject { start: time_end(t), end: None, grain: t.grain }
}

pub fn time_round(t: &TimeObject, grain: Grain) -> TimeObject {
    match grain {
        Grain::Week => {
            let day_rounded = time_round(t, Grain::Day);
            let (year, week, _) = to_week_date(day_rounded.start.mjd);
            let new_day = from_week_date(year, week as i64, 1);
            TimeObject {
                start: UtcTime { mjd: new_day, psecs: day_rounded.start.psecs },
                grain: Grain::Week,
                end: None,
            }
        }
        Grain::Quarter => {
            let month_time = time_round(t, Grain::Month);
            let (_, month, _) = to_gregorian(month_time.start.mjd);
            let mut new_time = time_plus(&month_time, Grain::Month, -(((month as i64) - 1).rem_euclid(3)));
            new_time.grain = Grain::Quarter;
            new_time
        }
        _ => {
            let (year, month, dom) = to_gregorian(t.start.mjd);
            let (hours, mins, secs) = time_of_day(t.start.psecs);
            let new_month = if grain > Grain::Month { 1 } else { month as i64 };
            let new_dom = if grain > Grain::Day { 1 } else { dom as i64 };
            let new_day = from_gregorian(year, new_month, new_dom);
            let new_hours = if grain > Grain::Hour { 0 } else { hours };
            let new_mins = if grain > Grain::Minute { 0 } else { mins };
            let new_secs = if grain > Grain::Second { 0 } else { secs };
            TimeObject {
                start: UtcTime { mjd: new_day, psecs: time_of_day_to_psecs(new_hours, new_mins, new_secs) },
                grain,
                end: None,
            }
        }
    }
}

pub fn time_interval(ty: TimeIntervalType, a: &TimeObject, b: &TimeObject) -> TimeObject {
    let g = a.grain.min(b.grain);
    let g2 = if a.grain < Grain::Day && b.grain < Grain::Day { g } else { b.grain };
    TimeObject {
        start: a.start,
        grain: g,
        end: Some(match ty {
            TimeIntervalType::Open => b.start,
            TimeIntervalType::Closed => b.end.unwrap_or_else(|| grain_add(b.start, g2, 1)),
        }),
    }
}

pub fn time_starts_before_the_end_of(t1: &TimeObject, t2: &TimeObject) -> bool {
    t1.start < time_end(t2)
}

pub fn time_before(t1: &TimeObject, t2: &TimeObject) -> bool {
    t1.start < t2.start
}

pub fn time_intersect(t1: &TimeObject, t2: &TimeObject) -> Option<TimeObject> {
    let (t1, t2) = if t1.start > t2.start { (t2, t1) } else { (t1, t2) };
    let (s1, s2) = (t1.start, t2.start);
    let e1 = time_end(t1);
    let e2 = time_end(t2);
    let g = t1.grain.min(t2.grain);
    if e1 <= s2 {
        None
    } else if e1 < e2 || (s1 == s2 && e1 == e2 && t1.end.is_some()) {
        Some(TimeObject { start: s2, end: t1.end, grain: g })
    } else {
        Some(TimeObject { grain: g, ..*t2 })
    }
}

/// `diffStartTime`: absolute difference in picoseconds.
pub fn diff_start_time(x: &TimeObject, y: &TimeObject) -> i128 {
    diff_psecs(x.start, y.start).abs()
}

fn to_midnight(mjd: i64) -> UtcTime {
    UtcTime { mjd, psecs: 0 }
}

// ----- predicates ----------------------------------------------------------------------------------

pub type SeriesFn = Arc<dyn Fn(&TimeObject, &TimeContext) -> (Seq, Seq) + Send + Sync>;

pub fn series_fn(f: impl Fn(&TimeObject, &TimeContext) -> (Seq, Seq) + Send + Sync + 'static) -> SeriesFn {
    Arc::new(f)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmPm {
    Am,
    Pm,
}

#[derive(Clone, Default)]
pub struct TimeDatePred {
    pub second: Option<i64>,
    pub minute: Option<i64>,
    pub hour: Option<(bool, i64)>,
    pub ampm: Option<AmPm>,
    pub day_of_week: Option<i64>,
    pub day_of_month: Option<i64>,
    pub month: Option<i64>,
    pub year: Option<i64>,
}

#[derive(Clone)]
pub enum Predicate {
    Series(SeriesFn),
    Empty,
    TimeDate(TimeDatePred),
    Intersect(Arc<Predicate>, Arc<Predicate>),
    TimeIntervals(TimeIntervalType, Arc<Predicate>, Arc<Predicate>),
    ReplaceIntersect(Arc<Predicate>, Arc<Predicate>, Arc<Predicate>),
}

pub fn mk_empty_predicate() -> Predicate {
    Predicate::Empty
}

pub fn mk_series_predicate(f: SeriesFn) -> Predicate {
    Predicate::Series(f)
}

fn td_pred(f: impl FnOnce(&mut TimeDatePred)) -> Predicate {
    let mut p = TimeDatePred::default();
    f(&mut p);
    Predicate::TimeDate(p)
}

pub fn mk_second_predicate(n: i64) -> Predicate {
    td_pred(|p| p.second = Some(n))
}
pub fn mk_minute_predicate(n: i64) -> Predicate {
    td_pred(|p| p.minute = Some(n))
}
pub fn mk_hour_predicate(is12h: bool, h: i64) -> Predicate {
    td_pred(|p| p.hour = Some((is12h, h)))
}
pub fn mk_ampm_predicate(a: AmPm) -> Predicate {
    td_pred(|p| p.ampm = Some(a))
}
pub fn mk_day_of_the_week_predicate(n: i64) -> Predicate {
    td_pred(|p| p.day_of_week = Some(n))
}
pub fn mk_day_of_the_month_predicate(n: i64) -> Predicate {
    td_pred(|p| p.day_of_month = Some(n))
}
pub fn mk_month_predicate(n: i64) -> Predicate {
    td_pred(|p| p.month = Some(n))
}
pub fn mk_year_predicate(n: i64) -> Predicate {
    td_pred(|p| p.year = Some(n))
}

fn unify<T: PartialEq + Copy>(a: Option<T>, b: Option<T>) -> Option<Option<T>> {
    match (a, b) {
        (None, b) => Some(b),
        (a, None) => Some(a),
        (Some(x), Some(y)) => {
            if x == y {
                Some(Some(x))
            } else {
                None
            }
        }
    }
}

fn unify_td(a: &TimeDatePred, b: &TimeDatePred) -> Option<TimeDatePred> {
    Some(TimeDatePred {
        second: unify(a.second, b.second)?,
        minute: unify(a.minute, b.minute)?,
        hour: unify(a.hour, b.hour)?,
        ampm: unify(a.ampm, b.ampm)?,
        day_of_week: unify(a.day_of_week, b.day_of_week)?,
        day_of_month: unify(a.day_of_month, b.day_of_month)?,
        month: unify(a.month, b.month)?,
        year: unify(a.year, b.year)?,
    })
}

pub fn mk_intersect_predicate(a: &Predicate, b: &Predicate) -> Predicate {
    match (a, b) {
        (Predicate::Empty, _) => Predicate::Empty,
        (_, Predicate::Empty) => Predicate::Empty,
        (Predicate::TimeDate(x), Predicate::TimeDate(y)) => match unify_td(x, y) {
            Some(p) => Predicate::TimeDate(p),
            None => Predicate::Empty,
        },
        _ => Predicate::Intersect(Arc::new(a.clone()), Arc::new(b.clone())),
    }
}

pub fn mk_replace_intersect_predicate(a: &Predicate, b: &Predicate, c: &Predicate) -> Predicate {
    Predicate::ReplaceIntersect(Arc::new(a.clone()), Arc::new(b.clone()), Arc::new(c.clone()))
}

fn contains_time_intervals_predicate(p: &Predicate) -> bool {
    match p {
        Predicate::TimeIntervals(..) => true,
        Predicate::Intersect(a, b) => contains_time_intervals_predicate(a) || contains_time_intervals_predicate(b),
        _ => false,
    }
}

pub fn mk_time_intervals_predicate(ty: TimeIntervalType, a: &Predicate, b: &Predicate) -> Predicate {
    if matches!(a, Predicate::Empty) {
        return Predicate::Empty;
    }
    if matches!(b, Predicate::Empty) {
        return Predicate::Empty;
    }
    if contains_time_intervals_predicate(a) || contains_time_intervals_predicate(b) {
        return Predicate::Empty;
    }
    Predicate::TimeIntervals(ty, Arc::new(a.clone()), Arc::new(b.clone()))
}

pub fn is_empty_predicate(p: &Predicate) -> bool {
    matches!(p, Predicate::Empty)
}

// ----- seasons / weekdays / periodic -----------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SeasonName {
    Spring,
    Summer,
    Fall,
    Winter,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Season {
    start_year: i64,
    name: SeasonName,
}

fn season_start(s: Season) -> i64 {
    match s.name {
        SeasonName::Spring => from_gregorian(s.start_year, 3, 20),
        SeasonName::Summer => from_gregorian(s.start_year, 6, 21),
        SeasonName::Fall => from_gregorian(s.start_year, 9, 23),
        SeasonName::Winter => from_gregorian(s.start_year, 12, 21),
    }
}

fn next_season(s: Season) -> Season {
    match s.name {
        SeasonName::Winter => Season { start_year: s.start_year + 1, name: SeasonName::Spring },
        SeasonName::Spring => Season { start_year: s.start_year, name: SeasonName::Summer },
        SeasonName::Summer => Season { start_year: s.start_year, name: SeasonName::Fall },
        SeasonName::Fall => Season { start_year: s.start_year, name: SeasonName::Winter },
    }
}

fn prev_season(s: Season) -> Season {
    match s.name {
        SeasonName::Spring => Season { start_year: s.start_year - 1, name: SeasonName::Winter },
        SeasonName::Summer => Season { start_year: s.start_year, name: SeasonName::Spring },
        SeasonName::Fall => Season { start_year: s.start_year, name: SeasonName::Summer },
        SeasonName::Winter => Season { start_year: s.start_year, name: SeasonName::Fall },
    }
}

fn season_end(s: Season) -> i64 {
    season_start(next_season(s)) - 1
}

fn season_of(day: i64) -> Season {
    let (year, _, _) = to_gregorian(day);
    for name in [SeasonName::Winter, SeasonName::Fall, SeasonName::Summer, SeasonName::Spring] {
        let s = Season { start_year: year, name };
        if season_start(s) <= day {
            return s;
        }
    }
    Season { start_year: year - 1, name: SeasonName::Winter }
}

fn season_to_obj(s: Season) -> TimeObject {
    TimeObject {
        start: to_midnight(season_start(s)),
        grain: Grain::Day,
        end: Some(to_midnight(season_end(s))),
    }
}

pub fn season_predicate() -> Predicate {
    Predicate::Series(series_fn(|t, _| {
        let day = t.start.mjd;
        let current = season_of(day);
        let future = Seq::new(std::iter::successors(Some(current), |s| Some(next_season(*s))).map(season_to_obj));
        let past = Seq::new(
            std::iter::successors(Some(prev_season(current)), |s| Some(prev_season(*s))).map(season_to_obj),
        );
        (past, future)
    }))
}

pub fn weekday_predicate() -> Predicate {
    fn next_weekday((d, dow): (i64, u32)) -> (i64, u32) {
        if dow < 5 {
            (d + 1, dow + 1)
        } else {
            (d + (8 - dow as i64), 1)
        }
    }
    fn prev_weekday((d, dow): (i64, u32)) -> (i64, u32) {
        if dow == 1 {
            (d - 3, 5)
        } else if dow == 7 {
            (d - 2, 5)
        } else {
            (d - 1, dow - 1)
        }
    }
    fn obj(d: i64) -> TimeObject {
        TimeObject { start: to_midnight(d), grain: Grain::Day, end: None }
    }
    Predicate::Series(series_fn(|time, _| {
        let day = time.start.mjd;
        let dow = day_of_week(day);
        let past =
            Seq::new(std::iter::successors(Some(prev_weekday((day, dow))), |x| Some(prev_weekday(*x))).map(|(d, _)| obj(d)));
        let days = std::iter::successors(Some(next_weekday((day, dow))), |x| Some(next_weekday(*x))).map(|(d, _)| d);
        let future = if dow <= 5 {
            Seq::new(std::iter::once(day).chain(days).map(obj))
        } else {
            Seq::new(days.map(obj))
        };
        (past, future)
    }))
}

pub fn time_sequence(grain: Grain, step: i64, anchor: TimeObject) -> (Seq, Seq) {
    let past = Seq::new(
        std::iter::successors(Some(time_plus(&anchor, grain, -step)), move |t| Some(time_plus(t, grain, -step))),
    );
    let future = Seq::new(std::iter::successors(Some(anchor), move |t| Some(time_plus(t, grain, step))));
    (past, future)
}

pub fn periodic_predicate(grain: Grain, delta: i64, given: TimeObject) -> Predicate {
    Predicate::Series(series_fn(move |t, _| {
        let (past, future) = time_sequence(grain, delta, given);
        if time_before(t, &given) {
            let t = *t;
            let (newer, older) = past.span(move |x| time_before(&t, x));
            let mut newer = newer;
            newer.reverse();
            (older, future.prepend(newer))
        } else {
            let t = *t;
            let (older, newer) = future.span(move |x| time_before(x, &t));
            let mut older = older;
            older.reverse();
            (past.prepend(older), newer)
        }
    }))
}

// ----- predicate runners -----------------------------------------------------------------------------

fn run_second_predicate(n: i64) -> SeriesFn {
    series_fn(move |t, _| {
        let (_, _, s_ps) = time_of_day(t.start.psecs);
        let s = s_ps / PSECS_PER_SEC;
        let anchor = time_plus(&time_round(t, Grain::Second), Grain::Second, (n - s).rem_euclid(60));
        time_sequence(Grain::Minute, 1, anchor)
    })
}

fn run_minute_predicate(n: i64) -> SeriesFn {
    series_fn(move |t, _| {
        let (_, m, _) = time_of_day(t.start.psecs);
        let rounded = time_round(t, Grain::Minute);
        let anchor = time_plus(&rounded, Grain::Minute, (n - m).rem_euclid(60));
        time_sequence(Grain::Hour, 1, anchor)
    })
}

fn run_hour_predicate(ampm: Option<AmPm>, is12h: bool, n: i64) -> SeriesFn {
    series_fn(move |time, _| {
        let (h, _, _) = time_of_day(time.start.psecs);
        let step: i64 = if is12h && n <= 12 && ampm.is_none() { 12 } else { 24 };
        let n2 = match ampm {
            Some(AmPm::Am) => n.rem_euclid(12),
            Some(AmPm::Pm) => n.rem_euclid(12) + 12,
            None => n,
        };
        let rounded = time_round(time, Grain::Hour);
        let anchor = time_plus(&rounded, Grain::Hour, (n2 - h).rem_euclid(step));
        time_sequence(Grain::Hour, step, anchor)
    })
}

fn run_day_of_the_week_predicate(n: i64) -> SeriesFn {
    series_fn(move |t, _| {
        let dow = day_of_week(t.start.mjd) as i64;
        let until = (n - dow).rem_euclid(7);
        let anchor = time_plus(&time_round(t, Grain::Day), Grain::Day, until);
        time_sequence(Grain::Day, 7, anchor)
    })
}

fn run_day_of_the_month_predicate(n: i64) -> SeriesFn {
    series_fn(move |time, _| {
        let enough_days = move |t: &TimeObject| {
            let (year, month, _) = to_gregorian(t.start.mjd);
            n <= month_length(year, month) as i64
        };
        let add_days = move |t: TimeObject| time_plus(&t, Grain::Day, n - 1);
        let add_month = |i: i64, t: &TimeObject| time_plus(t, Grain::Month, i);
        let rounded = time_round(time, Grain::Month);
        let (_, _, dom) = to_gregorian(time.start.mjd);
        let anchor = if (dom as i64) <= n { rounded } else { add_month(1, &rounded) };
        let past = Seq::new(
            std::iter::successors(Some(add_month(-1, &anchor)), move |t| Some(add_month(-1, t)))
                .filter(enough_days)
                .map(add_days),
        );
        let future = Seq::new(
            std::iter::successors(Some(anchor), move |t| Some(add_month(1, t))).filter(enough_days).map(add_days),
        );
        (past, future)
    })
}

fn run_month_predicate(n: i64) -> SeriesFn {
    series_fn(move |t, _| {
        let rounded = time_plus(&time_round(t, Grain::Year), Grain::Month, n - 1);
        let anchor = if time_starts_before_the_end_of(t, &rounded) { rounded } else { time_plus(&rounded, Grain::Year, 1) };
        time_sequence(Grain::Year, 1, anchor)
    })
}

fn run_year_predicate(n: i64) -> SeriesFn {
    series_fn(move |t, _| {
        let (tyear, _, _) = to_gregorian(t.start.mjd);
        let y = time_plus(&time_round(t, Grain::Year), Grain::Year, n - tyear);
        if tyear <= n {
            (Seq::empty(), Seq::from_vec(vec![y]))
        } else {
            (Seq::from_vec(vec![y]), Seq::empty())
        }
    })
}

/// Limits how deep into lists of segments to look.
const SAFE_MAX: usize = 10;
const SAFE_MAX_INTERVAL: usize = 12;

fn run_compose_with_replacement(pred1: SeriesFn, pred2: SeriesFn, pred3: SeriesFn) -> SeriesFn {
    series_fn(move |now, ctx| {
        let (past1, future1) = pred1(now, ctx);
        let (past2, future2) = pred2(now, ctx);
        let (past3, future3) = pred3(now, ctx);

        fn compute_serie(lists: [Seq; 3]) -> Seq {
            let [a, b, c] = lists;
            Seq::new(a.iter().zip(b.iter()).zip(c.iter()).map(|((t1, t2), t3)| match time_intersect(&t1, &t2) {
                Some(_) => t3,
                None => t2,
            }))
        }
        let min_time = ctx.min_time;
        let max_time = ctx.max_time;
        let backward_bounded = move |s: &Seq| {
            Seq::new(s.iter().take(SAFE_MAX).take_while(move |t| time_starts_before_the_end_of(&min_time, t)))
        };
        let forward_bounded = move |s: &Seq| {
            Seq::new(s.iter().take(SAFE_MAX).take_while(move |t| time_starts_before_the_end_of(t, &max_time)))
        };
        let backward = compute_serie([backward_bounded(&past1), backward_bounded(&past2), backward_bounded(&past3)]);
        let forward = compute_serie([forward_bounded(&future1), forward_bounded(&future2), forward_bounded(&future3)]);
        (backward, forward)
    })
}

/// `runCompose`: performs best when `pred1` has a smaller grain than `pred2`.
fn run_compose(pred1: SeriesFn, pred2: SeriesFn) -> SeriesFn {
    series_fn(move |now, ctx| {
        let (past, future) = pred2(now, ctx);
        let compute_serie = |tokens: Seq| -> Seq {
            let pred1 = pred1.clone();
            let ctx = ctx.clone();
            Seq::new(tokens.iter().take(SAFE_MAX).flat_map(move |time1| {
                let fixed = TimeContext { min_time: time1, max_time: time1, ..ctx.clone() };
                let (_, fut) = pred1(&time1, &fixed);
                Seq::new(
                    fut.iter()
                        .take_while(move |this| time_starts_before_the_end_of(this, &time1))
                        .filter_map(move |x| time_intersect(&time1, &x)),
                )
                .iter()
            }))
        };
        let min_time = ctx.min_time;
        let max_time = ctx.max_time;
        let backward = compute_serie(past.take_while(move |t| time_starts_before_the_end_of(&min_time, t)));
        let forward = compute_serie(future.take_while(move |t| time_starts_before_the_end_of(t, &max_time)));
        (backward, forward)
    })
}

/// `timeSeqMap`: applies `f` to each interval yielded by `g`; intervals including "now" are in the
/// future.
pub fn time_seq_map(
    dont_reverse: bool,
    f: Arc<dyn Fn(&TimeObject, &TimeContext) -> Option<TimeObject> + Send + Sync>,
    g: Predicate,
) -> SeriesFn {
    let g = run_predicate(&g);
    series_fn(move |now, ctx| {
        let apply_f = |ts: Seq| -> Seq {
            let f = f.clone();
            let ctx = ctx.clone();
            Seq::new(ts.iter().take(SAFE_MAX_INTERVAL).filter_map(move |t| f(&t, &ctx)))
        };
        let (first_past, first_future) = g(now, ctx);
        let past1 = apply_f(first_past);
        let future1 = apply_f(first_future);

        let now_t = *now;
        let (new_future, still_past) = past1.span(move |x| time_starts_before_the_end_of(&now_t, x));
        let min_time = ctx.min_time;
        let old_past = still_past.take_while(move |x| time_starts_before_the_end_of(&min_time, x));

        // `break p` = `span (not . p)`
        let (new_past, still_future) = future1.span(move |x| !time_starts_before_the_end_of(&now_t, x));
        let max_time = ctx.max_time;
        let old_future = still_future.take_while(move |x| time_starts_before_the_end_of(x, &max_time));

        let apply_rev = |mut v: Vec<TimeObject>| {
            if !dont_reverse {
                v.reverse();
            }
            v
        };
        let sorted_past = apply_rev(new_past);
        let sorted_future = apply_rev(new_future);
        (old_past.prepend(sorted_past), old_future.prepend(sorted_future))
    })
}

fn run_time_intervals_predicate(ty: TimeIntervalType, pred1: &Predicate, pred2: &Predicate) -> SeriesFn {
    let p2 = run_predicate(pred2);
    let f: Arc<dyn Fn(&TimeObject, &TimeContext) -> Option<TimeObject> + Send + Sync> =
        Arc::new(move |this_segment, ctx| {
            let (_, fut) = p2(this_segment, ctx);
            fut.head().map(|first_future| time_interval(ty, this_segment, &first_future))
        });
    time_seq_map(true, f, pred1.clone())
}

pub fn run_predicate(p: &Predicate) -> SeriesFn {
    match p {
        Predicate::Empty => series_fn(|_, _| (Seq::empty(), Seq::empty())),
        Predicate::Series(f) => f.clone(),
        Predicate::TimeDate(td) => {
            if td.hour.is_none() && td.ampm.is_some() {
                return series_fn(|_, _| (Seq::empty(), Seq::empty()));
            }
            let mut to_compose: Vec<SeriesFn> = Vec::new();
            if let Some(n) = td.second {
                to_compose.push(run_second_predicate(n));
            }
            if let Some(n) = td.minute {
                to_compose.push(run_minute_predicate(n));
            }
            if let Some((is12h, h)) = td.hour {
                to_compose.push(run_hour_predicate(td.ampm, is12h, h));
            }
            if let Some(n) = td.day_of_week {
                to_compose.push(run_day_of_the_week_predicate(n));
            }
            if let Some(n) = td.day_of_month {
                to_compose.push(run_day_of_the_month_predicate(n));
            }
            if let Some(n) = td.month {
                to_compose.push(run_month_predicate(n));
            }
            if let Some(n) = td.year {
                to_compose.push(run_year_predicate(n));
            }
            // foldr1 runCompose
            let mut it = to_compose.into_iter().rev();
            let mut acc = it.next().expect("TimeDatePredicate invariant: at least one field is set");
            for p in it {
                acc = run_compose(p, acc);
            }
            acc
        }
        Predicate::Intersect(a, b) => run_compose(run_predicate(a), run_predicate(b)),
        Predicate::TimeIntervals(ty, a, b) => run_time_intervals_predicate(*ty, a, b),
        Predicate::ReplaceIntersect(a, b, c) => {
            run_compose_with_replacement(run_predicate(a), run_predicate(b), run_predicate(c))
        }
    }
}

// ----- TimeData -----------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Form {
    DayOfWeek,
    TimeOfDay { hours: Option<i64>, is12h: bool },
    Month(i64),
    PartOfDay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IntervalDirection {
    Before,
    After,
}

#[derive(Clone)]
pub struct TimeData {
    pub time_pred: Predicate,
    pub latent: bool,
    /// Needed for intersect.
    pub time_grain: Grain,
    pub not_immediate: bool,
    pub form: Option<Form>,
    pub direction: Option<IntervalDirection>,
    /// Allows specific this+Time.
    pub ok_for_this_next: bool,
    pub holiday: Option<Arc<str>>,
    /// Hack to prevent double timezone parsing.
    pub has_timezone: bool,
}

impl std::fmt::Debug for TimeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TimeData{{latent={}, grain={:?}, form={:?}, direction={:?}, holiday={:?}, hasTimezone={}}}",
            self.latent, self.time_grain, self.form, self.direction, self.holiday, self.has_timezone
        )
    }
}

/// Haskell's `Eq TimeData`: ignores the predicate, `okForThisNext` and the holiday.
impl PartialEq for TimeData {
    fn eq(&self, o: &TimeData) -> bool {
        self.latent == o.latent
            && self.time_grain == o.time_grain
            && self.not_immediate == o.not_immediate
            && self.form == o.form
            && self.direction == o.direction
            && self.has_timezone == o.has_timezone
    }
}
impl Eq for TimeData {}

impl Hash for TimeData {
    fn hash<H: Hasher>(&self, h: &mut H) {
        (self.latent, self.time_grain, self.not_immediate, self.form, self.direction).hash(h)
    }
}

/// `timedata'`.
pub fn timedata() -> TimeData {
    TimeData {
        time_pred: Predicate::Empty,
        latent: false,
        time_grain: Grain::Second,
        not_immediate: false,
        form: None,
        direction: None,
        ok_for_this_next: false,
        holiday: None,
        has_timezone: false,
    }
}

// ----- JSON ----------------------------------------------------------------------------------------------

fn pad(n: usize, x: i64) -> String {
    format!("{:0width$}", x, width = n)
}

fn show_gregorian(y: i64, m: u32, d: u32) -> String {
    let ys = if y < 0 { format!("-{:04}", -y) } else { format!("{:04}", y) };
    format!("{}-{:02}-{:02}", ys, m, d)
}

/// Round half to even of `num / den` (non-negative).
fn round_div_half_even(num: i128, den: i128) -> i128 {
    let q = num / den;
    let r = num % den;
    match (2 * r).cmp(&den) {
        Ordering::Less => q,
        Ordering::Greater => q + 1,
        Ordering::Equal => {
            if q % 2 == 0 {
                q
            } else {
                q + 1
            }
        }
    }
}

/// `toRFC3339 . fromUTC t (timeZoneFromSeries series t)`. Duckling works "as if in UTC", so `t`
/// already holds local wall-clock time; the zone only supplies the printed offset (no conversion).
pub fn to_rfc3339(series: &TimeZoneSeries, t: UtcTime) -> String {
    let zone = series.zone_at(t);
    let offset = zone.offset_minutes as i64;
    let local = t;
    let (y, mo, d) = to_gregorian(local.mjd);
    let (h, m, s_ps) = time_of_day(local.psecs);
    let whole = s_ps / PSECS_PER_SEC;
    let frac_ps = s_ps % PSECS_PER_SEC;
    let ms = round_div_half_even(frac_ps as i128, 1_000_000_000) as i64;
    let (sign, t2) = if offset < 0 { ("-", -offset) } else { ("+", offset) };
    format!(
        "{}T{}:{}:{}.{}{}{}:{}",
        show_gregorian(y, mo, d),
        pad(2, h),
        pad(2, m),
        pad(2, whole),
        pad(3, ms),
        sign,
        pad(2, t2 / 60),
        pad(2, t2 % 60)
    )
}

fn instant_json(series: &TimeZoneSeries, t: UtcTime, g: Grain) -> Json {
    Json::obj([("value", Json::str(to_rfc3339(series, t))), ("grain", g.json())])
}

fn time_value(series: &TimeZoneSeries, t: &TimeObject) -> Json {
    match t.end {
        None => {
            let mut j = instant_json(series, t.start, t.grain);
            if let Json::Obj(m) = &mut j {
                m.insert("type".to_string(), Json::str("value"));
            }
            j
        }
        Some(e) => Json::obj([
            ("type", Json::str("interval")),
            ("from", instant_json(series, t.start, t.grain)),
            ("to", instant_json(series, e, t.grain)),
        ]),
    }
}

fn open_interval(series: &TimeZoneSeries, d: IntervalDirection, t: &TimeObject) -> Json {
    let inst = instant_json(series, t.start, t.grain);
    match d {
        IntervalDirection::Before => Json::obj([("type", Json::str("interval")), ("to", inst)]),
        IntervalDirection::After => Json::obj([("type", Json::str("interval")), ("from", inst)]),
    }
}

thread_local! {
    /// Set when a Time node needed the reference time while probing was enabled (see
    /// [`probe_reference_time_use`]).
    static REFERENCE_TIME_USED: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    static RESOLVE_TC: std::cell::RefCell<Option<TimeContext>> = const { std::cell::RefCell::new(None) };
}

fn time_context_from(ctx: &Context) -> TimeContext {
    let utc = ctx.reference_time.utc;
    let ref_time = TimeObject { start: utc, grain: Grain::Second, end: None };
    TimeContext {
        ref_time,
        tz_series: Arc::new(ctx.reference_time.series.clone()),
        max_time: time_plus(&ref_time, Grain::Year, 2000),
        min_time: time_plus(&ref_time, Grain::Year, -2000),
    }
}

/// Reuse one [`TimeContext`] for every Time resolve in a parse (same reference time).
pub fn with_resolve_context<R>(ctx: &Context, f: impl FnOnce() -> R) -> R {
    let tc = time_context_from(ctx);
    RESOLVE_TC.with(|c| *c.borrow_mut() = Some(tc));
    let r = f();
    RESOLVE_TC.with(|c| *c.borrow_mut() = None);
    r
}

/// Runs `f` and reports whether resolving any Time node consumed the reference time. Upstream
/// evaluates the reference time lazily, so an invalid one only fails requests that resolve a
/// Time entity; the HTTP layer uses this to reproduce that.
pub fn probe_reference_time_use<R>(f: impl FnOnce() -> R) -> (R, bool) {
    REFERENCE_TIME_USED.with(|c| c.set(Some(false)));
    let r = f();
    let used = REFERENCE_TIME_USED.with(|c| c.replace(None)).unwrap_or(false);
    (r, used)
}

impl TimeData {
    pub fn resolve(&self, ctx: &Context, opts: &Options) -> Option<(Json, bool)> {
        if !opts.with_latent && self.latent {
            return None;
        }
        REFERENCE_TIME_USED.with(|c| {
            if c.get().is_some() {
                c.set(Some(true));
            }
        });
        let owned;
        let tc = if let Some(cached) = RESOLVE_TC.with(|c| c.borrow().clone()) {
            cached
        } else {
            owned = time_context_from(ctx);
            owned
        };
        let series = tc.tz_series.clone();
        let (past, future) = run_predicate(&self.time_pred)(&tc.ref_time, &tc);
        let value = match (future.get(0), future.get(1)) {
            (None, _) => past.head()?,
            (Some(ahead), Some(next_ahead))
                if self.not_immediate && time_intersect(&ahead, &tc.ref_time).is_some() =>
            {
                next_ahead
            }
            (Some(ahead), _) => ahead,
        };
        let values: Vec<TimeObject> = if future.is_empty() { past.to_vec(3) } else { future.to_vec(3) };
        let (v, vs) = match self.direction {
            None => (
                time_value(&series, &value),
                values.iter().map(|t| time_value(&series, t)).collect::<Vec<_>>(),
            ),
            Some(d) => (
                open_interval(&series, d, &value),
                values.iter().map(|t| open_interval(&series, d, t)).collect::<Vec<_>>(),
            ),
        };
        let mut obj = match v {
            Json::Obj(m) => m,
            _ => Default::default(),
        };
        obj.insert("values".to_string(), Json::Arr(vs));
        if let Some(h) = &self.holiday {
            obj.insert("holidayBeta".to_string(), Json::str(h.to_string()));
        }
        Some((Json::Obj(obj), self.latent))
    }
}
