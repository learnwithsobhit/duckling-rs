//! `Duckling.Quantity.ES.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::is_positive;
use crate::dims::quantity::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: QuantityData) -> Option<Token> {
    Some(Token::Quantity(d))
}

/// `(value, unit)` of a simple quantity without a product.
fn qv(t: &Token) -> Option<(f64, Unit)> {
    match t {
        Token::Quantity(QuantityData { value: Some(v), unit: Some(u), aproduct: None, .. }) => {
            Some((*v, u.clone()))
        }
        _ => None,
    }
}

fn numeral_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

/// `getValue opsMap match` with the ES `opsMap`.
fn get_value(m: &str, v: f64) -> f64 {
    match to_lower(m).as_str() {
        "miligram" | "miligramos" | "mg" | "mgs" => v / 1000.0,
        "kilogramo" | "kilogramos" | "kg" | "kgs" => v * 1000.0,
        _ => v,
    }
}

const QUANTITIES: [(&str, &str, fn() -> Unit); 10] = [
    ("<quantity> bowls", "(bol(es)?|tazón(es)?|cuencos?|platos? (soperos?)|(hondos?))", || Unit::Bowl),
    ("<quantity> cups", "(tazas?)", || Unit::Cup),
    ("<quantity> dishes", "(platos?|fuentes?)", || Unit::Dish),
    ("<quantity> grams", "(((m(ili)?)|(k(ilo)?))?g(ramo)?s?)", || Unit::Gram),
    ("<quantity> ounces", "((onzas?)|oz)", || Unit::Ounce),
    ("<quantity> pints", "(pintas?)", || Unit::Pint),
    ("<quantity> pounds", "((lb|libra)s?)", || Unit::Pound),
    ("<quantity> quarts", "(cuartos? de galón)", || Unit::Quart),
    ("<quantity> tablespoons", "(cucharadas? (grande)?)", || Unit::Tablespoon),
    ("<quantity> teaspoons", "(cucharaditas?)", || Unit::Teaspoon),
];

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v: Vec<Rule> = vec![
        rule("<quantity> de producto", vec![dimension(Dim::Quantity), regex("de (\\w+)")], |ts| {
            let q = match ts.first()? {
                Token::Quantity(q) => q,
                _ => return None,
            };
            let p = groups(&ts[1..])?.first()?.clone();
            let mut q = q.clone();
            q.aproduct = Some(to_lower(&p));
            tok(q)
        }),
        rule(
            "over/above/exceeding/beyond/at least/more than <quantity>",
            vec![
                regex("(?<!no )m(á|a)s( grande| pesado)? (de|que)|mayor de|por encima de|excesivo|fuera de|por lo menos|como m(í|i)nim(o|a)|al menos"),
                pred(is_simple_quantity),
            ],
            |ts| {
                let (from, u) = qv(ts.get(1)?)?;
                tok(with_min(from, unit_only(u)))
            },
        ),
        rule(
            "under/below/less/lower/at most/no more than <dist>",
            vec![
                regex("no m(á|a)s que|menos de|por debajo de|como mucho|como m(á|a)xim(o|a)|a lo sumo|menos (que|de)"),
                pred(is_simple_quantity),
            ],
            |ts| {
                let (to, u) = qv(ts.get(1)?)?;
                tok(with_max(to, unit_only(u)))
            },
        ),
        rule(
            "between|from <numeral> and|to <quantity>",
            vec![regex("entre|de"), pred(is_positive), regex("a|y"), pred(is_simple_quantity)],
            |ts| {
                let from = numeral_value(ts.get(1)?)?;
                let (to, u) = qv(ts.get(3)?)?;
                if from < to { tok(with_interval(from, to, unit_only(u))) } else { None }
            },
        ),
        rule(
            "between|from <quantity> to|and <quantity>",
            vec![regex("entre/de"), pred(is_simple_quantity), regex("a/y"), pred(is_simple_quantity)],
            |ts| {
                let (from, u1) = qv(ts.get(1)?)?;
                let (to, u2) = qv(ts.get(3)?)?;
                if from < to && u1 == u2 { tok(with_interval(from, to, unit_only(u1))) } else { None }
            },
        ),
        rule(
            "<numeral> - <quantity>",
            vec![pred(is_positive), regex("\\-"), pred(is_simple_quantity)],
            |ts| {
                let from = numeral_value(ts.first()?)?;
                let (to, u) = qv(ts.get(2)?)?;
                if from < to { tok(with_interval(from, to, unit_only(u))) } else { None }
            },
        ),
        rule(
            "<quantity> - <quantity>",
            vec![pred(is_simple_quantity), regex("\\-"), pred(is_simple_quantity)],
            |ts| {
                let (from, u1) = qv(ts.first()?)?;
                let (to, u2) = qv(ts.get(2)?)?;
                if from < to && u1 == u2 { tok(with_interval(from, to, unit_only(u1))) } else { None }
            },
        ),
        rule(
            "about|exactly <quantity>",
            vec![regex("exactamente|precisamente|a?cerca( de)?|aproximadamente|casi"), dimension(Dim::Quantity)],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        rule("<quantity> (latent)", vec![pred(is_positive)], |ts| {
            let v = numeral_value(ts.first()?)?;
            let mut q = value_only(v);
            q.latent = true;
            tok(q)
        }),
    ];
    for (name, re, unit) in QUANTITIES {
        v.push(rule(name, vec![pred(is_positive), regex(re)], move |ts| {
            let n = numeral_value(ts.first()?)?;
            let m = match ts.get(1)? {
                Token::RegexMatch(g) => g.0.first()?.clone(),
                _ => return None,
            };
            tok(quantity(unit(), get_value(&m, n)))
        }));
    }
    for (name, re, unit) in QUANTITIES {
        v.push(rule(name, vec![regex(&format!("una? {}", re))], move |ts| {
            let m = match ts.first()? {
                Token::RegexMatch(g) => g.0.first()?.clone(),
                _ => return None,
            };
            tok(quantity(unit(), get_value(&m, 1.0)))
        }));
    }
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
