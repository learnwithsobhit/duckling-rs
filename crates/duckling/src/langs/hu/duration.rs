//! `Duckling.Duration.HU.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::{duration, minutes_from_hour_mixed_fraction};
use crate::dims::numeral::is_natural;
use crate::dims::time_grain::Grain;
use crate::dims::util::parse_integer;
use crate::types::{groups, pred, regex, rule, Rule, Token};

fn tok(d: crate::dims::duration::DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

fn floor_v(t: &Token) -> Option<i64> {
    match t {
        Token::Numeral(n) => Some(n.value.floor() as i64),
        _ => None,
    }
}

fn rule_duration_quarter_of_an_hour() -> Rule {
    rule("quarter of an hour", vec![regex("(negyed[\\s-]?óra)")], |_| {
        tok(duration(Grain::Minute, 15))
    })
}

fn rule_duration_half_an_hour() -> Rule {
    rule("half an hour", vec![regex("(fél[\\s-]?óra)")], |_| tok(duration(Grain::Minute, 30)))
}

fn rule_duration_three_quarters_of_an_hour() -> Rule {
    rule(
        "three-quarters of an hour",
        vec![regex("(3/4[\\s-]?óra|háromnegyed[\\s-]?óra)")],
        |_| tok(duration(Grain::Minute, 45)),
    )
}

fn rule_duration_dot_numeral_hours() -> Rule {
    rule("number.number hours", vec![regex("(\\d+)\\.(\\d+) óra")], |ts| {
        let g = groups(ts)?;
        let hh = parse_integer(g.first()?)?;
        let mnum = parse_integer(g.get(1)?)?;
        let mden = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
        tok(minutes_from_hour_mixed_fraction(hh, mnum, mden))
    })
}

fn rule_duration_and_half_hour() -> Rule {
    rule(
        "<integer> and an half hour",
        vec![pred(is_natural), regex("és fél[\\s-]?óra")],
        |ts| tok(duration(Grain::Minute, 30 + 60 * floor_v(ts.first()?)?)),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_duration_quarter_of_an_hour(),
        rule_duration_half_an_hour(),
        rule_duration_three_quarters_of_an_hour(),
        rule_duration_dot_numeral_hours(),
        rule_duration_and_half_hour(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
