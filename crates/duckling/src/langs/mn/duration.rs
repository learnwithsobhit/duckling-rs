//! `Duckling.Duration.MN.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::duration::{duration, n_plus_one_half, DurationData};
use crate::dims::numeral::{is_integer, is_natural};
use crate::dims::time_grain::Grain;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

static GRAINS_MAP: Lazy<HashMap<&'static str, Grain>> = Lazy::new(|| {
    [
        ("жил", Grain::Year),
        ("сар", Grain::Month),
        ("өдөр", Grain::Day),
        ("цаг", Grain::Hour),
        ("минут", Grain::Minute),
    ]
    .into_iter()
    .collect()
});

fn rule_halves() -> Rule {
    rule("half of a grain", vec![regex("хагас\\s?(жил|сар|өдөр|цаг|минут)")], |ts| {
        let g = groups(ts)?;
        let grain = *GRAINS_MAP.get(to_lower(g.first()?).as_str())?;
        n_plus_one_half(grain, 0).and_then(tok)
    })
}

fn rule_numeral_quotes() -> Rule {
    rule("<integer> + '\"", vec![pred(is_natural), regex("(['\"])")], |ts| {
        match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(n), Token::RegexMatch(g)) => match &**g.0.first()? {
                "'" => tok(duration(Grain::Minute, n.value.floor() as i64)),
                "\"" => tok(duration(Grain::Second, n.value.floor() as i64)),
                _ => None,
            },
            _ => None,
        }
    })
}

fn rule_duration_precision() -> Rule {
    rule(
        "about|exactly <duration>",
        vec![regex("(ойролцоогоор|яг)"), dimension(Dim::Duration)],
        |ts| ts.get(1).map(|t| (*t).clone()),
    )
}

fn rule_grain_as_duration() -> Rule {
    rule("a <unit-of-duration>", vec![dimension(Dim::TimeGrain)], |ts| match ts.first()? {
        Token::TimeGrain(grain) => tok(duration(*grain, 1)),
        _ => None,
    })
}

fn rule_positive_duration() -> Rule {
    rule(
        "<positive-numeral> <time-grain>",
        vec![
            crate::dims::numeral::number_with(|n| n.value, |v| !is_integer(v) && v > 0.0),
            dimension(Dim::TimeGrain),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(n), Token::TimeGrain(grain)) => {
                tok(duration(Grain::Second, grain.in_seconds(n.value).floor() as i64))
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_positive_duration(),
        rule_duration_precision(),
        rule_numeral_quotes(),
        rule_grain_as_duration(),
        rule_halves(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
