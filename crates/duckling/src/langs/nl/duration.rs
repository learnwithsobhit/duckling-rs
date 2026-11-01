//! `Duckling.Duration.NL.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::{duration, minutes_from_hour_mixed_fraction, DurationData};
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

fn rule_composite_commas_and() -> Rule {
    rule(
        "composite <duration> (with ,/and)",
        vec![pred(is_natural), dimension(Dim::TimeGrain), regex(",|en|plus"), dimension(Dim::Duration)],
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
        // ruleDurationQuarterOfAnHour
        rule("quarter of an hour", vec![regex("1/4\\s?(h|u(ur)?)|kwartier")], |_| {
            tok(duration(Grain::Minute, 15))
        }),
        // ruleDurationKwartier
        rule("<integer> kwartier", vec![pred(is_natural), regex("kwartier")], |ts| {
            tok(duration(Grain::Minute, 15 * floor_v(ts.first()?)?))
        }),
        // ruleCompositeDurationCommasAnd
        rule_composite_commas_and(),
        // ruleCompositeDuration
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
        // ruleDurationHalfAnHour
        rule("half an hour", vec![regex("(1/2\\s?uur|half uur)")], |_| tok(duration(Grain::Minute, 30))),
        // ruleDurationThreeQuartersOfAnHour
        rule("three-quarters of an hour", vec![regex("3/4\\s?uur")], |_| {
            tok(duration(Grain::Minute, 45))
        }),
        // ruleDurationDotNumeralHours
        rule("number,number uur", vec![regex("(\\d+)\\,(\\d+) *(uur|uren)")], |ts| {
            let g = groups(ts)?;
            let hh = parse_integer(g.first()?)?;
            let mnum = parse_integer(g.get(1)?)?;
            let mden = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
            tok(minutes_from_hour_mixed_fraction(hh, mnum, mden))
        }),
        // ruleDurationAndHalfHour
        rule(
            "<integer> and an half hour",
            vec![pred(is_natural), regex("en een half (uur|uren)")],
            |ts| tok(duration(Grain::Minute, 30 + 60 * floor_v(ts.first()?)?)),
        ),
        // ruleDurationOneAndHalfHour
        rule("1 and an half hour", vec![regex("anderhalf uur")], |_| tok(duration(Grain::Minute, 90))),
        // ruleDurationPrecision
        rule(
            "about|exactly <duration>",
            vec![regex("(ongeveer|precies|plusminus|exact)"), dimension(Dim::Duration)],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        // ruleNumeralQuotes
        rule("<integer> + '\"", vec![pred(is_natural), regex("(['\"])")], |ts| {
            let v = floor_v(ts.first()?)?;
            match ts.get(1)? {
                Token::RegexMatch(g) => match g.0.first()?.as_ref() {
                    "'" => tok(duration(Grain::Minute, v)),
                    "\"" => tok(duration(Grain::Second, v)),
                    _ => None,
                },
                _ => None,
            }
        }),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
