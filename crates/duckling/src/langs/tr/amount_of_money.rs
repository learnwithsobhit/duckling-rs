//! `Duckling.AmountOfMoney.TR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::amount_of_money::*;
use crate::dims::numeral::{is_natural, is_positive};
use crate::types::{pred, regex, rule, Rule, Token};

fn tok(d: AmountOfMoneyData) -> Option<Token> {
    Some(Token::AmountOfMoney(d))
}

fn aom(t: &Token) -> Option<&AmountOfMoneyData> {
    match t {
        Token::AmountOfMoney(d) => Some(d),
        _ => None,
    }
}

fn num(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

fn rule_unit_amount() -> Rule {
    rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
        let c = aom(ts.first()?)?.currency;
        let v = num(ts.get(1)?)?;
        tok(with_value(v, currency_only(c)))
    })
}

fn rule_pounds() -> Rule {
    rule("₺", vec![regex("₺")], |_| tok(currency_only(Currency::TRY)))
}

fn rule_cent() -> Rule {
    rule("cent", vec![regex("kuruş?")], |_| tok(currency_only(Currency::Cent)))
}

fn rule_a_dollar_coin() -> Rule {
    rule("a <dollar coin>", vec![regex("kuruş"), pred(is_dollar_coin)], |ts| {
        ts.get(1).map(|t| (*t).clone())
    })
}

fn rule_num_dollar_coins() -> Rule {
    rule("X <dollar coins>", vec![pred(is_natural), pred(is_dollar_coin)], |ts| {
        let c = num(ts.first()?)?;
        let d = aom(ts.get(1)?)?;
        let val = d.value?;
        tok(with_value(c * val, currency_only(d.currency)))
    })
}

fn rule_intersect_and_x_cents() -> Rule {
    rule(
        "intersect (and X cents)",
        vec![pred(is_without_cents), regex("(lira|tl)"), pred(is_cents)],
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
        let c = num(ts.get(1)?)?;
        tok(with_cents(c, fd.clone()))
    })
}

fn rule_intersect_and_numeral() -> Rule {
    rule(
        "intersect (and number)",
        vec![pred(is_without_cents), regex("(lira|tl)"), pred(is_natural)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = num(ts.get(2)?)?;
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

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_unit_amount(),
        rule_cent(),
        rule_a_dollar_coin(),
        rule_num_dollar_coins(),
        rule_intersect(),
        rule_intersect_and_numeral(),
        rule_intersect_and_x_cents(),
        rule_intersect_x_cents(),
        rule_pounds(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
