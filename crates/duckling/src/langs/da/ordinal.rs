//! `Duckling.Ordinal.DA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::{get_int_value, number_with};
use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{dimension, group, groups, regex, rule, Dim, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

const ORDINALS_MAP: &[(&str, i64)] = &[
    ("første", 1),
    ("anden", 2),
    ("tredje", 3),
    ("fjerde", 4),
    ("femte", 5),
    ("sjette", 6),
    ("syvende", 7),
    ("ottende", 8),
    ("niende", 9),
    ("tiende", 10),
    ("elfte", 11),
    ("tolvte", 12),
    ("trettende", 13),
    ("fjortende", 14),
    ("femtende", 15),
    ("sekstende", 16),
    ("syttende", 17),
    ("attende", 18),
    ("nittende", 19),
    ("tyvende", 20),
    ("tenogtyvende", 21),
    ("toogtyvende", 22),
    ("treogtyvende", 23),
    ("fireogtyvende", 24),
    ("femogtyvende", 25),
    ("seksogtyvende", 26),
    ("syvogtyvende", 27),
    ("otteogtyvende", 28),
    ("niogtyvende", 29),
    ("tredivte", 30),
    ("enogtredivte", 31),
];

const CARDINALS_MAP: &[(&str, i64)] = &[
    ("tyvende", 20),
    ("tredivte", 30),
    ("fyrrende", 40),
    ("fyrretyvende", 40),
    ("halvtredsende", 50),
    ("halvtredsindstyvende", 50),
    ("tressende", 60),
    ("tresindstyvende", 60),
    ("halvfjerdsende", 70),
    ("halvfjerdsindstyvende", 70),
    ("firsende", 80),
    ("firsindsstyvende", 80),
    ("halvfemsende", 90),
    ("halvfemsindstyvende", 90),
];

const ONE_VAL_MAP: &[(&str, i64)] = &[
    ("", 0),
    ("enog", 1),
    ("toog", 2),
    ("treog", 3),
    ("fireog", 4),
    ("femog", 5),
    ("seksog", 6),
    ("syvog", 7),
    ("otteog", 8),
    ("niog", 9),
];

fn lookup(table: &[(&str, i64)], k: &str) -> Option<i64> {
    table.iter().find(|(n, _)| *n == k).map(|(_, v)| *v)
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleOrdinalDigits
        rule("ordinal (digits)", vec![regex("0*(\\d+)(\\.|ste?)")], |ts| {
            parse_int(&group(ts, 0)?).and_then(ordinal)
        }),
        // ruleOrdinalsFirstst
        rule(
            "ordinals (first..19st)",
            vec![regex(
                "(første|anden|tredje|fjerde|femte|sjette|syvende|ottende|niende|tiende|elfte|tolvte|trettende|fjortende|femtende|sekstende|syttende|attende|nittende)",
            )],
            |ts| lookup(ORDINALS_MAP, &to_lower(&group(ts, 0)?)).and_then(ordinal),
        ),
        // ruleSpelledOutOrdinals
        rule(
            "ordinals, 20 to 99, spelled-out",
            vec![regex(&format!(
                "{}{}",
                "((?:en|to|tre|fire|fem|seks|syv|otte|ni)og)?",
                "(tyvende|tredivte|fyrr(?:etyv)?ende|halvtreds(?:indstyv)?ende|tres(?:indstyv|s)?ende|halvfjerds(?:indstyv)?ende|firs(?:indstyv)?ende|halvfems(?:indstyv)?ende)"
            ))],
            |ts| {
                let g = groups(ts)?;
                let one_val = lookup(ONE_VAL_MAP, &to_lower(g.first()?))?;
                let ten_val = lookup(CARDINALS_MAP, &to_lower(g.get(1)?))?;
                ordinal(one_val + ten_val)
            },
        ),
        // ruleSpelledOutBigOrdinals
        rule(
            "ordinals, above 99, spelled out",
            vec![number_with(|n| n.value, |v| v > 99.0), regex("og"), dimension(Dim::Ordinal)],
            |ts| match (ts.first()?, ts.get(2)?) {
                (Token::Numeral(n), Token::Ordinal(o)) => {
                    let numnum = get_int_value(n.value)?;
                    ordinal(numnum + o.value)
                }
                _ => None,
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
