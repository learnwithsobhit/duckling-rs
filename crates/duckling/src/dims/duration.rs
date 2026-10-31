//! `Duckling.Duration.Types` (rules are per language).

use crate::dims::time_grain::Grain;
use crate::json::Json;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DurationData {
    pub value: i64,
    pub grain: Grain,
}

impl DurationData {
    pub fn resolve(&self) -> Json {
        Json::obj([
            ("type", Json::str("value")),
            ("value", Json::Int(self.value)),
            ("unit", self.grain.json()),
            (self.grain.show(), Json::Int(self.value)),
            (
                "normalized",
                Json::obj([
                    ("unit", Json::str("second")),
                    ("value", Json::Int(self.grain.in_seconds_int(self.value))),
                ]),
            ),
        ])
    }

    /// `withGrain`: convert to `g`, rounding to the nearest integer (banker's rounding, like `round`).
    pub fn with_grain(&self, g: Grain) -> DurationData {
        if g == self.grain {
            return self.clone();
        }
        let v = self.grain.in_seconds(self.value as f64) / g.in_seconds(1.0);
        DurationData { value: hs_round(v), grain: g }
    }

    /// Semigroup `<>`: sum in the finer grain.
    pub fn combine(&self, other: &DurationData) -> DurationData {
        let g = self.grain.min(other.grain);
        let a = self.with_grain(g);
        let b = other.with_grain(g);
        DurationData { value: a.value + b.value, grain: g }
    }
}

/// Haskell's `round :: Double -> Int` (round half to even).
pub fn hs_round(x: f64) -> i64 {
    let r = x.round();
    if (x - x.trunc()).abs() == 0.5 {
        // tie: choose the even neighbour
        let t = x.trunc();
        if (t as i64) % 2 == 0 {
            t as i64
        } else {
            r as i64
        }
    } else {
        r as i64
    }
}

// ----- helpers (`Duckling.Duration.Helpers`) ----------------------------------------------

use crate::dims::numeral::{is_natural, number_between};
use crate::dims::util::{parse_integer, to_lower};
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};
use once_cell::sync::Lazy;

pub fn duration(grain: Grain, n: i64) -> DurationData {
    DurationData { value: n, grain }
}

fn tok(d: DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

pub fn minutes_from_hour_mixed_fraction(h: i128, n: i128, d: i128) -> DurationData {
    duration(Grain::Minute, (60 * h + (n * 60) / d) as i64)
}

pub fn seconds_from_hour_mixed_fraction(m: i128, s: i128, d: i128) -> DurationData {
    duration(Grain::Second, (60 * m + (s * 60) / d) as i64)
}

pub fn n_plus_one_half(grain: Grain, n: i64) -> Option<DurationData> {
    match grain {
        Grain::Minute => Some(duration(Grain::Second, 30 + 60 * n)),
        Grain::Hour => Some(duration(Grain::Minute, 30 + 60 * n)),
        Grain::Day => Some(duration(Grain::Hour, 12 + 24 * n)),
        Grain::Month => Some(duration(Grain::Day, 15 + 30 * n)),
        Grain::Year => Some(duration(Grain::Month, 6 + 12 * n)),
        _ => None,
    }
}

pub fn times_one_quarter(grain: Grain, n: i64) -> Option<DurationData> {
    match grain {
        Grain::Minute => Some(duration(Grain::Second, 15 + 60 * n)),
        Grain::Hour => Some(duration(Grain::Minute, 15 + 60 * n)),
        Grain::Day => Some(duration(Grain::Hour, 6 + 24 * n)),
        Grain::Month => Some(duration(Grain::Day, 7 + 30 * n)),
        Grain::Year => Some(duration(Grain::Month, 3 + 12 * n)),
        _ => None,
    }
}

pub fn times_three_quarter(grain: Grain, n: i64) -> Option<DurationData> {
    match grain {
        Grain::Minute => Some(duration(Grain::Second, 45 + 60 * n)),
        Grain::Hour => Some(duration(Grain::Minute, 45 + 60 * n)),
        Grain::Day => Some(duration(Grain::Hour, 18 + 24 * n)),
        Grain::Month => Some(duration(Grain::Day, 21 + 30 * n)),
        Grain::Year => Some(duration(Grain::Month, 9 + 12 * n)),
        _ => None,
    }
}

/// `inCoarsestGrain`.
pub fn in_coarsest_grain(g: Grain, v: f64) -> DurationData {
    let seconds = g.in_seconds(v).floor() as i64 as f64;
    let mut grain = g;
    loop {
        let in_grain = seconds / grain.in_seconds(1.0);
        if let Some(n) = crate::dims::numeral::get_int_value(in_grain) {
            return duration(grain, n);
        }
        let next = grain.lower();
        if next == grain {
            // `lower Second = Second`: Haskell would loop forever, but any integral number of
            // seconds is representable, so this is unreachable for finite inputs.
            return duration(grain, in_grain as i64);
        }
        grain = next;
    }
}

fn is_grain(value: Grain) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn floor_v(t: &Token) -> Option<i64> {
    match t {
        Token::Numeral(n) => Some(n.value.floor() as i64),
        _ => None,
    }
}

// ----- common rules (`Duckling.Duration.Rules`) -------------------------------------------

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule(
        "<integer> <unit-of-duration>",
        vec![pred(is_natural), dimension(Dim::TimeGrain)],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(n), Token::TimeGrain(g)) => tok(duration(*g, n.value.floor() as i64)),
            _ => None,
        },
    )]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}

// ----- EN rules (`Duckling.Duration.EN.Rules`) --------------------------------------------

fn is_natural_between_1_60(t: &Token) -> bool {
    is_natural(t) && number_between(1.0, 60.0)(t)
}

fn mixed_fraction(
    ts: &[&Token],
    f: impl Fn(i128, i128, i128) -> DurationData,
) -> Option<Token> {
    let g = groups(ts)?;
    let a = parse_integer(g.first()?)?;
    let b = parse_integer(g.get(1)?)?;
    let den = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
    tok(f(a, b, den))
}

static EN_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_composite_commas_and(),
        rule(
            "quarter of an hour",
            vec![regex("(1/4\\s?h(our)?|(a\\s)?quarter of an hour)")],
            |_| tok(duration(Grain::Minute, 15)),
        ),
        rule("half an hour (abbrev).", vec![regex("1/2\\s?h")], |_| tok(duration(Grain::Minute, 30))),
        rule(
            "three-quarters of an hour",
            vec![regex("(3/4\\s?h(our)?|three(\\s|-)quarters of an hour)")],
            |_| tok(duration(Grain::Minute, 45)),
        ),
        rule("fortnight", vec![regex("(a|one)? fortnight")], |_| tok(duration(Grain::Day, 14))),
        rule(
            "<integer> more <unit-of-duration>",
            vec![pred(is_natural), regex("more|additional|extra|less|fewer"), dimension(Dim::TimeGrain)],
            |ts| match (ts.first()?, ts.get(2)?) {
                (Token::Numeral(n), Token::TimeGrain(g)) => tok(duration(*g, n.value.floor() as i64)),
                _ => None,
            },
        ),
        rule(
            "number.number hours",
            vec![regex("(\\d+)\\.(\\d+)"), pred(is_grain(Grain::Hour))],
            |ts| mixed_fraction(ts, minutes_from_hour_mixed_fraction),
        ),
        rule(
            "<integer> and an half hour",
            vec![pred(is_natural), regex("and (an? )?half hours?")],
            |ts| tok(duration(Grain::Minute, 30 + 60 * floor_v(ts.first()?)?)),
        ),
        rule(
            "<integer> and a half minutes",
            vec![pred(is_natural), regex("and (an? )?half min(ute)?s?")],
            |ts| tok(duration(Grain::Second, 30 + 60 * floor_v(ts.first()?)?)),
        ),
        rule("a <unit-of-duration>", vec![regex("an?"), dimension(Dim::TimeGrain)], |ts| match ts.get(1)? {
            Token::TimeGrain(g) => tok(duration(*g, 1)),
            _ => None,
        }),
        rule(
            "half a <time-grain>",
            vec![regex("(1/2|half)( an?)?"), dimension(Dim::TimeGrain)],
            |ts| match ts.get(1)? {
                Token::TimeGrain(g) => n_plus_one_half(*g, 0).and_then(tok),
                _ => None,
            },
        ),
        rule(
            "a <unit-of-duration> and a half",
            vec![regex("an?|one"), dimension(Dim::TimeGrain), regex("and (a )?half")],
            |ts| match ts.get(1)? {
                Token::TimeGrain(g) => n_plus_one_half(*g, 1).and_then(tok),
                _ => None,
            },
        ),
        rule(
            "<integer> hour and <integer>",
            vec![pred(is_natural), regex("hours?( and)?"), pred(is_natural_between_1_60)],
            |ts| {
                let h = floor_v(ts.first()?)?;
                let m = floor_v(ts.get(2)?)?;
                tok(duration(Grain::Minute, m + 60 * h))
            },
        ),
        rule(
            "about|exactly <duration>",
            vec![regex("(about|around|approximately|exactly)"), dimension(Dim::Duration)],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        rule(
            "<integer> + '\"",
            vec![pred(is_natural), regex("(['\"])")],
            |ts| {
                let v = floor_v(ts.first()?)?;
                match ts.get(1)? {
                    Token::RegexMatch(g) => match g.0.first()?.as_ref() {
                        "'" => tok(duration(Grain::Minute, v)),
                        "\"" => tok(duration(Grain::Second, v)),
                        _ => None,
                    },
                    _ => None,
                }
            },
        ),
        rule(
            "composite <duration>",
            vec![pred(is_natural), dimension(Dim::TimeGrain), dimension(Dim::Duration)],
            |ts| match (ts.first()?, ts.get(1)?, ts.get(2)?) {
                (Token::Numeral(n), Token::TimeGrain(g), Token::Duration(dd)) if *g > dd.grain => {
                    tok(duration(*g, n.value.floor() as i64).combine(dd))
                }
                _ => None,
            },
        ),
        rule(
            "composite <duration> and <duration>",
            vec![dimension(Dim::Duration), regex(",|and"), dimension(Dim::Duration)],
            |ts| match (ts.first()?, ts.get(2)?) {
                (Token::Duration(a), Token::Duration(dd)) if a.grain > dd.grain => {
                    tok(duration(a.grain, a.value).combine(dd))
                }
                _ => None,
            },
        ),
        rule_composite_commas_and(), // listed twice upstream
        rule(
            "number.number minutes",
            vec![regex("(\\d+)\\.(\\d+)"), pred(is_grain(Grain::Minute))],
            |ts| mixed_fraction(ts, seconds_from_hour_mixed_fraction),
        ),
        rule(
            "<Integer> and <Integer> quarter of hour",
            vec![
                pred(is_natural),
                regex("and (a |an |one |two |three )?quarters?( of)?( an)?"),
                pred(is_grain(Grain::Hour)),
            ],
            |ts| {
                let h = floor_v(ts.first()?)?;
                let m = match ts.get(1)? {
                    Token::RegexMatch(g) => g.0.first()?.clone(),
                    _ => return None,
                };
                let q = match to_lower(m.as_ref()).trim() {
                    "a" | "an" | "one" => 1,
                    "two" => 2,
                    "three" => 3,
                    _ => 1,
                };
                tok(duration(Grain::Minute, 15 * q + 60 * h))
            },
        ),
    ]
});

fn rule_composite_commas_and() -> Rule {
    rule(
        "composite <duration> (with ,/and)",
        vec![pred(is_natural), dimension(Dim::TimeGrain), regex(",|and"), dimension(Dim::Duration)],
        |ts| match (ts.first()?, ts.get(1)?, ts.get(3)?) {
            (Token::Numeral(n), Token::TimeGrain(g), Token::Duration(dd)) if *g > dd.grain => {
                tok(duration(*g, n.value.floor() as i64).combine(dd))
            }
            _ => None,
        },
    )
}

pub fn en_rules() -> &'static [Rule] {
    &EN_RULES
}
