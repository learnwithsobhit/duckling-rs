//! `Duckling.Distance.DE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance::*;
use crate::dims::numeral::is_positive;
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

fn tok(d: DistanceData) -> Option<Token> {
    Some(Token::Distance(d))
}

fn simple(t: &Token) -> Option<(f64, Unit)> {
    match t {
        Token::Distance(DistanceData { value: Some(v), unit: Some(u), .. }) => Some((*v, *u)),
        _ => None,
    }
}

fn numeral_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = vec![
        rule(
            "between|from <numeral> to|and <dist>",
            vec![regex("zwischen|von"), pred(is_positive), regex("bis|und"), pred(is_simple_distance)],
            |ts| {
                let from = numeral_value(ts.get(1)?)?;
                let (to, u) = simple(ts.get(3)?)?;
                if from < to { tok(with_interval(from, to, unit_only(u))) } else { None }
            },
        ),
        rule(
            "between|from <dist> to|and <dist>",
            vec![regex("zwischen|von"), pred(is_simple_distance), regex("und|bis"), pred(is_simple_distance)],
            |ts| {
                let (from, u1) = simple(ts.get(1)?)?;
                let (to, u2) = simple(ts.get(3)?)?;
                if from < to && u1 == u2 { tok(with_interval(from, to, unit_only(u1))) } else { None }
            },
        ),
        rule(
            "under/less/lower/no more than <dist>",
            vec![regex("unter|höchstens|maximal|(weniger|nicht mehr) als"), pred(is_simple_distance)],
            |ts| {
                let (to, u) = simple(ts.get(1)?)?;
                tok(with_max(to, unit_only(u)))
            },
        ),
        rule(
            "over/above/at least/more than <dist>",
            vec![
                regex("über|(mehr|nicht weniger) als|mindestens|wenigstens|minimal"),
                pred(is_simple_distance),
            ],
            |ts| {
                let (to, u) = simple(ts.get(1)?)?;
                tok(with_min(to, unit_only(u)))
            },
        ),
        rule(
            "<numeral> - <dist>",
            vec![pred(is_positive), regex("-"), pred(is_simple_distance)],
            |ts| {
                let from = numeral_value(ts.first()?)?;
                let (to, u) = simple(ts.get(2)?)?;
                if from < to { tok(with_interval(from, to, unit_only(u))) } else { None }
            },
        ),
        rule(
            "<dist> - <dist>",
            vec![pred(is_simple_distance), regex("-"), pred(is_simple_distance)],
            |ts| {
                let (from, u1) = simple(ts.first()?)?;
                let (to, u2) = simple(ts.get(2)?)?;
                if from < to && u1 == u2 { tok(with_interval(from, to, unit_only(u1))) } else { None }
            },
        ),
        rule(
            "about|exactly <dist>",
            vec![
                regex("genau|exakt|präzise|ungefähr|(in )?etwa|nahe?( an)?|um( die)?|fast|rund|gut"),
                dimension(Dim::Distance),
            ],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
    ];
    let distances: [(&str, &str, Unit); 6] = [
        ("miles", "meilen?", Unit::Mile),
        ("inch", "(\"|''|zoll)", Unit::Inch),
        ("km", "k(ilo)?m(etern?)?", Unit::Kilometre),
        ("meters", "m(etern?)?", Unit::Metre),
        ("centimeters", "(cm|[zc]entimetern?)", Unit::Centimetre),
        ("millimeters", "(mm|millimetern?)", Unit::Millimetre),
    ];
    for (name, re, u) in distances {
        v.push(rule(name, vec![dimension(Dim::Distance), regex(re)], move |ts| match ts.first()? {
            Token::Distance(dd) => tok(with_unit(u, dd.clone())),
            _ => None,
        }));
    }
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
