//! `Duckling.Duration.DA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::{duration, minutes_from_hour_mixed_fraction};
use crate::dims::numeral::is_natural;
use crate::dims::time_grain::Grain;
use crate::dims::util::parse_integer;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: crate::dims::duration::DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

fn floor_v(t: &Token) -> Option<i64> {
    match t {
        Token::Numeral(n) => Some(n.value.floor() as i64),
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleAUnitofduration
        rule("a <unit-of-duration>", vec![regex("en|et?"), dimension(Dim::TimeGrain)], |ts| {
            match ts.get(1)? {
                Token::TimeGrain(g) => tok(duration(*g, 1)),
                _ => None,
            }
        }),
        // ruleAboutDuration
        rule(
            "about <duration>",
            vec![regex("(omkring|cirka|ca.)"), dimension(Dim::Duration)],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        // ruleExactlyDuration
        rule("exactly <duration>", vec![regex("pr(æ)cis"), dimension(Dim::Duration)], |ts| {
            ts.get(1).map(|t| (*t).clone())
        }),
        // ruleFortnight
        rule("fortnight", vec![regex("(a|one)? fortnight")], |_| tok(duration(Grain::Day, 14))),
        // ruleHalfAnHour
        rule("half an hour", vec![regex("(1/2|en halv) time")], |_| tok(duration(Grain::Minute, 30))),
        // ruleIntegerAndAnHalfHours
        rule(
            "<integer> and an half hours",
            vec![pred(is_natural), regex("og (en )?halv timer?")],
            |ts| tok(duration(Grain::Minute, 30 + 60 * floor_v(ts.first()?)?)),
        ),
        // ruleIntegerMoreUnitofduration
        rule(
            "<integer> more <unit-of-duration>",
            vec![pred(is_natural), dimension(Dim::TimeGrain), regex("mere|mindre")],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(n), Token::TimeGrain(g)) => tok(duration(*g, n.value.floor() as i64)),
                _ => None,
            },
        ),
        // ruleNumeralnumberHours
        rule(
            "number.number hours",
            vec![regex("(\\d+)\\,(\\d+)"), regex("timer?")],
            |ts| {
                let g = groups(ts)?;
                let hh = parse_integer(g.first()?)?;
                let mnum = parse_integer(g.get(1)?)?;
                let mden = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
                tok(minutes_from_hour_mixed_fraction(hh, mnum, mden))
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
