//! `Duckling.AmountOfMoney.ZH.Rules`.

use once_cell::sync::Lazy;

use crate::dims::amount_of_money::*;
use crate::dims::numeral::{decimals_to_double, get_int_value, is_natural, is_positive, number_between, one_of};
use crate::types::{groups, pred, regex, rule, Rule, Token};

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

/// `Duckling.AmountOfMoney.Helpers.isCent`.
fn is_cent(t: &Token) -> bool {
    match t {
        Token::AmountOfMoney(AmountOfMoneyData { value: Some(c), currency: Currency::Cent, .. }) => {
            get_int_value(*c).map(|i| (0..=9).contains(&i)).unwrap_or(false)
        }
        _ => false,
    }
}

fn simple_and_dime(t: &Token) -> bool {
    is_simple_amount_of_money(t) && is_dime(t)
}

fn simple_and_cent(t: &Token) -> bool {
    is_simple_amount_of_money(t) && is_cent(t)
}

fn simple_and_cents(t: &Token) -> bool {
    is_simple_amount_of_money(t) && is_cents(t)
}

fn simple_and_without_cents(t: &Token) -> bool {
    is_simple_amount_of_money(t) && is_without_cents(t)
}

fn rule_cny() -> Rule {
    rule("cny", vec![regex("人民币|人民幣")], |_| tok(currency_only(Currency::CNY)))
}

fn rule_cny_prefix() -> Rule {
    rule("cny prefix", vec![regex("人民币|人民幣"), pred(is_positive)], |ts| {
        tok(with_value(num(ts.get(1)?)?, currency_only(Currency::CNY)))
    })
}

fn rule_hkd() -> Rule {
    rule("HKD", vec![regex("港幣")], |_| tok(currency_only(Currency::HKD)))
}

fn rule_hkd_prefix() -> Rule {
    rule("hkd prefix", vec![regex("港幣"), pred(is_positive)], |ts| {
        tok(with_value(num(ts.get(1)?)?, currency_only(Currency::HKD)))
    })
}

fn rule_cent() -> Rule {
    rule("cent", vec![regex("分|仙")], |_| tok(currency_only(Currency::Cent)))
}

fn rule_dime() -> Rule {
    rule("dime", vec![pred(is_positive), regex("角|毛|毫")], |ts| {
        tok(with_cents(num(ts.first()?)? * 10.0, currency_only(Currency::Cent)))
    })
}

fn rule_dollar() -> Rule {
    rule("dollar", vec![regex("元|圆|块|蚊|個")], |_| tok(currency_only(Currency::Dollar)))
}

fn rule_precision() -> Rule {
    rule(
        "exactly/about <amount-of-money>",
        vec![regex("刚好|恰好|大概"), pred(is_money_with_value)],
        |ts| ts.get(1).map(|t| (*t).clone()),
    )
}

fn rule_precision2() -> Rule {
    rule("about <amount-of-money>", vec![pred(is_money_with_value), regex("左右")], |ts| {
        ts.first().map(|t| (*t).clone())
    })
}

fn rule_intersect_dimes_and_cents() -> Rule {
    rule(
        "intersect (X dimes and X cents)",
        vec![pred(simple_and_dime), pred(simple_and_cent)],
        |ts| {
            let d = aom(ts.first()?)?.value?;
            let c = aom(ts.get(1)?)?.value?;
            tok(with_cents(c + d, currency_only(Currency::Cent)))
        },
    )
}

fn rule_intersect_dollars_and_dimes_cents() -> Rule {
    rule(
        "intersect (X dollars and X dimes/cents)",
        vec![pred(simple_and_without_cents), pred(simple_and_cents)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = aom(ts.get(1)?)?.value?;
            tok(with_cents(c, fd.clone()))
        },
    )
}

fn rule_intersect_dollars_and_a_half() -> Rule {
    rule(
        "<amount-of-money> and a half",
        vec![pred(simple_and_without_cents), regex("半")],
        |ts| tok(with_cents(50.0, aom(ts.first()?)?.clone())),
    )
}

fn rule_intersect() -> Rule {
    rule(
        "intersect (implicit 0 delimited cents)",
        vec![pred(simple_and_without_cents), regex("0|零|〇"), one_of((1..=9).map(|i| i as f64).collect())],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = num(ts.get(2)?)?;
            tok(with_cents(c, fd.clone()))
        },
    )
}

fn rule_intersect2() -> Rule {
    rule(
        "intersect (implicit unitless cents)",
        vec![pred(simple_and_dime), pred(is_natural)],
        |ts| {
            let v = aom(ts.first()?)?.value?;
            let c = num(ts.get(1)?)?;
            tok(with_cents(v + c, currency_only(Currency::Cent)))
        },
    )
}

fn rule_intersect3() -> Rule {
    rule(
        "intersect (implicit unitless dimes)",
        vec![pred(simple_and_without_cents), pred(is_natural)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let d = num(ts.get(1)?)?;
            tok(with_cents(d * 10.0, fd.clone()))
        },
    )
}

fn rule_one_dollar_and_a_half() -> Rule {
    rule("one dollar and a half (short form)", vec![regex("個半")], |_| {
        tok(with_value(1.5, currency_only(Currency::Dollar)))
    })
}

fn rule_one_dollar_and() -> Rule {
    rule(
        "one dollar and x dimes (short form)",
        vec![regex("個"), pred(number_between(1.0, 10.0))],
        |ts| {
            let v = num(ts.get(1)?)?;
            tok(with_value(1.0 + decimals_to_double(v), currency_only(Currency::Dollar)))
        },
    )
}

fn rule_interval_numeral_dash() -> Rule {
    rule(
        "<numeral> - <amount-of-money>",
        vec![pred(is_positive), regex("-|~|到|至"), pred(is_simple_amount_of_money)],
        |ts| {
            let from = num(ts.first()?)?;
            let d = aom(ts.get(2)?)?;
            let to = d.value?;
            if from < to {
                tok(with_interval(from, to, currency_only(d.currency)))
            } else {
                None
            }
        },
    )
}

fn rule_interval_dash() -> Rule {
    rule(
        "<amount-of-money> - <amount-of-money>",
        vec![pred(is_simple_amount_of_money), regex("-|~|到|至"), pred(is_simple_amount_of_money)],
        |ts| {
            let a = aom(ts.first()?)?;
            let b = aom(ts.get(2)?)?;
            let (from, to) = (a.value?, b.value?);
            if from < to && a.currency == b.currency {
                tok(with_interval(from, to, currency_only(a.currency)))
            } else {
                None
            }
        },
    )
}

fn rule_interval_bound() -> Rule {
    rule(
        "under/less/lower/no more than <amount-of-money> (最多|至少|最少)",
        vec![regex("(最多|至少|最少|起碼)"), pred(is_simple_amount_of_money)],
        |ts| {
            let m = groups(ts)?.first()?.to_string();
            let d = aom(ts.get(1)?)?;
            let to = d.value?;
            match m.as_str() {
                "最多" => tok(with_max(to, currency_only(d.currency))),
                "最少" | "至少" | "起碼" => tok(with_min(to, currency_only(d.currency))),
                _ => None,
            }
        },
    )
}

fn rule_interval_bound2() -> Rule {
    rule(
        "under/less/lower/no more than <amount-of-money> (以下|以上)",
        vec![pred(is_simple_amount_of_money), regex("(以下|以上)")],
        |ts| {
            let d = aom(ts.first()?)?;
            let to = d.value?;
            let m = groups(&ts[1..])?.first()?.to_string();
            match m.as_str() {
                "以下" => tok(with_max(to, currency_only(d.currency))),
                "以上" => tok(with_min(to, currency_only(d.currency))),
                _ => None,
            }
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_cent(),
        rule_cny(),
        rule_cny_prefix(),
        rule_hkd(),
        rule_hkd_prefix(),
        rule_dime(),
        rule_dollar(),
        rule_intersect(),
        rule_intersect2(),
        rule_intersect3(),
        rule_intersect_dimes_and_cents(),
        rule_intersect_dollars_and_dimes_cents(),
        rule_intersect_dollars_and_a_half(),
        rule_interval_dash(),
        rule_interval_numeral_dash(),
        rule_interval_bound(),
        rule_interval_bound2(),
        rule_precision(),
        rule_precision2(),
        rule_one_dollar_and_a_half(),
        rule_one_dollar_and(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
