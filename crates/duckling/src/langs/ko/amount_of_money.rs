//! `Duckling.AmountOfMoney.KO.Rules`.

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

fn currency(name: &str, re: &str, c: Currency) -> Rule {
    rule(name, vec![regex(re)], move |_| tok(currency_only(c)))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
            let c = aom(ts.first()?)?.currency;
            let v = num(ts.get(1)?)?;
            tok(with_value(v, currency_only(c)))
        }),
        rule("about <amount-of-money>", vec![regex("약|대충|얼추"), pred(is_money_with_value)], |ts| {
            Some((*ts.get(1)?).clone())
        }),
        rule("<amount-of-money> about", vec![pred(is_money_with_value), regex("정도|쯤")], |ts| {
            Some((*ts.first()?).clone())
        }),
        currency("AUD", "호주달러", Currency::AUD),
        currency("cent", "센트", Currency::Cent),
        currency("dirham", "디르함", Currency::AED),
        currency("$", "달러|불", Currency::Dollar),
        currency("€", "€|유로", Currency::EUR),
        rule(
            "exactly <amount-of-money>",
            vec![regex("딱|정확히"), pred(is_money_with_value)],
            |ts| Some((*ts.get(1)?).clone()),
        ),
        currency("INR", "루피|인도루피", Currency::INR),
        rule("intersect (X cents)", vec![pred(is_without_cents), pred(is_cents)], |ts| {
            let fd = aom(ts.first()?)?;
            let c = aom(ts.get(1)?)?.value?;
            tok(with_cents(c, fd.clone()))
        }),
        currency("₩", "₩|원", Currency::KRW),
        currency("£", "파운드|영국파운드", Currency::Pound),
        rule(
            "between|from <numeral> to|and <amount-of-money>",
            vec![pred(is_positive), regex("에서"), pred(is_simple_amount_of_money)],
            |ts| {
                let from = num(ts.first()?)?;
                let d = aom(ts.get(2)?)?;
                tok(with_interval(from, d.value?, currency_only(d.currency)))
            },
        ),
        rule(
            "between|from <amount-of-money> to|and <amount-of-money>",
            vec![pred(is_simple_amount_of_money), regex("부터"), pred(is_simple_amount_of_money)],
            |ts| {
                let a = aom(ts.first()?)?;
                let b = aom(ts.get(2)?)?;
                if a.currency == b.currency {
                    tok(with_interval(a.value?, b.value?, currency_only(a.currency)))
                } else {
                    None
                }
            },
        ),
        rule(
            "<numeral> - <amount-of-money>",
            vec![pred(is_natural), regex("-"), pred(is_simple_amount_of_money)],
            |ts| {
                let from = num(ts.first()?)?;
                let d = aom(ts.get(2)?)?;
                tok(with_interval(from, d.value?, currency_only(d.currency)))
            },
        ),
        rule(
            "<amount-of-money> - <amount-of-money>",
            vec![pred(is_simple_amount_of_money), regex("-"), pred(is_simple_amount_of_money)],
            |ts| {
                let a = aom(ts.first()?)?;
                let b = aom(ts.get(2)?)?;
                if a.currency == b.currency {
                    tok(with_interval(a.value?, b.value?, currency_only(a.currency)))
                } else {
                    None
                }
            },
        ),
        rule(
            "under/less/lower/no more than <amount-of-money>",
            vec![pred(is_simple_amount_of_money), regex("미만|이하")],
            |ts| {
                let d = aom(ts.first()?)?;
                tok(with_max(d.value?, currency_only(d.currency)))
            },
        ),
        rule(
            "over/above/at least/more than <amount-of-money>",
            vec![pred(is_simple_amount_of_money), regex("초과|이상")],
            |ts| {
                let d = aom(ts.first()?)?;
                tok(with_min(d.value?, currency_only(d.currency)))
            },
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
