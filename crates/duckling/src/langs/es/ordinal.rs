//! `Duckling.Ordinal.ES.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::to_lower;
use crate::types::{groups, regex, rule, Rule, Token};

static ORDINALS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("primer", 1), ("primero", 1), ("primeros", 1), ("primera", 1), ("primeras", 1),
        ("segundo", 2), ("segunda", 2), ("segundas", 2), ("segundos", 2),
        ("terceros", 3), ("tercera", 3), ("terceras", 3), ("tercero", 3), ("tercer", 3),
        ("cuarta", 4), ("cuartas", 4), ("cuartos", 4), ("cuarto", 4),
        ("quinto", 5), ("quinta", 5), ("quintas", 5), ("quintos", 5),
        ("sextos", 6), ("sexto", 6), ("sexta", 6), ("sextas", 6),
        ("séptimas", 7), ("septimas", 7), ("séptima", 7), ("septimos", 7), ("septima", 7),
        ("séptimo", 7), ("séptimos", 7), ("septimo", 7),
        ("octavas", 8), ("octavo", 8), ("octavos", 8), ("octava", 8),
        ("novenos", 9), ("novena", 9), ("noveno", 9), ("novenas", 9),
        ("décimos", 10), ("decimo", 10), ("decimos", 10), ("décimo", 10), ("decimas", 10),
        ("décima", 10), ("decima", 10), ("décimas", 10),
    ]
    .into_iter()
    .collect()
});

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule(
        "ordinals (primero..10)",
        vec![regex("((primer|segund|tercer|cuart|quint|sext|s[eé]ptim|octav|noven|d[eé]cim)(os?|as?)|(prim|terc)er)")],
        |ts| {
            let g = groups(ts)?;
            ORDINALS
                .get(to_lower(g.first()?).as_str())
                .map(|v| Token::Ordinal(OrdinalData { value: *v }))
        },
    )]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
