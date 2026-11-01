//! `Duckling.Duration.HI.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::duration::*;
use crate::dims::numeral::is_natural;
use crate::dims::time_grain::Grain;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

static AVADHI_MAP: Lazy<HashMap<&'static str, Grain>> = Lazy::new(|| {
    [
        ("मिनट", Grain::Minute),
        ("क्षण", Grain::Minute),
        ("घंटा", Grain::Hour),
        ("दिवस", Grain::Day),
        ("दिन", Grain::Day),
        ("महीना", Grain::Month),
        ("माह", Grain::Month),
        ("मास", Grain::Month),
        ("वर्ष", Grain::Year),
        ("साल", Grain::Year),
        ("बरस", Grain::Year),
    ]
    .into_iter()
    .collect()
});

fn rule_aadha() -> Rule {
    rule(
        "half of a duration",
        vec![regex(
            "आधा ((साल|वर्ष|बरस)|(महीना|मास|माह)|(दिन|दिवस)|(घंटा)|(मिनट|क्षण))",
        )],
        |ts| {
            let grain = *AVADHI_MAP.get(to_lower(groups(ts)?.first()?).as_str())?;
            n_plus_one_half(grain, 0).and_then(tok)
        },
    )
}

fn rule_duration_pandrah_minat() -> Rule {
    rule(
        "quarter of an hour",
        vec![regex("पंद्रह मिनट")],
        |_| tok(duration(Grain::Minute, 15)),
    )
}

fn rule_duration_pakhwaada() -> Rule {
    rule(
        "fortnight",
        vec![regex("((एक पखवाड़ा)|(पखवाड़ा))")],
        |_| tok(duration(Grain::Day, 14)),
    )
}

fn rule_duration_din() -> Rule {
    rule(
        "a day",
        vec![regex("((एक दिन)|(एक दिवस)|(दिन|दिवस))")],
        |_| tok(duration(Grain::Day, 1)),
    )
}

fn rule_duration_ek_saal() -> Rule {
    rule(
        "one year",
        vec![regex("एक (साल|वर्ष|बरस)")],
        |_| tok(duration(Grain::Year, 1)),
    )
}

fn rule_duration_precise_imprecise() -> Rule {
    rule(
        "about|exactly <duration>",
        vec![regex("(लगभग|बिल्कुल|केवल)"), dimension(Dim::Duration)],
        |ts| ts.get(1).map(|t| (*t).clone()),
    )
}

fn rule_paune_duration() -> Rule {
    rule(
        "Paune duration",
        vec![regex("पौने"), pred(is_natural), dimension(Dim::TimeGrain)],
        |ts| match (ts.get(1)?, ts.get(2)?) {
            (Token::Numeral(n), Token::TimeGrain(grain)) => {
                times_three_quarter(*grain, (n.value - 1.0).floor() as i64).and_then(tok)
            }
            _ => None,
        },
    )
}

fn rule_sava_duration() -> Rule {
    rule(
        "Sava duration",
        vec![regex("सवा"), pred(is_natural), dimension(Dim::TimeGrain)],
        |ts| match (ts.get(1)?, ts.get(2)?) {
            (Token::Numeral(n), Token::TimeGrain(grain)) => {
                times_one_quarter(*grain, n.value.floor() as i64).and_then(tok)
            }
            _ => None,
        },
    )
}

fn rule_saade_duration() -> Rule {
    rule(
        "Saade duration",
        vec![
            regex("साढ़े|साड़े"),
            pred(is_natural),
            dimension(Dim::TimeGrain),
        ],
        |ts| match (ts.get(1)?, ts.get(2)?) {
            (Token::Numeral(n), Token::TimeGrain(grain)) => {
                n_plus_one_half(*grain, n.value.floor() as i64).and_then(tok)
            }
            _ => None,
        },
    )
}

fn rule_composite_duration() -> Rule {
    rule(
        "composite <duration>",
        vec![
            pred(is_natural),
            dimension(Dim::TimeGrain),
            dimension(Dim::Duration),
        ],
        |ts| match (ts.first()?, ts.get(1)?, ts.get(2)?) {
            (Token::Numeral(n), Token::TimeGrain(g), Token::Duration(dd)) if *g > dd.grain => {
                tok(duration(*g, n.value.floor() as i64).combine(dd))
            }
            _ => None,
        },
    )
}

fn rule_composite_duration_commas_and() -> Rule {
    rule(
        "composite <duration> (with ,/और)",
        vec![
            pred(is_natural),
            dimension(Dim::TimeGrain),
            regex(",|और"),
            dimension(Dim::Duration),
        ],
        |ts| match (ts.first()?, ts.get(1)?, ts.get(3)?) {
            (Token::Numeral(n), Token::TimeGrain(g), Token::Duration(dd)) if *g > dd.grain => {
                tok(duration(*g, n.value.floor() as i64).combine(dd))
            }
            _ => None,
        },
    )
}

fn rule_n_and_half_duration() -> Rule {
    rule(
        "डेढ़|ढाई <duration>",
        vec![regex("(डेढ़|ढाई)"), dimension(Dim::TimeGrain)],
        |ts| match ts.get(1)? {
            Token::TimeGrain(grain) => {
                let h = match to_lower(groups(ts)?.first()?).as_str() {
                    "डेढ़" => Some(1),
                    "ढाई" => Some(2),
                    _ => None,
                }?;
                n_plus_one_half(*grain, h).and_then(tok)
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_duration_pandrah_minat(),
        rule_duration_pakhwaada(),
        rule_duration_din(),
        rule_aadha(),
        rule_duration_ek_saal(),
        rule_duration_precise_imprecise(),
        rule_paune_duration(),
        rule_sava_duration(),
        rule_saade_duration(),
        rule_composite_duration(),
        rule_composite_duration_commas_and(),
        rule_n_and_half_duration(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
