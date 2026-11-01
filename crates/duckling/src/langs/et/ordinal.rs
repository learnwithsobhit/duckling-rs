//! `Duckling.Ordinal.ET.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{group, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

const ORDINALS: &[(&str, i64)] = &[
    ("esimene", 1),
    ("teine", 2),
    ("kolmas", 3),
    ("neljas", 4),
    ("viies", 5),
    ("kuues", 6),
    ("seitsmes", 7),
    ("kaheksas", 8),
    ("üheksas", 9),
    ("kümnes", 10),
    ("üheteistkümnes", 11),
    ("kaheteistkümnes", 12),
    ("kolmeteistkümnes", 13),
    ("neljateistkümnes", 14),
    ("viieteistkümnes", 15),
    ("kuueteistkümnes", 16),
    ("seitsmeteistkümnes", 17),
    ("kaheksateistkümnes", 18),
    ("üheksateistkümnes", 19),
];

fn lookup(table: &[(&str, i64)], k: &str) -> Option<i64> {
    table.iter().find(|(n, _)| *n == k).map(|(_, v)| *v)
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleOrdinalDigits
        rule("ordinal (digits)", vec![regex("0*(\\d+)\\.")], |ts| {
            parse_int(&group(ts, 0)?).and_then(ordinal)
        }),
        // ruleOrdinalsFirstth
        rule(
            "ordinals (first..19th)",
            vec![regex("(esimene|teine|kolmas|neljas|viies|kuues|seitsmes|kaheksas|üheksas|kümnes|üheteistkümnes|kaheteistkümnes|kolmeteistkümnes|neljateistkümnes|viieteistkümnes|kuueteistkümnes|seitsmeteistkümnes|kaheksateistkümnes|üheksateistkümnes)")],
            |ts| lookup(ORDINALS, &to_lower(&group(ts, 0)?)).and_then(ordinal),
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
