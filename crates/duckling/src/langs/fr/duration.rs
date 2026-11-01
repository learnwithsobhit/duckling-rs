//! `Duckling.Duration.FR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::duration;
use crate::dims::numeral::is_natural;
use crate::dims::time_grain::Grain;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn rule_numeral_quotes() -> Rule {
    rule("<integer> + '\"", vec![pred(is_natural), regex("(['\"])")], |ts| {
        match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(n), Token::RegexMatch(g)) => match &**g.0.first()? {
                "'" => Some(Token::Duration(duration(Grain::Minute, n.value.floor() as i64))),
                "\"" => Some(Token::Duration(duration(Grain::Second, n.value.floor() as i64))),
                _ => None,
            },
            _ => None,
        }
    })
}

fn rule_une_unit_of_duration() -> Rule {
    rule(
        "une <unit-of-duration>",
        vec![regex("une|la|le?"), dimension(Dim::TimeGrain)],
        |ts| match ts.get(1)? {
            Token::TimeGrain(g) => Some(Token::Duration(duration(*g, 1))),
            _ => None,
        },
    )
}

fn fixed(name: &str, re: &str, grain: Grain, n: i64) -> Rule {
    rule(name, vec![regex(re)], move |_| Some(Token::Duration(duration(grain, n))))
}

fn rule_duration_environ() -> Rule {
    // Upstream pattern has a single item but the production needs two tokens: never fires.
    rule("environ <duration>", vec![regex("environ")], |ts| match (ts.first(), ts.get(1)) {
        (Some(_), Some(t)) => Some((*t).clone()),
        _ => None,
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_une_unit_of_duration(),
        fixed("un quart d'heure", r"(1/4\s?h(eure)?|(un|1) quart d'heure)", Grain::Minute, 15),
        fixed("une demi heure", r"(1/2\s?h(eure)?|(1|une) demi(e)?(\s|-)heure)", Grain::Minute, 30),
        fixed(
            "trois quarts d'heure",
            r"(3/4\s?h(eure)?|(3|trois) quart(s)? d'heure)",
            Grain::Minute,
            45,
        ),
        rule_duration_environ(),
        rule_numeral_quotes(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}

#[allow(dead_code)]
fn _unused(ts: &[&Token]) {
    let _ = groups(ts);
}
