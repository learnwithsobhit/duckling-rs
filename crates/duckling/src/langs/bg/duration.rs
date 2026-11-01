//! `Duckling.Duration.BG.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::*;
use crate::dims::numeral::{is_integer, is_natural, number_with};
use crate::dims::time_grain::Grain;
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

fn tok(d: DurationData) -> Option<Token> {
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
        rule(
            "<positive-numeral> <time-grain> and a half",
            vec![pred(is_natural), dimension(Dim::TimeGrain), regex("и половина")],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(n), Token::TimeGrain(g)) => {
                    n_plus_one_half(*g, n.value.floor() as i64).and_then(tok)
                }
                _ => None,
            },
        ),
        rule(
            "<time-grain> and a half",
            vec![dimension(Dim::TimeGrain), regex("и половина")],
            |ts| match ts.first()? {
                Token::TimeGrain(g) => n_plus_one_half(*g, 1).and_then(tok),
                _ => None,
            },
        ),
        rule(
            "<positive-numeral> <time-grain>",
            vec![
                number_with(|n| n.value, |v| !is_integer(v) && v > 0.0),
                dimension(Dim::TimeGrain),
            ],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(n), Token::TimeGrain(g)) => {
                    tok(duration(Grain::Second, g.in_seconds(n.value).floor() as i64))
                }
                _ => None,
            },
        ),
        rule(
            "about|exactly <duration>",
            vec![regex("(към|приблизително|примерно|някъде)"), dimension(Dim::Duration)],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        rule(
            "<integer> + '\"",
            vec![pred(is_natural), regex("(['\"])")],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(_), Token::RegexMatch(g)) => {
                    let v = floor_v(ts.first()?)?;
                    match g.0.first()?.as_ref() {
                        "'" => tok(duration(Grain::Minute, v)),
                        "\"" => tok(duration(Grain::Second, v)),
                        _ => None,
                    }
                }
                _ => None,
            },
        ),
        rule("a <unit-of-duration>", vec![dimension(Dim::TimeGrain)], |ts| match ts.first()? {
            Token::TimeGrain(g) => tok(duration(*g, 1)),
            _ => None,
        }),
        rule(
            "half of a <time-grain>",
            vec![regex("половин"), dimension(Dim::TimeGrain)],
            |ts| match ts.get(1)? {
                Token::TimeGrain(g) => n_plus_one_half(*g, 0).and_then(tok),
                _ => None,
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
