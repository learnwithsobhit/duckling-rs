//! `Duckling.Numeral.AR.Rules` / `Duckling.Numeral.AR.Helpers` /
//! `Duckling.Numeral.AR.EG.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::{parse_int, parse_integer, to_lower};
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

/// `numeralToStringMap` / `parseArabicIntAsText`.
fn parse_arabic_int_as_text(s: &str) -> String {
    s.chars()
        .filter_map(|c| match c {
            '٠' => Some('0'),
            '١' => Some('1'),
            '٢' => Some('2'),
            '٣' => Some('3'),
            '٤' => Some('4'),
            '٥' => Some('5'),
            '٦' => Some('6'),
            '٧' => Some('7'),
            '٨' => Some('8'),
            '٩' => Some('9'),
            _ => None,
        })
        .collect()
}

/// `parseArabicIntegerFromText`.
fn parse_arabic_integer_from_text(s: &str) -> Option<i64> {
    parse_int(&parse_arabic_int_as_text(s))
}

fn parse_arabic_integer_from_text_i128(s: &str) -> Option<i128> {
    parse_integer(&parse_arabic_int_as_text(s))
}

/// `parseArabicDoubleAsText`.
fn parse_arabic_double_as_text(s: &str) -> String {
    s.chars()
        .filter_map(|c| match c {
            '٠' => Some('0'),
            '١' => Some('1'),
            '٢' => Some('2'),
            '٣' => Some('3'),
            '٤' => Some('4'),
            '٥' => Some('5'),
            '٦' => Some('6'),
            '٧' => Some('7'),
            '٨' => Some('8'),
            '٩' => Some('9'),
            '٫' => Some('.'),
            _ => None,
        })
        .collect()
}

/// `parseArabicDoubleFromText`.
fn parse_arabic_double_from_text(s: &str) -> Option<f64> {
    parse_double(&parse_arabic_double_as_text(s))
}

pub(crate) fn parse_arabic_int_as_text_pub(s: &str) -> String {
    parse_arabic_int_as_text(s)
}

pub(crate) fn parse_arabic_integer_from_text_i128_pub(s: &str) -> Option<i128> {
    parse_arabic_integer_from_text_i128(s)
}

/// `Duckling.Numeral.AR.Helpers.digitsMap`.
static DIGITS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("عشر", 2),
        ("ثلاث", 3),
        ("اربع", 4),
        ("أربع", 4),
        ("خمس", 5),
        ("ست", 6),
        ("سبع", 7),
        ("ثمان", 8),
        ("تسع", 9),
    ]
    .into_iter()
    .collect()
});

/// `Duckling.Numeral.AR.EG.Helpers.digitsMap`.
static EG_DIGITS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("اتنين", 2),
        ("تلات", 3),
        ("اربع", 4),
        ("أربع", 4),
        ("خمس", 5),
        ("ست", 6),
        ("سبع", 7),
        ("تمان", 8),
        ("تسع", 9),
    ]
    .into_iter()
    .collect()
});

fn sum2(ts: &[&Token]) -> Option<Token> {
    match (ts.first()?, ts.get(2)?) {
        (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
        _ => None,
    }
}

fn rule_integer_numeric() -> Rule {
    rule("Arabic integer numeric", vec![regex("([٠-٩]{1,18})")], |ts| {
        parse_arabic_integer_from_text(groups(ts)?.first()?).and_then(integer)
    })
}

fn rule_fractions_numeric() -> Rule {
    rule("Arabic fractional number numeric", vec![regex("([٠-٩]+)/([٠-٩]+)")], |ts| {
        let g = groups(ts)?;
        let n = parse_arabic_integer_from_text(g.first()?).and_then(integer)?;
        let d = parse_arabic_integer_from_text(g.get(1)?).and_then(integer)?;
        divide(&n, &d).and_then(|t| not_ok_for_any_time(&t))
    })
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex(r"(\d*\.\d+)")], |ts| {
        parse_decimal(true, groups(ts)?.first()?)
    })
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule("decimal with thousands separator", vec![regex(r"(\d+(,\d\d\d)+\.\d+)")], |ts| {
        let m = groups(ts)?.first()?.replace(',', "");
        parse_double(&m).and_then(double)
    })
}

fn rule_integer_0() -> Rule {
    rule("integer 0", vec![regex("صفر")], |_| integer(0))
}

fn rule_integer_7() -> Rule {
    rule("integer 7", vec![regex("سبع[ةه]?")], |_| integer(7))
}

fn rule_integer_8() -> Rule {
    rule("integer 8", vec![regex("ثما??ني?[ةه]?")], |_| integer(8))
}

fn rule_integer_9() -> Rule {
    rule("integer 9", vec![regex("تسع[ةه]?")], |_| integer(9))
}

fn rule_integer_10() -> Rule {
    rule("integer 10", vec![regex("عشر[ةه]?")], |_| integer(10))
}

fn rule_integer_11() -> Rule {
    rule("integer 11", vec![regex("([إاأ]حد[يى]? عشر[ةه]?)")], |_| integer(11))
}

fn rule_integer_12() -> Rule {
    rule("integer 12", vec![regex("([إا]?ثن(ت)?[يىا] ?عشر[ةه]?)")], |_| integer(12))
}

fn rule_integer_20_90() -> Rule {
    rule("integer (20..90)", vec![regex("(عشر|ثلاث|[أا]ربع|خمس|ست|سبع|ثمان|تسع)(ون|ين)")], |ts| {
        DIGITS_MAP.get(groups(ts)?.first()?.as_ref()).and_then(|v| integer(*v * 10))
    })
}

fn rule_integer_1() -> Rule {
    rule("integer 1", vec![regex("واحد[ةه]?")], |_| integer(1))
}

fn rule_integer_13_19() -> Rule {
    rule(
        "integer (13..19)",
        vec![pred(number_between(3.0, 10.0)), number_with(|n| n.value, |v| v == 10.0)],
        |ts| match ts.first()? {
            Token::Numeral(n) => double(n.value + 10.0),
            _ => None,
        },
    )
}

fn rule_integer_21_99() -> Rule {
    rule(
        "integer 21..99",
        vec![pred(number_between(1.0, 10.0)), regex("و"), one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0])],
        sum2,
    )
}

fn rule_integer_101_999() -> Rule {
    rule(
        "integer 101..999",
        vec![
            one_of(vec![100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0]),
            regex("و"),
            pred(number_between(1.0, 100.0)),
        ],
        sum2,
    )
}

fn rule_integer_2() -> Rule {
    rule("integer 2", vec![regex("[إا]ثنت?[اي]ن")], |_| integer(2))
}

fn rule_integer_3() -> Rule {
    rule("integer 3", vec![regex("(ثلاث[ةه]?)")], |_| integer(3))
}

fn rule_integer_4() -> Rule {
    rule("integer 4", vec![regex("([أا]ربع[ةه]?)")], |_| integer(4))
}

fn rule_integer_5() -> Rule {
    rule("integer 5", vec![regex("خمس[ةه]?")], |_| integer(5))
}

fn rule_integer_6() -> Rule {
    rule("integer 6", vec![regex("ست[ةه]?")], |_| integer(6))
}

fn hundreds(name: &str, re: &str, n: i64) -> Rule {
    rule(name, vec![regex(re)], move |_| integer(n))
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule("integer with thousands separator ,", vec![regex(r"(\d{1,3}(,\d\d\d){1,5})")], |ts| {
        let m = groups(ts)?.first()?.replace(',', "");
        parse_double(&m).and_then(double)
    })
}

fn rule_multiply() -> Rule {
    rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
        multiply(ts.first()?, ts.get(1)?)
    })
}

fn rule_numeral_dot_numeral() -> Rule {
    rule(
        "number dot number",
        vec![dimension(Dim::Numeral), regex("فاصل[ةه]"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + decimals_to_double(b.value)),
            _ => None,
        },
    )
}

fn rule_numerals_prefix_with_minus() -> Rule {
    rule("numbers prefix with -, minus", vec![regex("-"), dimension(Dim::Numeral)], |ts| {
        match ts.get(1)? {
            Token::Numeral(n) => double(n.value * -1.0),
            _ => None,
        }
    })
}

fn rule_powers_of_ten() -> Rule {
    rule(
        "powers of tens",
        vec![regex("(ما?[ئي][ةه]|مئت(ان|ين)|مئات|[أا]لف(ان|ين)?|[آا]لاف|ملايين|مليون(ان|ين)?)")],
        |ts| match to_lower(groups(ts)?.first()?).as_str() {
            "مئة" | "مئه" | "مائة" | "مائه" | "مئات" => {
                double(1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t))
            }
            "مئتين" | "مئتان" => double(2e2).and_then(|t| with_grain(2, &t)),
            "ألف" | "الف" => {
                double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t))
            }
            "الفين" | "الفان" => double(2e3).and_then(|t| with_grain(3, &t)),
            "الاف" | "آلاف" => {
                double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t))
            }
            "ملايين" | "مليون" => {
                double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t))
            }
            "مليونين" | "مليونان" => double(2e6).and_then(|t| with_grain(6, &t)),
            _ => None,
        },
    )
}

fn rule_integer_200() -> Rule {
    rule("integer (200)", vec![regex("مائتان|مائتين")], |_| integer(200))
}

fn rule_arabic_decimal() -> Rule {
    rule(
        "Arabic decimal number with Arabic decimal separator",
        vec![regex("([٠-٩]*٫[٠-٩]+)")],
        |ts| parse_arabic_double_from_text(groups(ts)?.first()?).and_then(double),
    )
}

fn rule_arabic_commas() -> Rule {
    rule(
        "Arabic number with commas and optional Arabic decimal separator",
        vec![regex("([٠-٩]{1,3}(٬[٠-٩]{3}){1,5}(٫[٠-٩]+)?)")],
        |ts| {
            let m = groups(ts)?.first()?.replace('٬', "");
            parse_arabic_double_from_text(&m).and_then(double)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_integer_numeric(),
        rule_fractions_numeric(),
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_integer_0(),
        rule_integer_7(),
        rule_integer_8(),
        rule_integer_9(),
        rule_integer_10(),
        rule_integer_11(),
        rule_integer_12(),
        rule_integer_20_90(),
        rule_integer_1(),
        rule_integer_13_19(),
        rule_integer_21_99(),
        rule_integer_101_999(),
        rule_integer_2(),
        rule_integer_3(),
        rule_integer_4(),
        rule_integer_5(),
        rule_integer_6(),
        hundreds("integer 300", "(ثلاث)ما?[ئي][ةه]", 300),
        hundreds("integer 400", "([أا]ربع)ما?[ئي][ةه]", 400),
        hundreds("integer 500", "(خمس)ما?[ئي][ةه]", 500),
        hundreds("integer 600", "(ست)ما?[ئي][ةه]", 600),
        hundreds("integer 700", "(سبع)ما?[ئي][ةه]", 700),
        hundreds("integer 800", "(ثمان[ي]?)ما?[ئي][ةه]", 800),
        hundreds("integer 900", "(تسع)ما?[ئي][ةه]", 900),
        rule_integer_with_thousands_separator(),
        rule_multiply(),
        rule_numeral_dot_numeral(),
        rule_numerals_prefix_with_minus(),
        rule_powers_of_ten(),
        rule_integer_200(),
        rule_arabic_decimal(),
        rule_arabic_commas(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}

fn eg_101_999_ones() -> Vec<f64> {
    let mut v = Vec::new();
    for t in [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0] {
        for o in 1..=9 {
            v.push(t + o as f64);
        }
    }
    v
}

fn eg_fixed(name: &str, re: &str, n: i64) -> Rule {
    rule(name, vec![regex(re)], move |_| integer(n))
}

static EG_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        eg_fixed("integer 2", "[إأا]?تني[ي]?ن", 2),
        eg_fixed("integer 3", "تلات[هة]?", 3),
        eg_fixed("integer 8", "تمان(ي[هة])?", 8),
        eg_fixed("integer 11", "(إأا)?حداشر", 11),
        eg_fixed("integer 12", "[إأا]?[تط]ناشر", 12),
        eg_fixed("integer 13", "[تط]ل(ا)?[تط]اشر", 13),
        eg_fixed("integer 14", "[إأا]ربع[تط]اشر", 14),
        eg_fixed("integer 15", "خمس[تط]اشر", 15),
        eg_fixed("integer 16", "س[تط]اشر", 16),
        eg_fixed("integer 17", "سبع[تط]اشر", 17),
        eg_fixed("integer 18", "[تط]من[تط]اشر", 18),
        eg_fixed("integer 19", "تسع[تط]اشر", 19),
        rule("integer (30, 80)", vec![regex("([تط]لا[تط]|[تط]مان)(ين)")], |ts| {
            EG_DIGITS_MAP.get(groups(ts)?.first()?.as_ref()).and_then(|v| integer(*v * 10))
        }),
        eg_fixed("integer (100)", "مي[هةت]", 100),
        eg_fixed("integer (200)", "م(ي)?تين", 200),
        eg_fixed("integer (300)", "([تط]و?لا?[تط]و?)مي[هةت]", 300),
        eg_fixed("integer (400)", "(رو?بعو?)مي[هةت]", 400),
        eg_fixed("integer (500)", "(خو?مسو?)مي[هةت]", 500),
        eg_fixed("integer (600)", "(سو?تو?)مي[هةت]", 600),
        eg_fixed("integer (700)", "(سو?بعو?)مي[هةت]", 700),
        eg_fixed("integer (800)", "([تط]و?منو?)مي[هةت]", 800),
        eg_fixed("integer (900)", "([تط]و?سعو?)مي[هةت]", 900),
        rule(
            "integer 101..999",
            vec![
                one_of(vec![100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0]),
                regex(r"\s"),
                one_of(eg_101_999_ones()),
            ],
            sum2,
        ),
        eg_fixed("integer (3000)", "([تط]لا?[تط])( ?[اآ])?لاف", 3000),
        eg_fixed("integer (4000)", "[أا]ربع(ت| ?[اآ])لاف", 4000),
        eg_fixed("integer (5000)", "خمس(ت| ?[اآ])لاف", 5000),
        eg_fixed("integer (6000)", "س(ت| ?[اآ])لاف", 6000),
        eg_fixed("integer (7000)", "سبع(ت| ?[اآ])لاف", 7000),
        eg_fixed("integer (8000)", "[تط]م[ا]?ن(ت| ?[اآ])لاف", 8000),
        eg_fixed("integer (9000)", "([تط]سع)(ت| ?[اآ])لاف", 9000),
    ]
});

pub fn eg_rules() -> &'static [Rule] {
    &EG_RULES
}
