//! `Duckling.Ordinal.Types` (rules are per language).

use crate::json::Json;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrdinalData {
    pub value: i64,
}

impl OrdinalData {
    pub fn resolve(&self) -> Json {
        Json::obj([("type", Json::str("value")), ("value", Json::Int(self.value))])
    }
}

pub fn is_between(x: i64, low: i64, high: i64) -> bool {
    low <= x && x <= high
}

// ----- EN rules (`Duckling.Ordinal.EN.Rules`) ---------------------------------------------

use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};
use once_cell::sync::Lazy;

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

const ORDINALS: &[(&str, i64)] = &[
    ("first", 1), ("second", 2), ("third", 3), ("fourth", 4), ("fifth", 5), ("sixth", 6),
    ("seventh", 7), ("eighth", 8), ("ninth", 9), ("tenth", 10), ("eleventh", 11),
    ("twelfth", 12), ("thirteenth", 13), ("fourteenth", 14), ("fifteenth", 15),
    ("sixteenth", 16), ("seventeenth", 17), ("eighteenth", 18), ("nineteenth", 19),
    ("twentieth", 20), ("thirtieth", 30), ("fortieth", 40), ("fiftieth", 50),
    ("sixtieth", 60), ("seventieth", 70), ("eightieth", 80), ("ninetieth", 90),
];

const CARDINALS: &[(&str, i64)] = &[
    ("twenty", 20), ("thirty", 30), ("forty", 40), ("fifty", 50), ("sixty", 60),
    ("seventy", 70), ("eighty", 80), ("ninety", 90),
];

fn lookup(table: &[(&str, i64)], k: &str) -> Option<i64> {
    table.iter().find(|(n, _)| *n == k).map(|(_, v)| *v)
}

static EN_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "ordinals (first..twentieth,thirtieth,...)",
            vec![regex("(first|second|third|fourth|fifth|sixth|seventh|eighth|ninth|tenth|eleventh|twelfth|thirteenth|fourteenth|fifteenth|sixteenth|seventeenth|eighteenth|nineteenth|twentieth|thirtieth|fortieth|fiftieth|sixtieth|seventieth|eightieth|ninetieth)")],
            |ts| lookup(ORDINALS, &to_lower(groups(ts)?.first()?)).and_then(ordinal),
        ),
        rule(
            "ordinals (composite, e.g. eighty-seven, forty\u{2014}seventh, twenty ninth, thirtythird)",
            vec![regex("(twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety)[\\s\\-\\\u{2014}]?(first|second|third|fourth|fifth|sixth|seventh|eighth|ninth)")],
            |ts| {
                let g = groups(ts)?;
                let tt = lookup(CARDINALS, &to_lower(g.first()?))?;
                let uu = lookup(ORDINALS, &to_lower(g.get(1)?))?;
                ordinal(tt + uu)
            },
        ),
        rule("ordinal (digits)", vec![regex("0*(\\d+) ?(st|nd|rd|th)")], |ts| {
            parse_int(groups(ts)?.first()?).and_then(ordinal)
        }),
    ]
});

pub fn en_rules() -> &'static [Rule] {
    &EN_RULES
}
