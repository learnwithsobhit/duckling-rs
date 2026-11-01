//! `Duckling.Numeral.ML.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{groups, regex, rule, Rule};

static ZERO_TO_NINE_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("പൂജ്യം", 0),
        ("ഒന്ന്", 1),
        ("രണ്ട്", 2),
        ("മുന്ന്", 3),
        ("നാല്", 4),
        ("അഞ്ച്", 5),
        ("ആറ്", 6),
        ("ഏഴ്", 7),
        ("എട്ട്", 8),
        ("ഒൻപത്", 9),
    ]
    .into_iter()
    .collect()
});

fn rule_zero_to_nine() -> Rule {
    rule(
        "integer (0..9)",
        vec![regex("(പൂജ്യം|ഒന്ന്|രണ്ട്|മുന്ന്|നാല്|അഞ്ച്|ആറ്|ഏഴ്|എട്ട്|ഒൻപത്)")],
        |ts| {
            ZERO_TO_NINE_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(integer)
        },
    )
}

static TEN_TO_NINETEEN_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("പത്ത്", 10),
        ("പതിനൊന്ന്", 11),
        ("പന്ത്രണ്ട്", 12),
        ("പതിമൂന്ന്", 13),
        ("പതിനാല്", 14),
        ("പതിനഞ്ച്", 15),
        ("പതിനാറ്", 16),
        ("പതിനേഴ്", 17),
        ("പതിനെട്ട്", 18),
        ("പത്തൊമ്പത്", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_ten_to_nineteen() -> Rule {
    rule(
        "integer (10..19)",
        vec![regex(
            "(പത്ത്|പതിനൊന്ന്|പന്ത്രണ്ട്|പതിമൂന്ന്|പതിനാല്|പതിനഞ്ച്|പതിനാറ്|പതിനേഴ്|പതിനെട്ട്|പത്തൊമ്പത്)",
        )],
        |ts| {
            TEN_TO_NINETEEN_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(integer)
        },
    )
}

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ഇരുപത്", 20),
        ("ഇരുപത്തി", 20),
        ("മുപ്പത്", 30),
        ("മുപ്പത്തി", 30),
        ("നാല്പത്", 40),
        ("നാല്പത്തി", 40),
        ("അമ്പത്", 50),
        ("അമ്പത്തി", 50),
        ("അറുപത്", 60),
        ("അറുപത്തി", 60),
        ("എഴുപത്", 70),
        ("എഴുപത്തി", 70),
        ("എൺപത്", 80),
        ("എൺപത്തി", 80),
        ("തൊണ്ണൂറ്", 90),
        ("തൊണ്ണൂറ്റി", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_tens() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(ഇരുപത്|മുപ്പത്|നാല്പത്|അമ്പത്|അറുപത്|എഴുപത്|എൺപത്|തൊണ്ണൂറ്)")],
        |ts| {
            TENS_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(integer)
        },
    )
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer ([2-9][1-9])",
        vec![regex("(ഇരുപത്തി|മുപ്പത്തി|നാല്പത്തി|അമ്പത്തി|അറുപത്തി|എഴുപത്തി|എൺപത്തി|തൊണ്ണൂറ്റി)(ഒന്ന്|രണ്ട്|മുന്ന്|നാല്|അഞ്ച്|ആറ്|ഏഴ്|എട്ട്|ഒൻപത്)")],
        |ts| {
            let g = groups(ts)?;
            let v1 = *TENS_MAP.get(to_lower(g.first()?).as_str())?;
            let v2 = *ZERO_TO_NINE_MAP.get(to_lower(g.get(1)?).as_str())?;
            integer(v1 + v2)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_zero_to_nine(),
        rule_ten_to_nineteen(),
        rule_composite_tens(),
        rule_tens(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
