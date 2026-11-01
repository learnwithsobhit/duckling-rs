//! `Duckling.AmountOfMoney.VI.Rules`.

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

fn rule_unit_amount() -> Rule {
    rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
        let c = aom(ts.first()?)?.currency;
        let v = numeral_value(ts.get(1)?)?;
        tok(with_value(v, currency_only(c)))
    })
}

fn rule_ng() -> Rule {
    rule("đồng", vec![regex("đ(ồng)?")], |_| tok(currency_only(Currency::VND)))
}

fn rule_dollar() -> Rule {
    rule("$", vec![regex("đô( la)?( mỹ)?")], |_| tok(currency_only(Currency::Dollar)))
}

fn rule_vnd() -> Rule {
    rule("VNĐ", vec![regex(r"vn(Đ|D|\$)")], |_| tok(currency_only(Currency::VND)))
}

fn rule_cent() -> Rule {
    rule("cent", vec![regex("xen|xu|cắc|penn(y|ies)")], |_| tok(currency_only(Currency::Cent)))
}

fn rule_pounds() -> Rule {
    rule("£", vec![regex("pounds?|pao")], |_| tok(currency_only(Currency::Pound)))
}

fn rule_intersect() -> Rule {
    rule("intersect", vec![pred(is_without_cents), pred(is_natural)], |ts| {
        let fd = aom(ts.first()?)?;
        let c = numeral_value(ts.get(1)?)?;
        tok(with_cents(c, fd.clone()))
    })
}

fn rule_intersect_and_numeral() -> Rule {
    rule(
        "intersect and number",
        vec![pred(is_without_cents), regex("và"), pred(is_natural)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = numeral_value(ts.get(2)?)?;
            tok(with_cents(c, fd.clone()))
        },
    )
}

fn rule_intersect_xxuxen() -> Rule {
    rule("intersect (X xu|xen)", vec![pred(is_without_cents), pred(is_cents)], |ts| {
        let fd = aom(ts.first()?)?;
        let c = aom(ts.get(1)?)?.value?;
        tok(with_cents(c, fd.clone()))
    })
}

fn rule_intersect_vxxuxen() -> Rule {
    rule(
        "intersect (và X xu|xen)",
        vec![pred(is_without_cents), regex("và"), pred(is_cents)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = aom(ts.get(2)?)?.value?;
            tok(with_cents(c, fd.clone()))
        },
    )
}

fn rule_dirham() -> Rule {
    rule("AED", vec![regex(r"AED\.|dirhams?")], |_| tok(currency_only(Currency::AED)))
}

fn rule_precision() -> Rule {
    rule(
        "about|exactly <amount-of-money>",
        vec![regex("đúng|chính xác|khoảng( chừng)?|tầm|xấp xỉ|gần"), pred(is_money_with_value)],
        |ts| ts.get(1).map(|t| (*t).clone()),
    )
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> to|and <amount-of-money>",
        vec![regex("từ|giữa|khoảng"), pred(is_positive), regex("đến|tới|và"), pred(is_simple_amount_of_money)],
        |ts| {
            let from = numeral_value(ts.get(1)?)?;
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
        "từ|giữa <amount-of-money> đến|tới|và <amount-of-money>",
        vec![
            regex("từ|giữa"),
            pred(is_simple_amount_of_money),
            regex("đến|tới|và"),
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
        vec![pred(is_natural), regex("-"), pred(is_simple_amount_of_money)],
        |ts| {
            let from = numeral_value(ts.first()?)?;
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
    )
}

fn rule_interval_max() -> Rule {
    rule(
        "under/less/lower/no more than <amount-of-money>",
        vec![
            regex("dưới|(bé|ít|thấp|kém|không( cao| nhiều)?) hơn|không (đến|tới|quá)|(nhiều|cao) nhất"),
            pred(is_simple_amount_of_money),
        ],
        |ts| {
            let (to, c) = vc(ts.get(1)?)?;
            tok(with_max(to, currency_only(c)))
        },
    )
}

fn rule_interval_max2() -> Rule {
    rule(
        "under <amount-of-money>",
        vec![regex("từ"), pred(is_simple_amount_of_money), regex("trở xuống")],
        |ts| {
            let (to, c) = vc(ts.get(1)?)?;
            tok(with_max(to, currency_only(c)))
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "over/above/at least/more than <amount-of-money>",
        vec![
            regex("trên|(lớn |nhiều |cao |không (thấp|ít|kém) )?hơn|(ít|thấp) nhất"),
            pred(is_simple_amount_of_money),
        ],
        |ts| {
            let (to, c) = vc(ts.get(1)?)?;
            tok(with_min(to, currency_only(c)))
        },
    )
}

fn rule_interval_min2() -> Rule {
    rule(
        "above <amount-of-money>",
        vec![regex("từ"), pred(is_simple_amount_of_money), regex("trở lên")],
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
        rule_dirham(),
        rule_dollar(),
        rule_intersect(),
        rule_intersect_and_numeral(),
        rule_intersect_vxxuxen(),
        rule_intersect_xxuxen(),
        rule_interval_between_numeral(),
        rule_interval_between(),
        rule_interval_dash(),
        rule_interval_numeral_dash(),
        rule_interval_max(),
        rule_interval_max2(),
        rule_interval_min(),
        rule_interval_min2(),
        rule_ng(),
        rule_pounds(),
        rule_precision(),
        rule_vnd(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
