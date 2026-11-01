//! `Duckling.Numeral.MY.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::types::{groups, pred, regex, rule, Rule, Token};

static INTEGER09_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("၀", 0),
        ("၁", 1),
        ("၂", 2),
        ("၃", 3),
        ("၄", 4),
        ("၅", 5),
        ("၆", 6),
        ("၇", 7),
        ("၈", 8),
        ("၉", 9),
    ]
    .into_iter()
    .collect()
});

static INTEGER_PALI_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [("ပထမ", 1), ("ဒုတိယ", 2), ("တတိယ", 3)].into_iter().collect()
});

static INTEGER2_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("တစ်", 1),
        ("နှစ်", 2),
        ("သုံး", 3),
        ("လေး", 4),
        ("ငါး", 5),
        ("ခြေါက်", 6),
        ("ခုနှစ်", 7),
        ("ရှစ်", 8),
        ("ကိုး", 9),
        ("တစ်ဆယ်", 10),
    ]
    .into_iter()
    .collect()
});

fn rule_integer5() -> Rule {
    rule(
        "integer (11..99) ",
        vec![pred(number_between(1.0, 10.0)), regex("ဆယ့်"), pred(number_between(1.0, 10.0))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(v1), Token::Numeral(v2)) => double(v1.value + v2.value * 10.0),
            _ => None,
        },
    )
}

fn rule_integer09() -> Rule {
    rule("integer (0..9) - numeric", vec![regex("(၀|၁|၂|၃|၄|၅|၆|၇|၈|၉)")], |ts| {
        let g = groups(ts)?;
        INTEGER09_MAP.get(g.first()?.as_ref()).and_then(|v| integer(*v))
    })
}

fn rule_integer3() -> Rule {
    rule(
        "integer (11..19) ",
        vec![regex("ဆယ့်"), pred(number_between(1.0, 10.0))],
        |ts| match ts.get(1)? {
            Token::Numeral(v) => double(v.value + 10.0),
            _ => None,
        },
    )
}

fn rule_integer_pali() -> Rule {
    rule("integer (1..3) - pali", vec![regex("(ပထမ|ဒုတိယ|တတိယ)")], |ts| {
        let g = groups(ts)?;
        INTEGER_PALI_MAP.get(g.first()?.as_ref()).and_then(|v| integer(*v))
    })
}

fn rule_integer6() -> Rule {
    rule("integer (100..900)", vec![pred(number_between(1.0, 10.0)), regex("ရာ")], |ts| {
        match ts.first()? {
            Token::Numeral(v) => double(v.value * 100.0),
            _ => None,
        }
    })
}

fn rule_integer7() -> Rule {
    rule("integer (1000..9000)", vec![pred(number_between(1.0, 10.0)), regex("ထောင်")], |ts| {
        match ts.first()? {
            Token::Numeral(v) => double(v.value * 1000.0),
            _ => None,
        }
    })
}

fn rule_integer8() -> Rule {
    rule("integer (10000..90000)", vec![pred(number_between(1.0, 10.0)), regex("သောင်း")], |ts| {
        match ts.first()? {
            Token::Numeral(v) => double(v.value * 10000.0),
            _ => None,
        }
    })
}

fn rule_integer() -> Rule {
    rule("integer 0", vec![regex("သုံည|မရှိ")], |_| integer(0))
}

fn rule_integer4() -> Rule {
    rule("integer (10..90)", vec![pred(number_between(1.0, 10.0)), regex("ဆယ်")], |ts| {
        match ts.first()? {
            Token::Numeral(v) => double(v.value * 10.0),
            _ => None,
        }
    })
}

fn rule_integer2() -> Rule {
    rule(
        "integer (1..10)",
        vec![regex("(တစ်|နှစ်|သုံး|လေး|ငါး|ခြေါက်|ခုနှစ်|ရှစ်|ကိုး|တစ်ဆယ်)")],
        |ts| {
            let g = groups(ts)?;
            INTEGER2_MAP.get(g.first()?.as_ref()).and_then(|v| integer(*v))
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_integer(),
        rule_integer2(),
        rule_integer3(),
        rule_integer4(),
        rule_integer5(),
        rule_integer6(),
        rule_integer7(),
        rule_integer8(),
        rule_integer09(),
        rule_integer_pali(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
