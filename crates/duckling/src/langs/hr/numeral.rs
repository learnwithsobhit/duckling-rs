//! `Duckling.Numeral.HR.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn rule_numbers_prefix_with_negative_or_minus() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex("-|minus|negativ"), pred(is_positive)],
        |ts| match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * -1.0),
            _ => None,
        },
    )
}

fn rule_few() -> Rule {
    rule("few", vec![regex("nekoliko")], |_| integer(3))
}

fn rule_ten() -> Rule {
    rule("ten", vec![regex("deset|cener")], |_| integer(10).and_then(|t| with_grain(1, &t)))
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule("decimal with thousands separator", vec![regex(r"(\d+(\.\d\d\d)+\,\d+)")], |ts| {
        let m = groups(ts)?.first()?.to_string();
        let fmt = m.replace('.', "").replace(',', ".");
        parse_double(&fmt).and_then(double)
    })
}

fn rule_decimal_number() -> Rule {
    rule("decimal number", vec![regex(r"(\d*,\d+)")], |ts| {
        parse_decimal(false, groups(ts)?.first()?)
    })
}

static INTEGER3_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("sto", 100),
        ("dvjesta", 200),
        ("dvjesto", 200),
        ("tristo", 300),
        ("cetiristo", 400),
        ("četiristo", 400),
        ("petsto", 500),
        ("šesto", 600),
        ("sesto", 600),
        ("sedamsto", 700),
        ("osamsto", 800),
        ("devetsto", 900),
    ]
    .into_iter()
    .collect()
});

fn rule_integer3() -> Rule {
    rule(
        "integer (100..900)",
        vec![regex(
            r"(sto|dvjest(o|a)|tristo|(c|č)etiristo|petsto|(š|s)esto|sedamsto|osamsto|devetsto)",
        )],
        |ts| {
            INTEGER3_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .and_then(|v| integer(*v))
        },
    )
}

fn rule_single() -> Rule {
    rule("single", vec![regex("sam")], |_| integer(1).and_then(|t| with_grain(1, &t)))
}

fn powers_of_ten_prod(match_: &str) -> Option<Token> {
    let (v, grain) = match match_ {
        "stotinu" | "stotina" | "stotine" => (1e2, 2),
        "tisuca" | "tisucu" | "tisuce" | "tisuća" | "tisuću" | "tisuće" => (1e3, 3),
        "milijun" | "milijuna" | "milijon" | "milijona" => (1e6, 6),
        _ => return None,
    };
    double(v).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
}

fn rule_powers_of_ten() -> Rule {
    rule(
        "powers of tens",
        vec![regex(r"(stotin(u|a|e)|tisu(c|ć)(a|u|e)|milij(u|o)na?)")],
        |ts| powers_of_ten_prod(&to_lower(groups(ts)?.first()?)),
    )
}

fn rule_numbers_i() -> Rule {
    rule(
        "numbers i",
        vec![
            one_of(vec![70.0, 20.0, 60.0, 50.0, 40.0, 90.0, 30.0, 80.0]),
            regex("i"),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
            _ => None,
        },
    )
}

fn rule_sum() -> Rule {
    rule(
        "intersect",
        vec![pred(has_grain), pred(|t| !is_multipliable(t) && is_positive(t))],
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

fn rule_numbers_suffixes_kmg() -> Rule {
    rule(
        "numbers suffixes (K, M, G)",
        vec![dimension(Dim::Numeral), regex(r"([kmg])(?=[\W\$€]|$)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(nd), Token::RegexMatch(g)) => match to_lower(g.0.first()?).as_str() {
                "k" => double(nd.value * 1e3),
                "m" => double(nd.value * 1e6),
                "g" => double(nd.value * 1e9),
                _ => None,
            },
            _ => None,
        },
    )
}

fn rule_a_pair() -> Rule {
    rule("a pair", vec![regex("par")], |_| integer(2).and_then(|t| with_grain(1, &t)))
}

fn rule_dozen() -> Rule {
    rule("dozen", vec![regex("tucet?")], |_| integer(12).and_then(|t| with_grain(1, &t)))
}

static INTEGER_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ništa", 0),
        ("ništica", 0),
        ("nistica", 0),
        ("nista", 0),
        ("nula", 0),
        ("jednoga", 1),
        ("jedna", 1),
        ("jednog", 1),
        ("jednu", 1),
        ("jedan", 1),
        ("dvoma", 2),
        ("dvije", 2),
        ("dvje", 2),
        ("dva", 2),
        ("dvama", 2),
        ("trima", 3),
        ("tri", 3),
        ("četiri", 4),
        ("cetiri", 4),
        ("pet", 5),
        ("šest", 6),
        ("sedam", 7),
        ("osam", 8),
        ("devet", 9),
        ("jedanaest", 11),
        ("dvanaest", 12),
        ("trinaest", 13),
        ("cetrnaest", 14),
        ("četrnaest", 14),
        ("petnaest", 15),
        ("šesnaest", 16),
        ("sesnaest", 16),
        ("sedamnaest", 17),
        ("osamnaest", 18),
        ("devetnaest", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_integer() -> Rule {
    rule(
        "integer (0..19)",
        vec![regex(
            r"(ni(s|š)ta|ni(s|š)tica|nula|jedanaest|dvanaest|trinaest|jeda?n(a|u|o(ga?)?)?|dv(i?je)?(a|o)?(ma)?|tri(ma)?|(č|c)etiri|(č|c)etrnaest|petnaest|pet|(s|š)esnaest|(š|s)est|sedamnaest|sedam|osamnaest|osam|devetnaest|devet)",
        )],
        |ts| {
            INTEGER_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .and_then(|v| integer(*v))
        },
    )
}

fn rule_integer4() -> Rule {
    rule(
        "integer 21..99",
        vec![
            one_of(vec![70.0, 20.0, 60.0, 50.0, 40.0, 90.0, 30.0, 80.0]),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
            _ => None,
        },
    )
}

static INTEGER2_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("dvadeset", 20),
        ("trideset", 30),
        ("cetrdeset", 40),
        ("četrdeset", 40),
        ("pedeset", 50),
        ("sesdeset", 60),
        ("šesdeset", 60),
        ("sedamdeset", 70),
        ("osamdeset", 80),
        ("devedeset", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_integer2() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex(
            r"(dvadeset|trideset|(c|č)etrdeset|pedeset|(š|s)esdeset|sedamdeset|osamdeset|devedeset)",
        )],
        |ts| {
            INTEGER2_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .and_then(|v| integer(*v))
        },
    )
}

fn rule_numbers() -> Rule {
    rule(
        "numbers 100..999",
        vec![
            pred(number_between(1.0, 10.0)),
            number_with(|n| n.value, |v| v == 100.0),
            pred(number_between(0.0, 100.0)),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(100.0 * a.value + b.value),
            _ => None,
        },
    )
}

fn rule_number_dot_number() -> Rule {
    rule(
        "number dot number",
        vec![dimension(Dim::Numeral), regex("cijela|to(c|č)ka|zarez"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + decimals_to_double(b.value)),
            _ => None,
        },
    )
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule(
        "integer with thousands separator .",
        vec![regex(r"(\d{1,3}(\.\d\d\d){1,5})")],
        |ts| {
            let fmt = groups(ts)?.first()?.replace('.', "");
            parse_double(&fmt).and_then(double)
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
        rule_a_pair(),
        rule_decimal_number(),
        rule_decimal_with_thousands_separator(),
        rule_dozen(),
        rule_few(),
        rule_integer(),
        rule_integer2(),
        rule_integer3(),
        rule_integer4(),
        rule_integer_with_thousands_separator(),
        rule_multiply(),
        rule_number_dot_number(),
        rule_numbers(),
        rule_numbers_i(),
        rule_numbers_prefix_with_negative_or_minus(),
        rule_numbers_suffixes_kmg(),
        rule_powers_of_ten(),
        rule_single(),
        rule_sum(),
        rule_ten(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
