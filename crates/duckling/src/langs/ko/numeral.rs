//! `Duckling.Numeral.KO.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn lookup_int(map: &HashMap<&str, i64>, ts: &[&Token]) -> Option<Token> {
    let g = groups(ts)?;
    map.get(g.first()?.as_ref()).copied().and_then(integer)
}

static INTEGER_FOR_ORDINALS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [("한", 1), ("첫", 1), ("두", 2), ("세", 3), ("네", 4)].into_iter().collect()
});

static INTEGER_TYPE_AND_ORDINALS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("열", 10),
        ("스물", 20),
        ("서른", 30),
        ("마흔", 40),
        ("쉰", 50),
        ("예순", 60),
        ("일흔", 70),
        ("여든", 80),
        ("아흔", 90),
    ]
    .into_iter()
    .collect()
});

static INTEGER_TYPE1: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("영", 0),
        ("일", 1),
        ("이", 2),
        ("삼", 3),
        ("사", 4),
        ("오", 5),
        ("육", 6),
        ("칠", 7),
        ("팔", 8),
        ("구", 9),
    ]
    .into_iter()
    .collect()
});

static INTEGER_TYPE1_POWERS: Lazy<HashMap<&'static str, (f64, i64)>> = Lazy::new(|| {
    [
        ("십", (10.0, 1)),
        ("백", (1e2, 2)),
        ("천", (1e3, 3)),
        ("만", (1e4, 4)),
        ("억", (1e8, 8)),
        ("조", (1e12, 12)),
    ]
    .into_iter()
    .collect()
});

static INTEGER_TYPE2: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("하나", 1),
        ("둘", 2),
        ("셋", 3),
        ("넷", 4),
        ("다섯", 5),
        ("여섯", 6),
        ("일곱", 7),
        ("여덟", 8),
        ("아홉", 9),
    ]
    .into_iter()
    .collect()
});

fn get_digit(c: char) -> Option<&'static str> {
    match c {
        '영' => Some("0"),
        '일' => Some("1"),
        '이' => Some("2"),
        '삼' => Some("3"),
        '사' => Some("4"),
        '오' => Some("5"),
        '육' => Some("6"),
        '칠' => Some("7"),
        '팔' => Some("8"),
        '구' => Some("9"),
        _ => None,
    }
}

fn not_multipliable_and_positive(t: &Token) -> bool {
    !is_multipliable(t) && is_positive(t)
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("decimal number", vec![regex("(\\d*\\.\\d+)")], |ts| {
            parse_decimal(true, groups(ts)?.first()?)
        }),
        rule("decimal with thousands separator", vec![regex("(\\d+(,\\d\\d\\d)+\\.\\d+)")], |ts| {
            parse_double(&groups(ts)?.first()?.replace(',', "")).and_then(double)
        }),
        rule("few 몇", vec![regex("몇")], |_| integer(3)),
        rule(
            "fraction",
            vec![dimension(Dim::Numeral), regex("분(의|에)"), dimension(Dim::Numeral)],
            |ts| match (ts.first()?, ts.get(2)?) {
                (Token::Numeral(a), Token::Numeral(b)) => double(b.value / a.value),
                _ => None,
            },
        ),
        rule("half - 반", vec![regex("반")], |_| double(0.5)),
        rule("integer 0", vec![regex("영|공|빵")], |_| integer(0)),
        rule("integer (1..4) - for ordinals", vec![regex("(한|첫|두|세|네)")], |ts| {
            lookup_int(&INTEGER_FOR_ORDINALS, ts)
        }),
        rule("integer - TYPE 1", vec![regex("(영|일|이|삼|사|오|육|칠|팔|구)")], |ts| {
            lookup_int(&INTEGER_TYPE1, ts)
        }),
        rule("integer - TYPE 1: powers of ten", vec![regex("(십|백|천|만|억|조)")], |ts| {
            let g = groups(ts)?;
            let (value, grain) = *INTEGER_TYPE1_POWERS.get(g.first()?.as_ref())?;
            double(value).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
        }),
        rule(
            "intersect 2 numbers",
            vec![pred(has_grain), pred(not_multipliable_and_positive)],
            |ts| match (ts.first()?, ts.get(1)?) {
                (
                    Token::Numeral(NumeralData { value: val1, grain: Some(g), .. }),
                    Token::Numeral(NumeralData { value: val2, .. }),
                ) if 10f64.powi(*g as i32) > *val2 => double(val1 + val2),
                _ => None,
            },
        ),
        rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
            multiply(ts.first()?, ts.get(1)?)
        }),
        rule("integer (1..10) - TYPE 2", vec![regex("(하나|둘|셋|넷|다섯|여섯|일곱|여덟|아홉)")], |ts| {
            lookup_int(&INTEGER_TYPE2, ts)
        }),
        rule(
            "integer (21..99) - TYPE 2",
            vec![
                one_of(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
                one_of(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]),
            ],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
                _ => None,
            },
        ),
        rule(
            "integer (20..90) - TYPE 2 and ordinals",
            vec![regex("(열|스물|서른|마흔|쉰|예순|일흔|여든|아흔)")],
            |ts| lookup_int(&INTEGER_TYPE_AND_ORDINALS, ts),
        ),
        rule("integer with thousands separator ,", vec![regex("(\\d{1,3}(,\\d\\d\\d){1,5})")], |ts| {
            parse_double(&groups(ts)?.first()?.replace(',', "")).and_then(double)
        }),
        rule(
            "number dot number - 삼점사",
            vec![dimension(Dim::Numeral), regex("(점|쩜)((영|일|이|삼|사|오|육|칠|팔|구)+)")],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(n), Token::RegexMatch(g)) => {
                    let match_s = g.0.get(1)?;
                    let digits: String = match_s.chars().filter_map(get_digit).collect();
                    let v2 = parse_double(&digits)?;
                    double(n.value + decimals_to_double(v2))
                }
                _ => None,
            },
        ),
        rule(
            "numbers prefix with -, 마이너스, or 마이나스",
            vec![regex("-|마이너스\\s?|마이나스\\s?"), dimension(Dim::Numeral)],
            |ts| match ts.get(1)? {
                Token::Numeral(n) => double(n.value * (-1.0)),
                _ => None,
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
