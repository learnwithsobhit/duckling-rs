//! `Duckling.Numeral.EN.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn rule_dozen() -> Rule {
    rule("a dozen of", vec![regex("(a )?dozens?( of)?")], |_| {
        integer(12).and_then(|t| with_multipliable(&t)).and_then(|t| not_ok_for_any_time(&t))
    })
}

static ZERO_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("naught", 0), ("nil", 0), ("nought", 0), ("none", 0), ("zero", 0), ("zilch", 0),
        ("one", 1), ("two", 2), ("three", 3), ("four", 4), ("five", 5), ("six", 6),
        ("seven", 7), ("eight", 8), ("nine", 9), ("ten", 10), ("eleven", 11), ("twelve", 12),
        ("thirteen", 13), ("fourteen", 14), ("fifteen", 15), ("sixteen", 16),
        ("seventeen", 17), ("eighteen", 18), ("nineteen", 19),
    ]
    .into_iter()
    .collect()
});

static INFORMAL: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("single", 1), ("a couple", 2), ("a couple of", 2), ("couple", 2), ("couples", 2),
        ("couple of", 2), ("couples of", 2), ("a pair", 2), ("a pair of", 2), ("pair", 2),
        ("pairs", 2), ("pair of", 2), ("pairs of", 2), ("a few", 3), ("few", 3),
    ]
    .into_iter()
    .collect()
});

fn rule_to_nineteen() -> Rule {
    // e.g. fourteen must be before four, otherwise four will always shadow fourteen
    rule(
        "integer (0..19)",
        vec![regex("(none|zilch|naught|nought|nil|zero|one|single|two|(a )?(pair|couple)s?( of)?|three|(a )?few|fourteen|four|fifteen|five|sixteen|six|seventeen|seven|eighteen|eight|nineteen|nine|ten|eleven|twelve|thirteen)")],
        |ts| {
            let g = groups(ts)?;
            let x = to_lower(g.first()?);
            if let Some(v) = ZERO_NINETEEN.get(x.as_str()) {
                if let Some(t) = integer(*v) {
                    return Some(t);
                }
            }
            INFORMAL.get(x.as_str()).and_then(|v| integer(*v)).and_then(|t| not_ok_for_any_time(&t))
        },
    )
}

static TENS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("twenty", 20), ("thirty", 30), ("forty", 40), ("fourty", 40), ("fifty", 50),
        ("sixty", 60), ("seventy", 70), ("eighty", 80), ("ninety", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_tens() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(twenty|thirty|fou?rty|fifty|sixty|seventy|eighty|ninety)")],
        |ts| {
            let g = groups(ts)?;
            TENS.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

static POWERS_OF_TEN: Lazy<HashMap<&'static str, i32>> = Lazy::new(|| {
    [
        ("hundred", 2), ("thousand", 3), ("lakh", 5), ("lkh", 5), ("l", 5), ("lac", 5),
        ("million", 6), ("cr", 7), ("crore", 7), ("krore", 7), ("kr", 7), ("koti", 7),
        ("billion", 9), ("trillion", 12),
    ]
    .into_iter()
    .collect()
});

/// Haskell's `10 ^ n` on `Double`.
fn pow10(n: i32) -> f64 {
    let mut r = 1.0f64;
    for _ in 0..n {
        r *= 10.0;
    }
    r
}

fn rule_powers_of_ten() -> Rule {
    rule(
        "powers of tens",
        vec![regex("(hundred|thousand|l(ac|(a?kh)?)|million|((k|c)r(ore)?|koti)|billion)s?")],
        |ts| {
            let g = groups(ts)?;
            let grain = *POWERS_OF_TEN.get(to_lower(g.first()?).as_str())?;
            double(pow10(grain))
                .and_then(|t| with_grain(grain as i64, &t))
                .and_then(|t| with_multipliable(&t))
        },
    )
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer 21..99",
        vec![
            one_of((2..=9).map(|i| (i * 10) as f64).collect()),
            regex(r"[\s\-]+"),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(t), Token::Numeral(u)) => double(t.value + u.value),
            _ => None,
        },
    )
}

fn rule_skip_hundreds1() -> Rule {
    rule(
        "one eleven",
        vec![
            regex("(one|two|three|four|five|six|seven|eight|nine)"),
            regex("(ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|fou?rty|fifty|sixty|seventy|eighty|ninety)"),
        ],
        |ts| {
            let m1 = match ts.first()? {
                Token::RegexMatch(g) => to_lower(g.0.first()?),
                _ => return None,
            };
            let m2 = match ts.get(1)? {
                Token::RegexMatch(g) => to_lower(g.0.first()?),
                _ => return None,
            };
            let hundreds = *ZERO_NINETEEN.get(m1.as_str())?;
            let rest = ZERO_NINETEEN.get(m2.as_str()).or_else(|| TENS.get(m2.as_str()))?;
            integer(hundreds * 100 + rest)
        },
    )
}

fn rule_skip_hundreds2() -> Rule {
    rule(
        "one twenty two",
        vec![
            regex("(one|two|three|four|five|six|seven|eight|nine)"),
            regex("(twenty|thirty|fou?rty|fifty|sixty|seventy|eighty|ninety)"),
            regex("(one|two|three|four|five|six|seven|eight|nine)"),
        ],
        |ts| {
            let g = |i: usize| match ts.get(i) {
                Some(Token::RegexMatch(g)) => g.0.first().map(|s| to_lower(s)),
                _ => None,
            };
            let hundreds = *ZERO_NINETEEN.get(g(0)?.as_str())?;
            let tens = *TENS.get(g(1)?.as_str())?;
            let rest = *ZERO_NINETEEN.get(g(2)?.as_str())?;
            integer(hundreds * 100 + tens + rest)
        },
    )
}

fn rule_dot_spelled_out() -> Rule {
    rule(
        "one point 2",
        vec![dimension(Dim::Numeral), regex("point|dot"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) => {
                double(a.value + decimals_to_double(b.value))
            }
            _ => None,
        },
    )
}

fn rule_leading_dot_spelled_out() -> Rule {
    rule("point 77", vec![regex("point|dot"), pred(|t| !has_grain(t))], |ts| match ts.get(1)? {
        Token::Numeral(n) => double(decimals_to_double(n.value)),
        _ => None,
    })
}

fn rule_decimals() -> Rule {
    rule("decimal number", vec![regex(r"(\d*\.\d+)")], |ts| {
        let g = groups(ts)?;
        parse_decimal(true, g.first()?)
    })
}

fn rule_commas() -> Rule {
    rule("comma-separated numbers", vec![regex(r"(\d+(,\d\d\d)+(\.\d+)?)")], |ts| {
        let g = groups(ts)?;
        parse_double(&g.first()?.replace(',', "")).and_then(double)
    })
}

fn rule_suffixes() -> Rule {
    rule(
        "suffixes (K,M,G))",
        vec![dimension(Dim::Numeral), regex(r"(k|m|g)(?=[\W$€¢£]|$)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(nd), Token::RegexMatch(g)) => {
                let x = match to_lower(g.0.first()?).as_str() {
                    "k" => 1e3,
                    "m" => 1e6,
                    "g" => 1e9,
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
        vec![regex(r"(-|minus|negative)(?!\s*-)"), pred(is_positive)],
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

fn rule_sum() -> Rule {
    rule(
        "intersect 2 numbers",
        vec![
            pred(|t| has_grain(t) && is_positive(t)),
            pred(|t| !is_multipliable(t) && is_positive(t)),
        ],
        |ts| sum_with(ts, 1),
    )
}

fn rule_sum_and() -> Rule {
    rule(
        "intersect 2 numbers (with and)",
        vec![
            pred(|t| has_grain(t) && is_positive(t)),
            regex("and"),
            pred(|t| !is_multipliable(t) && is_positive(t)),
        ],
        |ts| sum_with(ts, 2),
    )
}

fn rule_multiply() -> Rule {
    rule("compose by multiplication", vec![pred(is_positive), pred(is_multipliable)], |ts| {
        multiply(ts.first()?, ts.get(1)?)
    })
}

fn rule_legal_parentheses() -> Rule {
    rule(
        "<integer> '('<integer>')'",
        vec![
            number_with(|n| n.value, |v| is_integer(v) && v > 0.0),
            regex(r"\("),
            number_with(|n| n.value, |v| is_integer(v) && v > 0.0),
            regex(r"\)"),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) if a.value == b.value => double(a.value),
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_to_nineteen(),
        rule_tens(),
        rule_powers_of_ten(),
        rule_composite_tens(),
        rule_skip_hundreds1(),
        rule_skip_hundreds2(),
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
        rule_legal_parentheses(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
