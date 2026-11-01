//! `Duckling.Distance.KM.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance::{
    is_simple_distance, unit_only, with_interval, with_max, with_min, with_unit, DistanceData, Unit,
};
use crate::dims::numeral::is_positive;
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

fn tok(d: DistanceData) -> Option<Token> {
    Some(Token::Distance(d))
}

fn unit_rule(name: &str, re: &str, unit: Unit) -> Rule {
    rule(name, vec![dimension(Dim::Distance), regex(re)], move |ts| match ts.first()? {
        Token::Distance(dd) => tok(with_unit(unit, dd.clone())),
        _ => None,
    })
}

fn rule_precision() -> Rule {
    rule("about|exactly <dist>", vec![regex("\\~|ប្រហែល"), dimension(Dim::Distance)], |ts| {
        ts.get(1).map(|t| (*t).clone())
    })
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> to|and <dist>",
        vec![regex("ចន្លោះ(ពី)?|ចាប់ពី"), pred(is_positive), regex("និង|ដល់"), pred(is_simple_distance)],
        |ts| match (ts.get(1)?, ts.get(3)?) {
            (
                Token::Numeral(n),
                Token::Distance(DistanceData { value: Some(to), unit: Some(u), .. }),
            ) if n.value < *to => tok(with_interval(n.value, *to, unit_only(*u))),
            _ => None,
        },
    )
}

fn rule_interval_between() -> Rule {
    rule(
        "between|from <dist> to|and <dist>",
        vec![
            regex("ចន្លោះ(ពី)?|ចាប់ពី"),
            pred(is_simple_distance),
            regex("និង|ដល់"),
            pred(is_simple_distance),
        ],
        |ts| match (ts.get(1)?, ts.get(3)?) {
            (
                Token::Distance(DistanceData { value: Some(from), unit: Some(u1), .. }),
                Token::Distance(DistanceData { value: Some(to), unit: Some(u2), .. }),
            ) if *from < *to && u1 == u2 => tok(with_interval(*from, *to, unit_only(*u1))),
            _ => None,
        },
    )
}

fn rule_interval_numeral_dash() -> Rule {
    rule(
        "<numeral> - <dist>",
        vec![pred(is_positive), regex("-"), pred(is_simple_distance)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (
                Token::Numeral(n),
                Token::Distance(DistanceData { value: Some(to), unit: Some(u), .. }),
            ) if n.value < *to => tok(with_interval(n.value, *to, unit_only(*u))),
            _ => None,
        },
    )
}

fn rule_interval_dash() -> Rule {
    rule(
        "<dist> - <dist>",
        vec![pred(is_simple_distance), regex("-"), pred(is_simple_distance)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (
                Token::Distance(DistanceData { value: Some(from), unit: Some(u1), .. }),
                Token::Distance(DistanceData { value: Some(to), unit: Some(u2), .. }),
            ) if *from < *to && u1 == u2 => tok(with_interval(*from, *to, unit_only(*u1))),
            _ => None,
        },
    )
}

fn rule_interval_max() -> Rule {
    rule(
        "Max Rule",
        vec![regex("ក្រោម|តិចជាង|មិនដល់|យ៉ាងច្រើន|មិនលើស"), pred(is_simple_distance)],
        |ts| match ts.get(1)? {
            Token::Distance(DistanceData { value: Some(to), unit: Some(u), .. }) => {
                tok(with_max(*to, unit_only(*u)))
            }
            _ => None,
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "Min Rule",
        vec![regex("លើស(ពី)?|មិនតិចជាង|លើ|ច្រើនជាង|យ៉ាងតិច|យ៉ាងហោច"), pred(is_simple_distance)],
        |ts| match ts.get(1)? {
            Token::Distance(DistanceData { value: Some(to), unit: Some(u), .. }) => {
                tok(with_min(*to, unit_only(*u)))
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = vec![
        rule_interval_between_numeral(),
        rule_interval_between(),
        rule_interval_max(),
        rule_interval_min(),
        rule_interval_numeral_dash(),
        rule_interval_dash(),
        rule_precision(),
    ];
    v.extend([
        unit_rule("km", "km|គីឡូ(ម៉ែត្រ)?", Unit::Kilometre),
        unit_rule("meters", "m|ម៉ែត្រ", Unit::Metre),
        unit_rule("centimeters", "cm|សង់ទីម៉ែត្រ", Unit::Centimetre),
        unit_rule("millimeters", "mm|មីលីម៉ែត្រ", Unit::Millimetre),
    ]);
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
