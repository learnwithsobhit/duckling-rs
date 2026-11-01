//! `Duckling.Ordinal.MN.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

static ORDINALS_FIRSTTH_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("нэг", 1),
        ("хоёр", 2),
        ("гурав", 3),
        ("дөрөв", 4),
        ("тав", 5),
        ("зургаа", 6),
        ("долоо", 7),
        ("найм", 8),
        ("ес", 9),
        ("арав", 10),
    ]
    .into_iter()
    .collect()
});

static CARDINALS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("арван", 10),
        ("хорин", 20),
        ("хорь", 20),
        ("гучин", 30),
        ("гуч", 30),
        ("дөчин", 40),
        ("дөч", 40),
        ("тавин", 50),
        ("тавь", 50),
        ("жаран", 60),
        ("жар", 60),
        ("далан", 70),
        ("дал", 70),
        ("наян", 80),
        ("ная", 80),
        ("ерэн", 90),
        ("ер", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_ordinals_firstth() -> Rule {
    rule(
        "ordinals (first..19th)",
        vec![regex("(нэг|хоёр|гурав|дөрөв|тав|зургаа|долоо|найм|ес|арав) ?(дугаар|дүгээр|дахь|дэх)")],
        |ts| {
            let g = groups(ts)?;
            ORDINALS_FIRSTTH_MAP.get(to_lower(g.first()?).as_str()).copied().and_then(ordinal)
        },
    )
}

fn rule_ordinal() -> Rule {
    // Haskell has one regex pattern item but matches two RegexMatch tokens, so this never fires.
    rule(
        "ordinal 10..99",
        vec![regex(
            "(арван|хорин|гучин|дөчин|тавин|жаран|далан|наян|ерэн) ?(нэг|хоёр|гурав|дөрөв|тав|зургаа|долоо|найм|ес) ?(дугаар|дүгээр|дахь|дэх)",
        )],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::RegexMatch(g1), Token::RegexMatch(g2)) => {
                let dozen = *CARDINALS_MAP.get(to_lower(g1.0.first()?).as_str())?;
                let unit = *ORDINALS_FIRSTTH_MAP.get(to_lower(g2.0.first()?).as_str())?;
                ordinal(dozen + unit)
            }
            _ => None,
        },
    )
}

fn rule_integer2() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(хорь|гуч|дөч|тавь|жар|дал|ная|ер) ?(дугаар|дүгээр|дахь|дэх)")],
        |ts| {
            let g = groups(ts)?;
            CARDINALS_MAP.get(to_lower(g.first()?).as_str()).copied().and_then(ordinal)
        },
    )
}

fn rule_ordinal_digits() -> Rule {
    rule("ordinal (digits)", vec![regex("0*(\\d+)-?(ын|ийн|р|с|)")], |ts| {
        parse_int(groups(ts)?.first()?).and_then(ordinal)
    })
}

fn rule_ordinal_digits2() -> Rule {
    rule(
        "ordinal (digits)",
        vec![regex("(?<!\\d|\\.)0*(\\d+)(\\.(?!\\d)| ?(дугаар|дүгээр|дахь|дэх))")],
        |ts| parse_int(groups(ts)?.first()?).and_then(ordinal),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_ordinal_digits(),
        rule_ordinal_digits2(),
        rule_ordinal(),
        rule_integer2(),
        rule_ordinals_firstth(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
