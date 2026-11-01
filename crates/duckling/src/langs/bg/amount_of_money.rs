//! `Duckling.AmountOfMoney.BG.Rules`.

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

fn vc(t: &Token) -> Option<(f64, Currency)> {
    let d = aom(t)?;
    Some((d.value?, d.currency))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
            let c = aom(ts.first()?)?.currency;
            let v = numeral_value(ts.get(1)?)?;
            tok(with_value(v, currency_only(c)))
        }),
        rule("лв", vec![regex("ле?ва?")], |_| tok(currency_only(Currency::BGN))),
        rule("$", vec![regex("долар(а|и)?")], |_| tok(currency_only(Currency::Dollar))),
        rule("cent", vec![regex("ст(отинк(a|и))?|цента?|пени(та)?|пенса?|ц")], |_| {
            tok(currency_only(Currency::Cent))
        }),
        rule("€", vec![regex("евр(о|а)")], |_| tok(currency_only(Currency::EUR))),
        rule("intersect", vec![pred(is_without_cents), pred(is_natural)], |ts| {
            let fd = aom(ts.first()?)?;
            let c = numeral_value(ts.get(1)?)?;
            tok(with_cents(c, fd.clone()))
        }),
        rule(
            "intersect (and number)",
            vec![pred(is_without_cents), regex("и"), pred(is_natural)],
            |ts| {
                let fd = aom(ts.first()?)?;
                let c = numeral_value(ts.get(2)?)?;
                tok(with_cents(c, fd.clone()))
            },
        ),
        rule(
            "intersect (and X cents)",
            vec![pred(is_without_cents), regex("и"), pred(is_cents)],
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
        rule(
            "between|from <numeral> to|and <amount-of-money>",
            vec![regex("между|от"), pred(is_positive), regex("до|и"), pred(is_simple_amount_of_money)],
            |ts| {
                let from = numeral_value(ts.get(1)?)?;
                let (to, c) = vc(ts.get(3)?)?;
                if from < to {
                    tok(with_interval(from, to, currency_only(c)))
                } else {
                    None
                }
            },
        ),
        rule(
            "between|from <amount-of-money> to|and <amount-of-money>",
            vec![
                regex("между|от"),
                pred(is_simple_amount_of_money),
                regex("до|и"),
                pred(is_simple_amount_of_money),
            ],
            |ts| {
                let (from, c1) = vc(ts.get(1)?)?;
                let (to, c2) = vc(ts.get(3)?)?;
                if from < to && c1 == c2 {
                    tok(with_interval(from, to, currency_only(c1)))
                } else {
                    None
                }
            },
        ),
        rule(
            "under/less/lower/no more than <amount-of-money>",
            vec![regex("под|по-малко от|не повече от"), pred(is_simple_amount_of_money)],
            |ts| {
                let (to, c) = vc(ts.get(1)?)?;
                tok(with_max(to, currency_only(c)))
            },
        ),
        rule(
            "over/above/at least/more than <amount-of-money>",
            vec![regex("над|поне|повече от"), pred(is_simple_amount_of_money)],
            |ts| {
                let (to, c) = vc(ts.get(1)?)?;
                tok(with_min(to, currency_only(c)))
            },
        ),
        rule(
            "<numeral> - <amount-of-money>",
            vec![pred(is_positive), regex("-"), pred(is_simple_amount_of_money)],
            |ts| {
                let from = numeral_value(ts.first()?)?;
                let (to, c) = vc(ts.get(2)?)?;
                if from < to {
                    tok(with_interval(from, to, currency_only(c)))
                } else {
                    None
                }
            },
        ),
        rule(
            "<amount-of-money> - <amount-of-money>",
            vec![pred(is_simple_amount_of_money), regex("-"), pred(is_simple_amount_of_money)],
            |ts| {
                let (from, c1) = vc(ts.first()?)?;
                let (to, c2) = vc(ts.get(2)?)?;
                if from < to && c1 == c2 {
                    tok(with_interval(from, to, currency_only(c1)))
                } else {
                    None
                }
            },
        ),
        rule("£", vec![regex("паунд(а|и)?")], |_| tok(currency_only(Currency::Pound))),
        rule(
            "about|exactly <amount-of-money>",
            vec![regex("точно|около|приблизително|близо (до)?|почти"), pred(is_money_with_value)],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
