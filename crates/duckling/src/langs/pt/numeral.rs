//! `Duckling.Numeral.PT.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn rule_dozen() -> Rule {
    rule("a dozen of", vec![regex("(uma )?d(u|ú)zias?( de)?")], |_| {
        integer(12).and_then(|t| with_multipliable(&t)).and_then(|t| not_ok_for_any_time(&t))
    })
}

static ZERO_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("zero", 0), ("um", 1), ("uma", 1), ("dois", 2), ("duas", 2), ("tres", 3), ("três", 3),
        ("quatro", 4), ("cinco", 5), ("seis", 6), ("sete", 7), ("oito", 8), ("nove", 9),
        ("dez", 10), ("onze", 11), ("doze", 12), ("treze", 13), ("catorze", 14),
        ("quatorze", 14), ("quinze", 15), ("dezesseis", 16), ("dezasseis", 16),
        ("dezessete", 17), ("dezassete", 17), ("dezoito", 18), ("dezenove", 19),
        ("dezanove", 19),
    ]
    .into_iter()
    .collect()
});

static INFORMAL: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("um par", 2), ("um par de", 2), ("par", 2), ("pares", 2), ("par de", 2),
        ("pares de", 2), ("um pouco", 3), ("pouco", 3),
    ]
    .into_iter()
    .collect()
});

fn rule_to_nineteen() -> Rule {
    rule(
        "integer (0..19)",
        vec![regex("(zero|d(oi|ua)s|(uma? )?par(es)?( de)?|tr(e|ê)s|(um )?pouco|uma?|(c|qu)atorze|quatro|quinze|cinco|dez[ea]sseis|seis|dez[ea]ssete|sete|dezoito|oito|dez[ea]nove|nove|dez|onze|doze|treze)")],
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
        ("vinte", 20), ("trinta", 30), ("quarenta", 40), ("cincoenta", 50),
        ("cinquenta", 50), ("cinqüenta", 50), ("sessenta", 60), ("setenta", 70),
        ("oitenta", 80), ("noventa", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_tens() -> Rule {
    rule(
        "tens (20..90)",
        vec![regex("(vinte|trinta|quarenta|cin(co|q[uü])enta|sessenta|setenta|oitenta|noventa)")],
        |ts| {
            let g = groups(ts)?;
            TENS.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

static CENTS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("cem", 100), ("cento", 100), ("duzentos", 200), ("trezentos", 300),
        ("quatrocentos", 400), ("quinhetos", 500), ("seiscentos", 600),
        ("setecentos", 700), ("oitocentos", 800), ("novecentos", 900),
    ]
    .into_iter()
    .collect()
});

fn rule_cent() -> Rule {
    rule(
        "hundreds (100..900)",
        vec![regex("(cem|cento|duzentos|trezentos|quatrocentos|quinhetos|seiscentos|setecentos|oitocentos|novecentos)")],
        |ts| {
            let g = groups(ts)?;
            CENTS.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_powers_of_ten() -> Rule {
    rule(
        "powers of tens",
        vec![regex("(milhao|milhão|milhões|milhoes|bilhao|bilhão|bilhões|bilhoes|mil)")],
        |ts| {
            let g = groups(ts)?;
            let (v, grain) = match to_lower(g.first()?).as_str() {
                "mil" => (1e3, 3),
                "milhao" | "milhão" | "milhões" | "milhoes" => (1e6, 6),
                "bilhao" | "bilhão" | "bilhões" | "bilhoes" => (1e9, 9),
                _ => return None,
            };
            double(v).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
        },
    )
}

fn nums2(ts: &[&Token], second: usize) -> Option<(f64, f64)> {
    match (ts.first()?, ts.get(second)?) {
        (Token::Numeral(a), Token::Numeral(b)) => Some((a.value, b.value)),
        _ => None,
    }
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer 21..99",
        vec![one_of((2..=9).map(|i| (i * 10) as f64).collect()), pred(number_between(1.0, 10.0))],
        |ts| {
            let (t, u) = nums2(ts, 1)?;
            double(t + u)
        },
    )
}

fn rule_decs_and() -> Rule {
    rule(
        "number (21..29 31..39 .. 91..99)",
        vec![
            one_of((2..=9).map(|i| (i * 10) as f64).collect()),
            regex("e"),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| {
            let (a, b) = nums2(ts, 2)?;
            double(a + b)
        },
    )
}

fn rule_composite_cents() -> Rule {
    rule(
        "integer 101..999",
        vec![one_of((1..=9).map(|i| (i * 100) as f64).collect()), pred(number_between(1.0, 100.0))],
        |ts| {
            let (a, b) = nums2(ts, 1)?;
            double(a + b)
        },
    )
}

fn rule_cents_and() -> Rule {
    rule(
        "number (101..199 201..299 .. 901..999)",
        vec![
            one_of((1..=9).map(|i| (i * 100) as f64).collect()),
            regex("e"),
            pred(number_between(1.0, 100.0)),
        ],
        |ts| {
            let (a, b) = nums2(ts, 2)?;
            double(a + b)
        },
    )
}

fn rule_skip_hundreds() -> Rule {
    rule(
        "one twenty two",
        vec![pred(number_between(1.0, 10.0)), pred(number_between(10.0, 100.0))],
        |ts| {
            let (h, r) = nums2(ts, 1)?;
            double(h * 100.0 + r)
        },
    )
}

fn rule_dot_spelled_out() -> Rule {
    rule(
        "one point 2",
        vec![dimension(Dim::Numeral), regex("ponto"), pred(|t| !has_grain(t))],
        |ts| {
            let (a, b) = nums2(ts, 2)?;
            double(a + decimals_to_double(b))
        },
    )
}

fn rule_leading_dot_spelled_out() -> Rule {
    rule("point 77", vec![regex("ponto"), pred(|t| !has_grain(t))], |ts| match ts.get(1)? {
        Token::Numeral(n) => double(decimals_to_double(n.value)),
        _ => None,
    })
}

fn rule_decimals() -> Rule {
    rule("decimal number", vec![regex(r"(\d*\,\d+)")], |ts| {
        let g = groups(ts)?;
        parse_decimal(false, g.first()?)
    })
}

fn rule_commas() -> Rule {
    rule("dot-separated numbers", vec![regex(r"(\d+(\.\d\d\d)+(\,\d+)?)")], |ts| {
        let g = groups(ts)?;
        parse_decimal(false, &g.first()?.replace('.', ""))
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
        vec![regex(r"(-|menos|negativo)(?!\s*\-)"), pred(is_positive)],
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
        vec![pred(has_grain), pred(|t| !is_multipliable(t) && is_positive(t))],
        |ts| sum_with(ts, 1),
    )
}

fn rule_sum_and() -> Rule {
    rule(
        "intersect 2 numbers (with and)",
        vec![pred(has_grain), regex("e"), pred(|t| !is_multipliable(t) && is_positive(t))],
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
        rule_to_nineteen(),
        rule_tens(),
        rule_cent(),
        rule_powers_of_ten(),
        rule_composite_tens(),
        rule_composite_cents(),
        rule_skip_hundreds(),
        rule_dot_spelled_out(),
        rule_leading_dot_spelled_out(),
        rule_decimals(),
        rule_commas(),
        rule_suffixes(),
        rule_negative(),
        rule_sum(),
        rule_decs_and(),
        rule_cents_and(),
        rule_sum_and(),
        rule_multiply(),
        rule_dozen(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
