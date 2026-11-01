//! `Duckling.Ordinal.AR.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::to_lower;
use crate::types::{groups, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

static ORDINALS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("اول", 1),
        ("أول", 1),
        ("حاد", 1),
        ("حادي", 1),
        ("واحد", 1),
        ("ثان", 2),
        ("ثاني", 2),
        ("ثالث", 3),
        ("رابع", 4),
        ("خامس", 5),
        ("سادس", 6),
        ("سابع", 7),
        ("ثامن", 8),
        ("تاسع", 9),
        ("عاشر", 10),
    ]
    .into_iter()
    .collect()
});

static CARDINALS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("عشر", 20),
        ("ثلاث", 30),
        ("اربع", 40),
        ("خمس", 50),
        ("ست", 60),
        ("سبع", 70),
        ("ثمان", 80),
        ("تسع", 90),
    ]
    .into_iter()
    .collect()
});

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "ordinals (composite, e.g., eighty-seven)",
            vec![regex("ال(واحد|حادي?|ثاني?|ثالث|رابع|خامس|سادس|سابع|ثامن|تاسع|عاشر) و ?ال(عشر|ثلاث|اربع|خمس|ست|سبع|ثمان|تسع)(ون|ين)")],
            |ts| {
                let g = groups(ts)?;
                let tt = ORDINALS_MAP.get(to_lower(g.first()?).as_str())?;
                let uu = CARDINALS_MAP.get(to_lower(g.get(1)?).as_str())?;
                ordinal(*tt + *uu)
            },
        ),
        rule(
            "ordinals (first..tenth)",
            vec![regex("(?:ال)?([أا]ول|ثاني?|ثالث|رابع|خامس|سادس|سابع|ثامن|تاسع|عاشر)[ةهى]?")],
            |ts| {
                ORDINALS_MAP
                    .get(to_lower(groups(ts)?.first()?).as_str())
                    .and_then(|v| ordinal(*v))
            },
        ),
        rule("ordinals (eleventh)", vec![regex("ال([اأإ]حد[يى]?|حاد(ي[ةه]?)?) ?عشر[ةه]?")], |_| {
            ordinal(11)
        }),
        rule("ordinals (twelveth)", vec![regex("ال([اأإ]ثن[يى]?|ثان(ي[ةه]?)?) ?عشر[ةه]?")], |_| {
            ordinal(12)
        }),
        rule(
            "ordinals (thirtieth..nineteenth)",
            vec![regex("ال(ثالث|رابع|خامس|سادس|سابع|ثامن|تاسع)[ةه]? ?عشرة?")],
            |ts| {
                let uu = ORDINALS_MAP.get(to_lower(groups(ts)?.first()?).as_str())?;
                ordinal(10 + *uu)
            },
        ),
        rule(
            "ordinals (twenty, thirty..ninety)",
            vec![regex("ال(عشر|ثلاث|اربع|خمس|ست|سبع|ثمان|تسع)(ون|ين)")],
            |ts| {
                CARDINALS_MAP
                    .get(to_lower(groups(ts)?.first()?).as_str())
                    .and_then(|v| ordinal(*v))
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
