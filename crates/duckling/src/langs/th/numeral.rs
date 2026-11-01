//! `Duckling.Numeral.TH.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{
    dimension, groups, pred, regex, rule, single_string_lookup_rule, Dim, Rule, Token,
};

fn first_group(t: &Token) -> Option<String> {
    match t {
        Token::RegexMatch(g) => Some(to_lower(g.0.first()?)),
        _ => None,
    }
}

fn rule_dozen() -> Rule {
    rule("a dozen of", vec![regex("โหล?( ของ)?")], |_| {
        integer(12).and_then(|t| with_multipliable(&t)).and_then(|t| not_ok_for_any_time(&t))
    })
}

static ZERO_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ไม่มี", 0),
        ("ศูนย์", 0),
        ("หนึ่ง", 1),
        ("เอ็ด", 1),
        ("สอง", 2),
        ("สาม", 3),
        ("สี่", 4),
        ("ห้า", 5),
        ("หก", 6),
        ("เจ็ด", 7),
        ("แปด", 8),
        ("เก้า", 9),
        ("สิบ", 10),
        ("สิบเอ็ด", 11),
        ("สิบหนึ่ง", 11),
        ("สิบสอง", 12),
        ("สิบสาม", 13),
        ("สิบสี่", 14),
        ("สิบห้า", 15),
        ("สิบหก", 16),
        ("สิบเจ็ด", 17),
        ("สิบแปด", 18),
        ("สิบเก้า", 19),
    ]
    .into_iter()
    .collect()
});

static INFORMAL: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [("อันนึง", 1), ("คู่นึง", 2), ("คู่ของ", 2)].into_iter().collect()
});

fn rule_to_nineteen() -> Rule {
    rule(
        "integer (0..19)",
        // e.g. fourteen must be before four, otherwise four will always shadow fourteen
        vec![regex(
            "(ไม่มี|ศูนย์|สิบหนึ่ง|หนึ่ง|(คู่)s?( ของ)?|(คู่)s?( นึง)?|สิบเอ็ด|เอ็ด|สิบสอง|สิบสาม|สิบสี่|สิบห้า|สิบหก|สิบเจ็ด|สิบแปด|สิบเก้า|สอง|สาม|สี่|ห้า|หก|เจ็ด|แปด|เก้า|สิบ)",
        )],
        |ts| {
            let x = to_lower(groups(ts)?.first()?);
            ZERO_NINETEEN
                .get(x.as_str())
                .and_then(|v| integer(*v))
                .or_else(|| {
                    INFORMAL.get(x.as_str()).and_then(|v| integer(*v)).and_then(|t| not_ok_for_any_time(&t))
                })
        },
    )
}

const TENS: &[(&str, i64)] = &[
    ("ยี่สิบ", 20),
    ("สามสิบ", 30),
    ("สี่สิบ", 40),
    ("ห้าสิบ", 50),
    ("หกสิบ", 60),
    ("เจ็ดสิบ", 70),
    ("แปดสิบ", 80),
    ("เก้าสิบ", 90),
];

fn rule_tens() -> Rule {
    single_string_lookup_rule(TENS, "integer (20..90)", |v| integer(*v))
}

const HUNDRED_TWENTY_TO_TWENTY_NINE: &[(&str, i64)] = &[
    ("ร้อยยี่สิบ", 120),
    ("ร้อยยี่สิบเอ็ด", 121),
    ("ร้อยยี่สิบหนึ่ง", 121),
    ("ร้อยยี่สิบสอง", 122),
    ("ร้อยยี่สิบสาม", 123),
    ("ร้อยยี่สิบสี่", 124),
    ("ร้อยยี่สิบห้า", 125),
    ("ร้อยยี่สิบหก", 126),
    ("ร้อยยี่สิบเจ็ด", 127),
    ("ร้อยยี่สิบแปด", 128),
    ("ร้อยยี่สิบเก้า", 129),
    ("หนึ่งร้อยยี่สิบ", 120),
    ("หนึ่งร้อยยี่สิบเอ็ด", 121),
    ("หนึ่งร้อยยี่สิบหนึ่ง", 121),
    ("หนึ่งร้อยยี่สิบสอง", 122),
    ("หนึ่งร้อยยี่สิบสาม", 123),
    ("หนึ่งร้อยยี่สิบสี่", 124),
    ("หนึ่งร้อยยี่สิบห้า", 125),
    ("หนึ่งร้อยยี่สิบหก", 126),
    ("หนึ่งร้อยยี่สิบเจ็ด", 127),
    ("หนึ่งร้อยยี่สิบแปด", 128),
    ("หนึ่งร้อยยี่สิบเก้า", 129),
    ("สองร้อยยี่สิบ", 220),
    ("สองร้อยยี่สิบเอ็ด", 221),
    ("สองร้อยยี่สิบหนึ่ง", 221),
    ("สองร้อยยี่สิบสอง", 222),
    ("สองร้อยยี่สิบสาม", 223),
    ("สองร้อยยี่สิบสี่", 224),
    ("สองร้อยยี่สิบห้า", 225),
    ("สองร้อยยี่สิบหก", 226),
    ("สองร้อยยี่สิบเจ็ด", 227),
    ("สองร้อยยี่สิบแปด", 228),
    ("สองร้อยยี่สิบเก้า", 229),
    ("สามร้อยยี่สิบ", 320),
    ("สามร้อยยี่สิบเอ็ด", 321),
    ("สามร้อยยี่สิบหนึ่ง", 321),
    ("สามร้อยยี่สิบสอง", 322),
    ("สามร้อยยี่สิบสาม", 323),
    ("สามร้อยยี่สิบสี่", 324),
    ("สามร้อยยี่สิบห้า", 325),
    ("สามร้อยยี่สิบหก", 326),
    ("สามร้อยยี่สิบเจ็ด", 327),
    ("สามร้อยยี่สิบแปด", 328),
    ("สามร้อยยี่สิบเก้า", 329),
    ("สี่ร้อยยี่สิบ", 420),
    ("สี่ร้อยยี่สิบเอ็ด", 421),
    ("สี่ร้อยยี่สิบหนึ่ง", 421),
    ("สี่ร้อยยี่สิบสอง", 422),
    ("สี่ร้อยยี่สิบสาม", 423),
    ("สี่ร้อยยี่สิบสี่", 424),
    ("สี่ร้อยยี่สิบห้า", 425),
    ("สี่ร้อยยี่สิบหก", 426),
    ("สี่ร้อยยี่สิบเจ็ด", 427),
    ("สี่ร้อยยี่สิบแปด", 428),
    ("สี่ร้อยยี่สิบเก้า", 429),
    ("ห้าร้อยยี่สิบ", 520),
    ("ห้าร้อยยี่สิบเอ็ด", 521),
    ("ห้าร้อยยี่สิบหนึ่ง", 521),
    ("ห้าร้อยยี่สิบสอง", 522),
    ("ห้าร้อยยี่สิบสาม", 523),
    ("ห้าร้อยยี่สิบสี่", 524),
    ("ห้าร้อยยี่สิบห้า", 525),
    ("ห้าร้อยยี่สิบหก", 526),
    ("ห้าร้อยยี่สิบเจ็ด", 527),
    ("ห้าร้อยยี่สิบแปด", 528),
    ("ห้าร้อยยี่สิบเก้า", 529),
    ("หกร้อยยี่สิบ", 620),
    ("หกร้อยยี่สิบเอ็ด", 621),
    ("หกร้อยยี่สิบหนึ่ง", 621),
    ("หกร้อยยี่สิบสอง", 622),
    ("หกร้อยยี่สิบสาม", 623),
    ("หกร้อยยี่สิบสี่", 624),
    ("หกร้อยยี่สิบห้า", 625),
    ("หกร้อยยี่สิบหก", 626),
    ("หกร้อยยี่สิบเจ็ด", 627),
    ("หกร้อยยี่สิบแปด", 628),
    ("หกร้อยยี่สิบเก้า", 629),
    ("เจ็ดร้อยยี่สิบ", 720),
    ("เจ็ดร้อยยี่สิบเอ็ด", 721),
    ("เจ็ดร้อยยี่สิบหนึ่ง", 721),
    ("เจ็ดร้อยยี่สิบสอง", 722),
    ("เจ็ดร้อยยี่สิบสาม", 723),
    ("เจ็ดร้อยยี่สิบสี่", 724),
    ("เจ็ดร้อยยี่สิบห้า", 725),
    ("เจ็ดร้อยยี่สิบหก", 726),
    ("เจ็ดร้อยยี่สิบเจ็ด", 727),
    ("เจ็ดร้อยยี่สิบแปด", 728),
    ("เจ็ดร้อยยี่สิบเก้า", 729),
    ("แปดร้อยยี่สิบ", 820),
    ("แปดร้อยยี่สิบเอ็ด", 821),
    ("แปดร้อยยี่สิบหนึ่ง", 821),
    ("แปดร้อยยี่สิบสอง", 822),
    ("แปดร้อยยี่สิบสาม", 823),
    ("แปดร้อยยี่สิบสี่", 824),
    ("แปดร้อยยี่สิบห้า", 825),
    ("แปดร้อยยี่สิบหก", 826),
    ("แปดร้อยยี่สิบเจ็ด", 827),
    ("แปดร้อยยี่สิบแปด", 828),
    ("แปดร้อยยี่สิบเก้า", 829),
    ("เก้าร้อยยี่สิบ", 920),
    ("เก้าร้อยยี่สิบเอ็ด", 921),
    ("เก้าร้อยยี่สิบหนึ่ง", 921),
    ("เก้าร้อยยี่สิบสอง", 922),
    ("เก้าร้อยยี่สิบสาม", 923),
    ("เก้าร้อยยี่สิบสี่", 924),
    ("เก้าร้อยยี่สิบห้า", 925),
    ("เก้าร้อยยี่สิบหก", 926),
    ("เก้าร้อยยี่สิบเจ็ด", 927),
    ("เก้าร้อยยี่สิบแปด", 928),
    ("เก้าร้อยยี่สิบเก้า", 929),
];

fn rule_x_hundred_twenty_to_x_hundred_twenty_nine() -> Rule {
    single_string_lookup_rule(HUNDRED_TWENTY_TO_TWENTY_NINE, "integer (x20,x21,...,x29)", |v| {
        integer(*v)
    })
}

fn rule_powers_of_ten() -> Rule {
    rule(
        "powers of tens",
        vec![regex("(ร้อย|พัน|หมื่น|แสน|ล้าน|สิบล้าน|ร้อยล้าน|พันล้าน)")],
        |ts| {
            let (value, grain) = match to_lower(groups(ts)?.first()?).as_str() {
                "ร้อย" => (1e2, 2),
                "พัน" => (1e3, 3),
                "หมื่น" => (1e4, 4),
                "แสน" => (1e5, 5),
                "ล้าน" => (1e6, 6),
                "สิบล้าน" => (1e7, 7),
                "ร้อยล้าน" => (1e8, 8),
                "พันล้าน" => (1e9, 9),
                _ => return None,
            };
            double(value).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
        },
    )
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer 21..99",
        vec![
            one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
            regex("[\\s\\-]+"),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(tens), Token::Numeral(units)) => double(tens.value + units.value),
            _ => None,
        },
    )
}

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| TENS.iter().copied().collect());

fn rule_sum_ten_digits() -> Rule {
    rule(
        "สามสิบสี่",
        vec![
            regex("(ยี่สิบ|สามสิบ|สี่สิบ|ห้าสิบ|หกสิบ|เจ็ดสิบ|แปดสิบ|เก้าสิบ)"),
            regex("(หนึ่ง|เอ็ด|สอง|สาม|สี่|ห้า|หก|เจ็ด|แปด|เก้า|สิบ)"),
        ],
        |ts| {
            let x1 = first_group(ts.first()?)?;
            let x2 = first_group(ts.get(1)?)?;
            let hundreds = TENS_MAP.get(x1.as_str())?;
            let rest = ZERO_NINETEEN.get(x2.as_str())?;
            integer(hundreds + rest)
        },
    )
}

fn rule_skip_hundreds1() -> Rule {
    rule(
        "one eleven",
        vec![
            regex("(หนึ่ง|สอง|สาม|สี่|ห้า|หก|เจ็ด|แปด|เก้า)"),
            regex("(สิบ|สิบเอ็ด|สิบสอง|สิบสาม|สิบสี่|สิบห้า|สิบหก|สิบเจ็ด|สิบแปด|สิบเก้า|ยี่สิบ|สามสิบ|สี่สิบ|ห้าสิบ|หกสิบ|เจ็ดสิบ|แปดสิบ|เก้าสิบ)"),
        ],
        |ts| {
            let x1 = first_group(ts.first()?)?;
            let x2 = first_group(ts.get(1)?)?;
            let hundreds = ZERO_NINETEEN.get(x1.as_str())?;
            let rest = ZERO_NINETEEN.get(x2.as_str()).or_else(|| TENS_MAP.get(x2.as_str()))?;
            integer(hundreds * 100 + rest)
        },
    )
}

fn rule_skip_hundreds2() -> Rule {
    rule(
        "one twenty two",
        vec![
            regex("(หนึ่ง|สอง|สาม|สี่|ห้า|หก|เจ็ด|แปด|เก้า)"),
            regex("(ยี่สิบ|สามสิบ|สี่สิบ|ห้าสิบ|หกสิบ|เจ็ดสิบ|แปดสิบ|เก้าสิบ)"),
            regex("(หนึ่ง|สอง|สาม|สี่|ห้า|หก|เจ็ด|แปด|เก้า)"),
        ],
        |ts| {
            let x1 = first_group(ts.first()?)?;
            let x2 = first_group(ts.get(1)?)?;
            let x3 = first_group(ts.get(2)?)?;
            let hundreds = ZERO_NINETEEN.get(x1.as_str())?;
            let tens = TENS_MAP.get(x2.as_str())?;
            let rest = ZERO_NINETEEN.get(x3.as_str())?;
            integer(hundreds * 100 + tens + rest)
        },
    )
}

fn rule_dot_spelled_out() -> Rule {
    rule(
        "one point 2",
        vec![dimension(Dim::Numeral), regex("จุด"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(nd1), Token::Numeral(nd2)) => {
                double(nd1.value + decimals_to_double(nd2.value))
            }
            _ => None,
        },
    )
}

fn rule_leading_dot_spelled_out() -> Rule {
    rule("point 77", vec![regex("จุด"), pred(|t| !has_grain(t))], |ts| match ts.get(1)? {
        Token::Numeral(nd) => double(decimals_to_double(nd.value)),
        _ => None,
    })
}

fn rule_decimals() -> Rule {
    rule("decimal number", vec![regex("(\\d*\\.\\d+)")], |ts| {
        parse_decimal(true, groups(ts)?.first()?)
    })
}

fn rule_commas() -> Rule {
    rule("comma-separated numbers", vec![regex("(\\d+(,\\d\\d\\d)+(\\.\\d+)?)")], |ts| {
        parse_double(&groups(ts)?.first()?.replace(',', "")).and_then(double)
    })
}

fn rule_suffixes() -> Rule {
    rule(
        "suffixes (กิโลกรัม,กรัม))",
        vec![dimension(Dim::Numeral), regex("(กิโลกรัม|กรัม)(?=[\\W$€¢£]|$)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(nd), Token::RegexMatch(g)) => {
                let x = match to_lower(g.0.first()?).as_str() {
                    "กิโลกรัม" => 1e3,
                    "กรัม" => 1e1,
                    _ => return None,
                };
                double(nd.value * x)
            }
            _ => None,
        },
    )
}

fn rule_negative() -> Rule {
    rule(
        "negative numbers",
        vec![regex("(-|ลบ)(?!\\s*-)"), pred(is_positive)],
        |ts| match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * -1.0),
            _ => None,
        },
    )
}

fn sum_with(ts: &[&Token], second: usize) -> Option<Token> {
    match (ts.first()?, ts.get(second)?) {
        (Token::Numeral(a), Token::Numeral(b)) => {
            let g = a.grain?;
            if 10f64.powf(g as f64) > b.value {
                double(a.value + b.value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn grain_and_positive(t: &Token) -> bool {
    has_grain(t) && is_positive(t)
}

fn not_multipliable_and_positive(t: &Token) -> bool {
    !is_multipliable(t) && is_positive(t)
}

fn rule_sum() -> Rule {
    rule(
        "intersect 2 numbers",
        vec![pred(grain_and_positive), pred(not_multipliable_and_positive)],
        |ts| sum_with(ts, 1),
    )
}

fn rule_sum_and() -> Rule {
    rule(
        "intersect 2 numbers (with and)",
        vec![pred(grain_and_positive), regex("และ"), pred(not_multipliable_and_positive)],
        |ts| sum_with(ts, 2),
    )
}

fn rule_multiply() -> Rule {
    rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
        multiply(ts.first()?, ts.get(1)?)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_x_hundred_twenty_to_x_hundred_twenty_nine(),
        rule_skip_hundreds1(),
        rule_skip_hundreds2(),
        rule_to_nineteen(),
        rule_tens(),
        rule_powers_of_ten(),
        rule_composite_tens(),
        rule_sum_ten_digits(),
        rule_dot_spelled_out(),
        rule_leading_dot_spelled_out(),
        rule_decimals(),
        rule_commas(),
        rule_suffixes(),
        rule_negative(),
        rule_sum(),
        rule_sum_and(),
        rule_multiply(),
        rule_dozen(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
