//! `Duckling.AmountOfMoney.GA.Rules`.

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

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleUnitAmount
        rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
            let c = aom(ts.first()?)?.currency;
            let v = numeral_value(ts.get(1)?)?;
            tok(with_value(v, currency_only(c)))
        }),
        // ruleAmountofmoneyGlan
        rule(
            "<amount-of-money> glan",
            vec![pred(is_money_with_value), regex("glan|baileach|(go )?d(í|i)reach")],
            |ts| ts.first().cloned().cloned(),
        ),
        // ruleCent
        rule("cent", vec![regex("cents?|g?ch?eint(eanna)?")], |_| {
            tok(currency_only(Currency::Cent))
        }),
        // ruleDollar
        rule("$", vec![regex("n?dh?oll?ai?rs?")], |_| tok(currency_only(Currency::Dollar))),
        // ruleInr
        rule("INR", vec![regex("r(ú|u)pa(í|i)")], |_| tok(currency_only(Currency::INR))),
        // ruleIntersect
        rule("intersect", vec![pred(is_without_cents), pred(is_natural)], |ts| {
            let fd = aom(ts.first()?)?;
            let c = numeral_value(ts.get(1)?)?;
            tok(with_cents(c, fd.clone()))
        }),
        // ruleIntersectAgusNumeral
        rule(
            "intersect (agus number)",
            vec![pred(is_without_cents), regex("agus|is"), pred(is_natural)],
            |ts| {
                let fd = aom(ts.first()?)?;
                let c = numeral_value(ts.get(2)?)?;
                tok(with_cents(c, fd.clone()))
            },
        ),
        // ruleIntersectAndXCents
        rule(
            "intersect (and X cents)",
            vec![pred(is_without_cents), regex("agus|is"), pred(is_cents)],
            |ts| {
                let fd = aom(ts.first()?)?;
                let c = aom(ts.get(2)?)?.value?;
                tok(with_cents(c, fd.clone()))
            },
        ),
        // ruleIntersectXCents
        rule("intersect (X cents)", vec![pred(is_without_cents), pred(is_cents)], |ts| {
            let fd = aom(ts.first()?)?;
            let c = aom(ts.get(1)?)?.value?;
            tok(with_cents(c, fd.clone()))
        }),
        // rulePounds
        rule("£", vec![regex("pounds?|b?ph?unt")], |_| tok(currency_only(Currency::Pound))),
        // ruleThartArAmountofmoney
        rule(
            "thart ar <amount-of-money>",
            vec![regex("thart( ar)?|beagnach|breis (is|agus)"), pred(is_money_with_value)],
            |ts| ts.get(1).cloned().cloned(),
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
