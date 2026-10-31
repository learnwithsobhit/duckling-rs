//! `Duckling.Numeral.Rules`: language-independent numeral rules.

use once_cell::sync::Lazy;

use crate::dims::numeral::*;
use crate::types::{group, groups, regex, rule, Rule, Token};

fn rule_integer_numeric() -> Rule {
    rule("integer (numeric)", vec![regex(r"(\d{1,18})")], |ts| {
        let m = group(ts, 0)?;
        parse_int_token(&m)
    })
}

fn rule_fractions() -> Rule {
    rule("fractional number", vec![regex(r"(\d+)/(\d+)")], |ts| {
        let g = groups(ts)?;
        let n = parse_decimal(false, g.first()?)?;
        let d = parse_decimal(false, g.get(1)?)?;
        divide(&n, &d).and_then(|t| not_ok_for_any_time(&t))
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![rule_integer_numeric(), rule_fractions()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}

#[allow(dead_code)]
fn _unused(_: &Token) {}
