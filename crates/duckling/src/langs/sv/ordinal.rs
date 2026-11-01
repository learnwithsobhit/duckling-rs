//! `Duckling.Ordinal.SV.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{group, groups, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

const ORDINALS: &[(&str, i64)] = &[
    ("första", 1),
    ("förste", 1),
    ("andra", 2),
    ("andre", 2),
    ("tredje", 3),
    ("fjärde", 4),
    ("femte", 5),
    ("sjätte", 6),
    ("sjunde", 7),
    ("åttonde", 8),
    ("nionde", 9),
    ("tionde", 10),
    ("elfte", 11),
    ("tolfte", 12),
    ("trettonde", 13),
    ("fjortonde", 14),
    ("femtonde", 15),
    ("sextonde", 16),
    ("sjuttonde", 17),
    ("artonde", 18),
    ("nittonde", 19),
    ("tjugonde", 20),
    ("trettionde", 30),
    ("fyrtionde", 40),
    ("femtionde", 50),
    ("sextionde", 60),
    ("sjuttionde", 70),
    ("åttionde", 80),
    ("nittionde", 90),
];

const CARDINALS: &[(&str, i64)] = &[
    ("tjugo", 20),
    ("trettio", 30),
    ("fyrtio", 40),
    ("femtio", 50),
    ("sextio", 60),
    ("sjuttio", 70),
    ("åttio", 80),
    ("nittio", 90),
];

fn lookup(table: &[(&str, i64)], k: &str) -> Option<i64> {
    table.iter().find(|(n, _)| *n == k).map(|(_, v)| *v)
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleOrdinals — regex omits trettonde/femtionde (upstream)
        rule(
            "ordinals (first..twentieth,thirtieth,...)",
            vec![regex("(första|förste|andra|andre|tredje|fjärde|femte|sjätte|sjunde|åttonde|nionde|tionde|elfte|tolfte|trettionde|fjortonde|femtonde|sextonde|sjuttonde|artonde|nittonde|tjugonde|trettionde|fyrtionde|femtonde|sextionde|sjuttionde|åttionde|nittionde)")],
            |ts| lookup(ORDINALS, &to_lower(&group(ts, 0)?)).and_then(ordinal),
        ),
        // ruleCompositeOrdinals
        rule(
            "ordinals (composite, e.g., eighty-seven)",
            vec![regex("(tjugo|trettio|fyrtio|femtio|sextio|sjuttio|åttio|nittio)(första|förste|andra|andre|tredje|fjärde|femte|sjätte|sjunde|åttonde|nionde)")],
            |ts| {
                let g = groups(ts)?;
                let tt = lookup(CARDINALS, &to_lower(g.first()?))?;
                let uu = lookup(ORDINALS, &to_lower(g.get(1)?))?;
                ordinal(tt + uu)
            },
        ),
        // ruleOrdinalDigits
        rule("ordinal (digits)", vec![regex("0*(\\d+):?(a|e)")], |ts| {
            parse_int(&group(ts, 0)?).and_then(ordinal)
        }),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
