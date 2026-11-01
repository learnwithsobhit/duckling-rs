//! `Duckling.Duration.RU.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::duration::*;
use crate::dims::numeral::{get_int_value, is_integer, is_natural};
use crate::dims::time_grain::Grain;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

static GRAINS_MAP: Lazy<HashMap<&'static str, Grain>> = Lazy::new(|| {
    [
        ("года", Grain::Year),
        ("месяца", Grain::Month),
        ("дня", Grain::Day),
        ("часа", Grain::Hour),
        ("минуты", Grain::Minute),
    ]
    .into_iter()
    .collect()
});

fn rule_halves() -> Rule {
    rule("half of a grain", vec![regex("пол\\s?(года|месяца|дня|часа|минуты)")], |ts| {
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
        vec![regex("(где-то|приблизительно|примерно|ровно)"), dimension(Dim::Duration)],
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
        "<positive-non-integer> <time-grain>",
        vec![
            crate::dims::numeral::number_with(|n| n.value, |v| !is_integer(v) && v > 0.0),
            dimension(Dim::TimeGrain),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(n), Token::TimeGrain(grain)) => tok(in_coarsest_grain(*grain, n.value)),
            _ => None,
        },
    )
}

fn hour_diminutive() -> Rule {
    rule("hour diminutive", vec![regex("час(ок|ик|очек)")], |_| tok(duration(Grain::Hour, 1)))
}

fn hours_diminutive() -> Rule {
    rule(
        "hour diminutive 2",
        vec![
            crate::dims::numeral::number_with(|n| n.value, is_integer),
            regex("час(иков|очков)"),
        ],
        |ts| match ts.first()? {
            Token::Numeral(n) => tok(duration(Grain::Hour, get_int_value(n.value)?)),
            _ => None,
        },
    )
}

fn minute_diminutive() -> Rule {
    rule("minute diminutive", vec![regex("минутк.|минуточк.")], |_| {
        tok(duration(Grain::Minute, 1))
    })
}

fn minutes_diminutive() -> Rule {
    rule(
        "minute diminutive",
        vec![
            crate::dims::numeral::number_with(|n| n.value, is_integer),
            regex("минутк.|минуток|минуточк.|минуточек"),
        ],
        |ts| match ts.first()? {
            Token::Numeral(n) => tok(duration(Grain::Minute, get_int_value(n.value)?)),
            _ => None,
        },
    )
}

fn rule_duration_quarter_of_an_hour() -> Rule {
    rule(
        "quarter of an hour",
        vec![regex("((одн(у|а|ой)|1)\\s)?четверт. (часа|ч|ч\\.)")],
        |_| tok(duration(Grain::Minute, 15)),
    )
}

fn rule_duration_three_quarters_of_an_hour() -> Rule {
    rule(
        "3 quarters of an hour",
        vec![
            crate::dims::numeral::number_with(|n| n.value, |v| v == 3.0),
            regex("четверт(и|ей) (часа|ч|ч\\.)"),
        ],
        |_| tok(duration(Grain::Minute, 45)),
    )
}

fn rule_duration_24h() -> Rule {
    rule("сутки", vec![regex("сутки")], |_| tok(duration(Grain::Hour, 24)))
}

fn rule_duration_24h_n() -> Rule {
    rule(
        "<integer> сутки",
        vec![crate::dims::numeral::number_with(|n| n.value, is_integer), regex("сутки|суток")],
        |ts| match ts.first()? {
            Token::Numeral(n) => tok(duration(Grain::Hour, get_int_value(n.value)? * 24)),
            _ => None,
        },
    )
}

fn rule_composite_duration() -> Rule {
    rule(
        "composite <duration>",
        vec![pred(is_natural), dimension(Dim::TimeGrain), dimension(Dim::Duration)],
        |ts| match (ts.first()?, ts.get(1)?, ts.get(2)?) {
            (Token::Numeral(n), Token::TimeGrain(g), Token::Duration(dd)) if *g > dd.grain => {
                tok(duration(*g, n.value.floor() as i64).combine(dd))
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
        hour_diminutive(),
        hours_diminutive(),
        minute_diminutive(),
        minutes_diminutive(),
        rule_duration_quarter_of_an_hour(),
        rule_duration_three_quarters_of_an_hour(),
        rule_duration_24h(),
        rule_duration_24h_n(),
        rule_composite_duration(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
