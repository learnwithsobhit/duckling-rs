//! `Duckling.Volume.DE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::is_positive;
use crate::dims::volume::*;
use crate::types::{dimension, pred, regex, rule, Dim, PatternItem, Rule, Token};

fn tok(d: VolumeData) -> Option<Token> {
    Some(Token::Volume(d))
}

fn vu(t: &Token) -> Option<(f64, Unit)> {
    match t {
        Token::Volume(VolumeData { value: Some(v), unit: Some(u), .. }) => Some((*v, *u)),
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
    let mut v: Vec<Rule> = vec![
        rule(
            "about <volume>",
            vec![
                regex("\\~|(ganz )?genau|präzise|(in )?etwa|ungefähr|um( die)?|fast"),
                dimension(Dim::Volume),
            ],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        rule(
            "between|from <numeral> and|to <volume>",
            vec![regex("zwischen|von"), pred(is_positive), regex("und|bis( zu)?"), pred(is_simple_volume)],
            |ts| {
                let from = numeral_value(ts.get(1)?)?;
                let (to, u) = vu(ts.get(3)?)?;
                if from < to { tok(with_interval(from, to, unit_only(u))) } else { None }
            },
        ),
        rule(
            "between|from <volume> to|and <volume>",
            vec![regex("zwischen|von"), pred(is_simple_volume), regex("bis( zu)?|und"), pred(is_simple_volume)],
            |ts| {
                let (from, u1) = vu(ts.get(1)?)?;
                let (to, u2) = vu(ts.get(3)?)?;
                if from < to && u1 == u2 { tok(with_interval(from, to, unit_only(u1))) } else { None }
            },
        ),
        rule(
            "at most <volume>",
            vec![regex("unter|weniger( als)?|höchstens|nicht mehr als"), pred(is_simple_volume)],
            |ts| {
                let (to, u) = vu(ts.get(1)?)?;
                tok(with_max(to, unit_only(u)))
            },
        ),
        rule(
            "more than <volume>",
            vec![regex("über|mindestens|wenigstens|mehr als|größer( als)?"), pred(is_simple_volume)],
            |ts| {
                let (from, u) = vu(ts.get(1)?)?;
                tok(with_min(from, unit_only(u)))
            },
        ),
    ];
    let volumes: [(&str, &str, Unit); 3] = [
        ("<latent vol> ml", "m(l|illiliter[ns]?)", Unit::Millilitre),
        ("<vol> hectoliters", "hektoliter[ns]?", Unit::Hectolitre),
        ("<vol> liters", "l(iter[ns]?)?", Unit::Litre),
    ];
    for (name, re, u) in volumes {
        v.push(rule(name, vec![regex(re)], move |_| tok(unit_only(u))));
    }
    let fractions: [(&str, &str, f64); 6] = [
        ("one", "ein(e[ns])?", 1.0),
        ("half", "(ein(e[ns])? )?halb(e[rn])? ", 1.0 / 2.0),
        ("third", "(ein(e[ns])? )?dritttel ", 1.0 / 3.0),
        ("fourth", "(ein(e[ns])? )?viertel ", 1.0 / 4.0),
        ("fifth", "(ein(e[ns])? )?fünftel ", 1.0 / 5.0),
        ("tenth", "(ein(e[ns])? )?zehntel ", 1.0 / 10.0),
    ];
    for (name, re, f) in fractions {
        let pattern: Vec<PatternItem> = vec![regex(re), pred(is_unit_only)];
        v.push(rule(name, pattern, move |ts| match ts.get(1)? {
            Token::Volume(VolumeData { unit: Some(u), .. }) => tok(volume(*u, f)),
            _ => None,
        }));
    }
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
