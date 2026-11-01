//! `Duckling.AmountOfMoney.PT.Rules`.

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

fn const_currency(name: &str, re: &str, c: Currency) -> Rule {
    rule(name, vec![regex(re)], move |_| tok(currency_only(c)))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
            let c = aom(ts.first()?)?.currency;
            let v = numeral_value(ts.get(1)?)?;
            tok(with_value(v, currency_only(c)))
        }),
        const_currency("BRL", "reais|r\\$", Currency::BRL),
        const_currency("cent", "centavos?", Currency::Cent),
        const_currency("$", "dolares?", Currency::Dollar),
        rule("intersect", vec![pred(is_without_cents), pred(is_natural)], |ts| {
            let fd = aom(ts.first()?)?;
            let c = numeral_value(ts.get(1)?)?;
            tok(with_cents(c, fd.clone()))
        }),
        rule(
            "intersect (and number)",
            vec![pred(is_without_cents), regex("e"), pred(is_natural)],
            |ts| {
                let fd = aom(ts.first()?)?;
                let c = numeral_value(ts.get(2)?)?;
                tok(with_cents(c, fd.clone()))
            },
        ),
        rule(
            "intersect (and X cents)",
            vec![pred(is_without_cents), regex("e"), pred(is_cents)],
            |ts| {
                let fd = aom(ts.first()?)?;
                let c = aom(ts.get(2)?)?.value?;
                tok(with_cents(c, fd.clone()))
            },
        ),
        rule("intersect (X cents)", vec![pred(is_without_cents), pred(is_cents)], |ts| {
            let fd = aom(ts.first()?)?;
            let c = aom(ts.get(1)?)?.value?;
            tok(with_cents(c, fd.clone()))
        }),
        const_currency("£", "libras?", Currency::Pound),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
