//! `Duckling.Quantity.AR.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::quantity::{quantity, QuantityData, Unit};
use crate::dims::util::to_lower;
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

fn tok(d: QuantityData) -> Option<Token> {
    Some(Token::Quantity(d))
}

fn numeral_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

fn match_text(t: &Token) -> Option<String> {
    match t {
        Token::RegexMatch(g) => g.0.first().map(|s| s.to_string()),
        _ => None,
    }
}

static OPS_MAP: Lazy<HashMap<&'static str, fn(f64) -> f64>> = Lazy::new(|| {
    let times2: fn(f64) -> f64 = |v| v * 2.0;
    let div500: fn(f64) -> f64 = |v| v / 500.0;
    let div1000: fn(f64) -> f64 = |v| v / 1000.0;
    let times1000: fn(f64) -> f64 = |v| v * 1000.0;
    let times2000: fn(f64) -> f64 = |v| v * 2000.0;
    [
        ("غرامان", times2),
        ("غرامين", times2),
        ("كوبان", times2),
        ("كوبين", times2),
        ("باوندان", times2),
        ("باوندين", times2),
        ("اونصتان", times2),
        ("اونصتين", times2),
        ("أونصتان", times2),
        ("أونصتين", times2),
        ("جرامان", times2),
        ("جرامين", times2),
        ("ميلي غرامان", div500),
        ("ميليغرامان", div500),
        ("ميلغرامان", div500),
        ("ميلي غرامين", div500),
        ("ميليغرامين", div500),
        ("ميلغرامين", div500),
        ("ميلي جرامان", div500),
        ("ميليجرامان", div500),
        ("ميلجرامان", div500),
        ("ميلي جرامين", div500),
        ("ميليجرامين", div500),
        ("ميلجرامين", div500),
        ("ميلي غرام", div1000),
        ("ميليغرام", div1000),
        ("ميلغرام", div1000),
        ("كيلوغرام", times1000),
        ("كيلو غرام", times1000),
        ("ميلي غرامات", div1000),
        ("ميليغرامات", div1000),
        ("ميلغرامات", div1000),
        ("ملغ", div1000),
        ("كغ", times1000),
        ("كغم", times1000),
        ("ميلي جرام", div1000),
        ("ميليجرام", div1000),
        ("ميلجرام", div1000),
        ("ميلي جرامات", div1000),
        ("ميليجرامات", div1000),
        ("ميلجرامات", div1000),
        ("كيلوغرامات", times1000),
        ("كيلو غرامات", times1000),
        ("ملج", div1000),
        ("كيلوجرام", times1000),
        ("كيلو جرام", times1000),
        ("كيلوجرامات", times1000),
        ("كيلو جرامات", times1000),
        ("كج", times1000),
        ("كجم", times1000),
        ("كيلوغرامان", times2000),
        ("كيلوغرامين", times2000),
        ("كيلو غرامان", times2000),
        ("كيلو غرامين", times2000),
        ("كيلوجرامان", times2000),
        ("كيلوجرامين", times2000),
        ("كيلو جرامان", times2000),
        ("كيلو جرامين", times2000),
    ]
    .into_iter()
    .collect()
});

fn get_value(m: &str, v: f64) -> f64 {
    match OPS_MAP.get(to_lower(m).as_str()) {
        Some(op) => op(v),
        None => v,
    }
}

const QUANTITIES: [(&str, &str, Unit); 4] = [
    ("<quantity> cups", "(كوب(ان|ين)?|[أا]كواب)", Unit::Cup),
    (
        "<quantity> grams",
        "(((كيلو|مي?لي?) ?)?((غ|ج)رام(ات|ين|ان)?)|ك(غ|ج)م?|مل(غ|ج)|(غ|ج)م)",
        Unit::Gram,
    ),
    ("<quantity> lb", "(باوند(ان|ين)?)", Unit::Pound),
    ("<quantity> oz", "([أا]ونص([ةه]|تان|تين|ات))", Unit::Ounce),
];

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v: Vec<Rule> = vec![rule(
        "<quantity> of product",
        vec![dimension(Dim::Quantity), regex("من ([ء-ي]+)")],
        |ts| {
            let q = match ts.first()? {
                Token::Quantity(q) => q.clone(),
                _ => return None,
            };
            let product = match ts.get(1)? {
                Token::RegexMatch(g) => g.0.first()?.to_string(),
                _ => return None,
            };
            let mut q = q;
            q.aproduct = Some(product);
            tok(q)
        },
    )];
    for (name, re, unit) in QUANTITIES {
        v.push(rule(
            name,
            vec![
                crate::types::pred(|t| matches!(t, Token::Numeral(n) if n.value > 0.0)),
                regex(re),
            ],
            move |ts| {
                let n = numeral_value(ts.first()?)?;
                let m = match_text(ts.get(1)?)?;
                tok(quantity(unit.clone(), get_value(&m, n)))
            },
        ));
    }
    for (name, re, unit) in QUANTITIES {
        v.push(rule(name, vec![regex(re)], move |ts| {
            let m = match_text(ts.first()?)?;
            tok(quantity(unit.clone(), get_value(&m, 1.0)))
        }));
    }
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
