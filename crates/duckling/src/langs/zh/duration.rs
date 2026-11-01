//! `Duckling.Duration.ZH.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::*;
use crate::dims::numeral::{decimals_to_double, is_natural};
use crate::dims::time_grain::Grain;
use crate::dims::util::parse_integer;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

fn is_grain(value: Grain) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn mixed_fraction(ts: &[&Token], f: impl Fn(i128, i128, i128) -> DurationData) -> Option<Token> {
    let g = groups(ts)?;
    let a = parse_integer(g.first()?)?;
    let b = parse_integer(g.get(1)?)?;
    let den = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
    tok(f(a, b, den))
}

fn rule_duration_n_five_minutes() -> Rule {
    rule("number of five minutes", vec![pred(is_natural), regex("個字")], |ts| {
        match ts.first()? {
            Token::Numeral(n) => tok(duration(Grain::Minute, n.value.floor() as i64 * 5)),
            _ => None,
        }
    })
}

fn rule_duration_half_a_time_grain() -> Rule {
    rule("half a <time-grain>", vec![regex("半"), dimension(Dim::TimeGrain)], |ts| {
        match ts.get(1)? {
            Token::TimeGrain(g) => n_plus_one_half(*g, 0).and_then(tok),
            _ => None,
        }
    })
}

fn rule_duration_one_grain_and_half() -> Rule {
    rule(
        "a <unit-of-duration> and a half",
        vec![regex("一個半|個半"), dimension(Dim::TimeGrain)],
        |ts| match ts.get(1)? {
            Token::TimeGrain(g) => n_plus_one_half(*g, 1).and_then(tok),
            _ => None,
        },
    )
}

fn rule_duration_one_grain_and_half2() -> Rule {
    rule(
        "a <unit-of-duration> and a half",
        vec![dimension(Dim::TimeGrain), regex("半(鐘)?")],
        |ts| match ts.first()? {
            Token::TimeGrain(g) => n_plus_one_half(*g, 1).and_then(tok),
            _ => None,
        },
    )
}

fn rule_duration_and_half_grain() -> Rule {
    rule(
        "<integer> and a half <unit-of-duration>",
        vec![pred(is_natural), dimension(Dim::TimeGrain), regex("半(鐘)?")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(n), Token::TimeGrain(g)) => {
                n_plus_one_half(*g, n.value.floor() as i64).and_then(tok)
            }
            _ => None,
        },
    )
}

fn rule_duration_and_half_grain2() -> Rule {
    rule(
        "<integer> and a half <unit-of-duration>",
        vec![pred(is_natural), regex("個半"), dimension(Dim::TimeGrain)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(n), Token::TimeGrain(g)) => {
                n_plus_one_half(*g, n.value.floor() as i64).and_then(tok)
            }
            _ => None,
        },
    )
}

fn rule_duration_dot_numeral_hours() -> Rule {
    rule(
        "number.number hours",
        vec![regex("(\\d+)\\.(\\d+)"), pred(is_grain(Grain::Hour))],
        |ts| mixed_fraction(ts, minutes_from_hour_mixed_fraction),
    )
}

fn rule_duration_dot_numeral_hours2() -> Rule {
    rule(
        "number.number hours",
        vec![pred(is_natural), regex("點"), pred(is_natural), pred(is_grain(Grain::Hour))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(m), Token::Numeral(s)) => tok(duration(
                Grain::Minute,
                ((m.value + decimals_to_double(s.value)) * 60.0).floor() as i64,
            )),
            _ => None,
        },
    )
}

fn rule_duration_dot_numeral_minutes() -> Rule {
    rule(
        "number.number minutes",
        vec![regex("(\\d+)\\.(\\d+)"), pred(is_grain(Grain::Minute))],
        |ts| mixed_fraction(ts, seconds_from_hour_mixed_fraction),
    )
}

fn rule_duration_dot_numeral_minutes2() -> Rule {
    rule(
        "number.number minutes",
        vec![pred(is_natural), regex("點"), pred(is_natural), pred(is_grain(Grain::Minute))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(m), Token::Numeral(s)) => tok(duration(
                Grain::Second,
                ((m.value + decimals_to_double(s.value)) * 60.0).floor() as i64,
            )),
            _ => None,
        },
    )
}

fn rule_duration_hours_and_minutes() -> Rule {
    rule(
        "<integer> hours and <integer> minutes",
        vec![
            pred(is_natural),
            pred(is_grain(Grain::Hour)),
            pred(is_natural),
            pred(is_grain(Grain::Minute)),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(h), Token::Numeral(m)) => tok(duration(
                Grain::Minute,
                m.value.floor() as i64 + 60 * h.value.floor() as i64,
            )),
            _ => None,
        },
    )
}

fn rule_duration_minutes_and_seconds() -> Rule {
    rule(
        "<integer> minutes and <integer> seconds",
        vec![
            pred(is_natural),
            pred(is_grain(Grain::Minute)),
            pred(is_natural),
            pred(is_grain(Grain::Second)),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(m), Token::Numeral(s)) => tok(duration(
                Grain::Second,
                s.value.floor() as i64 + 60 * m.value.floor() as i64,
            )),
            _ => None,
        },
    )
}

fn rule_duration_hours_and_seconds() -> Rule {
    rule(
        "<integer> hours and <integer> seconds",
        vec![
            pred(is_natural),
            pred(is_grain(Grain::Hour)),
            pred(is_natural),
            pred(is_grain(Grain::Second)),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(h), Token::Numeral(s)) => tok(duration(
                Grain::Second,
                s.value.floor() as i64 + 3600 * h.value.floor() as i64,
            )),
            _ => None,
        },
    )
}

fn rule_duration_hours_and_minutes_and_seconds() -> Rule {
    rule(
        "<integer> hours and <integer> minutes and <integer> seconds",
        vec![
            pred(is_natural),
            pred(is_grain(Grain::Hour)),
            pred(is_natural),
            pred(is_grain(Grain::Minute)),
            pred(is_natural),
            pred(is_grain(Grain::Second)),
        ],
        |ts| match (ts.first()?, ts.get(2)?, ts.get(4)?) {
            (Token::Numeral(h), Token::Numeral(m), Token::Numeral(s)) => tok(duration(
                Grain::Second,
                s.value.floor() as i64
                    + 60 * m.value.floor() as i64
                    + 3600 * h.value.floor() as i64,
            )),
            _ => None,
        },
    )
}

fn rule_composite_duration_and() -> Rule {
    rule(
        "composite <duration> and <duration>",
        vec![dimension(Dim::Duration), regex("零"), dimension(Dim::Duration)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Duration(a), Token::Duration(dd)) if a.grain > dd.grain => {
                tok(duration(a.grain, a.value).combine(dd))
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_duration_n_five_minutes(),
        rule_duration_half_a_time_grain(),
        rule_duration_one_grain_and_half(),
        rule_duration_one_grain_and_half2(),
        rule_duration_and_half_grain(),
        rule_duration_and_half_grain2(),
        rule_duration_dot_numeral_hours(),
        rule_duration_dot_numeral_hours2(),
        rule_duration_dot_numeral_minutes(),
        rule_duration_dot_numeral_minutes2(),
        rule_duration_hours_and_minutes(),
        rule_duration_minutes_and_seconds(),
        rule_duration_hours_and_seconds(),
        rule_duration_hours_and_minutes_and_seconds(),
        rule_composite_duration_and(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
