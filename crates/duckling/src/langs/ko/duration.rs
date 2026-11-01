//! `Duckling.Duration.KO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::{duration, minutes_from_hour_mixed_fraction};
use crate::dims::numeral::is_natural;
use crate::dims::time_grain::Grain;
use crate::dims::util::parse_integer;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: crate::dims::duration::DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

/// `Duckling.Duration.Helpers.isGrain`.
fn is_grain(value: Grain) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("a day - 하루", vec![regex("하루")], |_| tok(duration(Grain::Day, 1))),
        rule("about <duration>", vec![regex("대충|약"), dimension(Dim::Duration)], |ts| {
            ts.get(1).map(|t| (*t).clone())
        }),
        rule("exactly <duration>", vec![regex("정확히"), dimension(Dim::Duration)], |ts| {
            ts.get(1).map(|t| (*t).clone())
        }),
        rule("half an hour", vec![pred(is_grain(Grain::Hour)), regex("반")], |_| {
            tok(duration(Grain::Minute, 30))
        }),
        rule(
            "<integer> and an half hours",
            vec![pred(is_natural), regex("시간반")],
            |ts| match ts.first()? {
                Token::Numeral(n) => tok(duration(Grain::Minute, 30 + 60 * n.value.floor() as i64)),
                _ => None,
            },
        ),
        rule("number.number hours", vec![regex("(\\d+)\\.(\\d+)"), regex("시간")], |ts| {
            let g = groups(ts)?;
            let hh = parse_integer(g.first()?)?;
            let mnum = parse_integer(g.get(1)?)?;
            let mden = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
            tok(minutes_from_hour_mixed_fraction(hh, mnum, mden))
        }),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
