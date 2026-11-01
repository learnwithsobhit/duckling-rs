//! `Duckling.Ordinal.NB.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{group, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

fn ordinal_from_word(m: &str) -> Option<Token> {
    match m {
        "første" => ordinal(1),
        "andre" => ordinal(2),
        "tredje" => ordinal(3),
        "fjerde" => ordinal(4),
        "femte" => ordinal(5),
        "sjette" => ordinal(6),
        "syvende" => ordinal(7),
        "åttende" => ordinal(8),
        "niende" => ordinal(9),
        "tiende" => ordinal(10),
        "ellevte" => ordinal(11),
        "tolvte" => ordinal(12),
        "trettende" => ordinal(13),
        "fjortende" => ordinal(14),
        "femtende" => ordinal(15),
        "sekstende" => ordinal(16),
        "syttende" => ordinal(17),
        "attende" => ordinal(18),
        "nittende" => ordinal(19),
        "tyvende" => ordinal(20),
        "tjuende" => ordinal(20),
        "enogtjuende" => ordinal(21),
        "enogtyvende" => ordinal(21),
        "toogtyvende" => ordinal(22),
        "toogtjuende" => ordinal(22),
        "treogtyvende" => ordinal(23),
        "treogtjuende" => ordinal(23),
        "fireogtjuende" => ordinal(24),
        "fireogtyvende" => ordinal(24),
        "femogtyvende" => ordinal(25),
        "femogtjuende" => ordinal(25),
        "seksogtjuende" => ordinal(26),
        "seksogtyvende" => ordinal(26),
        "syvogtyvende" => ordinal(27),
        "syvogtjuende" => ordinal(27),
        "åtteogtyvende" => ordinal(28),
        "åtteogtjuende" => ordinal(28),
        "niogtyvende" => ordinal(29),
        "niogtjuende" => ordinal(29),
        "tredefte" => ordinal(30),
        "enogtredefte" => ordinal(31),
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleOrdinalDigits
        rule("ordinal (digits)", vec![regex("0*(\\d+)(\\.|ste?)")], |ts| {
            parse_int(&group(ts, 0)?).and_then(ordinal)
        }),
        // ruleOrdinalsFirstst
        rule(
            "ordinals (first..31st)",
            vec![regex("(første|andre|tredje|fjerde|femtende|femte|sjette|syvende|åttende|niende|tiende|ellevte|tolvte|trettende|fjortende|sekstende|syttende|attende|nittende|tyvende|tjuende|enogtyvende|toogtyvende|treogtyvende|fireogtyvende|femogtyvende|seksogtyvende|syvogtyvende|åtteogtyvende|niogtyvende|enogtjuende|toogtjuende|treogtjuende|fireogtjuende|femogtjuende|seksogtjuende|syvogtjuende|åtteogtyvend|niogtjuende|tredefte|enogtredefte)")],
            |ts| ordinal_from_word(&to_lower(&group(ts, 0)?)),
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
