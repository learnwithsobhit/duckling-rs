//! `Duckling.Duration.KA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::*;
use crate::dims::numeral::is_natural;
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

fn rule_composite_duration() -> Rule {
    rule(
        "composite <duration>",
        vec![pred(is_natural), dimension(Dim::TimeGrain), regex(",|და"), dimension(Dim::Duration)],
        |ts| match (ts.first()?, ts.get(1)?, ts.get(3)?) {
            (Token::Numeral(_), Token::TimeGrain(g), Token::Duration(dd)) if *g > dd.grain => {
                tok(duration(*g, floor_v(ts.first()?)?).combine(dd))
            }
            _ => None,
        },
    )
}

fn rule_composite_duration1() -> Rule {
    rule(
        "composite <duration>",
        vec![dimension(Dim::TimeGrain), regex(",|და"), dimension(Dim::Duration)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::TimeGrain(g), Token::Duration(dd)) if *g > dd.grain => {
                tok(duration(*g, 1).combine(dd))
            }
            _ => None,
        },
    )
}

fn rule_duration_year() -> Rule {
    rule(
        "<integer> year",
        vec![pred(is_natural), regex("წელიწად(ის)?(ი)?(ში)?|წლი(ის)?(ში)?|წელშ?ი")],
        |ts| tok(duration(Grain::Month, 12 * floor_v(ts.first()?)?)),
    )
}

fn rule_duration_and_half_year() -> Rule {
    rule(
        "<integer> and an half year",
        vec![pred(is_natural), regex("წელიწადნახევა?(რის)?(რი)?(რში)?|წლინახევრ(ის)?(არში)?")],
        |ts| tok(duration(Grain::Month, 6 + 12 * floor_v(ts.first()?)?)),
    )
}

fn rule_duration_and_half_year1() -> Rule {
    rule(
        "<integer> and an half year",
        vec![regex("წელიწადნახევა?(რის)?(რი)?(რში)?|წლინახევრ(ის)?(არში)?")],
        |_| tok(duration(Grain::Month, 18)),
    )
}

fn rule_duration_and_half_month() -> Rule {
    rule(
        "<integer> and an half month",
        vec![pred(is_natural), regex("თვენახევა?(რის)?(რი)?(რში)?")],
        |ts| tok(duration(Grain::Day, 15 + 30 * floor_v(ts.first()?)?)),
    )
}

fn rule_duration_and_half_month1() -> Rule {
    rule(
        "month and an half month",
        vec![regex("თვენახევა?(რის)?(რი)?(რში)?")],
        |_| tok(duration(Grain::Day, 45)),
    )
}

fn rule_duration_and_half_week() -> Rule {
    rule(
        "<integer> and an half week",
        vec![pred(is_natural), regex("თვენახევა?(რის)?(რი)?(რში)?")],
        |ts| tok(duration(Grain::Day, 3 + 7 * floor_v(ts.first()?)?)),
    )
}

fn rule_duration_and_half_week1() -> Rule {
    rule(
        "week and an half week",
        vec![regex("კვირანახევა?(რის)?(რი)?(რში)?")],
        |_| tok(duration(Grain::Day, 10)),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_composite_duration(),
        rule_composite_duration1(),
        rule_duration_and_half_year(),
        rule_duration_and_half_year1(),
        rule_duration_and_half_month(),
        rule_duration_and_half_month1(),
        rule_duration_and_half_week(),
        rule_duration_and_half_week1(),
        rule_duration_year(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
