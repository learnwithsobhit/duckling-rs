//! `Duckling.Time.ZH.CN.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time::helpers::*;
use crate::types::{regex, rule, Rule};

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule("national day", vec![regex("(国庆|國慶)(节|節)?")], |_| tt(month_day(10, 1)))]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
