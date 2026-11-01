//! `Duckling.Duration.DE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::*;
use crate::dims::numeral::{is_natural, number_between};
use crate::dims::time_grain::Grain;
use crate::dims::util::parse_integer;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

fn is_grain(value: Grain) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn floor_v(t: &Token) -> Option<i64> {
    match t {
        Token::Numeral(n) => Some(n.value.floor() as i64),
        _ => None,
    }
}

fn is_natural_between_1_60(t: &Token) -> bool {
    is_natural(t) && number_between(1.0, 60.0)(t)
}

fn mixed_fraction(ts: &[&Token], f: impl Fn(i128, i128, i128) -> DurationData) -> Option<Token> {
    let g = groups(ts)?;
    let a = parse_integer(g.first()?)?;
    let b = parse_integer(g.get(1)?)?;
    let den = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
    tok(f(a, b, den))
}

fn rule_and_half_minute() -> Rule {
    rule(
        "<integer> and a half minutes",
        vec![pred(is_natural), regex("(und )?(ein(en?)? ?)?halb(en?)?"), pred(is_grain(Grain::Minute))],
        |ts| tok(duration(Grain::Second, 30 + 60 * floor_v(ts.first()?)?)),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("quarter of an hour", vec![regex("(einer? )?Viertelstunde")], |_| {
            tok(duration(Grain::Minute, 15))
        }),
        rule(
            "half an hour",
            vec![regex("(1/2\\s?|(eine )?halbe |(einer )?halben )stunde")],
            |_| tok(duration(Grain::Minute, 30)),
        ),
        rule(
            "three-quarters of an hour",
            vec![regex("3/4\\s?stunde|(einer? )?dreiviertel stunde|drei viertelstunden")],
            |_| tok(duration(Grain::Minute, 45)),
        ),
        rule(
            "about|exactly <duration>",
            vec![regex("ungef(ä|a)hr|zirka|genau|exakt"), dimension(Dim::Duration)],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        rule(
            "number,number hours",
            vec![regex("(\\d+),(\\d+)"), pred(is_grain(Grain::Hour))],
            |ts| mixed_fraction(ts, minutes_from_hour_mixed_fraction),
        ),
        rule(
            "number,number minutes",
            vec![regex("(\\d+),(\\d+)"), pred(is_grain(Grain::Minute))],
            |ts| mixed_fraction(ts, seconds_from_hour_mixed_fraction),
        ),
        rule("fortnight", vec![regex("zwei Wochen")], |_| tok(duration(Grain::Day, 14))),
        rule(
            "<integer> and a half hour",
            vec![pred(is_natural), regex("(und )?(ein(en?)? )?halb(en?)?"), pred(is_grain(Grain::Hour))],
            |ts| tok(duration(Grain::Minute, 30 + 60 * floor_v(ts.first()?)?)),
        ),
        rule_and_half_minute(),
        rule("a <unit-of-duration>", vec![regex("ein(en?)?"), dimension(Dim::TimeGrain)], |ts| {
            match ts.get(1)? {
                Token::TimeGrain(g) => tok(duration(*g, 1)),
                _ => None,
            }
        }),
        rule(
            "half a <time-grain>",
            vec![regex("(ein(en)?)?(1/2|halbe?)"), dimension(Dim::TimeGrain)],
            |ts| match ts.get(1)? {
                Token::TimeGrain(g) => n_plus_one_half(*g, 0).and_then(tok),
                _ => None,
            },
        ),
        rule(
            "composite <duration> (with ,/and)",
            vec![pred(is_natural), dimension(Dim::TimeGrain), regex(",|und"), dimension(Dim::Duration)],
            |ts| match (ts.first()?, ts.get(1)?, ts.get(3)?) {
                (Token::Numeral(n), Token::TimeGrain(g), Token::Duration(dd)) if *g > dd.grain => {
                    tok(duration(*g, n.value.floor() as i64).combine(dd))
                }
                _ => None,
            },
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
            "composite <duration> and <duration>",
            vec![dimension(Dim::Duration), regex(",|und"), dimension(Dim::Duration)],
            |ts| match (ts.first()?, ts.get(2)?) {
                (Token::Duration(a), Token::Duration(dd)) if a.grain > dd.grain => {
                    tok(duration(a.grain, a.value).combine(dd))
                }
                _ => None,
            },
        ),
        rule(
            "<integer> hour and <integer>",
            vec![
                pred(is_natural),
                regex("(ein(en?) )?stunden?( und)?"),
                pred(is_natural_between_1_60),
                pred(is_grain(Grain::Minute)),
            ],
            |ts| {
                let h = floor_v(ts.first()?)?;
                let m = floor_v(ts.get(2)?)?;
                tok(duration(Grain::Minute, m + 60 * h))
            },
        ),
        rule_and_half_minute(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
