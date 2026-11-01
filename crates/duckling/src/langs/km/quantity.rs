//! `Duckling.Quantity.KM.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::is_positive;
use crate::dims::quantity::{
    is_simple_quantity, quantity, unit_only, with_interval, with_max, with_min, QuantityData, Unit,
};
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: QuantityData) -> Option<Token> {
    Some(Token::Quantity(d))
}

fn with_product(p: String, mut qd: QuantityData) -> QuantityData {
    qd.aproduct = Some(p);
    qd
}

static UNITS_MAP: Lazy<HashMap<&'static str, Unit>> = Lazy::new(|| {
    [
        ("ចាន", Unit::Bowl),
        ("ពែង", Unit::Cup),
        ("កែវ", Unit::Cup),
        ("ថូ", Unit::Pint),
        ("ស្លាបព្រា", Unit::Tablespoon),
        ("ស្លាបព្រាបាយ", Unit::Tablespoon),
        ("ស្លាបព្រាកាហ្វេ", Unit::Teaspoon),
        ("នាក់", Unit::Custom("For Persons".to_string())),
        ("ក្បាល", Unit::Custom("For Animals".to_string())),
        ("ដើម", Unit::Custom("For Trees".to_string())),
        ("ទង", Unit::Custom("For Flowers".to_string())),
        ("ខ្នង", Unit::Custom("For Buildings".to_string())),
        ("គ្រឿង", Unit::Custom("For Vehicles/Devices".to_string())),
        ("កញ្ចប់", Unit::Custom("For Packages".to_string())),
        ("ឈុត", Unit::Custom("Sets".to_string())),
    ]
    .into_iter()
    .collect()
});

static OPS_MAP: Lazy<HashMap<&'static str, fn(f64) -> f64>> = Lazy::new(|| {
    let div1000: fn(f64) -> f64 = |v| v / 1000.0;
    let times1000: fn(f64) -> f64 = |v| v * 1000.0;
    [("មីលីក្រាម", div1000), ("គីឡូក្រាម", times1000)].into_iter().collect()
});

fn get_value(m: &str, v: f64) -> f64 {
    match OPS_MAP.get(to_lower(m).as_str()) {
        Some(op) => op(v),
        None => v,
    }
}

fn rule_quantity_of_product() -> Rule {
    rule(
        "<quantity> of product",
        vec![regex("(មនុស្ស|បងប្អូន|សត្វ|ឆ្កែ|ឆ្មា|ដើមឈើ|ផ្កា|កុលាប|ផ្ទះ)"), dimension(Dim::Quantity)],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::RegexMatch(_), Token::Quantity(qd)) => {
                let m = groups(ts)?.first()?.to_string();
                tok(with_product(m, qd.clone()))
            }
            _ => None,
        },
    )
}

fn rule_numeral_units() -> Rule {
    rule(
        "<number><units>",
        vec![
            dimension(Dim::Numeral),
            regex("(ចាន|ពែង|កែវ|ថូ|ស្លាបព្រា|ស្លាបព្រាបាយ|ស្លាបព្រាកាហ្វេ|នាក់|ក្បាល|ដើម|ទង|ខ្នង|គ្រឿង|កញ្ចប់|ឈុត)"),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(n), Token::RegexMatch(_)) => {
                let m = groups(&ts[1..])?.first()?;
                let unit = UNITS_MAP.get(m.as_ref())?.clone();
                tok(quantity(unit, n.value))
            }
            _ => None,
        },
    )
}

fn rule_numeral_units2() -> Rule {
    rule(
        "<quantity> grams",
        vec![dimension(Dim::Numeral), regex("((មីលី|គីឡូ)?ក្រាម)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(nd), Token::RegexMatch(_)) => {
                let m = groups(&ts[1..])?.first()?.to_string();
                tok(quantity(Unit::Gram, get_value(&m, nd.value)))
            }
            _ => None,
        },
    )
}

fn rule_precision() -> Rule {
    rule("about|exactly <quantity>", vec![regex("\\~|ប្រហែល"), dimension(Dim::Quantity)], |ts| {
        ts.get(1).map(|t| (*t).clone())
    })
}

fn qv_no_product(t: &Token) -> Option<(f64, Unit)> {
    match t {
        Token::Quantity(QuantityData {
            value: Some(v),
            unit: Some(u),
            aproduct: None,
            ..
        }) => Some((*v, u.clone())),
        _ => None,
    }
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> and|to <quantity>",
        vec![
            regex("ចន្លោះ(ពី)?|ចាប់ពី"),
            pred(is_positive),
            regex("និង|ដល់"),
            pred(is_simple_quantity),
        ],
        |ts| {
            let from = match ts.get(1)? {
                Token::Numeral(n) => n.value,
                _ => return None,
            };
            let (to, u) = qv_no_product(ts.get(3)?)?;
            if from < to {
                tok(with_interval(from, to, unit_only(u)))
            } else {
                None
            }
        },
    )
}

fn rule_interval_between() -> Rule {
    rule(
        "between|from <quantity> to|and <quantity>",
        vec![
            regex("ចន្លោះ(ពី)?|ចាប់ពី"),
            pred(is_simple_quantity),
            regex("និង|ដល់"),
            pred(is_simple_quantity),
        ],
        |ts| {
            let (from, u1) = qv_no_product(ts.get(1)?)?;
            let (to, u2) = qv_no_product(ts.get(3)?)?;
            if from < to && u1 == u2 {
                tok(with_interval(from, to, unit_only(u1)))
            } else {
                None
            }
        },
    )
}

fn rule_interval_numeral_dash() -> Rule {
    rule(
        "<numeral> - <quantity>",
        vec![pred(is_positive), regex("\\-"), pred(is_simple_quantity)],
        |ts| {
            let from = match ts.first()? {
                Token::Numeral(n) => n.value,
                _ => return None,
            };
            let (to, u) = qv_no_product(ts.get(2)?)?;
            if from < to {
                tok(with_interval(from, to, unit_only(u)))
            } else {
                None
            }
        },
    )
}

fn rule_interval_dash() -> Rule {
    rule(
        "<quantity> - <quantity>",
        vec![pred(is_simple_quantity), regex("\\-"), pred(is_simple_quantity)],
        |ts| {
            let (from, u1) = qv_no_product(ts.first()?)?;
            let (to, u2) = qv_no_product(ts.get(2)?)?;
            if from < to && u1 == u2 {
                tok(with_interval(from, to, unit_only(u1)))
            } else {
                None
            }
        },
    )
}

fn rule_interval_max() -> Rule {
    rule(
        "Max Rule",
        vec![regex("ក្រោម|តិចជាង|មិនដល់|យ៉ាងច្រើន|មិនលើស"), pred(is_simple_quantity)],
        |ts| {
            let (to, u) = qv_no_product(ts.get(1)?)?;
            tok(with_max(to, unit_only(u)))
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "Min Rule",
        vec![regex("លើស(ពី)?|មិនតិចជាង|លើ|ច្រើនជាង|យ៉ាងតិច|យ៉ាងហោច"), pred(is_simple_quantity)],
        |ts| {
            let (from, u) = qv_no_product(ts.get(1)?)?;
            tok(with_min(from, unit_only(u)))
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_numeral_units(),
        rule_quantity_of_product(),
        rule_precision(),
        rule_interval_between_numeral(),
        rule_interval_between(),
        rule_interval_numeral_dash(),
        rule_interval_dash(),
        rule_interval_max(),
        rule_interval_min(),
        rule_numeral_units2(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
