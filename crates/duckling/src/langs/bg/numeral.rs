//! `Duckling.Numeral.BG.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn zero_nineteen_map() -> HashMap<&'static str, i64> {
    HashMap::from([
        ("нула", 0),
        ("един", 1),
        ("една", 1),
        ("едно", 1),
        ("два", 2),
        ("две", 2),
        ("три", 3),
        ("четири", 4),
        ("пет", 5),
        ("шест", 6),
        ("седем", 7),
        ("осем", 8),
        ("девет", 9),
        ("десет", 10),
        ("единадесет", 11),
        ("единайсет", 11),
        ("дванадесет", 12),
        ("дванайсет", 12),
        ("тринадесет", 13),
        ("тринайсет", 13),
        ("четиринадесет", 14),
        ("четиринайсет", 14),
        ("петнадесет", 15),
        ("петнайсет", 15),
        ("шестнадесет", 16),
        ("шестнайсет", 16),
        ("седемнадесет", 17),
        ("седемнайсет", 17),
        ("осемнадесет", 18),
        ("осемнайсет", 18),
        ("деветнадесет", 19),
        ("деветнайсет", 19),
    ])
}

fn lookup_0_19(s: &str) -> Option<i64> {
    zero_nineteen_map().get(s).copied()
}

fn rx_group(t: &Token, i: usize) -> Option<String> {
    match t {
        Token::RegexMatch(g) => g.0.get(i).map(|s| s.to_string()),
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "number (0..19)",
            vec![regex("(нула|едина(де|й)сет|двана(де|й)сет|трина(де|й)сет|четирина(де|й)сет|петна(де|й)сет|шестна(де|й)сет|седемна(де|й)сет|осемна(де|й)сет|деветна(де|й)сет|един|една|едно|два|две|три|четири|пет|шест|седем|осем|девет|десет)")],
            |ts| {
                let x = to_lower(groups(ts)?.first()?);
                lookup_0_19(&x).and_then(integer)
            },
        ),
        rule(
            "integer (20..90)",
            vec![regex("((два|три|четири|пет|шест|седем|осем|девет)десет)")],
            |ts| {
                let match_ = to_lower(groups(ts)?.get(1)?);
                let x = lookup_0_19(&match_)?;
                integer(x * 10)
            },
        ),
        rule(
            "powers of tens",
            vec![regex("(хиляд(а|и)|милион(а|и)?|милиард(а|и)?)")],
            |ts| {
                match to_lower(groups(ts)?.first()?).as_str() {
                    "хиляд" => double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t)),
                    "милион" => double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t)),
                    "милиард" => double(1e9).and_then(|t| with_grain(9, &t)).and_then(|t| with_multipliable(&t)),
                    _ => None,
                }
            },
        ),
        rule(
            "integer 21..99",
            vec![one_of((2..=9).map(|i| (i * 10) as f64).collect()), regex("и"), pred(number_between(1.0, 10.0))],
            |ts| match (ts.first()?, ts.get(2)?) {
                (Token::Numeral(tens), Token::Numeral(units)) => double(tens.value + units.value),
                _ => None,
            },
        ),
        rule(
            "integer (100..900)",
            vec![regex("(сто|двеста|триста|(четири|пет|шест|седем|осем|девет)стотин)")],
            |ts| match to_lower(groups(ts)?.first()?).as_str() {
                "сто" => integer(100),
                "двеста" => integer(200),
                "триста" => integer(300),
                "четиристотин" => integer(400),
                "петстотин" => integer(500),
                "шестстотин" => integer(600),
                "седемстотин" => integer(700),
                "осемстотин" => integer(800),
                "деветстотин" => integer(900),
                _ => None,
            },
        ),
        rule(
            "integer 101..999",
            vec![one_of((2..=9).map(|i| (i * 100) as f64).collect()), pred(number_between(1.0, 100.0))],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(hundreds), Token::Numeral(tens)) => double(hundreds.value + tens.value),
                _ => None,
            },
        ),
        rule(
            "one point 2",
            vec![dimension(Dim::Numeral), regex("цяло и"), pred(|t| !has_grain(t))],
            |ts| match (ts.first()?, ts.get(2)?) {
                (Token::Numeral(nd1), Token::Numeral(nd2)) => {
                    double(nd1.value + decimals_to_double(nd2.value))
                }
                _ => None,
            },
        ),
        rule("decimal number", vec![regex("(\\d*\\.\\d+)")], |ts| {
            parse_decimal(true, groups(ts)?.first()?)
        }),
        rule(
            "comma-separated numbers",
            vec![regex("(\\d+(,\\d\\d\\d)+(\\.\\d+)?)")],
            |ts| {
                let match_ = groups(ts)?.first()?.replace(',', "");
                parse_double(&match_).and_then(double)
            },
        ),
        rule(
            "suffixes (K,M,G))",
            vec![dimension(Dim::Numeral), regex("((к|м|г)|(К|М|Г))(?=[\\W$€¢£]|$)")],
            |ts| match ts.first()? {
                Token::Numeral(nd) => {
                    let match_ = to_lower(&rx_group(ts.get(1)?, 0)?);
                    let x = match match_.as_str() {
                        "к" => Some(1e3),
                        "К" => Some(1e3),
                        "м" => Some(1e6),
                        "М" => Some(1e6),
                        "г" => Some(1e9),
                        "Г" => Some(1e9),
                        _ => None,
                    }?;
                    double(nd.value * x)
                }
                _ => None,
            },
        ),
        rule(
            "negative numbers",
            vec![regex("-|минус\\s?"), dimension(Dim::Numeral)],
            |ts| match ts.get(1)? {
                Token::Numeral(nd) => double(nd.value * -1.0),
                _ => None,
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
