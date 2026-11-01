//! `Duckling.AmountOfMoney.ID.Rules`.

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

fn vc(t: &Token) -> Option<(f64, Currency)> {
    let d = aom(t)?;
    Some((d.value?, d.currency))
}

fn rule_unit_amount() -> Rule {
    rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
        let c = aom(ts.first()?)?.currency;
        let v = num(ts.get(1)?)?;
        tok(with_value(v, currency_only(c)))
    })
}

fn rule_dollar() -> Rule {
    rule("$", vec![regex("dolar?")], |_| tok(currency_only(Currency::Dollar)))
}

fn rule_idr() -> Rule {
    rule("IDR", vec![regex(r"rp\.?|rupiah")], |_| tok(currency_only(Currency::IDR)))
}

fn rule_pounds() -> Rule {
    rule("£", vec![regex("pound( sterling)?")], |_| tok(currency_only(Currency::Pound)))
}

fn rule_intersect() -> Rule {
    rule("intersect", vec![pred(is_without_cents), pred(is_natural)], |ts| {
        let fd = aom(ts.first()?)?;
        let c = num(ts.get(1)?)?;
        tok(with_cents(c, fd.clone()))
    })
}

fn rule_jpy() -> Rule {
    rule("JPY", vec![regex(r"¥\.")], |_| tok(currency_only(Currency::JPY)))
}

fn rule_precision() -> Rule {
    rule(
        "about|exactly <amount-of-money>",
        vec![regex("tepatnya|kira-kira|sekitar|hampir"), pred(is_money_with_value)],
        |ts| ts.get(1).map(|t| (*t).clone()),
    )
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> to|and <amount-of-money>",
        vec![regex("sekitar|dari|antara"), pred(is_positive), regex("sampai|hingga"), pred(is_simple_amount_of_money)],
        |ts| {
            let from = num(ts.get(1)?)?;
            let (to, c) = vc(ts.get(3)?)?;
            if from < to {
                tok(with_interval(from, to, currency_only(c)))
            } else {
                None
            }
        },
    )
}

fn rule_interval_between() -> Rule {
    rule(
        "between|from <amount-of-money> to|and <amount-of-money>",
        vec![
            regex("sekitar|dari|antara|kira-kira"),
            pred(is_simple_amount_of_money),
            regex("sampai|hingga"),
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
    )
}

fn rule_interval_numeral_dash() -> Rule {
    rule(
        "<numeral> - <amount-of-money>",
        vec![pred(is_natural), regex(r"\-"), pred(is_simple_amount_of_money)],
        |ts| {
            let from = num(ts.first()?)?;
            let (to, c) = vc(ts.get(2)?)?;
            if from < to {
                tok(with_interval(from, to, currency_only(c)))
            } else {
                None
            }
        },
    )
}

fn rule_interval_dash() -> Rule {
    rule(
        "<amount-of-money> - <amount-of-money>",
        vec![pred(is_simple_amount_of_money), regex(r"\-"), pred(is_simple_amount_of_money)],
        |ts| {
            let (from, c1) = vc(ts.first()?)?;
            let (to, c2) = vc(ts.get(2)?)?;
            if from < to && c1 == c2 {
                tok(with_interval(from, to, currency_only(c1)))
            } else {
                None
            }
        },
    )
}

fn rule_interval_max() -> Rule {
    rule(
        "under/less/lower/no more than <amount-of-money>",
        vec![regex("(kurang|lebih murah) dari|tidak sampai|di bawah"), pred(is_simple_amount_of_money)],
        |ts| {
            let (to, c) = vc(ts.get(1)?)?;
            tok(with_max(to, currency_only(c)))
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "over/above/at least/more than <amount-of-money>",
        vec![regex("di atas|lebih dari|melebihi|melewati"), pred(is_simple_amount_of_money)],
        |ts| {
            let (to, c) = vc(ts.get(1)?)?;
            tok(with_min(to, currency_only(c)))
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_unit_amount(),
        rule_dollar(),
        rule_idr(),
        rule_intersect(),
        rule_jpy(),
        rule_pounds(),
        rule_precision(),
        rule_interval_between_numeral(),
        rule_interval_between(),
        rule_interval_max(),
        rule_interval_min(),
        rule_interval_numeral_dash(),
        rule_interval_dash(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
