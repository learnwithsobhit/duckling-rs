//! `Duckling.Numeral.ES.Rules` and the ES locale numeral rules (`Duckling.Numeral.ES.{AR,CL,CO,ES,MX,PE,VE}.Rules`).

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

static ZERO_TO_FIFTEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("zero", 0), ("cero", 0), ("un", 1), ("una", 1), ("uno", 1), ("dos", 2), ("trés", 3),
        ("tres", 3), ("cuatro", 4), ("cinco", 5), ("seis", 6), ("séis", 6), ("siete", 7),
        ("ocho", 8), ("nueve", 9), ("diez", 10), ("dies", 10), ("once", 11), ("doce", 12),
        ("trece", 13), ("catorce", 14), ("quince", 15),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral_zero_to_fifteen() -> Rule {
    rule(
        "number (0..15)",
        vec![regex("((c|z)ero|un(o|a)?|dos|tr(é|e)s|cuatro|cinco|s(e|é)is|siete|ocho|nueve|die(z|s)|once|doce|trece|catorce|quince)")],
        |ts| {
            let g = groups(ts)?;
            ZERO_TO_FIFTEEN.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn num_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

fn rule_below_ten_with_two_digits() -> Rule {
    rule(
        "integer (0-9) with two digits",
        vec![regex("((c|z)ero)|0"), pred(number_between(1.0, 10.0))],
        |ts| double(num_value(ts.get(1)?)?),
    )
}

static SIXTEEN_TO_TWENTY_NINE: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("dieciseis", 16), ("diesiséis", 16), ("diesiseis", 16), ("dieciséis", 16),
        ("diecisiete", 17), ("dieciocho", 18), ("diecinueve", 19), ("veintiuno", 21),
        ("veintiuna", 21), ("veintidos", 22), ("veintidós", 22), ("veintitrés", 23),
        ("veintitres", 23), ("veinticuatro", 24), ("veinticinco", 25), ("veintiséis", 26),
        ("veintiseis", 26), ("veintisiete", 27), ("veintiocho", 28), ("veintinueve", 29),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral_sixteen_to_twenty_nine() -> Rule {
    rule(
        "number (16..19 21..29)",
        vec![regex("(die(c|s)is(é|e)is|diecisiete|dieciocho|diecinueve|veintiun(o|a)|veintid(o|ó)s|veintitr(é|e)s|veinticuatro|veinticinco|veintis(é|e)is|veintisiete|veintiocho|veintinueve|treinta)")],
        |ts| {
            let g = groups(ts)?;
            SIXTEEN_TO_TWENTY_NINE.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_numeral_sixteen_to_nineteen_with_diez() -> Rule {
    rule(
        "number (16..19, two words)",
        vec![number_with(|n| n.value, |v| v == 10.0), regex("y"), pred(number_between(6.0, 10.0))],
        |ts| double(10.0 + num_value(ts.get(2)?)?),
    )
}

static BY_TENS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("veinte", 20), ("treinta", 30), ("cuarenta", 40), ("cincuenta", 50), ("sesenta", 60),
        ("setenta", 70), ("ochenta", 80), ("noventa", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral_twenty_to_ninety_tens() -> Rule {
    rule(
        "number (20..90)",
        vec![regex("(veinte|treinta|cuarenta|cincuenta|sesenta|setenta|ochenta|noventa)")],
        |ts| {
            let g = groups(ts)?;
            BY_TENS.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_numeral_twenty_one_to_ninety_nine() -> Rule {
    rule(
        "number (21..29 31..39 41..49 51..59 61..69 71..79 81..89 91..99)",
        vec![one_of((2..=9).map(|i| (i * 10) as f64).collect()), regex("y"), pred(number_between(1.0, 10.0))],
        |ts| double(num_value(ts.first()?)? + num_value(ts.get(2)?)?),
    )
}

static BIG_NUMBERS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("cien", 100), ("cientos", 100), ("ciento", 100), ("doscientos", 200),
        ("trescientos", 300), ("cuatrocientos", 400), ("quinientos", 500), ("seiscientos", 600),
        ("setecientos", 700), ("ochocientos", 800), ("novecientos", 900), ("mil", 1000),
        ("millon", 1000000), ("millón", 1000000), ("un millon", 1000000), ("un millón", 1000000),
        ("millones", 1000000),
    ]
    .into_iter()
    .collect()
});

fn rule_big_numeral() -> Rule {
    rule(
        "big number 100 to 1K",
        vec![regex("(cien(to|tos)?|doscientos|trescientos|cuatrocientos|quinientos|seiscientos|setecientos|ochocientos|novecientos|(un )?mill(o|ó)n)")],
        |ts| {
            let g = groups(ts)?;
            BIG_NUMBERS.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_big_numeral_multipliable() -> Rule {
    rule("1K or 1M in multipliable form", vec![regex("(mil(lones)?)")], |ts| {
        let g = groups(ts)?;
        BIG_NUMBERS
            .get(to_lower(g.first()?).as_str())
            .and_then(|v| integer(*v))
            .and_then(|t| with_multipliable(&t))
    })
}

fn rule_two_part_hundreds() -> Rule {
    rule("2..9 cientos", vec![pred(number_between(2.0, 10.0)), regex("cientos")], |ts| {
        double(100.0 * num_value(ts.first()?)?)
    })
}

fn rule_numeral_hundreds_and_smaller() -> Rule {
    rule(
        "<hundreds> 0..99",
        vec![number_with(|n| n.value, |v| is_multiple(100, v)), pred(number_between(0.0, 100.0))],
        |ts| {
            let (v1, v2) = (num_value(ts.first()?)?, num_value(ts.get(1)?)?);
            if v1 > 0.0 && v1 < 1000.0 {
                double(v1 + v2)
            } else {
                None
            }
        },
    )
}

fn rule_numeral_multiply() -> Rule {
    rule("2..999 <multipliable>", vec![pred(number_between(2.0, 1000.0)), pred(is_multipliable)], |ts| {
        double(num_value(ts.first()?)? * num_value(ts.get(1)?)?)
    })
}

fn rule_numeral_thousands_and() -> Rule {
    rule(
        "<thousands> 0..999",
        vec![number_with(|n| n.value, |v| is_multiple(1000, v)), pred(number_between(0.0, 999.0))],
        |ts| {
            let (v1, v2) = (num_value(ts.first()?)?, num_value(ts.get(1)?)?);
            if 0.0 < v1 && v1 < 1000000.0 {
                double(v1 + v2)
            } else {
                None
            }
        },
    )
}

fn rule_numeral_millions_and() -> Rule {
    rule(
        "<millions> 0..999999",
        vec![number_with(|n| n.value, |v| is_multiple(1000000, v)), pred(number_between(0.0, 999999.0))],
        |ts| {
            let (v1, v2) = (num_value(ts.first()?)?, num_value(ts.get(1)?)?);
            if 0.0 < v1 {
                double(v1 + v2)
            } else {
                None
            }
        },
    )
}

fn rule_numeral_dot_numeral() -> Rule {
    rule(
        "number dot number",
        vec![dimension(Dim::Numeral), regex("(co(n|ma)|punto)"), pred(|t| !has_grain(t))],
        |ts| double(num_value(ts.first()?)? + decimals_to_double(num_value(ts.get(2)?)?)),
    )
}

fn rule_leading_dot_numeral() -> Rule {
    rule("dot number", vec![regex("coma|punto"), pred(|t| !has_grain(t))], |ts| {
        double(decimals_to_double(num_value(ts.get(1)?)?))
    })
}

fn rule_numerals_suffixes_kmg() -> Rule {
    rule(
        "numbers suffixes (K, M, G)",
        vec![dimension(Dim::Numeral), regex(r"([kmg])(?=[\W\$€]|$)")],
        |ts| {
            let v = num_value(ts.first()?)?;
            match ts.get(1)? {
                Token::RegexMatch(g) => match to_lower(g.0.first()?).as_str() {
                    "k" => double(v * 1e3),
                    "m" => double(v * 1e6),
                    "g" => double(v * 1e9),
                    _ => None,
                },
                _ => None,
            }
        },
    )
}

fn rule_numerals_prefix_with_negative_or_minus() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex("-|menos|negativ(o|a)"), pred(is_positive)],
        |ts| double(num_value(ts.get(1)?)? * -1.0),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_numeral_zero_to_fifteen(),
        rule_below_ten_with_two_digits(),
        rule_numeral_sixteen_to_twenty_nine(),
        rule_numeral_sixteen_to_nineteen_with_diez(),
        rule_numeral_twenty_to_ninety_tens(),
        rule_numeral_twenty_one_to_ninety_nine(),
        rule_big_numeral(),
        rule_big_numeral_multipliable(),
        rule_numeral_multiply(),
        rule_numeral_thousands_and(),
        rule_numeral_millions_and(),
        rule_two_part_hundreds(),
        rule_numeral_hundreds_and_smaller(),
        rule_numeral_dot_numeral(),
        rule_leading_dot_numeral(),
        rule_numerals_suffixes_kmg(),
        rule_numerals_prefix_with_negative_or_minus(),
    ]
});

/// `Duckling.Numeral.ES.Rules.rules`.
pub fn rules() -> &'static [Rule] {
    &RULES
}

// ----- locale rules ------------------------------------------------------------------------

/// Comma as decimal separator, dot as thousands separator (AR, CL, CO, ES, PE, VE).
fn comma_decimal_rules() -> Vec<Rule> {
    vec![
        rule("decimal number ,", vec![regex(r"(\d*,\d+)")], |ts| {
            let g = groups(ts)?;
            parse_decimal(false, g.first()?)
        }),
        rule(
            "decimal with thousands separator .",
            vec![regex(r"(\d+(\.\d\d\d)+,\d+)")],
            |ts| {
                let g = groups(ts)?;
                let fmt = g.first()?.replace('.', "").replace(',', ".");
                parse_double(&fmt).and_then(double)
            },
        ),
        rule(
            "integer with thousands separator .",
            vec![regex(r"(\d{1,3}(\.\d\d\d){1,5})")],
            |ts| {
                let g = groups(ts)?;
                parse_double(&g.first()?.replace('.', "")).and_then(double)
            },
        ),
    ]
}

/// `Duckling.Numeral.ES.MX.Rules`: dot as decimal separator, comma as thousands separator.
fn mx_rules() -> Vec<Rule> {
    vec![
        rule("decimal number .", vec![regex(r"(\d*\.\d+)")], |ts| {
            let g = groups(ts)?;
            parse_decimal(false, g.first()?)
        }),
        rule(
            "decimal with thousands separator",
            vec![regex(r"(\d+(\,\d\d\d)+\.\d+)")],
            |ts| {
                let g = groups(ts)?;
                parse_double(&g.first()?.replace(',', "")).and_then(double)
            },
        ),
        rule(
            "integer with thousands separator ,",
            vec![regex(r"(\d{1,3}(\,\d\d\d){1,5})")],
            |ts| {
                let g = groups(ts)?;
                parse_double(&g.first()?.replace(',', "")).and_then(double)
            },
        ),
    ]
}

static COMMA_RULES: Lazy<Vec<Rule>> = Lazy::new(comma_decimal_rules);
static MX_RULES: Lazy<Vec<Rule>> = Lazy::new(mx_rules);

/// `Duckling.Numeral.ES.{AR,CL,CO,ES,PE,VE}.Rules.rules` (identical sets).
pub fn comma_locale_rules() -> &'static [Rule] {
    &COMMA_RULES
}

/// `Duckling.Numeral.ES.MX.Rules.rules`.
pub fn mx_locale_rules() -> &'static [Rule] {
    &MX_RULES
}
