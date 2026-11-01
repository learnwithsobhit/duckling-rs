//! `Duckling.Ordinal.UK.Rules`.

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
        ("перш", 1),
        ("друг", 2),
        ("трет", 3),
        ("четверт", 4),
        ("п‘ят", 5),
        ("шост", 6),
        ("сьом", 7),
        ("восьм", 8),
        ("дев‘ят", 9),
        ("десят", 10),
        ("одинадцят", 11),
        ("дванадцят", 12),
        ("тринадцят", 13),
        ("чотирнадцят", 14),
        ("п‘ятнадцят", 15),
        ("шістнадцят", 16),
        ("сімнадцят", 17),
        ("вісімнадцят", 18),
        ("дев‘ятнадцят", 19),
        ("двадцят", 20),
    ]
    .into_iter()
    .collect()
});

fn rule_ordinals_firstth() -> Rule {
    rule(
        "ordinals (first..19th)",
        vec![regex(
            "(перш|друг|трет|четверт|п‘ят|шост|сьом|восьм|дев‘ят|десят|одинадцят|дванадцят|тринадцят|чотирнадцят|п‘ятнадцят|шістнадцят|сімнадцят|вісімнадцят|дев‘ятнадцят|двадцят)(ий|ій|а|я|е|є)",
        )],
        |ts| {
            let g = groups(ts)?;
            ORDINALS_FIRSTTH_MAP.get(to_lower(g.first()?).as_str()).copied().and_then(ordinal)
        },
    )
}

static ORDINAL_TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("двадцять", 20),
        ("тридцять", 30),
        ("сорок", 40),
        ("п‘ятдесят", 50),
        ("шістдесят", 60),
        ("сімдесят", 70),
        ("вісімдесят", 80),
        ("дев‘яносто", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_ordinal() -> Rule {
    rule(
        "ordinal 21..99",
        vec![
            regex("(двадцять|тридцять|сорок|п‘ятдесят|шістьдесят|сімдесят|вісімдесят|дев‘яносто)"),
            regex("(перш|друг|трет|четверт|п‘ят|шост|сьом|восьм|дев‘ят)(ий|ій|а|я|е|є)"),
        ],
        |ts| {
            let m1 = groups(ts)?.first()?;
            let m2 = match ts.get(1)? {
                Token::RegexMatch(g) => g.0.first()?,
                _ => return None,
            };
            let v1 = *ORDINAL_TENS_MAP.get(to_lower(m1).as_str())?;
            let v2 = *ORDINALS_FIRSTTH_MAP.get(to_lower(m2).as_str())?;
            ordinal(v1 + v2)
        },
    )
}

fn rule_ordinal_digits() -> Rule {
    rule(
        "ordinal (digits)",
        vec![regex("0*(\\d+)-?((и|і)?й|а|я|е|є)")],
        |ts| parse_int(groups(ts)?.first()?).and_then(ordinal),
    )
}

static RULES: Lazy<Vec<Rule>> =
    Lazy::new(|| vec![rule_ordinal(), rule_ordinal_digits(), rule_ordinals_firstth()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
