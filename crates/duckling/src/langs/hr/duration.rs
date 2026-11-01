//! `Duckling.Duration.HR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::{duration, minutes_from_hour_mixed_fraction};
use crate::dims::numeral::is_natural;
use crate::dims::time_grain::Grain;
use crate::dims::util::parse_int;
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

fn rule_exactly_duration() -> Rule {
    rule("exactly <duration>", vec![regex("to(c|č)no"), dimension(Dim::Duration)], |ts| {
        ts.get(1).map(|t| (*t).clone())
    })
}

fn rule_integer_and_an_half_hours() -> Rule {
    rule("<integer> and an half hours", vec![pred(is_natural), regex("i pol?a?")], |ts| {
        match ts.first()? {
            Token::Numeral(n) => {
                Some(Token::Duration(duration(Grain::Minute, 30 + 60 * (n.value.floor() as i64))))
            }
            _ => None,
        }
    })
}

fn rule_integer_more_unitofduration() -> Rule {
    rule(
        "<integer> more <unit-of-duration>",
        vec![pred(is_natural), regex("vi(s|š)e|manje"), dimension(Dim::TimeGrain)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(n), Token::TimeGrain(g)) => {
                Some(Token::Duration(duration(*g, n.value.floor() as i64)))
            }
            _ => None,
        },
    )
}

fn rule_quarter_of_an_hour() -> Rule {
    rule(
        "quarter of an hour",
        vec![regex(r"((1/4|frtalj|kvarat|(c|č)etvrt)\s?(h|sata)?)")],
        |_| Some(Token::Duration(duration(Grain::Minute, 15))),
    )
}

fn rule_about_duration() -> Rule {
    rule(
        "about <duration>",
        vec![regex("(oko|otprilike|odokativno)"), dimension(Dim::Duration)],
        |ts| ts.get(1).map(|t| (*t).clone()),
    )
}

fn rule_numbernumber_hours() -> Rule {
    rule("number.number hours", vec![regex(r"(\d+)\.(\d+)"), regex("sat(i|a)?")], |ts| {
        match ts.first()? {
            Token::RegexMatch(g) => {
                let h = parse_int(g.0.first()?)?;
                let m = g.0.get(1)?;
                let mnum = parse_int(m)?;
                let mden = 10i64.pow(m.chars().count() as u32);
                Some(Token::Duration(minutes_from_hour_mixed_fraction(
                    h as i128,
                    mnum as i128,
                    mden as i128,
                )))
            }
            _ => None,
        }
    })
}

fn rule_threequarters_of_an_hour() -> Rule {
    rule(
        "three-quarters of an hour",
        vec![regex(r"((3/4|tri-?frtalja|tri-?kvarat|tri-?(c|č)etvrt(ine)?)\s?(h|sata)?)")],
        |_| Some(Token::Duration(duration(Grain::Minute, 45))),
    )
}

fn rule_half_an_hour() -> Rule {
    rule("half an hour", vec![regex(r"(1/2\s?(h|sata)?|pol?a? sata)")], |_| {
        Some(Token::Duration(duration(Grain::Minute, 30)))
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_about_duration(),
        rule_exactly_duration(),
        rule_half_an_hour(),
        rule_integer_and_an_half_hours(),
        rule_integer_more_unitofduration(),
        rule_numbernumber_hours(),
        rule_quarter_of_an_hour(),
        rule_threequarters_of_an_hour(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
