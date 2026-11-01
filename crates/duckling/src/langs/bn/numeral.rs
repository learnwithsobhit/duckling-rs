//! `Duckling.Numeral.BN.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::parse_int;
use crate::types::{groups, regex, rule, Rule};

/// `bengaliMap` / `bengaliToArab`.
fn bengali_to_arab(c: char) -> char {
    match c {
        '০' => '0',
        '১' => '1',
        '২' => '2',
        '৩' => '3',
        '৪' => '4',
        '৫' => '5',
        '৬' => '6',
        '৭' => '7',
        '৮' => '8',
        '৯' => '9',
        _ => c,
    }
}

fn rule_bengali() -> Rule {
    rule(
        "bengali forms",
        vec![regex("([০১২৩৪৫৬৭৮৯]{1,18})")],
        |ts| {
            let mapped: String = groups(ts)?
                .first()?
                .chars()
                .map(bengali_to_arab)
                .collect();
            parse_int(&mapped).and_then(integer)
        },
    )
}

static RULE_NUMERAL_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("শূন্য", 0),
        ("এক", 1),
        ("দুই", 2),
        ("তিন", 3),
        ("চার", 4),
        ("পাঁচ", 5),
        ("ছয়", 6),
        ("সাত", 7),
        ("আট", 8),
        ("নয়", 9),
        ("দশ", 10),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral() -> Rule {
    rule(
        "number (0..10)",
        vec![regex("(শূন্য|এক|দুই|তিন|চার|পাঁচ|ছয়|সাত|আট|নয়|দশ)")],
        |ts| {
            RULE_NUMERAL_MAP
                .get(groups(ts)?.first()?.as_ref())
                .copied()
                .and_then(integer)
        },
    )
}

static ELEVEN_TO_NINETEEN_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("এগারো", 11),
        ("বারো", 12),
        ("তেরো", 13),
        ("চৌদ্দ", 14),
        ("পনেরো", 15),
        ("ষোল", 16),
        ("সতেরো", 17),
        ("আঠারো", 18),
        ("উনিশ", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_eleven_to_nineteen() -> Rule {
    rule(
        "number (11..19)",
        vec![regex("(এগারো|বারো|তেরো|চৌদ্দ|পনেরো|ষোল|সতেরো|আঠারো|উনিশ)")],
        |ts| {
            ELEVEN_TO_NINETEEN_MAP
                .get(groups(ts)?.first()?.as_ref())
                .copied()
                .and_then(integer)
        },
    )
}

static TWENTYONE_TO_TWENTYNINE_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("একুশ", 21),
        ("বাইশ", 22),
        ("তেইশ", 23),
        ("চব্বিশ", 24),
        ("পঁচিশ", 25),
        ("ছাব্বিশ", 26),
        ("সাতাশ", 27),
        ("আঠাশ", 28),
        ("ঊনত্রিশ", 29),
    ]
    .into_iter()
    .collect()
});

fn rule_twentyone_to_twentynine() -> Rule {
    rule(
        "number (21..29)",
        vec![regex("(একুশ|বাইশ|তেইশ|চব্বিশ|পঁচিশ|ছাব্বিশ|সাতাশ|আঠাশ|ঊনত্রিশ)")],
        |ts| {
            TWENTYONE_TO_TWENTYNINE_MAP
                .get(groups(ts)?.first()?.as_ref())
                .copied()
                .and_then(integer)
        },
    )
}

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("কুড়ি", 20),
        ("তিরিশ", 30),
        ("চল্লিশ", 40),
        ("পঞ্চাশ", 50),
        ("ষাট", 60),
        ("সত্তর", 70),
        ("আশি", 80),
        ("নব্বই", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_tens() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(কুড়ি|তিরিশ|চল্লিশ|পঞ্চাশ|ষাট|সত্তর|আশি|নব্বই)")],
        |ts| {
            TENS_MAP
                .get(groups(ts)?.first()?.as_ref())
                .copied()
                .and_then(integer)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_bengali(),
        rule_numeral(),
        rule_eleven_to_nineteen(),
        rule_twentyone_to_twentynine(),
        rule_tens(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
