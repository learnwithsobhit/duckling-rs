//! `Duckling.Duration.UK.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::*;
use crate::dims::numeral::is_natural;
use crate::dims::time_grain::Grain;
use crate::dims::util::parse_integer;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

fn floor_v(t: &Token) -> Option<i64> {
    match t {
        Token::Numeral(n) => Some(n.value.floor() as i64),
        _ => None,
    }
}

fn rule_half_an_hour() -> Rule {
    rule("half an hour", vec![regex("(1/2\\s?|пів\\s?)години?")], |_| {
        tok(duration(Grain::Minute, 30))
    })
}

fn rule_numeralnumber_hours() -> Rule {
    rule("number.number hours", vec![regex("(\\d+)\\.(\\d+) години?")], |ts| {
        let g = groups(ts)?;
        let hh = parse_integer(g.first()?)?;
        let mnum = parse_integer(g.get(1)?)?;
        let mden = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
        tok(minutes_from_hour_mixed_fraction(hh, mnum, mden))
    })
}

fn rule_integer_and_an_half_hours() -> Rule {
    rule(
        "<integer> and an half hours",
        vec![pred(is_natural), regex("з половиною години")],
        |ts| tok(duration(Grain::Minute, 30 + 60 * floor_v(ts.first()?)?)),
    )
}

fn rule_about_duration() -> Rule {
    rule("about <duration>", vec![regex("близько"), dimension(Dim::Duration)], |ts| {
        ts.get(1).map(|t| (*t).clone())
    })
}

fn rule_exactly_duration() -> Rule {
    rule("exactly <duration>", vec![regex("рівно"), dimension(Dim::Duration)], |ts| {
        ts.get(1).map(|t| (*t).clone())
    })
}

fn rule_duration_half_a_time_grain() -> Rule {
    rule(
        "half a <time-grain>",
        vec![regex("(1/2|пів)"), dimension(Dim::TimeGrain)],
        |ts| match ts.get(1)? {
            Token::TimeGrain(grain) => n_plus_one_half(*grain, 0).and_then(tok),
            _ => None,
        },
    )
}

fn rule_duration_one_grain_and_half() -> Rule {
    rule(
        "one and a hald <unit-of-duration>",
        vec![regex("півтори"), dimension(Dim::TimeGrain)],
        |ts| match ts.get(1)? {
            Token::TimeGrain(grain) => n_plus_one_half(*grain, 1).and_then(tok),
            _ => None,
        },
    )
}

fn rule_composite_duration_commas_and() -> Rule {
    rule(
        "composite <duration> (with ,/and)",
        vec![pred(is_natural), dimension(Dim::TimeGrain), regex(",|і"), dimension(Dim::Duration)],
        |ts| match (ts.first()?, ts.get(1)?, ts.get(3)?) {
            (Token::Numeral(n), Token::TimeGrain(g), Token::Duration(dd)) if *g > dd.grain => {
                tok(duration(*g, n.value.floor() as i64).combine(dd))
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_about_duration(),
        rule_exactly_duration(),
        rule_half_an_hour(),
        rule_integer_and_an_half_hours(),
        rule_numeralnumber_hours(),
        rule_duration_half_a_time_grain(),
        rule_duration_one_grain_and_half(),
        rule_composite_duration_commas_and(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
