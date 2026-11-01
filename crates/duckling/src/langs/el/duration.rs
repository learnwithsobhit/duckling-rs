//! `Duckling.Duration.EL.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use super::numeral::NUMERAL_MAP_EL;
use crate::dims::duration::*;
use crate::dims::numeral::is_natural;
use crate::dims::time_grain::Grain;
use crate::dims::util::{parse_integer, to_lower};
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

static TIME_GRAIN_MAP: Lazy<HashMap<&'static str, Grain>> = Lazy::new(|| {
    [
        ("λεπτο", Grain::Minute),
        ("ωρο", Grain::Hour),
        ("μερο", Grain::Day),
        ("ήμερο", Grain::Day),
        ("μηνο", Grain::Month),
        ("ετία", Grain::Year),
        ("ετίας", Grain::Year),
        ("ετή", Grain::Year),
        ("ετέ", Grain::Year),
        ("χρονο", Grain::Year),
    ]
    .into_iter()
    .collect()
});

fn tok(d: DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

fn floor_v(t: &Token) -> Option<i64> {
    match t {
        Token::Numeral(n) => Some(n.value.floor() as i64),
        _ => None,
    }
}

fn lookup_num(s: &str) -> Option<i64> {
    NUMERAL_MAP_EL.get(to_lower(s).as_str()).copied()
}

fn lookup_grain(s: &str) -> Option<Grain> {
    TIME_GRAIN_MAP.get(to_lower(s).as_str()).copied()
}

fn rule_duration_quarter_of_an_hour() -> Rule {
    rule(
        "quarter of an hour",
        vec![regex("(1/4|[εέ]ν(α|ός)\\s+τ[εέ]τ[αά]ρτου?)(\\s*ω|\\s+(της\\s+)?ώρας)?")],
        |_| tok(duration(Grain::Minute, 15)),
    )
}

fn rule_duration_half_an_hour() -> Rule {
    rule(
        "half an hour",
        vec![regex("(1/2\\s?((της )?ώρας?|ω)|μισάωρου?)")],
        |_| tok(duration(Grain::Minute, 30)),
    )
}

fn rule_numeral_quotes() -> Rule {
    rule("<integer> + '\"", vec![pred(is_natural), regex("(['\"])")], |ts| {
        match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(n), Token::RegexMatch(g)) => match &**g.0.first()? {
                "'" => tok(duration(Grain::Minute, n.value.floor() as i64)),
                "\"" => tok(duration(Grain::Second, n.value.floor() as i64)),
                _ => None,
            },
            _ => None,
        }
    })
}

fn rule_duration_numeral_more() -> Rule {
    rule(
        "<integer> more <unit-of-duration>",
        vec![pred(is_natural), dimension(Dim::TimeGrain), regex("ακόμα|λιγότερ[οη]")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(_), Token::TimeGrain(g)) => tok(duration(*g, floor_v(ts.first()?)?)),
            _ => None,
        },
    )
}

fn rule_duration_more_numeral() -> Rule {
    rule(
        "<integer> more <unit-of-duration>",
        vec![pred(is_natural), regex("ακόμα|λιγότερ[οη]"), dimension(Dim::TimeGrain)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(_), Token::TimeGrain(g)) => tok(duration(*g, floor_v(ts.first()?)?)),
            _ => None,
        },
    )
}

fn rule_numeral_with_grain() -> Rule {
    rule(
        "<number><grain> (one word)",
        vec![regex(
            "(δ[ιί]|τρ[ιί]|τετρ|πεν[θτ]|εξ|ε[πφ]τ|ο[κχ]τ|εννι|δεκ|δεκαπεν[θτ]|εικοσ|εικοσιπεν[θτ]|τριαντ|τριανταπεν[θτ]|σαραντ|σαρανταπεν[θτ]|πενηντ|πενηνταπεν[θτ]|εξηντ|ενενηντ)[αά]?(λεπτο|ωρο|ή?μερο|μηνο|ετία?|ετ[ήέ]|χρονο)ς?υ?",
        )],
        |ts| {
            let g = groups(ts)?;
            let n = lookup_num(g.first()?)?;
            let grain = lookup_grain(g.get(1)?)?;
            tok(duration(grain, n))
        },
    )
}

fn rule_duration_three_quarters_of_an_hour() -> Rule {
    rule(
        "three quarters of an hour",
        vec![regex("(3/4|τρ[ιί](α|ών)\\s+τ[εέ]τ[αά]ρτ(α|ων))(\\s*ω|\\s+(της\\s+)?ώρας)?")],
        |_| tok(duration(Grain::Minute, 45)),
    )
}

fn rule_duration_dot_numeral_hours() -> Rule {
    rule(
        "number.number hours",
        vec![regex("(\\d+),(\\d+)"), dimension(Dim::TimeGrain)],
        |ts| match ts.get(1)? {
            Token::TimeGrain(Grain::Hour) => {
                let g = groups(ts)?;
                let hh = parse_integer(g.first()?)?;
                let mnum = parse_integer(g.get(1)?)?;
                let mden = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
                tok(minutes_from_hour_mixed_fraction(hh, mnum, mden))
            }
            _ => None,
        },
    )
}

fn rule_half_duration() -> Rule {
    rule(
        "half a <grain>",
        vec![regex("μισ[ήό]ς?"), dimension(Dim::TimeGrain)],
        |ts| match ts.get(1)? {
            Token::TimeGrain(g) => n_plus_one_half(*g, 0).and_then(tok),
            _ => None,
        },
    )
}

fn rule_duration_and_a_half() -> Rule {
    rule(
        "<integer> and a half <grain>",
        vec![pred(is_natural), regex("και μισ[ήό]ς?"), dimension(Dim::TimeGrain)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(_), Token::TimeGrain(g)) => {
                n_plus_one_half(*g, floor_v(ts.first()?)?).and_then(tok)
            }
            _ => None,
        },
    )
}

fn rule_duration_and_a_half_one_word() -> Rule {
    rule(
        "<integer-and-half> <grain>",
        vec![
            regex(
                "(μιά|ενά|δυό|τρεισή|τεσσερι?σή|πεντέ|εξί|ε[πφ]τά|ο[κχ]τώ|εννιά|δεκά|εντεκά|δωδεκά)μισ[ιη]ς?",
            ),
            dimension(Dim::TimeGrain),
        ],
        |ts| match ts.get(1)? {
            Token::TimeGrain(g) => {
                let n = lookup_num(groups(ts)?.first()?)?;
                n_plus_one_half(*g, n).and_then(tok)
            }
            _ => None,
        },
    )
}

fn rule_duration_precision() -> Rule {
    rule(
        "about|exactly <duration>",
        vec![regex("(περίπου|πάνω κάτω|ακριβώς)"), dimension(Dim::Duration)],
        |ts| ts.get(1).map(|t| (*t).clone()),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_duration_quarter_of_an_hour(),
        rule_duration_half_an_hour(),
        rule_numeral_quotes(),
        rule_duration_numeral_more(),
        rule_duration_more_numeral(),
        rule_numeral_with_grain(),
        rule_duration_three_quarters_of_an_hour(),
        rule_duration_dot_numeral_hours(),
        rule_half_duration(),
        rule_duration_and_a_half(),
        rule_duration_and_a_half_one_word(),
        rule_duration_precision(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
