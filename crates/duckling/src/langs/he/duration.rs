//! `Duckling.Duration.HE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::{duration, minutes_from_hour_mixed_fraction};
use crate::dims::numeral::is_natural;
use crate::dims::time_grain::Grain;
use crate::dims::util::parse_integer;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: crate::dims::duration::DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

fn rule_quarter_of_an_hour() -> Rule {
    rule(
        "quarter of an hour",
        vec![regex("(1/4/s שעה|רבע שעה)")],
        |_| tok(duration(Grain::Minute, 15)),
    )
}

fn rule_half_an_hour() -> Rule {
    rule(
        "half an hour",
        vec![regex("(1/2/s שעה|חצי שעה)")],
        |_| tok(duration(Grain::Minute, 30)),
    )
}

fn rule_threequarters_of_an_hour() -> Rule {
    rule(
        "three-quarters of an hour",
        vec![regex("(3/4/s שעה|שלושת רבעי שעה)")],
        |_| tok(duration(Grain::Minute, 45)),
    )
}

fn rule_numbernumber_hours() -> Rule {
    rule(
        "number.number hours",
        vec![regex("(\\d+)\\.(\\d+)"), regex("שעה|שעות")],
        |ts| {
            let g = groups(ts)?;
            let hh = parse_integer(g.first()?)?;
            let mnum = parse_integer(g.get(1)?)?;
            let mden = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
            tok(minutes_from_hour_mixed_fraction(hh, mnum, mden))
        },
    )
}

fn rule_integer_and_an_half_hours() -> Rule {
    rule(
        "<integer> and an half hours",
        vec![pred(is_natural), regex("וחצי (שעות|שעה)")],
        |ts| match ts.first()? {
            Token::Numeral(n) => tok(duration(Grain::Minute, 30 + 60 * (n.value.floor() as i64))),
            _ => None,
        },
    )
}

fn rule_about_duration() -> Rule {
    rule(
        "about <duration>",
        vec![regex("(בערך|סביבות|בקירוב)"), dimension(Dim::Duration)],
        |ts| Some((*ts.get(1)?).clone()),
    )
}

fn rule_exactly_duration() -> Rule {
    rule(
        "exactly <duration>",
        vec![regex("בדיוק"), dimension(Dim::Duration)],
        |ts| Some((*ts.get(1)?).clone()),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_about_duration(),
        rule_exactly_duration(),
        rule_half_an_hour(),
        rule_integer_and_an_half_hours(),
        rule_numbernumber_hours(),
        rule_quarter_of_an_hour(),
        rule_threequarters_of_an_hour(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
