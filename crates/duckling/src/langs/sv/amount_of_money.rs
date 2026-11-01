//! `Duckling.AmountOfMoney.SV.Rules`.

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
        // ruleAboutAmountofmoney
        rule(
            "about <amount-of-money>",
            vec![regex("omkring|cirka|runt|ca"), pred(is_money_with_value)],
            |ts| ts.get(1).cloned().cloned(),
        ),
        // ruleCent
        rule("cent", vec![regex("cents?|penn(y|ies)|öre")], |_| tok(currency_only(Currency::Cent))),
        // ruleDirham
        rule("AED", vec![regex("dirhams?")], |_| tok(currency_only(Currency::AED))),
        // ruleExactlyAmountofmoney
        rule(
            "exactly <amount-of-money>",
            vec![regex("exakt|precis"), pred(is_money_with_value)],
            |ts| ts.get(1).cloned().cloned(),
        ),
        // ruleIntersect
        rule("intersect", vec![pred(is_without_cents), pred(is_natural)], |ts| {
            let fd = aom(ts.first()?)?;
            let c = numeral_value(ts.get(1)?)?;
            tok(with_cents(c, fd.clone()))
        }),
        // ruleIntersectAndNumeral
        rule(
            "intersect (and number)",
            vec![pred(is_without_cents), regex("och"), pred(is_natural)],
            |ts| {
                let fd = aom(ts.first()?)?;
                let c = numeral_value(ts.get(2)?)?;
                tok(with_cents(c, fd.clone()))
            },
        ),
        // ruleIntersectAndXCents
        rule(
            "intersect (and X cents)",
            vec![pred(is_without_cents), regex("och"), pred(is_cents)],
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
        // ruleNok
        rule("NOK", vec![regex("norska kronor|nkr")], |_| tok(currency_only(Currency::NOK))),
        // rulePounds
        rule("£", vec![regex("pund?")], |_| tok(currency_only(Currency::Pound))),
        // ruleSek
        rule("SEK", vec![regex("kr(onor)?")], |_| tok(currency_only(Currency::SEK))),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
