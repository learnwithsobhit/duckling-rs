//! `Duckling.Numeral.RO.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

static ZERO_TEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("zero", 0),
        ("nimic", 0),
        ("nicio", 0),
        ("nici o", 0),
        ("nici una", 0),
        ("nici unu", 0),
        ("nici unul", 0),
        ("un", 1),
        ("una", 1),
        ("unu", 1),
        ("unul", 1),
        ("intai", 1),
        ("întai", 1),
        ("intâi", 1),
        ("întâi", 1),
        ("o", 1),
        ("doi", 2),
        ("doua", 2),
        ("două", 2),
        ("trei", 3),
        ("patru", 4),
        ("cinci", 5),
        ("sase", 6),
        ("șase", 6),
        ("sapte", 7),
        ("șapte", 7),
        ("opt", 8),
        ("noua", 9),
        ("nouă", 9),
        ("zece", 10),
        ("zeci", 10),
    ]
    .into_iter()
    .collect()
});

static ELEVEN_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("un", 11),
        ("doi", 12),
        ("trei", 13),
        ("pai", 14),
        ("cin", 15),
        ("cinci", 15),
        ("sai", 16),
        ("șai", 16),
        ("sapti", 17),
        ("șapti", 17),
        ("sapte", 17),
        ("șapte", 17),
        ("opti", 18),
        ("opt", 18),
        ("noua", 19),
        ("nouă", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_numerals_prefix_with_or_minus() -> Rule {
    rule("numbers prefix with - or minus", vec![regex("-|minus"), pred(is_positive)], |ts| {
        match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * -1.0),
            _ => None,
        }
    })
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule("decimal with thousands separator", vec![regex(r"(\d+(\.\d\d\d)+,\d+)")], |ts| {
        // `Text.replace "," "." $ Text.replace "." Text.empty match`
        let fmt = groups(ts)?.first()?.replace('.', "").replace(',', ".");
        parse_double(&fmt).and_then(double)
    })
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex(r"(\d*,\d+)")], |ts| {
        parse_decimal(false, groups(ts)?.first()?)
    })
}

fn rule_integer3() -> Rule {
    rule(
        "integer 21..99",
        vec![one_of((2..=9).map(|i| (i * 10) as f64).collect()), pred(number_between(1.0, 10.0))],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
            _ => None,
        },
    )
}

fn rule_multiply() -> Rule {
    rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
        multiply(ts.first()?, ts.get(1)?)
    })
}

fn rule_multiply_de() -> Rule {
    rule(
        "compose by multiplication",
        vec![number_with(|n| n.value, |v| v >= 20.0), regex("de"), pred(is_multipliable)],
        |ts| multiply(ts.first()?, ts.get(2)?),
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

fn rule_intersect() -> Rule {
    rule(
        "intersect",
        vec![pred(has_grain), pred(|t| !is_multipliable(t) && is_positive(t))],
        |ts| sum_with(ts, 1),
    )
}

fn rule_intersect_cu_i() -> Rule {
    rule(
        "intersect (cu și)",
        vec![pred(has_grain), regex("[sș]i"), pred(|t| !is_multipliable(t) && is_positive(t))],
        |ts| sum_with(ts, 2),
    )
}

fn rule_numerals_suffixes_with_negativ() -> Rule {
    rule("numbers suffixes with (negativ)", vec![pred(is_positive), regex("neg(ativ)?")], |ts| {
        match ts.first()? {
            Token::Numeral(n) => double(n.value * -1.0),
            _ => None,
        }
    })
}

fn rule_powers_of_ten() -> Rule {
    rule(
        "powers of tens",
        vec![regex("(sut(a|e|ă)?|milio(n|ane)?|miliar(de?)?|mi[ei]?)")],
        |ts| {
            let (v, grain) = match to_lower(groups(ts)?.first()?).as_str() {
                "suta" | "sute" | "sută" => (1e2, 2),
                "mi" | "mie" | "mii" => (1e3, 3),
                "milio" | "milion" | "milioane" => (1e6, 6),
                "miliar" | "miliard" | "miliarde" => (1e9, 9),
                _ => return None,
            };
            double(v).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
        },
    )
}

fn rule_integer_zero_ten() -> Rule {
    rule(
        "integer (0..10)",
        vec![regex(
            "(zero|nimic|nici(\\s?o|\\sun(a|ul?))|una|unul?|doi|dou(a|ă)|trei|patru|cinci|(s|ș)ase|(s|ș)apte|opt|nou(a|ă)|zec[ei]|(i|î)nt(a|â)i|un|o)",
        )],
        |ts| ZERO_TEN.get(to_lower(groups(ts)?.first()?).as_str()).and_then(|v| integer(*v)),
    )
}

fn rule_integer() -> Rule {
    rule(
        "integer (11..19)",
        vec![regex(
            "(cin|sapti|opti)(s|ș)pe|(cinci|(s|ș)apte|opt)sprezece|(un|doi|trei|pai|(s|ș)ai|nou(a|ă))((s|ș)pe|sprezece)",
        )],
        |ts| {
            let g = groups(ts)?;
            let e1 = g.first().map(|s| s.as_ref()).unwrap_or("");
            let e2 = g.get(2).map(|s| s.as_ref()).unwrap_or("");
            let r = g.get(4).map(|s| s.as_ref()).unwrap_or("");
            let match_s = if !e1.is_empty() {
                e1
            } else if !e2.is_empty() {
                e2
            } else if !r.is_empty() {
                r
            } else {
                return None;
            };
            ELEVEN_NINETEEN.get(to_lower(match_s).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_integer2() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(dou[aă]|trei|patru|cinci|[sș]ai|[sș]apte|opt|nou[aă])\\s?zeci")],
        |ts| {
            let unit = ZERO_TEN.get(to_lower(groups(ts)?.first()?).as_str())?;
            integer(unit * 10).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t))
        },
    )
}

fn rule_integer_cu_separator_de_mii_dot() -> Rule {
    rule("integer cu separator de mii dot", vec![regex(r"(\d{1,3}(\.\d\d\d){1,5})")], |ts| {
        parse_double(&groups(ts)?.first()?.replace('.', "")).and_then(double)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_integer(),
        rule_integer_zero_ten(),
        rule_integer2(),
        rule_integer3(),
        rule_integer_cu_separator_de_mii_dot(),
        rule_intersect(),
        rule_intersect_cu_i(),
        rule_multiply(),
        rule_multiply_de(),
        rule_numerals_prefix_with_or_minus(),
        rule_numerals_suffixes_with_negativ(),
        rule_powers_of_ten(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
