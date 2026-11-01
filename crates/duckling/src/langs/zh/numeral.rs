//! `Duckling.Numeral.ZH.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

static INTEGER_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("〇", 0),
        ("零", 0),
        ("一", 1),
        ("壹", 1),
        ("兩", 2),
        ("两", 2),
        ("二", 2),
        ("貳", 2),
        ("三", 3),
        ("參", 3),
        ("四", 4),
        ("肆", 4),
        ("五", 5),
        ("伍", 5),
        ("六", 6),
        ("陸", 6),
        ("七", 7),
        ("柒", 7),
        ("八", 8),
        ("捌", 8),
        ("九", 9),
        ("玖", 9),
        ("十", 10),
        ("拾", 10),
    ]
    .into_iter()
    .collect()
});

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [("廿", 20), ("卅", 30), ("卌", 40)].into_iter().collect()
});

fn rule_integer() -> Rule {
    rule(
        "integer (0..10)",
        vec![regex("(〇|零|一|二|两|兩|三|四|五|六|七|八|九|十|壹|貳|參|肆|伍|陸|柒|捌|玖|拾)")],
        |ts| {
            let g = groups(ts)?;
            INTEGER_MAP.get(g.first()?.as_ref()).and_then(|v| integer(*v))
        },
    )
}

fn rule_tens() -> Rule {
    rule("integer (20,30,40)", vec![regex("(廿|卅|卌)")], |ts| {
        let g = groups(ts)?;
        TENS_MAP.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
    })
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer 21..49",
        vec![one_of(vec![20.0, 30.0, 40.0]), regex("[\\s\\-]+"), pred(number_between(1.0, 10.0))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(tens), Token::Numeral(units)) => double(tens.value + units.value),
            _ => None,
        },
    )
}

fn rule_numerals_prefix_with_negative_or_minus() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex("-|负|負"), pred(is_positive)],
        |ts| match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * -1.0),
            _ => None,
        },
    )
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule("decimal with thousands separator", vec![regex("(\\d+(,\\d\\d\\d)+\\.\\d+)")], |ts| {
        let g = groups(ts)?;
        parse_double(&g.first()?.replace(',', "")).and_then(double)
    })
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex("(\\d*\\.\\d+)")], |ts| {
        parse_decimal(true, groups(ts)?.first()?)
    })
}

fn rule_dot_spelled_out() -> Rule {
    rule(
        "one point 2",
        vec![dimension(Dim::Numeral), regex("點"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(nd1), Token::Numeral(nd2)) => {
                double(nd1.value + decimals_to_double(nd2.value))
            }
            _ => None,
        },
    )
}

fn rule_fraction() -> Rule {
    rule(
        "fraction",
        vec![dimension(Dim::Numeral), regex("分之|分|份"), dimension(Dim::Numeral)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(n1), Token::Numeral(n2)) => double(n2.value / n1.value),
            _ => None,
        },
    )
}

fn rule_mixed_fraction() -> Rule {
    rule(
        " mixed fraction",
        vec![
            dimension(Dim::Numeral),
            regex("又"),
            dimension(Dim::Numeral),
            regex("分之|分|份"),
            dimension(Dim::Numeral),
        ],
        |ts| match (ts.first()?, ts.get(2)?, ts.get(4)?) {
            (Token::Numeral(n1), Token::Numeral(n2), Token::Numeral(n3)) => {
                double(n3.value / n2.value + n1.value)
            }
            _ => None,
        },
    )
}

fn rule_numeral() -> Rule {
    rule("<number>个/個", vec![dimension(Dim::Numeral), regex("个|個")], |ts| {
        ts.first().map(|t| (*t).clone())
    })
}

fn rule_half() -> Rule {
    rule("half", vec![regex("(1|一)?半(半|个|個)?")], |_| {
        double(0.5).and_then(|t| with_multipliable(&t))
    })
}

fn rule_dozen() -> Rule {
    rule("a dozen of", vec![regex("打")], |_| {
        integer(12).and_then(|t| with_multipliable(&t)).and_then(|t| not_ok_for_any_time(&t))
    })
}

fn rule_pair() -> Rule {
    rule("a pair", vec![regex("雙|對")], |_| {
        integer(2).and_then(|t| with_multipliable(&t)).and_then(|t| not_ok_for_any_time(&t))
    })
}

fn numeral_suffix(name_suffix: &str, value: f64, grain: i64) -> Rule {
    rule(&format!("number suffix: {name_suffix}"), vec![regex(name_suffix)], move |_| {
        double(value).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
    })
}

fn rule_multiply() -> Rule {
    rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
        multiply(ts.first()?, ts.get(1)?)
    })
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule("integer with thousands separator ,", vec![regex("(\\d{1,3}(,\\d\\d\\d){1,5})")], |ts| {
        let fmt = groups(ts)?.first()?.replace(',', "");
        parse_double(&fmt).and_then(double)
    })
}

fn rule_numerals_intersect_nonconsecutive_unit() -> Rule {
    rule(
        "integer with nonconsecutive unit modifiers",
        vec![pred(is_positive), regex("零|〇"), pred(is_positive)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(n1), Token::Numeral(n2)) => {
                let d = diff_integer_digits(n1.value, n2.value);
                if d <= 1 {
                    None
                } else {
                    double(n1.value + n2.value)
                }
            }
            _ => None,
        },
    )
}

fn rule_numerals_intersect_consecutive_unit() -> Rule {
    rule(
        "integer with consecutive unit modifiers",
        vec![pred(is_positive), pred(is_positive)],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(n1), Token::Numeral(n2)) => {
                let d = diff_integer_digits(n1.value, n2.value);
                if d == 1 {
                    double(n1.value + n2.value)
                } else {
                    None
                }
            }
            _ => None,
        },
    )
}

fn rule_hundred_prefix() -> Rule {
    rule(
        "one hundred and <integer> (short form)",
        vec![regex("百|佰"), pred(number_between(1.0, 10.0))],
        |ts| match ts.get(1)? {
            Token::Numeral(n) => double(100.0 + n.value * 10.0),
            _ => None,
        },
    )
}

fn rule_thousand_prefix() -> Rule {
    rule(
        "one thousand and <integer> (short form)",
        vec![regex("千|仟"), pred(number_between(1.0, 10.0))],
        |ts| match ts.get(1)? {
            Token::Numeral(n) => double(1000.0 + n.value * 100.0),
            _ => None,
        },
    )
}

fn rule_ten_thousand_prefix() -> Rule {
    rule(
        "ten thousand and <integer> (short form)",
        vec![regex("万|萬"), pred(number_between(1.0, 10.0))],
        |ts| match ts.get(1)? {
            Token::Numeral(n) => double(10000.0 + n.value * 1000.0),
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = vec![
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_integer(),
        rule_integer_with_thousands_separator(),
        rule_numeral(),
        rule_numerals_intersect_consecutive_unit(),
        rule_numerals_intersect_nonconsecutive_unit(),
        rule_numerals_prefix_with_negative_or_minus(),
        rule_multiply(),
        rule_tens(),
        rule_composite_tens(),
        rule_half(),
        rule_dot_spelled_out(),
        rule_dozen(),
        rule_pair(),
        rule_fraction(),
        rule_mixed_fraction(),
        rule_hundred_prefix(),
        rule_thousand_prefix(),
        rule_ten_thousand_prefix(),
    ];
    v.extend([
        numeral_suffix("K", 1e3, 3),
        numeral_suffix("M", 1e6, 6),
        numeral_suffix("G", 1e9, 9),
        numeral_suffix("十|拾", 1e1, 1),
        numeral_suffix("百|佰", 1e2, 2),
        numeral_suffix("千|仟", 1e3, 3),
        numeral_suffix("万|萬", 1e4, 4),
        numeral_suffix("亿|億", 1e8, 8),
    ]);
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
