//! `Duckling.Duration.GA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::duration;
use crate::dims::numeral::number_with;
use crate::dims::time_grain::Grain;
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

fn tok(d: crate::dims::duration::DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleCoics
        rule("coicís", vec![regex("coic(í|i)s(í|i|e)?")], |_| tok(duration(Grain::Day, 14))),
        // ruleIntegerUnitofdurationInteger
        rule(
            "<unit-integer> <unit-of-duration> <tens-integer>",
            vec![
                number_with(|n| n.value, |v| v < 10.0),
                dimension(Dim::TimeGrain),
                number_with(|n| n.value, |v| matches!(v, 10.0 | 20.0 | 30.0 | 40.0 | 50.0)),
            ],
            |ts| match (ts.first()?, ts.get(1)?, ts.get(2)?) {
                (Token::Numeral(n1), Token::TimeGrain(g), Token::Numeral(n2)) => {
                    tok(duration(*g, (n1.value + n2.value).floor() as i64))
                }
                _ => None,
            },
        ),
        // ruleLeathuair
        rule("leathuair", vec![regex("leathuair(e|eanta)?")], |_| {
            tok(duration(Grain::Minute, 30))
        }),
        // ruleAonDurationAmhain
        rule(
            "aon X amhain",
            vec![
                number_with(|n| n.value, |v| v == 1.0),
                dimension(Dim::TimeGrain),
                number_with(|n| n.value, |v| v == 1.0),
            ],
            |ts| match ts.get(1)? {
                Token::TimeGrain(g) => tok(duration(*g, 1)),
                _ => None,
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
