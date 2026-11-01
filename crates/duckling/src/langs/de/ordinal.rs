//! `Duckling.Ordinal.DE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};

const ORDINAL_LIST: &[(&str, i64)] = &[
    ("erste", 1),
    ("zweite", 2),
    ("dritte", 3),
    ("vierte", 4),
    ("fünfte", 5),
    ("sechste", 6),
    ("siebte", 7),
    ("achte", 8),
    ("neunte", 9),
    ("zehnte", 10),
    ("elfte", 11),
    ("zwölfte", 12),
    ("dreizente", 13),
    ("vierzehnte", 14),
    ("fünfzehnte", 15),
    ("sechzente", 16),
    ("siebzehnte", 17),
    ("achtzehnte", 18),
    ("neunzehnte", 19),
    ("zwanzigste", 20),
    ("einundzwanzigste", 21),
    ("zweiundzwanzigste", 22),
    ("dreiundzwanzigste", 23),
    ("vierundzwanzigste", 24),
    ("fünfundzwanzigste", 25),
    ("sechsundzwanzigste", 26),
    ("siebenundzwanzigste", 27),
    ("achtundzwanzigste", 28),
    ("neunundzwanzigste", 29),
    ("dreissigste", 30),
    ("dreißigste", 30),
    ("einunddreissigste", 31),
    ("einunddreißigste", 31),
];

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let names: Vec<&str> = ORDINAL_LIST.iter().map(|(n, _)| *n).collect();
    let construction = format!("({})[rsn]?", names.join("|"));
    vec![
        rule("ordinal (1..31)", vec![regex(&construction)], |ts| {
            let m = to_lower(groups(ts)?.first()?);
            ORDINAL_LIST.iter().find(|(n, _)| *n == m).and_then(|(_, v)| ordinal(*v))
        }),
        rule(
            "ordinal (digits)",
            vec![regex(r"(?<!\d|\.)0*(\d+)(\.(?!\d)| ?(te(n|r|s)?)|(ste(n|r|s)?))")],
            |ts| {
                let v = parse_int(groups(ts)?.first()?)?;
                ordinal(v)
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
