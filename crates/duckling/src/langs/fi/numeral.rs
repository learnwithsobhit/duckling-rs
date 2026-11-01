//! `Duckling.Numeral.FI.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::{integer, parse_int_token};
use crate::dims::util::to_lower;
use crate::types::{group, groups, regex, rule, Rule, Token};

static NUMERAL_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("nolla", 0),
        ("yksi", 1),
        ("kaksi", 2),
        ("kolme", 3),
        ("neljä", 4),
        ("viisi", 5),
        ("kuusi", 6),
        ("seitsemän", 7),
        ("kahdeksan", 8),
        ("yhdeksän", 9),
        ("kymmenen", 10),
    ]
    .into_iter()
    .collect()
});

static ELEVEN_TO_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("yksitoista", 11),
        ("kaksitoista", 12),
        ("kolmetoista", 13),
        ("neljätoista", 14),
        ("viisitoista", 15),
        ("kuusitoista", 16),
        ("seitsemäntoista", 17),
        ("kahdeksantoista", 18),
        ("yhdeksäntoista", 19),
    ]
    .into_iter()
    .collect()
});

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("kaksikymmentä", 20),
        ("kolmekymmentä", 30),
        ("neljäkymmentä", 40),
        ("viisikymmentä", 50),
        ("kuusikymmentä", 60),
        ("seitsemänkymmentä", 70),
        ("kahdeksankymmentä", 80),
        ("yhdeksänkymmentä", 90),
    ]
    .into_iter()
    .collect()
});

fn lookup_int(map: &HashMap<&'static str, i64>, ts: &[&Token]) -> Option<Token> {
    let g = groups(ts)?;
    map.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
}

fn rule_integer_numeric() -> Rule {
    rule("integer (numeric)", vec![regex("(\\d{1,18})")], |ts| {
        parse_int_token(&group(ts, 0)?)
    })
}

fn rule_numeral() -> Rule {
    rule(
        "number (0..10)",
        vec![regex(
            "(nolla|yksi|kaksi|kolme|neljä|viisi|kuusi|seitsemän|kahdeksan|yhdeksän|kymmenen)",
        )],
        |ts| lookup_int(&NUMERAL_MAP, ts),
    )
}

fn rule_eleven_to_nineteen() -> Rule {
    rule(
        "number (11..19)",
        vec![regex(
            "(yksitoista|kaksitoista|kolmetoista|neljätoista|viisitoista|kuusitoista|seitsemäntoista|kahdeksantoista|yhdeksäntoista)",
        )],
        |ts| lookup_int(&ELEVEN_TO_NINETEEN, ts),
    )
}

fn rule_tens() -> Rule {
    rule(
        "integer (20,30..90)",
        vec![regex(
            "(kaksikymmentä|kolmekymmentä|neljäkymmentä|viisikymmentä|kuusikymmentä|seitsemänkymmentä|kahdeksankymmentä|yhdeksänkymmentä)",
        )],
        |ts| lookup_int(&TENS_MAP, ts),
    )
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer ([2-9][1-9])",
        vec![regex(
            "(kaksikymmentä|kolmekymmentä|neljäkymmentä|viisikymmentä|kuusikymmentä|seitsemänkymmentä|kahdeksankymmentä|yhdeksänkymmentä)(yksi|kaksi|kolme|neljä|viisi|kuusi|seitsemän|kahdeksan|yhdeksän)",
        )],
        |ts| {
            let g = groups(ts)?;
            let v1 = TENS_MAP.get(to_lower(g.first()?).as_str())?;
            let v2 = NUMERAL_MAP.get(to_lower(g.get(1)?).as_str())?;
            integer(v1 + v2)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_integer_numeric(),
        rule_numeral(),
        rule_eleven_to_nineteen(),
        rule_tens(),
        rule_composite_tens(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
