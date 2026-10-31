//! `Duckling.Volume.Types` (rules are per language).

use std::hash::{Hash, Hasher};

use crate::dims::interval::{self, Direction};
use crate::dims::util::hash_opt_f64;
use crate::json::Json;
use crate::types::{Context, Options};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Unit {
    Gallon,
    Hectolitre,
    Litre,
    Centilitre,
    Millilitre,
}

impl Unit {
    pub fn json(self) -> Json {
        Json::str(match self {
            Unit::Gallon => "gallon",
            Unit::Hectolitre => "hectolitre",
            Unit::Litre => "litre",
            Unit::Centilitre => "centilitre",
            Unit::Millilitre => "millilitre",
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VolumeData {
    pub value: Option<f64>,
    pub unit: Option<Unit>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl Eq for VolumeData {}

impl Hash for VolumeData {
    fn hash<H: Hasher>(&self, h: &mut H) {
        hash_opt_f64(self.value, h);
        self.unit.hash(h);
        hash_opt_f64(self.min_value, h);
        hash_opt_f64(self.max_value, h);
    }
}

fn single(u: Unit, v: f64) -> Json {
    Json::obj([("value", Json::Num(v)), ("unit", u.json())])
}

impl VolumeData {
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

// ----- helpers + rules (`Duckling.Volume.Helpers`, `.Rules`, `.EN.Rules`) --------------------

use crate::dims::numeral::is_positive;
use crate::types::{dimension, pred, regex, rule, Dim, PatternItem, Rule, Token};
use once_cell::sync::Lazy;

fn tok(d: VolumeData) -> Option<Token> {
    Some(Token::Volume(d))
}

pub fn volume(u: Unit, v: f64) -> VolumeData {
    VolumeData { unit: Some(u), value: Some(v), min_value: None, max_value: None }
}

pub fn unit_only(u: Unit) -> VolumeData {
    VolumeData { unit: Some(u), value: None, min_value: None, max_value: None }
}

pub fn value_only(v: f64) -> VolumeData {
    VolumeData { unit: None, value: Some(v), min_value: None, max_value: None }
}

pub fn with_interval(from: f64, to: f64, mut vd: VolumeData) -> VolumeData {
    vd.value = None;
    vd.min_value = Some(from);
    vd.max_value = Some(to);
    vd
}

pub fn with_min(from: f64, mut vd: VolumeData) -> VolumeData {
    vd.min_value = Some(from);
    vd
}

pub fn with_max(to: f64, mut vd: VolumeData) -> VolumeData {
    vd.max_value = Some(to);
    vd
}

pub fn is_simple_volume(t: &Token) -> bool {
    matches!(t, Token::Volume(VolumeData { value: Some(_), min_value: None, max_value: None, .. }))
}

pub fn is_unit_only(t: &Token) -> bool {
    matches!(
        t,
        Token::Volume(VolumeData { value: None, unit: Some(_), min_value: None, max_value: None })
    )
}

/// `(value, unit)` of a volume with both set.
fn vu(t: &Token) -> Option<(f64, Unit)> {
    match t {
        Token::Volume(VolumeData { value: Some(v), unit: Some(u), .. }) => Some((*v, *u)),
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
    vec![
        rule("number as volume", vec![pred(is_positive)], |ts| tok(value_only(numeral_value(ts.first()?)?))),
        rule("<number> <volume>", vec![pred(is_positive), pred(is_unit_only)], |ts| {
            let v = numeral_value(ts.first()?)?;
            match ts.get(1)? {
                Token::Volume(VolumeData { unit: Some(u), .. }) => tok(volume(*u, v)),
                _ => None,
            }
        }),
        rule(
            "<numeral> - <volume>",
            vec![pred(is_positive), regex("\\-"), pred(is_simple_volume)],
            |ts| {
                let from = numeral_value(ts.first()?)?;
                let (to, u) = vu(ts.get(2)?)?;
                if from < to { tok(with_interval(from, to, unit_only(u))) } else { None }
            },
        ),
        rule(
            "<volume> - <volume>",
            vec![pred(is_simple_volume), regex("\\-"), pred(is_simple_volume)],
            |ts| {
                let (from, u1) = vu(ts.first()?)?;
                let (to, u2) = vu(ts.get(2)?)?;
                if from < to && u1 == u2 { tok(with_interval(from, to, unit_only(u1))) } else { None }
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}

static EN_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v: Vec<Rule> = vec![
        rule(
            "about <volume>",
            vec![
                regex("\\~|exactly|precisely|about|approx(\\.|imately)?|close to|near( to)?|around|almost"),
                dimension(Dim::Volume),
            ],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        rule(
            "between|from <numeral> and|to <volume>",
            vec![regex("between|from"), pred(is_positive), regex("to|and"), pred(is_simple_volume)],
            |ts| {
                let from = numeral_value(ts.get(1)?)?;
                let (to, u) = vu(ts.get(3)?)?;
                if from < to { tok(with_interval(from, to, unit_only(u))) } else { None }
            },
        ),
        rule(
            "between|from <volume> to|and <volume>",
            vec![regex("between|from"), pred(is_simple_volume), regex("to|and"), pred(is_simple_volume)],
            |ts| {
                let (from, u1) = vu(ts.get(1)?)?;
                let (to, u2) = vu(ts.get(3)?)?;
                if from < to && u1 == u2 { tok(with_interval(from, to, unit_only(u1))) } else { None }
            },
        ),
        rule(
            "at most <volume>",
            vec![regex("under|below|at most|(less|lower|not? more) than"), pred(is_simple_volume)],
            |ts| {
                let (to, u) = vu(ts.get(1)?)?;
                tok(with_max(to, unit_only(u)))
            },
        ),
        rule(
            "more than <volume>",
            vec![
                regex("over|above|exceeding|beyond|at least|(more|larger|bigger|heavier) than"),
                pred(is_simple_volume),
            ],
            |ts| {
                let (from, u) = vu(ts.get(1)?)?;
                tok(with_min(from, unit_only(u)))
            },
        ),
    ];
    let volumes: [(&str, &str, Unit); 4] = [
        ("<latent vol> ml", "m(l(s?)|illilit(er|re)s?)", Unit::Millilitre),
        ("<vol> hectoliters", "hectolit(er|re)s?", Unit::Hectolitre),
        ("<vol> liters", "l(it(er|re)s?)?", Unit::Litre),
        ("<latent vol> gallon", "gal((l?ons?)|s)?", Unit::Gallon),
    ];
    for (name, re, u) in volumes {
        v.push(rule(name, vec![regex(re)], move |_| tok(unit_only(u))));
    }
    let fractions: [(&str, &str, f64); 6] = [
        ("one", "an? ", 1.0),
        ("half", "half(-|(( of)?( a(n?))?))?", 1.0 / 2.0),
        ("third", "third(-|(( of)?( a(n?))?))?", 1.0 / 3.0),
        ("fourth", "(quarter|fourth)(-|(( of)?( a(n?))?))?", 1.0 / 4.0),
        ("fifth", "fifth(-|(( of)?( a(n?))?))?", 1.0 / 5.0),
        ("tenth", "tenth(-|(( of)?( a(n?))?))?", 1.0 / 10.0),
    ];
    for (name, re, f) in fractions {
        let pattern: Vec<PatternItem> = vec![regex(re), pred(is_unit_only)];
        v.push(rule(name, pattern, move |ts| match ts.get(1)? {
            Token::Volume(VolumeData { unit: Some(u), .. }) => tok(volume(*u, f)),
            _ => None,
        }));
    }
    v
});

pub fn en_rules() -> &'static [Rule] {
    &EN_RULES
}
