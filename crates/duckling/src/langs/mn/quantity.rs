//! `Duckling.Quantity.MN.Rules`.

use once_cell::sync::Lazy;

use crate::dims::quantity::{quantity, Unit};
use crate::types::{regex, rule, Rule, Token};

fn tok(d: crate::dims::quantity::QuantityData) -> Option<Token> {
    Some(Token::Quantity(d))
}

fn numeral_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

type Qty = (&'static str, &'static str, Unit, fn(f64) -> f64);

const QUANTITIES: [Qty; 5] = [
    ("<quantity> milligrams", "(мг|миллиграмм)", Unit::Gram, |v| v / 1000.0),
    ("<quantity> grams", "г(рамм?)?", Unit::Gram, |v| v),
    ("<quantity> kilograms", "(кг|килограмм?)", Unit::Gram, |v| v * 1000.0),
    ("<quantity> lb", "фунт", Unit::Pound, |v| v),
    ("<quantity> oz", "унц", Unit::Ounce, |v| v),
];

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v: Vec<Rule> = QUANTITIES
        .into_iter()
        .map(|(name, re, u, convert)| {
            rule(
                name,
                vec![crate::dims::numeral::number_with(|n| n.value, |v| v > 0.0), regex(re)],
                move |ts| {
                    let n = numeral_value(ts.first()?)?;
                    tok(quantity(u.clone(), convert(n)))
                },
            )
        })
        .collect();
    for (name, re, u, convert) in QUANTITIES {
        v.push(rule(name, vec![regex(re)], move |_| tok(quantity(u.clone(), convert(1.0)))));
    }
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
