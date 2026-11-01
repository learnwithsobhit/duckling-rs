//! `Duckling.Numeral.TE.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule};

/// `teluguMap` / `teluguToArab`.
fn telugu_to_arab(c: char) -> char {
    match c {
        '౦' => '0',
        '౧' => '1',
        '౨' => '2',
        '౩' => '3',
        '౪' => '4',
        '౫' => '5',
        '౬' => '6',
        '౭' => '7',
        '౮' => '8',
        '౯' => '9',
        _ => c,
    }
}

fn rule_telugu() -> Rule {
    rule(
        "telugu forms",
        vec![regex("([౦౧౨౩౪౫౬౭౮౯]{1,18})")],
        |ts| {
            let mapped: String = groups(ts)?.first()?.chars().map(telugu_to_arab).collect();
            parse_int(&mapped).and_then(integer)
        },
    )
}

static ZERO_TO_NINE: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("సున్న", 0),
        ("ఒకటి", 1),
        ("రెండు", 2),
        ("మూడు", 3),
        ("నాలుగు", 4),
        ("ఐదు", 5),
        ("ఆరు", 6),
        ("ఏడు", 7),
        ("ఎనిమిది", 8),
        ("తొమ్మిది", 9),
    ]
    .into_iter()
    .collect()
});

fn rule_zero_to_nine() -> Rule {
    rule(
        "integer (0..9)",
        vec![regex("(సున్న|ఒకటి|రెండు|మూడు|నాలుగు|ఐదు|ఆరు|ఏడు|ఎనిమిది|తొమ్మిది)")],
        |ts| {
            ZERO_TO_NINE
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(integer)
        },
    )
}

static TEN_TO_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("పది", 10),
        ("పదకొండు", 11),
        ("పన్నెండు", 12),
        ("పదమూడు", 13),
        ("పద్నాల్గు", 14),
        ("పదిహేను", 15),
        ("పదహారు", 16),
        ("పదిహేడు", 17),
        ("పద్దెనిమిది", 18),
        ("పంతొమ్మిది", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_ten_to_nineteen() -> Rule {
    rule(
        "integer (10..19)",
        vec![regex(
            "(పదకొండు|పన్నెండు|పదమూడు|పద్నాల్గు|పదిహేను|పదహారు|పదిహేడు|పద్దెనిమిది|పంతొమ్మిది|పది)",
        )],
        |ts| {
            TEN_TO_NINETEEN
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(integer)
        },
    )
}

static TENS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ఇరవై", 20),
        ("ముప్పై", 30),
        ("నలబై", 40),
        ("యాబై", 50),
        ("అరవై", 60),
        ("డెబ్బై", 70),
        ("ఎనబై", 80),
        ("తొంబై", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_tens() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(ఇరవై|ముప్పై|నలబై|యాబై|అరవై|డెబ్బై|ఎనబై|తొంబై)")],
        |ts| {
            TENS.get(groups(ts)?.first()?.as_ref())
                .copied()
                .and_then(integer)
        },
    )
}

static HUNDREDS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("వంద", 100),
        ("వెయ్యి", 1000),
        ("లక్ష", 100000),
        ("కోటి", 10000000),
    ]
    .into_iter()
    .collect()
});

fn rule_hundreds() -> Rule {
    rule(
        "integer (100,1000,100000,10000000)",
        vec![regex("(వంద|వెయ్యి|లక్ష|కోటి)")],
        |ts| {
            HUNDREDS
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(integer)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_telugu(),
        rule_zero_to_nine(),
        rule_ten_to_nineteen(),
        rule_tens(),
        rule_hundreds(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
