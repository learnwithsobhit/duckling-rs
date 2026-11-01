//! `Duckling.Ordinal.CA.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::to_lower;
use crate::types::{groups, regex, rule, Rule, Token};

static ORDINALS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("primer", 1),
        ("primers", 1),
        ("primera", 1),
        ("primeres", 1),
        ("segon", 2),
        ("segona", 2),
        ("segones", 2),
        ("segons", 2),
        ("tercer", 3),
        ("tercera", 3),
        ("tercers", 3),
        ("terceres", 3),
        ("quart", 4),
        ("quarta", 4),
        ("quarts", 4),
        ("quartes", 4),
        ("cinquè", 5),
        ("cinquena", 5),
        ("cinquens", 5),
        ("cinquenes", 5),
        ("sisè", 6),
        ("sisena", 6),
        ("sisens", 6),
        ("sisenes", 6),
        ("setè", 7),
        ("setena", 7),
        ("setens", 7),
        ("setenes", 7),
        ("vuitè", 8),
        ("vuitena", 8),
        ("vuitens", 8),
        ("vuitenes", 8),
        ("novè", 9),
        ("novena", 9),
        ("novens", 9),
        ("novenes", 9),
        ("desè", 10),
        ("desena", 10),
        ("desens", 10),
        ("desenes", 10),
        ("onzè", 11),
        ("onzena", 11),
        ("dotzè", 12),
        ("dotzena", 12),
        ("tretzè", 13),
        ("tretzena", 13),
        ("catorzè", 14),
        ("catorzena", 14),
        ("quinzè", 15),
        ("quinzena", 15),
        ("setzè", 16),
        ("setzena", 16),
        ("dissetè", 17),
        ("dissetena", 17),
        ("divuitè", 18),
        ("divuitena", 18),
        ("dinovè", 19),
        ("dinovena", 19),
        ("vintè", 20),
        ("vintena", 20),
    ]
    .into_iter()
    .collect()
});

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule(
        "ordinals (primero..10)",
        vec![regex(
            "(primer(a|s|es)?|segon(a|s|es)?|tercer(a|s|es)?|quart(a|s|es)?|cinqu(è|(en(a|s|es)))|sis(è|en(a|s|es))|set(è|en(a|s|es))|vuit(è|en(a|s|es))|nov(è|en(a|s|es))|des(è|en(a|s|es))|onz(è|ena)|dotz(è|ena)|tretz(è|ena)|catorz(è|ena)|quinz(è|ena)|setz(è|ena)|disset(è|ena)|divuit(è|ena)|dinov(è|ena)|vint(è|ena))",
        )],
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
