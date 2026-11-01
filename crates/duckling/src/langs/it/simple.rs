//! `Duckling.Ordinal.IT.Rules`, `Duckling.TimeGrain.IT.Rules`, `Duckling.Duration.IT.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::duration;
use crate::dims::ordinal::OrdinalData;
use crate::dims::time_grain::Grain;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{dimension, groups, regex, rule, Dim, Rule, Token};

// ----- Ordinal ---------------------------------------------------------------------------

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

fn ordinals_primo(m: &str) -> Option<Token> {
    let v = match m {
        "primi" | "prima" | "primo" | "prime" => 1,
        "seconda" | "secondi" | "seconde" | "secondo" => 2,
        "terze" | "terzi" | "terzo" | "terza" => 3,
        "quarte" | "quarto" | "quarta" | "quarti" => 4,
        "quinto" | "quinta" | "quinti" | "quinte" => 5,
        "sesti" | "seste" | "sesta" | "sesto" => 6,
        "settimi" | "settima" | "settimo" | "settime" => 7,
        "ottavo" | "ottava" | "ottavi" | "ottave" => 8,
        "none" | "noni" | "nona" | "nono" => 9,
        "decimo" | "decima" | "decime" | "decimi" => 10,
        _ => return None,
    };
    ordinal(v)
}

static ORDINAL_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("ordinal (digits)", vec![regex("0*(\\d+) ?(ª|°|°)")], |ts| {
            parse_int(groups(ts)?.first()?).and_then(ordinal)
        }),
        rule(
            "ordinals (primo..10)",
            vec![regex("((prim|second|terz|quart|quint|sest|settim|ottav|non|decim)(o|a|i|e))")],
            |ts| ordinals_primo(&to_lower(groups(ts)?.first()?)),
        ),
    ]
});

pub fn ordinal_rules() -> &'static [Rule] {
    &ORDINAL_RULES
}

// ----- TimeGrain -------------------------------------------------------------------------

static TIME_GRAIN_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("seconde (grain)", "second[oi]", Grain::Second),
        ("minute (grain)", "minut[oi]", Grain::Minute),
        ("heure (grain)", "or[ae]", Grain::Hour),
        ("jour (grain)", "giorn[oi]", Grain::Day),
        ("semaine (grain)", "settiman[ae]", Grain::Week),
        ("mois (grain)", "mes[ei]", Grain::Month),
        ("trimestre (grain)", "trimestr[ei]", Grain::Quarter),
        ("année (grain)", "ann?[oi]", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn time_grain_rules() -> &'static [Rule] {
    &TIME_GRAIN_RULES
}

// ----- Duration --------------------------------------------------------------------------

fn dur(grain: Grain, n: i64) -> Option<Token> {
    Some(Token::Duration(duration(grain, n)))
}

static DURATION_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "une <unit-of-duration>",
            vec![regex("un[a']?"), dimension(Dim::TimeGrain)],
            |ts| match ts.get(1)? {
                Token::TimeGrain(g) => dur(*g, 1),
                _ => None,
            },
        ),
        rule("un quarto d'ora", vec![regex("un quarto d['i] ?ora")], |_| dur(Grain::Minute, 15)),
        rule("mezz'ora", vec![regex("mezz[a'] ?ora")], |_| dur(Grain::Minute, 30)),
        rule("tre quarti d'ora", vec![regex("(3|tre) quarti d['i] ?ora")], |_| {
            dur(Grain::Minute, 45)
        }),
    ]
});

pub fn duration_rules() -> &'static [Rule] {
    &DURATION_RULES
}
