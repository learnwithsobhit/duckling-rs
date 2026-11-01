//! `Duckling.Numeral.KA.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

static ZERO_NINETEEN_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ნოლ", 0),
        ("ნულ", 0),
        ("ნული", 0),
        ("ნოლი", 0),
        ("ერთი", 1),
        ("ორი", 2),
        ("ორ", 2),
        ("სამი", 3),
        ("სამ", 3),
        ("ოთხი", 4),
        ("ოთხ", 4),
        ("ხუთი", 5),
        ("ხუთ", 5),
        ("ექვსი", 6),
        ("ექვს", 6),
        ("შვიდი", 7),
        ("შვიდ", 7),
        ("რვა", 8),
        ("რვ", 8),
        ("ცხრა", 9),
        ("ცხრ", 9),
        ("ათი", 10),
        ("აათი", 10),
        ("თერთმეტი", 11),
        ("თერთმეტ", 11),
        ("თორმეტი", 12),
        ("თორმეტ", 12),
        ("ცამეტი", 13),
        ("ცამეტ", 13),
        ("თოთხმეტი", 14),
        ("თოთხმეტ", 14),
        ("თხუთმეტი", 15),
        ("თხუთმეტ", 15),
        ("თექვსმეტი", 16),
        ("თექვსმეტ", 16),
        ("ჩვიდმეტი", 17),
        ("ჩვიდმეტ", 17),
        ("თვრამეტი", 18),
        ("თვრამეტ", 18),
        ("ცხრამეტი", 19),
        ("ცხრამეტ", 19),
    ]
    .into_iter()
    .collect()
});

static INFORMAL_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ერთი", 1),
        ("წყვილი", 2),
        ("წყვილები", 2),
        ("ცოტა", 3),
        ("რამდენიმე", 3),
        ("რამოდენიმე", 3),
    ]
    .into_iter()
    .collect()
});

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ოცი", 20),
        ("ოცდა", 20),
        ("ოც", 20),
        ("ოცდაათ", 30),
        ("ოცდაათი", 30),
        ("ორმოც", 40),
        ("ორმოცი", 40),
        ("ორმოცდა", 40),
        ("ორმოცდაათ", 50),
        ("ორმოცდაათი", 50),
        ("სამოც", 60),
        ("სამოცი", 60),
        ("სამოცდა", 60),
        ("სამოცდაათ", 70),
        ("სამოცდაათი", 70),
        ("ოთხმოც", 80),
        ("ოთხმოცი", 80),
        ("ოთხმოცდა", 80),
        ("ოთხმოცდაათ", 90),
        ("ოთხმოცდაათი", 90),
    ]
    .into_iter()
    .collect()
});

static HUNDREDS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ასი", 100),
        ("ორასი", 200),
        ("სამასი", 300),
        ("ოთხასი", 400),
        ("ხუთასი", 500),
        ("ექვსასი", 600),
        ("შვიდასი", 700),
        ("რვაასი", 800),
        ("ცხრაასი", 900),
        ("ას", 100),
        ("ორას", 200),
        ("სამას", 300),
        ("ოთხას", 400),
        ("ხუთას", 500),
        ("ექვსას", 600),
        ("შვიდას", 700),
        ("რვაას", 800),
        ("ცხრაას", 900),
        ("ორ ას", 200),
        ("სამ ას", 300),
        ("ოთხ ას", 400),
        ("ხუთ ას", 500),
        ("ექვს ას", 600),
        ("შვიდ ას", 700),
        ("რვა ას", 800),
        ("ცხრა ას", 900),
        ("ორ ასი", 200),
        ("სამ ასი", 300),
        ("ოთხ ასი", 400),
        ("ხუთ ასი", 500),
        ("ექვს ასი", 600),
        ("შვიდ ასი", 700),
        ("რვა ასი", 800),
        ("ცხრა ასი", 900),
    ]
    .into_iter()
    .collect()
});

fn lookup_i64(map: &HashMap<&'static str, i64>, s: &str) -> Option<i64> {
    map.get(to_lower(s).as_str()).copied()
}

fn rule_to_nineteen() -> Rule {
    rule(
        "integer (0..19)",
        vec![regex(
            "(წყვილ(ებ)?ი|ცოტა|რამდენიმე|რამოდენიმე|ნოლი?|ნული?|ერთი|ორი?|სამი?|ოთხი?|ხუთი?|ექვსი?|შვიდი?|რვა|თერთმეტი?|თორმეტი?|ცამეტი?|თოთხმეტი?|თხუთმეტი?|თექვსმეტი?|ჩვიდმეტი?|თვრამეტი?|ცხრამეტი?|ცხრა|ა?ათი)",
        )],
        |ts| {
            let x = to_lower(groups(ts)?.first()?);
            lookup_i64(&ZERO_NINETEEN_MAP, &x)
                .and_then(integer)
                .or_else(|| lookup_i64(&INFORMAL_MAP, &x).and_then(integer).and_then(|t| not_ok_for_any_time(&t)))
        },
    )
}

fn rule_tens() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex(
            "(ოცდაათი?|ორმოცდაათი?|სამოცდაათი?|ოთხმოცდაათი?|ოცდა|ორმოცდა|სამოცდა|ოთხმოცდა|ოცი?|ორმოცი?|სამოცი?|ოთხმოცი?)",
        )],
        |ts| lookup_i64(&TENS_MAP, groups(ts)?.first()?).and_then(integer),
    )
}

fn rule_hundreds() -> Rule {
    rule(
        "integer (800..900)",
        vec![regex("(ასი?|ორ ?ასი?|სამ ?ასი?|ოთხ ?ასი?|ხუთ ?ასი?|ექვს ?ასი?|შვიდ ?ასი?|რვა ?ასი?|ცხრა ?ასი?)")],
        |ts| lookup_i64(&HUNDREDS_MAP, groups(ts)?.first()?).and_then(integer),
    )
}

fn rule_powers_of_ten() -> Rule {
    rule("powers of tens", vec![regex("(ათასი?|მილიონი?|მილიარდი?)")], |ts| {
        let (v, grain) = match to_lower(groups(ts)?.first()?).as_str() {
            "ათასი" => (1e3, 3),
            "ათას" => (1e3, 3),
            "მილიონი" => (1e6, 6),
            "მილიონ" => (1e6, 6),
            "მილიარდი" => (1e9, 9),
            "მილიარდ" => (1e9, 9),
            _ => return None,
        };
        double(v).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
    })
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer 21..99",
        vec![one_of(vec![20.0, 40.0, 60.0, 80.0]), pred(number_between(1.0, 20.0))],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(tens), Token::Numeral(units)) => double(tens.value + units.value),
            _ => None,
        },
    )
}

fn rule_composite_hundreds() -> Rule {
    rule(
        "integer 100..999",
        vec![
            one_of(vec![100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0]),
            one_of(vec![20.0, 40.0, 60.0, 80.0]),
            pred(number_between(1.0, 20.0)),
        ],
        |ts| match (ts.first()?, ts.get(1)?, ts.get(2)?) {
            (Token::Numeral(hundreds), Token::Numeral(tens), Token::Numeral(units)) => {
                double(hundreds.value + tens.value + units.value)
            }
            _ => None,
        },
    )
}

fn rule_composite_hundreds_and_units() -> Rule {
    rule(
        "integer 100..999",
        vec![
            one_of(vec![100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0]),
            pred(number_between(1.0, 20.0)),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(hundreds), Token::Numeral(units)) => double(hundreds.value + units.value),
            _ => None,
        },
    )
}

fn rule_composite_hundreds_and_tens() -> Rule {
    rule(
        "integer 100..999",
        vec![
            one_of(vec![100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0]),
            one_of(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(hundreds), Token::Numeral(tens)) => double(hundreds.value + tens.value),
            _ => None,
        },
    )
}

fn rule_dot_spelled_out() -> Rule {
    rule(
        "one point 2",
        vec![dimension(Dim::Numeral), regex("წერტილი|მთელი"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(nd1), Token::Numeral(nd2)) => {
                double(nd1.value + decimals_to_double(nd2.value))
            }
            _ => None,
        },
    )
}

fn rule_leading_dot_spelled_out() -> Rule {
    rule("point 77", vec![regex("წერტილი|მთელი"), pred(|t| !has_grain(t))], |ts| match ts.get(1)?
    {
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
        "suffixes (K,M,G))",
        vec![dimension(Dim::Numeral), regex("(k|m|g)(?=[\\W$€¢£]|$)")],
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
        vec![regex("(-|მინუს|მინ)(?!\\s*-)"), pred(is_positive)],
        |ts| match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * (-1.0)),
            _ => None,
        },
    )
}

fn rule_sum() -> Rule {
    rule(
        "intersect 2 numbers",
        vec![pred(|t| has_grain(t) && is_positive(t)), pred(|t| !is_multipliable(t) && is_positive(t))],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(a), Token::Numeral(b)) => {
                let g = a.grain?;
                if 10f64.powf(g as f64) > b.value {
                    double(a.value + b.value)
                } else {
                    None
                }
            }
            _ => None,
        },
    )
}

fn rule_sum_and() -> Rule {
    rule(
        "intersect 2 numbers (with and)",
        vec![
            pred(|t| has_grain(t) && is_positive(t)),
            regex("და"),
            pred(|t| !is_multipliable(t) && is_positive(t)),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) => {
                let g = a.grain?;
                if 10f64.powf(g as f64) > b.value {
                    double(a.value + b.value)
                } else {
                    None
                }
            }
            _ => None,
        },
    )
}

fn rule_multiply() -> Rule {
    rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
        multiply(ts.first()?, ts.get(1)?)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_to_nineteen(),
        rule_hundreds(),
        rule_composite_hundreds(),
        rule_composite_hundreds_and_units(),
        rule_composite_hundreds_and_tens(),
        rule_tens(),
        rule_powers_of_ten(),
        rule_composite_tens(),
        rule_dot_spelled_out(),
        rule_leading_dot_spelled_out(),
        rule_decimals(),
        rule_commas(),
        rule_suffixes(),
        rule_negative(),
        rule_sum(),
        rule_sum_and(),
        rule_multiply(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
