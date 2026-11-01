//! `Duckling.Ordinal.PT.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::to_lower;
use crate::types::{groups, pred, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

const ORDINALS: &[(&str, i64)] = &[
    ("primeir", 1), ("segund", 2), ("terceir", 3), ("quart", 4), ("quint", 5), ("sext", 6),
    ("setim", 7), ("sétim", 7), ("oitav", 8), ("non", 9), ("decim", 10), ("décim", 10),
];

const CARDINALS: &[(&str, i64)] = &[
    ("vi", 20), ("tri", 30), ("quadra", 40), ("qüinqua", 50), ("quinqua", 50), ("sexa", 60),
    ("septua", 70), ("octo", 80), ("nona", 90),
];

fn lookup(table: &[(&str, i64)], k: &str) -> Option<i64> {
    table.iter().find(|(n, _)| *n == k).map(|(_, v)| *v)
}

/// `Duckling.Ordinal.Helpers.oneOf`.
fn one_of(vs: Vec<i64>) -> crate::types::PatternItem {
    pred(move |t| matches!(t, Token::Ordinal(o) if vs.contains(&o.value)))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "ordinals (1..10)",
            vec![regex("(primeir|segund|terceir|quart|quint|sext|s[ée]tim|oitav|non|d[ée]cim)[ao]s?")],
            |ts| lookup(ORDINALS, &to_lower(groups(ts)?.first()?)).and_then(ordinal),
        ),
        rule(
            "cardinals (20 .. 90)",
            vec![regex("(vi|tri|quadra|q[üu]inqua|sexa|septua|octo|nona)g[ée]sim[ao]s?")],
            |ts| lookup(CARDINALS, &to_lower(groups(ts)?.first()?)).and_then(ordinal),
        ),
        rule(
            "ordinals (11..19)",
            vec![one_of((1..=9).map(|i| i * 10).collect()), one_of((1..=9).collect())],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Ordinal(t), Token::Ordinal(u)) => ordinal(t.value + u.value),
                _ => None,
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
