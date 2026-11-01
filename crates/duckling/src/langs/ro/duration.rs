//! `Duckling.Duration.RO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::duration;
use crate::dims::numeral::number_with;
use crate::dims::time_grain::Grain;
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

fn fixed(name: &str, re: &str, grain: Grain, n: i64) -> Rule {
    rule(name, vec![regex(re)], move |_| Some(Token::Duration(duration(grain, n))))
}

fn rule_o_unitofduration() -> Rule {
    rule("o <unit-of-duration>", vec![regex("o|un"), dimension(Dim::TimeGrain)], |ts| {
        match ts.get(1)? {
            Token::TimeGrain(g) => Some(Token::Duration(duration(*g, 1))),
            _ => None,
        }
    })
}

fn rule_exact_in_jur_de_duration() -> Rule {
    rule(
        "exact|in jur de <duration>",
        vec![regex("(exact|aproximativ|(i|î)n jur de)"), dimension(Dim::Duration)],
        |ts| ts.get(1).map(|t| (*t).clone()),
    )
}

fn rule_integer_de_unitofduration() -> Rule {
    rule(
        "<integer> <unit-of-duration>",
        vec![number_with(|n| n.value, |v| v >= 20.0), regex("de"), dimension(Dim::TimeGrain)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(n), Token::TimeGrain(g)) => {
                Some(Token::Duration(duration(*g, n.value.floor() as i64)))
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_exact_in_jur_de_duration(),
        fixed(
            "jumatate de ora",
            r"(1/2\s?(h|or(a|ă))|jum(a|ă)tate (de )?or(a|ă))",
            Grain::Minute,
            30,
        ),
        rule_o_unitofduration(),
        fixed("quarter of an hour", r"(1/4\s?(h|or(a|ă))|sfert de or(a|ă))", Grain::Minute, 15),
        fixed(
            "trei sferturi de ora",
            r"(3/4\s?(h|or(a|ă))|trei sferturi de or(a|ă))",
            Grain::Minute,
            45,
        ),
        rule_integer_de_unitofduration(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
