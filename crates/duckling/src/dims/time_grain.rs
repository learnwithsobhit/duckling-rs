//! `Duckling.TimeGrain.Types`.

use crate::json::Json;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Grain {
    NoGrain,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl Grain {
    pub const ALL: &'static [Grain] = &[
        Grain::NoGrain,
        Grain::Second,
        Grain::Minute,
        Grain::Hour,
        Grain::Day,
        Grain::Week,
        Grain::Month,
        Grain::Quarter,
        Grain::Year,
    ];

    /// `TextShow Grain`: lower-cased constructor name.
    pub fn text(self) -> &'static str {
        match self {
            Grain::NoGrain => "nograin",
            Grain::Second => "second",
            Grain::Minute => "minute",
            Grain::Hour => "hour",
            Grain::Day => "day",
            Grain::Week => "week",
            Grain::Month => "month",
            Grain::Quarter => "quarter",
            Grain::Year => "year",
        }
    }

    /// Haskell `show` (derived): capitalised constructor name.
    pub fn show(self) -> &'static str {
        match self {
            Grain::NoGrain => "NoGrain",
            Grain::Second => "Second",
            Grain::Minute => "Minute",
            Grain::Hour => "Hour",
            Grain::Day => "Day",
            Grain::Week => "Week",
            Grain::Month => "Month",
            Grain::Quarter => "Quarter",
            Grain::Year => "Year",
        }
    }

    pub fn json(self) -> Json {
        Json::str(self.text())
    }

    /// `inSeconds` on a `Double` count.
    pub fn in_seconds(self, n: f64) -> f64 {
        match self {
            Grain::NoGrain | Grain::Second => n,
            Grain::Minute => n * 60.0,
            Grain::Hour => n * (60.0 * 60.0),
            Grain::Day => n * ((60.0 * 60.0) * 24.0),
            Grain::Week => n * (((60.0 * 60.0) * 24.0) * 7.0),
            Grain::Month => n * (((60.0 * 60.0) * 24.0) * 30.0),
            Grain::Quarter => n * ((((60.0 * 60.0) * 24.0) * 30.0) * 3.0),
            Grain::Year => n * (((60.0 * 60.0) * 24.0) * 365.0),
        }
    }

    /// `inSeconds` on an integral count (Haskell `Int`).
    pub fn in_seconds_int(self, n: i64) -> i64 {
        match self {
            Grain::NoGrain | Grain::Second => n,
            Grain::Minute => n * 60,
            Grain::Hour => n * 3600,
            Grain::Day => n * 86400,
            Grain::Week => n * 604800,
            Grain::Month => n * 2592000,
            Grain::Quarter => n * 7776000,
            Grain::Year => n * 31536000,
        }
    }

    /// `lower`: the next finer grain.
    pub fn lower(self) -> Grain {
        match self {
            Grain::NoGrain | Grain::Second => Grain::Second,
            Grain::Year => Grain::Month,
            Grain::Month => Grain::Day,
            Grain::Minute => Grain::Second,
            Grain::Hour => Grain::Minute,
            Grain::Day => Grain::Hour,
            Grain::Week => Grain::Day,
            Grain::Quarter => Grain::Month,
        }
    }
}

// ----- EN rules (`Duckling.TimeGrain.EN.Rules`) -------------------------------------------

use crate::types::{regex, rule, Rule, Token};
use once_cell::sync::Lazy;

static EN_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain) ", "sec(ond)?s?", Grain::Second),
        ("minute (grain)", "m(in(ute)?s?)?", Grain::Minute),
        ("hour (grain)", "h(((ou)?rs?)|r)?", Grain::Hour),
        ("day (grain)", "days?", Grain::Day),
        ("week (grain)", "weeks?", Grain::Week),
        ("month (grain)", "months?", Grain::Month),
        ("quarter (grain)", "(quarter|qtr)s?", Grain::Quarter),
        ("year (grain)", "y(ea)?rs?", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn en_rules() -> &'static [Rule] {
    &EN_RULES
}
