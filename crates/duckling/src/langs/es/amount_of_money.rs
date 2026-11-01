//! `Duckling.AmountOfMoney.ES.Rules`.

use once_cell::sync::Lazy;

use crate::dims::amount_of_money::*;
use crate::dims::numeral::{is_natural, is_positive};
use crate::types::{pred, regex, rule, Rule, Token};

fn tok(d: AmountOfMoneyData) -> Option<Token> {
    Some(Token::AmountOfMoney(d))
}

fn numeral_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

fn aom(t: &Token) -> Option<&AmountOfMoneyData> {
    match t {
        Token::AmountOfMoney(d) => Some(d),
        _ => None,
    }
}

/// `(value, currency)` of an amount with a value.
fn vc(t: &Token) -> Option<(f64, Currency)> {
    let d = aom(t)?;
    Some((d.value?, d.currency))
}

fn rule_unit_amount() -> Rule {
    rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
        let c = aom(ts.first()?)?.currency;
        let v = numeral_value(ts.get(1)?)?;
        tok(with_value(v, currency_only(c)))
    })
}

fn rule_dollar() -> Rule {
    rule("dollar", vec![regex("d(ó|o)lar(es)?")], |_| tok(currency_only(Currency::Dollar)))
}

fn rule_cent() -> Rule {
    rule("cent", vec![regex("centavos?")], |_| tok(currency_only(Currency::Cent)))
}

fn rule_pounds() -> Rule {
    rule("£", vec![regex("(pound|libra)s?")], |_| tok(currency_only(Currency::Pound)))
}

fn rule_intersect_and_numeral() -> Rule {
    rule(
        "intersect (and number)",
        vec![pred(is_without_cents), regex("y"), pred(is_natural)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = numeral_value(ts.get(2)?)?;
            tok(with_cents(c, fd.clone()))
        },
    )
}

fn rule_intersect_x_cents() -> Rule {
    rule("intersect (X cents)", vec![pred(is_without_cents), pred(is_cents)], |ts| {
        let fd = aom(ts.first()?)?;
        let c = aom(ts.get(1)?)?.value?;
        tok(with_cents(c, fd.clone()))
    })
}

fn rule_intersect_and_x_cents() -> Rule {
    rule(
        "intersect (and X cents)",
        vec![pred(is_without_cents), regex("y"), pred(is_cents)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = aom(ts.get(2)?)?.value?;
            tok(with_cents(c, fd.clone()))
        },
    )
}

fn rule_intersect() -> Rule {
    rule("intersect", vec![pred(is_without_cents), pred(is_natural)], |ts| {
        let fd = aom(ts.first()?)?;
        let c = numeral_value(ts.get(1)?)?;
        tok(with_cents(c, fd.clone()))
    })
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> to|and <amount-of-money>",
        vec![regex("de|entre"), pred(is_positive), regex("a|y"), pred(is_simple_amount_of_money)],
        |ts| {
            let from = numeral_value(ts.get(1)?)?;
            let (to, c) = vc(ts.get(3)?)?;
            if from < to { tok(with_interval(from, to, currency_only(c))) } else { None }
        },
    )
}

fn rule_interval_between() -> Rule {
    rule(
        "between|from <amount-of-money> to|and <amount-of-money>",
        vec![regex("de|entre"), pred(is_simple_amount_of_money), regex("a|y"), pred(is_simple_amount_of_money)],
        |ts| {
            let (from, c1) = vc(ts.get(1)?)?;
            let (to, c2) = vc(ts.get(3)?)?;
            if from < to && c1 == c2 { tok(with_interval(from, to, currency_only(c1))) } else { None }
        },
    )
}

fn rule_interval_numeral_dash() -> Rule {
    rule(
        "<numeral> - <amount-of-money>",
        vec![pred(is_natural), regex("-"), pred(is_simple_amount_of_money)],
        |ts| {
            let from = numeral_value(ts.first()?)?;
            let (to, c) = vc(ts.get(2)?)?;
            if from < to { tok(with_interval(from, to, currency_only(c))) } else { None }
        },
    )
}

fn rule_interval_dash() -> Rule {
    rule(
        "<amount-of-money> - <amount-of-money>",
        vec![pred(is_simple_amount_of_money), regex("-"), pred(is_simple_amount_of_money)],
        |ts| {
            let (from, c1) = vc(ts.first()?)?;
            let (to, c2) = vc(ts.get(2)?)?;
            if from < to && c1 == c2 { tok(with_interval(from, to, currency_only(c1))) } else { None }
        },
    )
}

fn rule_interval_max() -> Rule {
    rule(
        "less/no more than <amount-of-money>",
        vec![regex("menos de|no más de"), pred(is_simple_amount_of_money)],
        |ts| {
            let (to, c) = vc(ts.get(1)?)?;
            tok(with_max(to, currency_only(c)))
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "no less/more than <amount-of-money>",
        vec![regex("más de|no menos de"), pred(is_simple_amount_of_money)],
        |ts| {
            let (to, c) = vc(ts.get(1)?)?;
            tok(with_min(to, currency_only(c)))
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_unit_amount(),
        rule_cent(),
        rule_dollar(),
        rule_intersect(),
        rule_intersect_and_numeral(),
        rule_intersect_and_x_cents(),
        rule_intersect_x_cents(),
        rule_pounds(),
        rule_interval_between_numeral(),
        rule_interval_between(),
        rule_interval_numeral_dash(),
        rule_interval_dash(),
        rule_interval_max(),
        rule_interval_min(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
