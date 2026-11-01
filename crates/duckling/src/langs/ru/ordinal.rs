//! `Duckling.Ordinal.RU.Rules`.

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
        ("перв", 1),
        ("втор", 2),
        ("трет", 3),
        ("четверт", 4),
        ("четвёрт", 4),
        ("пят", 5),
        ("шест", 6),
        ("седьм", 7),
        ("восьм", 8),
        ("девят", 9),
        ("десят", 10),
        ("одиннадцат", 11),
        ("двенадцат", 12),
        ("тринадцат", 13),
        ("четырнадцат", 14),
        ("пятнадцат", 15),
        ("шестнадцат", 16),
        ("семнадцат", 17),
        ("восемнадцат", 18),
        ("девятнадцат", 19),
        ("двадцат", 20),
        ("тридцат", 30),
        ("сороков", 40),
        ("пятидесят", 50),
        ("шестидесят", 60),
        ("семидесят", 70),
        ("восьмидесят", 80),
        ("девяност", 90),
        ("сот", 100),
    ]
    .into_iter()
    .collect()
});

static CARDINALS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("двадцать", 20),
        ("тридцать", 30),
        ("сорок", 40),
        ("пятьдесят", 50),
        ("шестьдесят", 60),
        ("семьдесят", 70),
        ("восемьдесят", 80),
        ("девяносто", 90),
        ("сто", 100),
    ]
    .into_iter()
    .collect()
});

fn rule_ordinals_firstth() -> Rule {
    rule(
        "ordinals (first..20th, then 30th, 40th, ..., 100th)",
        vec![regex(
            "(перв|втор|трет|четв[её]рт|пят|шест|седьм|восьм|девят|десят|одиннадцат|двенадцат|тринадцат|четырнадцат|пятнадцат|шестнадцат|семнадцат|восемнадцат|девятнадцат|двадцат|тридцат|сороков|пятидесят|шестидесят|семидесят|восьмидесят|девяност|сот)(ь(его|ему|ей|ем|им|их|и|е)|ого|ому|ый|ой|ий|ая|ое|ья|ом|ые|ым|ых)",
        )],
        |ts| {
            let g = groups(ts)?;
            ORDINALS_FIRSTTH_MAP.get(to_lower(g.first()?).as_str()).copied().and_then(ordinal)
        },
    )
}

fn rule_ordinal() -> Rule {
    rule(
        "ordinal 21..99",
        vec![
            regex("(двадцать|тридцать|сорок|пятьдесят|шестьдесят|семьдесят|восемьдесят|девяносто)"),
            regex("(перв|втор|трет|четв[её]рт|пят|шест|седьм|восьм|девят)(ье(го|й)?|ого|ому|ый|ой|ий|ая|ое|ья|ые|ым|ых)"),
        ],
        |ts| {
            let m1 = groups(ts)?.first()?;
            let m2 = match ts.get(1)? {
                Token::RegexMatch(g) => g.0.first()?,
                _ => return None,
            };
            let dozen = *CARDINALS_MAP.get(to_lower(m1).as_str())?;
            let unit = *ORDINALS_FIRSTTH_MAP.get(to_lower(m2).as_str())?;
            ordinal(dozen + unit)
        },
    )
}

fn rule_ordinal_digits() -> Rule {
    rule(
        "ordinal (digits)",
        vec![regex("0*(\\d+)-?((ы|о|и|а|е|ь)?(ее|й|я|е|го|му?))")],
        |ts| parse_int(groups(ts)?.first()?).and_then(ordinal),
    )
}

static RULES: Lazy<Vec<Rule>> =
    Lazy::new(|| vec![rule_ordinal(), rule_ordinal_digits(), rule_ordinals_firstth()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
