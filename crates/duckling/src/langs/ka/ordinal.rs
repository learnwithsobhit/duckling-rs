//! `Duckling.Ordinal.KA.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};

static ORDINALS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("პირველი", 1),
        ("პირველ", 1),
        ("მეერთე", 1),
        ("მეორე", 2),
        ("მესამე", 3),
        ("მეოთხე", 4),
        ("მეხუთე", 5),
        ("მეექვსე", 6),
        ("მეშვიდე", 7),
        ("მერვე", 8),
        ("მეცხრე", 9),
        ("მეათე", 10),
        ("მეთერთმეტე", 11),
        ("მეთორმეტე", 12),
        ("მეცამეტე", 13),
        ("მეთოთხმეტე", 14),
        ("მეთხუთმეტე", 15),
        ("მეთქვსმეტე", 16),
        ("მეჩვიდმეტე", 17),
        ("მეთვრამეტე", 18),
        ("მეცხრამეტე", 19),
        ("მეოცე", 20),
        ("ოცდამეათე", 30),
        ("მეორმოცე", 40),
        ("ორმოცდამეათე", 50),
        ("მესამოცე", 60),
        ("სამოცდამეათე", 70),
        ("მეოთხმოცე", 80),
        ("ოთხმოცდამეათე", 90),
    ]
    .into_iter()
    .collect()
});

static CARDINALS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ოცი", 20),
        ("ოცდა", 20),
        ("ოცდაათი", 30),
        ("ორმოცი", 40),
        ("ორმოცდა", 40),
        ("ორმოცდაათი", 50),
        ("სამოცი", 60),
        ("სამოცდა", 60),
        ("სამოცდაათი", 70),
        ("ოთხმოცი", 80),
        ("ოთხმოცდა", 80),
        ("ოთხმოცდაათი", 90),
    ]
    .into_iter()
    .collect()
});

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

fn rule_ordinals() -> Rule {
    rule(
        "ordinals (first..twentieth,thirtieth,...)",
        vec![regex(
            "(პირველი?|მეორე|მესამე|მეოთხე|მეხუთე|მეექვსე|მეშვიდე|მერვე|მეცხრე|მეათე|მეთერთმეტე|მეთოთხმეტე|მეცამეტე|მეთოთხმეტე|მეთხუთმეტე|მეთექვსმეტე|მეჩვიდმეტე|მეთვრამეტე|მეცხრამეტე|მეოცე|ოცდამეათე|მეორმოცე|ორმოცდამეათე|მესამოცე|სამოცდამეათე|მეოთხმოცე|ოთხმოცდამეათე)",
        )],
        |ts| ORDINALS_MAP.get(to_lower(groups(ts)?.first()?).as_str()).copied().and_then(ordinal),
    )
}

fn rule_composite_ordinals() -> Rule {
    rule(
        "ordinals (composite, e.g. eighty-seven, forty—seventh, twenty ninth, thirtythird)",
        vec![regex(
            "(ოცდა|ორმოცდა|სამოცდა|ოთხმოცდა)[\\s\\-\\—]?(მეერთე|მეორე|მესამე|მეოთხე|მეხუთე|მეექვსე|მეშვიდე|მერვე|მეცხრე|მეათე|მეთერთმეტე|მეთოთხმეტე|მეცამეტე|მეთოთხმეტე|მეთხუთმეტე|მეთექვსმეტე|მეჩვიდმეტე|მეთვრამეტე|მეცხრამეტე)",
        )],
        |ts| {
            let g = groups(ts)?;
            let tt = *CARDINALS_MAP.get(to_lower(g.first()?).as_str())?;
            let uu = *ORDINALS_MAP.get(to_lower(g.get(1)?).as_str())?;
            ordinal(tt + uu)
        },
    )
}

fn rule_ordinal_digits() -> Rule {
    rule("ordinal (digits)", vec![regex("0*(\\d+) ?(-ლი|-ე)")], |ts| {
        parse_int(groups(ts)?.first()?).and_then(ordinal)
    })
}

fn rule_ordinal_digits1() -> Rule {
    rule("ordinal (digits)", vec![regex("მე-? ?0*(\\d+)")], |ts| {
        parse_int(groups(ts)?.first()?).and_then(ordinal)
    })
}

fn rule_ordinal_digits2() -> Rule {
    rule("ordinal (digits)", vec![regex("მე-? ?0*(\\d+) ?(-ლი|-ე)")], |ts| {
        parse_int(groups(ts)?.first()?).and_then(ordinal)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_ordinals(),
        rule_composite_ordinals(),
        rule_ordinal_digits(),
        rule_ordinal_digits1(),
        rule_ordinal_digits2(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
