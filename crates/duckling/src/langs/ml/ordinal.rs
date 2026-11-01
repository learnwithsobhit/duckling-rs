//! `Duckling.Ordinal.ML.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

static ONE_TO_NINETEEN_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ഒന്നാം", 1),
        ("രണ്ടാം", 2),
        ("മൂന്നാം", 3),
        ("നാലാം", 4),
        ("അഞ്ചാം", 5),
        ("ആറാം", 6),
        ("ഏഴാം", 7),
        ("എട്ടാം", 8),
        ("ഒമ്പതാം", 9),
        ("പത്താം", 10),
        ("പതിനൊന്നാം", 11),
        ("പന്ത്രണ്ടാം", 12),
        ("പതിമൂന്നാം", 13),
        ("പതിനാലാം", 14),
        ("പതിനഞ്ചാം", 15),
        ("പതിനാറാം", 16),
        ("പതിനേഴാം", 17),
        ("പതിനെട്ടാം", 18),
        ("പത്തൊൻപതാം", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_one_to_nineteen() -> Rule {
    rule(
        "integer (1..19)",
        vec![regex("(ഒന്നാം|രണ്ടാം|മൂന്നാം|നാലാം|അഞ്ചാം|ആറാം|ഏഴാം|എട്ടാം|ഒമ്പതാം|പത്താം|പതിനൊന്നാം|പന്ത്രണ്ടാം|പതിമൂന്നാം|പതിനാലാം|പതിനഞ്ചാം|പതിനാറാം|പതിനേഴാം|പതിനെട്ടാം|പത്തൊൻപതാം)")],
        |ts| {
            ONE_TO_NINETEEN_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(ordinal)
        },
    )
}

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ഇരുപതാം", 20),
        ("മുപ്പത്തഞ്ചാം", 30),
        ("നാല്പതാം", 40),
        ("അമ്പതാം", 50),
        ("അറുപതാം", 60),
        ("എഴുപത്താം", 70),
        ("എൺപത്താം", 80),
        ("തൊണ്ണൂറാം", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_tens() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex(
            "(ഇരുപതാം|മുപ്പത്തഞ്ചാം|നാല്പതാം|അമ്പതാം|അറുപതാം|എഴുപത്താം|എൺപത്താം|തൊണ്ണൂറാം)",
        )],
        |ts| {
            TENS_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(ordinal)
        },
    )
}

static TENS_ORDINAL_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ഇരുപത്തി", 20),
        ("മുപ്പത്തി", 30),
        ("നാല്പത്തി", 40),
        ("അമ്പത്തി", 50),
        ("അറുപത്തി", 60),
        ("എഴുപത്തി", 70),
        ("എൺപത്തി", 80),
        ("തൊണ്ണൂറ്റി", 90),
    ]
    .into_iter()
    .collect()
});

/// `HashMap.filterWithKey (\_ v -> v <= 9) oneToNineteenMap`.
static ONE_TO_NINE_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    ONE_TO_NINETEEN_MAP
        .iter()
        .filter(|(_, v)| **v <= 9)
        .map(|(k, v)| (*k, *v))
        .collect()
});

fn rule_composite_tens() -> Rule {
    rule(
        "integer ([2-9][1-9])",
        vec![regex("(ഇരുപത്തി|മുപ്പത്തി|നാല്പത്തി|അമ്പത്തി|അറുപത്തി|എഴുപത്തി|എൺപത്തി|തൊണ്ണൂറ്റി)(ആദ്യം|രണ്ടാം|മൂന്നാം|നാലാം|അഞ്ചാം|ആറാം|ഏഴാം|എട്ടാം|ഒമ്പത)")],
        |ts| {
            let g = groups(ts)?;
            let v1 = *TENS_ORDINAL_MAP.get(to_lower(g.first()?).as_str())?;
            let v2 = *ONE_TO_NINE_MAP.get(to_lower(g.get(1)?).as_str())?;
            ordinal(v1 + v2)
        },
    )
}

fn rule_ordinal_digits() -> Rule {
    rule("ordinal (digits)", vec![regex("0*(\\d+)\\.")], |ts| {
        parse_int(groups(ts)?.first()?).and_then(ordinal)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_ordinal_digits(),
        rule_one_to_nineteen(),
        rule_tens(),
        rule_composite_tens(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
