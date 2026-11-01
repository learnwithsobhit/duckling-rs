//! `Duckling.Ordinal.BG.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};

fn ordinals_map() -> HashMap<&'static str, i64> {
    HashMap::from([
        ("първ", 1),
        ("втор", 2),
        ("трет", 3),
        ("четвърт", 4),
        ("пет", 5),
        ("шест", 6),
        ("седм", 7),
        ("осм", 8),
        ("девет", 9),
        ("десет", 10),
        ("единадесет", 11),
        ("дванадесет", 12),
        ("тринадесет", 13),
        ("четиринадесет", 14),
        ("петнадесет", 15),
        ("шестнадесет", 16),
        ("седемнадесет", 17),
        ("осемнадесет", 18),
        ("деветнадесет", 19),
        ("двадесет", 20),
    ])
}

fn dozens_map() -> HashMap<&'static str, i64> {
    HashMap::from([
        ("двадесет", 20),
        ("тридесет", 30),
        ("четирдесет", 40),
        ("петдесет", 50),
        ("шестдесет", 60),
        ("седемдесет", 70),
        ("осемдесет", 80),
        ("деветдесет", 90),
    ])
}

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

fn rx_group(t: &Token, i: usize) -> Option<String> {
    match t {
        Token::RegexMatch(g) => g.0.get(i).map(|s| s.to_string()),
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "ordinal 21..99",
            vec![
                regex("(двадесет|тридесет|четирдесет|петдесет|шестдесет|седемдесет|осемдесет|деветдесет)"),
                regex("и (първ|втор|трет|четвърт|пет|шест|седм|осм|девет)(и(ят?|те)?|а(та)?|о(то)?)"),
            ],
            |ts| {
                let dozen = *dozens_map().get(to_lower(&rx_group(ts.first()?, 0)?).as_str())?;
                let unit = *ordinals_map().get(to_lower(&rx_group(ts.get(1)?, 0)?).as_str())?;
                ordinal(dozen + unit)
            },
        ),
        rule(
            "ordinal (digits)",
            vec![regex("0*(\\d+)-?((в|р|м|т)(и(я(т)?|те)?|а(та)?|о(то)?))")],
            |ts| {
                let v = parse_int(groups(ts)?.first()?)?;
                ordinal(v)
            },
        ),
        rule(
            "ordinals (first..19th)",
            vec![regex("(първ|втор|трет|четвърт|пет|шест|седм|осм|девет|десет|единадесет|дванадесет|тринадесет|четиринадесет|петнадесет|шестнадесет|седемнадесет|осемнадесет|деветнадесет|двадесет)(и(я(т)?|те)?|а(та)?|о(то)?)")],
            |ts| {
                let match_ = to_lower(groups(ts)?.first()?);
                ordinals_map().get(match_.as_str()).copied().and_then(ordinal)
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
