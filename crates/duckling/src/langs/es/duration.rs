//! `Duckling.Duration.ES.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::duration;
use crate::dims::numeral::is_natural;
use crate::dims::time_grain::Grain;
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

fn tok(d: crate::dims::duration::DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("half of an hour", vec![regex("media horas?")], |_| tok(duration(Grain::Minute, 30))),
        rule("quarter of an hour", vec![regex("cuartos? de hora")], |_| {
            tok(duration(Grain::Minute, 15))
        }),
        rule(
            "three-quarters of an hour",
            vec![regex("tres cuartos? de horas?")],
            |_| tok(duration(Grain::Minute, 45)),
        ),
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
        rule(
            "composite <duration> (with and)",
            vec![pred(is_natural), dimension(Dim::TimeGrain), regex("y"), dimension(Dim::Duration)],
            |ts| match (ts.first()?, ts.get(1)?, ts.get(3)?) {
                (Token::Numeral(n), Token::TimeGrain(g), Token::Duration(dd)) if *g > dd.grain => {
                    tok(duration(*g, n.value.floor() as i64).combine(dd))
                }
                _ => None,
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
