//! `Duckling.Distance.Types` (rules are per language).

use std::hash::{Hash, Hasher};

use crate::dims::interval::{self, Direction};
use crate::dims::util::hash_opt_f64;
use crate::json::Json;
use crate::types::{Context, Options};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Unit {
    Centimetre,
    Foot,
    Inch,
    Kilometre,
    /// Ambiguous between Mile and Metre.
    M,
    Metre,
    Mile,
    Millimetre,
    Yard,
}

impl Unit {
    pub fn json(self) -> Json {
        Json::str(match self {
            Unit::Centimetre => "centimetre",
            Unit::Foot => "foot",
            Unit::Inch => "inch",
            Unit::Kilometre => "kilometre",
            Unit::M => "m",
            Unit::Metre => "metre",
            Unit::Mile => "mile",
            Unit::Millimetre => "millimetre",
            Unit::Yard => "yard",
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DistanceData {
    pub unit: Option<Unit>,
    pub value: Option<f64>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl Eq for DistanceData {}

impl Hash for DistanceData {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.unit.hash(h);
        hash_opt_f64(self.value, h);
        hash_opt_f64(self.min_value, h);
        hash_opt_f64(self.max_value, h);
    }
}

fn single(u: Unit, v: f64) -> Json {
    Json::obj([("value", Json::Num(v)), ("unit", u.json())])
}

impl DistanceData {
    pub fn resolve(&self, _ctx: &Context, _opts: &Options) -> Option<(Json, bool)> {
        let u = self.unit?;
        match (self.value, self.min_value, self.max_value) {
            (Some(v), _, _) => Some((interval::simple(single(u, v)), false)),
            (None, Some(from), Some(to)) => Some((interval::between(single(u, from), single(u, to)), false)),
            (None, Some(v), None) => Some((interval::open(Direction::Above, single(u, v)), false)),
            (None, None, Some(v)) => Some((interval::open(Direction::Under, single(u, v)), false)),
            (None, None, None) => None,
        }
    }
}

// ----- helpers (`Duckling.Distance.Helpers`) ----------------------------------------------

use crate::dims::numeral::is_positive;
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};
use once_cell::sync::Lazy;

pub fn distance(x: f64) -> DistanceData {
    DistanceData { value: Some(x), unit: None, min_value: None, max_value: None }
}

pub fn unit_only(u: Unit) -> DistanceData {
    DistanceData { unit: Some(u), value: None, min_value: None, max_value: None }
}

pub fn with_unit(u: Unit, mut dd: DistanceData) -> DistanceData {
    dd.unit = Some(u);
    dd
}

pub fn with_interval(from: f64, to: f64, mut dd: DistanceData) -> DistanceData {
    dd.min_value = Some(from);
    dd.max_value = Some(to);
    dd
}

pub fn with_min(from: f64, mut dd: DistanceData) -> DistanceData {
    dd.min_value = Some(from);
    dd
}

pub fn with_max(to: f64, mut dd: DistanceData) -> DistanceData {
    dd.max_value = Some(to);
    dd
}

pub fn is_simple_distance(t: &Token) -> bool {
    matches!(t, Token::Distance(DistanceData { value: Some(_), unit: Some(_), .. }))
}

// ----- `Duckling.DistanceUnits.Types` -------------------------------------------------------

/// Derived `Ord` of `SystemUnit`: all metric units sort before imperial ones.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SystemUnit {
    Metric(Metric),
    Imperial(Imperial),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Metric {
    Millimetre,
    Centimetre,
    Metre,
    Kilometre,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Imperial {
    Inch,
    Foot,
    Yard,
    Mile,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Deferrable {
    Definite(SystemUnit),
    /// "m": miles or metres.
    AmbiguousM,
}

fn to_system_unit(u: Unit) -> Deferrable {
    use Deferrable::*;
    match u {
        Unit::M => AmbiguousM,
        Unit::Millimetre => Definite(SystemUnit::Metric(Metric::Millimetre)),
        Unit::Centimetre => Definite(SystemUnit::Metric(Metric::Centimetre)),
        Unit::Metre => Definite(SystemUnit::Metric(Metric::Metre)),
        Unit::Kilometre => Definite(SystemUnit::Metric(Metric::Kilometre)),
        Unit::Inch => Definite(SystemUnit::Imperial(Imperial::Inch)),
        Unit::Foot => Definite(SystemUnit::Imperial(Imperial::Foot)),
        Unit::Yard => Definite(SystemUnit::Imperial(Imperial::Yard)),
        Unit::Mile => Definite(SystemUnit::Imperial(Imperial::Mile)),
    }
}

fn to_raw_unit(u: Deferrable) -> Unit {
    use Deferrable::*;
    match u {
        AmbiguousM => Unit::M,
        Definite(SystemUnit::Metric(Metric::Millimetre)) => Unit::Millimetre,
        Definite(SystemUnit::Metric(Metric::Centimetre)) => Unit::Centimetre,
        Definite(SystemUnit::Metric(Metric::Metre)) => Unit::Metre,
        Definite(SystemUnit::Metric(Metric::Kilometre)) => Unit::Kilometre,
        Definite(SystemUnit::Imperial(Imperial::Inch)) => Unit::Inch,
        Definite(SystemUnit::Imperial(Imperial::Foot)) => Unit::Foot,
        Definite(SystemUnit::Imperial(Imperial::Yard)) => Unit::Yard,
        Definite(SystemUnit::Imperial(Imperial::Mile)) => Unit::Mile,
    }
}

fn in_inches(u: Imperial, n: f64) -> f64 {
    match u {
        Imperial::Inch => n,
        Imperial::Foot => 12.0 * n,
        Imperial::Yard => 3.0 * (12.0 * n),
        Imperial::Mile => 5280.0 * (12.0 * n),
    }
}

fn in_metres(u: Metric, n: f64) -> f64 {
    match u {
        Metric::Millimetre => n / 1000.0,
        Metric::Centimetre => n / 100.0,
        Metric::Metre => n,
        Metric::Kilometre => n * 1000.0,
    }
}

fn in_si_units(val: f64, u: SystemUnit) -> f64 {
    match u {
        SystemUnit::Metric(m) => in_metres(m, val),
        SystemUnit::Imperial(i) => in_inches(i, val) * 0.0254,
    }
}

fn scale_units(target: SystemUnit, start: SystemUnit, v: f64) -> f64 {
    if start == target {
        v
    } else {
        in_si_units(v, start) / in_si_units(1.0, target)
    }
}

fn resolve_ambiguous(du: SystemUnit) -> SystemUnit {
    match du {
        SystemUnit::Metric(_) => SystemUnit::Metric(Metric::Metre),
        SystemUnit::Imperial(_) => SystemUnit::Imperial(Imperial::Mile),
    }
}

/// `distanceSum`: `cd1 <> cd2` on `ContextualDistance`; `None` is `Nonrelatable`.
pub fn distance_sum(v1: f64, u1: Unit, v2: f64, u2: Unit) -> Option<DistanceData> {
    use Deferrable::*;
    let reconcile = |av: f64, dv: f64, du: SystemUnit| -> (f64, Deferrable) {
        let resolved = resolve_ambiguous(du);
        let preferred = du.min(resolved);
        let combined = scale_units(preferred, resolved, av) + scale_units(preferred, du, dv);
        (combined, Definite(preferred))
    };
    let (v, u) = match (to_system_unit(u1), to_system_unit(u2)) {
        (AmbiguousM, AmbiguousM) => (v1 + v2, AmbiguousM),
        (AmbiguousM, Definite(du)) => reconcile(v1, v2, du),
        (Definite(du), AmbiguousM) => reconcile(v2, v1, du),
        (Definite(a), Definite(b)) => {
            let u = a.min(b);
            (scale_units(u, a, v1) + scale_units(u, b, v2), Definite(u))
        }
    };
    Some(with_unit(to_raw_unit(u), distance(v)))
}

// ----- rules ----------------------------------------------------------------------------

fn tok(d: DistanceData) -> Option<Token> {
    Some(Token::Distance(d))
}

/// Simple distance fields `(value, unit)` of a token.
fn simple(t: &Token) -> Option<(f64, Unit)> {
    match t {
        Token::Distance(DistanceData { value: Some(v), unit: Some(u), .. }) => Some((*v, *u)),
        _ => None,
    }
}

fn numeral_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule("number as distance", vec![dimension(Dim::Numeral)], |ts| {
        numeral_value(ts.first()?).and_then(|v| tok(distance(v)))
    })]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}

static EN_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = vec![
        rule(
            "between|from <numeral> to|and <dist>",
            vec![regex("between|from"), pred(is_positive), regex("to|and"), pred(is_simple_distance)],
            |ts| {
                let from = numeral_value(ts.get(1)?)?;
                let (to, u) = simple(ts.get(3)?)?;
                if from < to { tok(with_interval(from, to, unit_only(u))) } else { None }
            },
        ),
        rule(
            "between|from <dist> to|and <dist>",
            vec![regex("between|from"), pred(is_simple_distance), regex("to|and"), pred(is_simple_distance)],
            |ts| {
                let (from, u1) = simple(ts.get(1)?)?;
                let (to, u2) = simple(ts.get(3)?)?;
                if from < to && u1 == u2 { tok(with_interval(from, to, unit_only(u1))) } else { None }
            },
        ),
        rule(
            "under/less/lower/no more than <dist>",
            vec![regex("under|(less|lower|not? more) than"), pred(is_simple_distance)],
            |ts| {
                let (to, u) = simple(ts.get(1)?)?;
                tok(with_max(to, unit_only(u)))
            },
        ),
        rule(
            "over/above/at least/more than <dist>",
            vec![regex("over|above|at least|more than"), pred(is_simple_distance)],
            |ts| {
                let (to, u) = simple(ts.get(1)?)?;
                tok(with_min(to, unit_only(u)))
            },
        ),
        rule(
            "<numeral> - <dist>",
            vec![pred(is_positive), regex("-"), pred(is_simple_distance)],
            |ts| {
                let from = numeral_value(ts.first()?)?;
                let (to, u) = simple(ts.get(2)?)?;
                if from < to { tok(with_interval(from, to, unit_only(u))) } else { None }
            },
        ),
        rule(
            "<dist> - <dist>",
            vec![pred(is_simple_distance), regex("-"), pred(is_simple_distance)],
            |ts| {
                let (from, u1) = simple(ts.first()?)?;
                let (to, u2) = simple(ts.get(2)?)?;
                if from < to && u1 == u2 { tok(with_interval(from, to, unit_only(u1))) } else { None }
            },
        ),
        rule(
            "about|exactly <dist>",
            vec![
                regex("exactly|precisely|about|approx(\\.|imately)?|close to| near( to)?|around|almost"),
                dimension(Dim::Distance),
            ],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        rule(
            "composite <distance> (with ,/and)",
            vec![pred(is_simple_distance), regex(",|and"), pred(is_simple_distance)],
            |ts| {
                let (v1, u1) = simple(ts.first()?)?;
                let (v2, u2) = simple(ts.get(2)?)?;
                if u1 != u2 && v1 > 0.0 && v2 > 0.0 { distance_sum(v1, u1, v2, u2).and_then(tok) } else { None }
            },
        ),
        rule(
            "composite <distance>",
            vec![pred(is_simple_distance), pred(is_simple_distance)],
            |ts| {
                let (v1, u1) = simple(ts.first()?)?;
                let (v2, u2) = simple(ts.get(1)?)?;
                if u1 != u2 && v1 > 0.0 && v2 > 0.0 { distance_sum(v1, u1, v2, u2).and_then(tok) } else { None }
            },
        ),
    ];
    let distances: [(&str, &str, Unit); 9] = [
        ("miles", "mi(le(s)?)?", Unit::Mile),
        ("yard", "y(ar)?ds?", Unit::Yard),
        ("feet", "('|f(oo|ee)?ts?)", Unit::Foot),
        ("inch", "(\"|''|in(ch(es)?)?)", Unit::Inch),
        ("km", "k(ilo)?m?(et(er|re))?s?", Unit::Kilometre),
        ("meters", "met(er|re)s?", Unit::Metre),
        ("centimeters", "cm|centimet(er|re)s?", Unit::Centimetre),
        ("millimeters", "mm|millimet(er|re)s?", Unit::Millimetre),
        ("m (miles or meters)", "m", Unit::M),
    ];
    for (name, re, u) in distances {
        v.push(rule(name, vec![dimension(Dim::Distance), regex(re)], move |ts| match ts.first()? {
            Token::Distance(dd) => tok(with_unit(u, dd.clone())),
            _ => None,
        }));
    }
    v
});

pub fn en_rules() -> &'static [Rule] {
    &EN_RULES
}
