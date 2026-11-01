//! `Duckling.Numeral.HE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, group, pred, regex, rule, Dim, Rule, Token};

fn nv(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

fn grain_of(t: &Token) -> Option<i64> {
    match t {
        Token::Numeral(n) => n.grain,
        _ => None,
    }
}

fn add_if_grain(ts: &[&Token], a: usize, b: usize) -> Option<Token> {
    let val1 = nv(ts.get(a)?)?;
    let val2 = nv(ts.get(b)?)?;
    let g = grain_of(ts.get(a)?)?;
    if 10f64.powf(g as f64) > val2 {
        double(val1 + val2)
    } else {
        None
    }
}

fn rule_integer5() -> Rule {
    rule("integer 4", vec![regex("(ארבע(ה)?)")], |_| integer(4))
}

fn rule_intersect_numerals() -> Rule {
    rule(
        "intersect numbers",
        vec![pred(has_grain), pred(|t| !is_multipliable(t) && is_positive(t))],
        |ts| add_if_grain(ts, 0, 1),
    )
}

fn rule_intersect_with_and() -> Rule {
    rule(
        "intersect (with and)",
        vec![pred(has_grain), regex("ו"), pred(is_multipliable)],
        |ts| add_if_grain(ts, 0, 2),
    )
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer 21..99",
        vec![
            one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| double(nv(ts.first()?)? + nv(ts.get(1)?)?),
    )
}

fn rule_composite_tens_with_and() -> Rule {
    rule(
        "integer 21..99 (with and)",
        vec![
            one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
            regex("ו"),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| double(nv(ts.first()?)? + nv(ts.get(2)?)?),
    )
}

fn rule_numerals_prefix_with_negative_or_minus() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex("-|מינוס"), pred(is_positive)],
        |ts| double(nv(ts.get(1)?)? * -1.0),
    )
}

fn rule_integer10() -> Rule {
    rule("integer 9", vec![regex("(תשע(ה)?)")], |_| integer(9))
}

fn rule_integer15() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(עשרים|שלושים|ארבעים|חמישים|שישים|שבעים|שמונים|תשעים)")],
        |ts| match group(ts, 0)?.as_ref() {
            "עשרים" => integer(20),
            "שלושים" => integer(30),
            "ארבעים" => integer(40),
            "חמישים" => integer(50),
            "שישים" => integer(60),
            "שבעים" => integer(70),
            "שמונים" => integer(80),
            "תשעים" => integer(90),
            _ => None,
        },
    )
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex("(\\d*\\.\\d+)")], |ts| {
        parse_decimal(true, group(ts, 0)?.as_ref())
    })
}

fn rule_integer3() -> Rule {
    rule("integer 2", vec![regex("(שתיים|שניים)")], |_| integer(2))
}

fn rule_single() -> Rule {
    rule("single", vec![regex("יחיד")], |_| integer(1))
}

fn rule_integer13() -> Rule {
    rule("integer 12", vec![regex("(שניים עשר|תרי עשר)")], |_| integer(12))
}

fn rule_multiply() -> Rule {
    rule(
        "compose by multiplication",
        vec![dimension(Dim::Numeral), pred(is_multipliable)],
        |ts| multiply(ts.first()?, ts.get(1)?),
    )
}

fn rule_integer6() -> Rule {
    rule("integer 5", vec![regex("(חמ(ש|ישה))")], |_| integer(5))
}

fn rule_powers_of_ten() -> Rule {
    rule(
        "powers of tens",
        vec![regex("(מא(ה|ות)|אל(ף|פים)|מיליו(ן|נים))")],
        |ts| match to_lower(group(ts, 0)?.as_ref()).as_str() {
            "מאה" => double(1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t)),
            "מאות" => double(1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t)),
            "אלף" => double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t)),
            "אלפים" => double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t)),
            "מיליון" => double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t)),
            "מיליונים" => double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t)),
            _ => None,
        },
    )
}

fn rule_integer7() -> Rule {
    rule("integer 6", vec![regex("(שש(ה)?)")], |_| integer(6))
}

fn rule_integer14() -> Rule {
    rule(
        "integer 11..19",
        vec![pred(number_between(1.0, 10.0)), number_with(|n| n.value, |v| v == 10.0)],
        |ts| double(nv(ts.first()?)? + nv(ts.get(1)?)?),
    )
}

fn rule_integer8() -> Rule {
    rule("integer 7", vec![regex("(שבע(ה)?)")], |_| integer(7))
}

fn rule_couple() -> Rule {
    rule("couple", vec![regex("זוג( של)?")], |_| integer(2))
}

fn rule_integer16() -> Rule {
    rule(
        "integer 101..999",
        vec![
            one_of(vec![300.0, 600.0, 500.0, 100.0, 800.0, 200.0, 900.0, 700.0, 400.0]),
            pred(number_between(1.0, 100.0)),
        ],
        |ts| double(nv(ts.first()?)? + nv(ts.get(1)?)?),
    )
}

fn rule_integer9() -> Rule {
    rule("integer 8", vec![regex("(שמונה)")], |_| integer(8))
}

fn rule_integer() -> Rule {
    rule("integer 0", vec![regex("(אפס|כלום)")], |_| integer(0))
}

fn rule_integer4() -> Rule {
    rule("integer 3", vec![regex("(שלוש(ה)?)")], |_| integer(3))
}

fn rule_integer2() -> Rule {
    rule("integer 1", vec![regex("(אחד|אחת)")], |_| integer(1))
}

fn rule_integer11() -> Rule {
    rule("integer 10", vec![regex("(עשר(ה)?)")], |_| integer(10))
}

fn rule_numeral_dot_numeral() -> Rule {
    rule(
        "number dot number",
        vec![dimension(Dim::Numeral), regex("נקודה"), pred(|t| !has_grain(t))],
        |ts| double(nv(ts.first()?)? + decimals_to_double(nv(ts.get(2)?)?)),
    )
}

fn rule_commas() -> Rule {
    rule(
        "comma-separated numbers",
        vec![regex("(\\d+(,\\d\\d\\d)+(\\.\\d+)?)")],
        |ts| parse_double(&group(ts, 0)?.replace(',', "")).and_then(double),
    )
}

fn rule_half() -> Rule {
    rule("half", vec![regex("חצי")], |_| double(0.5))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_commas(),
        rule_composite_tens(),
        rule_composite_tens_with_and(),
        rule_couple(),
        rule_decimal_numeral(),
        rule_integer(),
        rule_integer10(),
        rule_integer11(),
        rule_integer13(),
        rule_integer14(),
        rule_integer15(),
        rule_integer16(),
        rule_integer2(),
        rule_integer3(),
        rule_integer4(),
        rule_integer5(),
        rule_integer6(),
        rule_integer7(),
        rule_integer8(),
        rule_integer9(),
        rule_intersect_numerals(),
        rule_intersect_with_and(),
        rule_multiply(),
        rule_numeral_dot_numeral(),
        rule_numerals_prefix_with_negative_or_minus(),
        rule_powers_of_ten(),
        rule_single(),
        rule_half(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
