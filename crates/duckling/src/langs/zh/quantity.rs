//! `Duckling.Quantity.ZH.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::{is_positive, number_between};
use crate::dims::quantity::{quantity, Unit};
use crate::dims::util::to_lower;
use crate::types::{groups, pred, regex, rule, Rule, Token};

fn get_value(match_s: &str, v: f64) -> f64 {
    static OPS: Lazy<HashMap<&'static str, fn(f64) -> f64>> = Lazy::new(|| {
        let mut m: HashMap<&'static str, fn(f64) -> f64> = HashMap::new();
        m.insert("千克", |x| x * 1000.0);
        m.insert("公斤", |x| x * 1000.0);
        m.insert("kg", |x| x * 1000.0);
        m.insert("斤", |x| x * 500.0);
        m.insert("两", |x| x * 50.0);
        m.insert("兩", |x| x * 50.0);
        m.insert("毫克", |x| x / 1000.0);
        m.insert("mg", |x| x / 1000.0);
        m
    });
    OPS.get(to_lower(match_s).as_str()).map(|f| f(v)).unwrap_or(v)
}

fn rule_numeral_quantities() -> Rule {
    rule(
        "<quantity> grams",
        vec![pred(is_positive), regex("(千克|公斤|斤|两|兩|克|毫克|kg|g|mg)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(nd), Token::RegexMatch(_)) => {
                let m = groups(&ts[1..])?.first()?.to_string();
                Some(Token::Quantity(quantity(Unit::Gram, get_value(&m, nd.value))))
            }
            _ => None,
        },
    )
}

fn rule_catty_tael() -> Rule {
    rule(
        "<quantity> catty <quantity> tael",
        vec![pred(is_positive), regex("斤"), pred(number_between(1.0, 10.0)), regex("两|兩")],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(x), Token::Numeral(y)) => {
                Some(Token::Quantity(quantity(Unit::Gram, x.value * 500.0 + y.value * 50.0)))
            }
            _ => None,
        },
    )
}

fn rule_catty_half() -> Rule {
    rule("<quantity> catty half", vec![pred(is_positive), regex("斤半")], |ts| match ts.first()? {
        Token::Numeral(x) => Some(Token::Quantity(quantity(Unit::Gram, x.value * 500.0 + 250.0))),
        _ => None,
    })
}

fn rule_tael_half() -> Rule {
    rule("<quantity> tael half", vec![pred(is_positive), regex("(两|兩)半")], |ts| {
        match ts.first()? {
            Token::Numeral(x) => Some(Token::Quantity(quantity(Unit::Gram, x.value * 50.0 + 25.0))),
            _ => None,
        }
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule_catty_tael(), rule_catty_half(), rule_tael_half(), rule_numeral_quantities()]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
