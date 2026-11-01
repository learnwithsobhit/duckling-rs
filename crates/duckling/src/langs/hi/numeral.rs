//! `Duckling.Numeral.HI.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, pred, regex, rule, Rule, Token};

/// `devanagariMap` / `devanagariToArab`.
fn devanagari_to_arab(c: char) -> char {
    match c {
        '०' => '0',
        '१' => '1',
        '२' => '2',
        '३' => '3',
        '४' => '4',
        '५' => '5',
        '६' => '6',
        '७' => '7',
        '८' => '8',
        '९' => '9',
        _ => c,
    }
}

fn rule_devanagari() -> Rule {
    rule(
        "devanagari forms",
        vec![regex("([०१२३४५६७८९]{1,18})")],
        |ts| {
            let mapped: String = groups(ts)?
                .first()?
                .chars()
                .map(devanagari_to_arab)
                .collect();
            parse_int(&mapped).and_then(integer)
        },
    )
}

static ZERO_TO_NINETY_NINE: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("शून्य", 0),
        ("एक", 1),
        ("दो", 2),
        ("तीन", 3),
        ("चार", 4),
        ("पाँच", 5),
        ("छः", 6),
        ("छह", 6),
        ("सात", 7),
        ("आठ", 8),
        ("नौ", 9),
        ("ग्यारह", 11),
        ("बारह", 12),
        ("तेरह", 13),
        ("चौदह", 14),
        ("पन्द्रह", 15),
        ("सोलह", 16),
        ("सत्रह", 17),
        ("अठारह", 18),
        ("उन्नीस", 19),
        ("इक्कीस", 21),
        ("बाईस", 22),
        ("तेईस", 23),
        ("चौबीस", 24),
        ("पच्चीस", 25),
        ("छब्बीस", 26),
        ("सत्ताईस", 27),
        ("अट्ठाईस", 28),
        ("उनतीस", 29),
        ("इकतीस", 31),
        ("इकत्तीस", 31),
        ("बत्तीस", 32),
        ("तैंतीस", 33),
        ("चौंतीस", 34),
        ("पैंतीस", 35),
        ("छत्तीस", 36),
        ("सैंतीस", 37),
        ("अड़तीस", 38),
        ("उनतालीस", 39),
        ("इकतालीस", 41),
        ("बयालीस", 42),
        ("तैंतालीस", 43),
        ("चौवालीस", 44),
        ("पैंतालीस", 45),
        ("छियालीस", 46),
        ("सैंतालीस", 47),
        ("अड़तालीस", 48),
        ("उनचास", 49),
        ("इक्यावन", 51),
        ("बावन", 52),
        ("तिरेपन", 53),
        ("चौवन", 54),
        ("पचपन", 55),
        ("छप्पन", 56),
        ("सत्तावन", 57),
        ("अट्ठावन", 58),
        ("उनसठ", 59),
        ("इकसठ", 61),
        ("बासठ", 62),
        ("तिरेसठ", 63),
        ("चौंसठ", 64),
        ("पैंसठ", 65),
        ("छियासठ", 66),
        ("सड़सठ", 67),
        ("अड़सठ", 68),
        ("उनहत्तर", 69),
        ("इकहत्तर", 71),
        ("बहत्तर", 72),
        ("तिहत्तर", 73),
        ("चौहत्तर", 74),
        ("पचहत्तर", 75),
        ("छिहत्तर", 76),
        ("सतहत्तर", 77),
        ("अठहत्तर", 78),
        ("उनासी", 79),
        ("इक्यासी", 81),
        ("बयासी", 82),
        ("तिरासी", 83),
        ("चौरासी", 84),
        ("पचासी", 85),
        ("छियासी", 86),
        ("सतासी", 87),
        ("अट्ठासी", 88),
        ("नवासी", 89),
        ("इक्यानवे", 91),
        ("बानवे", 92),
        ("तिरानवे", 93),
        ("चौरानवे", 94),
        ("पचानवे", 95),
        ("छियानवे", 96),
        ("सत्तानवे", 97),
        ("अट्ठानवे", 98),
        ("निन्यानवे", 99),
    ]
    .into_iter()
    .collect()
});

fn rule_zero_to_ninety_nine() -> Rule {
    rule(
        "number (0..99)",
        vec![regex("(शून्य|एक|दो|तीन|चार|पाँच|छे|छह|सात|आठ|नौ|ग्यारह|बारह|तेरह|चौदह|पन्द्रह|सोलह|सत्रह|अठारह|उन्नीस|इक्कीस|बाईस|तेईस|चौबीस|पच्चीस|छब्बीस|सत्ताईस|अट्ठाईस|उनतीस|इकतीस|बत्तीस|तैंतीस|चौंतीस|पैंतीस|छत्तीस|सैंतीस|अड़तीस|उनतालीस|इकतालीस|बयालीस|तैंतालीस|चौवालीस|पैंतालीस|छियालीस|सैंतालीस|अड़तालीस|उनचास|इक्यावन|बावन|तिरेपन|चौवन|पचपन|छप्पन|सत्तावन|अट्ठावन|उनसठ|इकसठ|बासठ|तिरेसठ|चौंसठ|पैंसठ|छियासठ|सड़सठ|अड़सठ|उनहत्तर|इकहत्तर|बहत्तर|तिहत्तर|चौहत्तर|पचहत्तर|छिहत्तर|सतहत्तर|अठहत्तर|उनासी|इक्यासी|बयासी|तिरासी|चौरासी|पचासी|छियासी|सतासी|अट्ठासी|नवासी|इक्यानवे|बानवे|तिरानवे|चौरानवे|पचानवे|छियानवे|सत्तानवे|अट्ठानवे|निन्यानवे)")],
        |ts| ZERO_TO_NINETY_NINE.get(groups(ts)?.first()?.as_ref()).copied().and_then(integer),
    )
}

static TENS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("दस", 10),
        ("बीस", 20),
        ("तीस", 30),
        ("चालीस", 40),
        ("पचास", 50),
        ("साठ", 60),
        ("सत्तर", 70),
        ("अस्सी", 80),
        ("नब्बे", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_tens() -> Rule {
    rule(
        "integer (20,30..90)",
        vec![regex("(दस|बीस|तीस|चालीस|पचास|साठ|सत्तर|अस्सी|नब्बे)")],
        |ts| {
            TENS.get(groups(ts)?.first()?.as_ref())
                .copied()
                .and_then(integer)
        },
    )
}

fn rule_powers_of_ten() -> Rule {
    rule(
        "powers of tens",
        vec![regex("(सौ|हज़ार|हज़ार)")],
        |ts| match to_lower(groups(ts)?.first()?).as_str() {
            "सौ" => double(1e2)
                .and_then(|t| with_grain(2, &t))
                .and_then(|t| with_multipliable(&t)),
            "हज़ार" => double(1e3)
                .and_then(|t| with_grain(3, &t))
                .and_then(|t| with_multipliable(&t)),
            "हज़ार" => double(1e3)
                .and_then(|t| with_grain(3, &t))
                .and_then(|t| with_multipliable(&t)),
            _ => None,
        },
    )
}

fn rule_multiply() -> Rule {
    rule(
        "compose by multiplication",
        vec![pred(is_positive), pred(is_multipliable)],
        |ts| multiply(ts.first()?, ts.get(1)?),
    )
}

fn rule_composite_hundreds() -> Rule {
    rule(
        "integer 100s..",
        vec![
            one_of((1..=50).map(|i| i as f64 * 100.0).collect()),
            regex("[\\s\\-]+"),
            pred(number_between(1.0, 99.0)),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(hundreds), Token::Numeral(remaining)) => {
                double(hundreds.value + remaining.value)
            }
            _ => None,
        },
    )
}

fn rule_composite_thousands() -> Rule {
    rule(
        "integer 100s..",
        vec![
            one_of((1..=50).map(|i| i as f64 * 1000.0).collect()),
            regex("[\\s\\-]+"),
            pred(number_between(1.0, 999.0)),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(thousands), Token::Numeral(remaining)) => {
                double(thousands.value + remaining.value)
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_devanagari(),
        rule_zero_to_ninety_nine(),
        rule_tens(),
        rule_powers_of_ten(),
        rule_multiply(),
        rule_composite_hundreds(),
        rule_composite_thousands(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
