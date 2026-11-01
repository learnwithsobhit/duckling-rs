//! `Duckling.AmountOfMoney.HR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::amount_of_money::*;
use crate::dims::numeral::{is_natural, is_positive};
use crate::dims::util::to_lower;
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

fn rule_a_currency() -> Rule {
    rule("a <currency>", vec![regex("jed(an|na|no)"), pred(is_currency_only)], |ts| {
        let c = aom(ts.get(1)?)?.currency;
        tok(with_value(1.0, currency_only(c)))
    })
}

fn rule_unit_amount() -> Rule {
    rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
        let c = aom(ts.first()?)?.currency;
        let v = num(ts.get(1)?)?;
        tok(with_value(v, currency_only(c)))
    })
}

fn rule_intersect_and_number() -> Rule {
    rule(
        "intersect (and number)",
        vec![pred(is_without_cents), regex("i"), pred(is_natural)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = num(ts.get(2)?)?;
            tok(with_cents(c, fd.clone()))
        },
    )
}

fn rule_sar() -> Rule {
    rule("SAR", vec![regex("saudijskirijal|saudi rijal?")], |_| tok(currency_only(Currency::SAR)))
}

fn rule_dollar() -> Rule {
    rule("$", vec![regex("dolar(a|i|e)?")], |_| tok(currency_only(Currency::Dollar)))
}

fn rule_cent() -> Rule {
    rule("cent", vec![regex("cent(i|a)?|penij(i|a)?|c|¢|lp|lip(a|e)")], |_| {
        tok(currency_only(Currency::Cent))
    })
}

fn rule_intersect_ix_lipa() -> Rule {
    rule(
        "intersect (i X lipa)",
        vec![pred(is_without_cents), regex("i"), pred(is_cents)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = aom(ts.get(2)?)?.value?;
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

fn rule_pound() -> Rule {
    rule("£", vec![regex("funt(a|e|i)?")], |_| tok(currency_only(Currency::Pound)))
}

fn rule_hrk() -> Rule {
    rule("HRK", vec![regex("kn|(hrvatsk(a|ih|e) )?kun(a|e)")], |_| {
        tok(currency_only(Currency::HRK))
    })
}

fn rule_intersect() -> Rule {
    rule("intersect", vec![pred(is_without_cents), pred(is_natural)], |ts| {
        let fd = aom(ts.first()?)?;
        let c = num(ts.get(1)?)?;
        tok(with_cents(c, fd.clone()))
    })
}

fn rule_other_pounds() -> Rule {
    rule("other pounds", vec![regex("(egipatska|libanonska) ?funta")], |ts| {
        match to_lower(groups(ts)?.first()?).as_str() {
            "egipatska" => tok(currency_only(Currency::EGP)),
            "libanonska" => tok(currency_only(Currency::LBP)),
            _ => None,
        }
    })
}

fn rule_inr() -> Rule {
    rule("INR", vec![regex("rupija?")], |_| tok(currency_only(Currency::INR)))
}

fn rule_kwd() -> Rule {
    rule("KWD", vec![regex("kuvajtski ?dinar")], |_| tok(currency_only(Currency::KWD)))
}

fn rule_qar() -> Rule {
    rule("QAR", vec![regex("katarski(i| )rijal")], |_| tok(currency_only(Currency::QAR)))
}

fn rule_aed() -> Rule {
    rule("AED", vec![regex("drahma?")], |_| tok(currency_only(Currency::AED)))
}

fn rule_precision() -> Rule {
    rule(
        "about|exactly <amount-of-money>",
        vec![regex("oko|otprilike|u blizini|skoro|približno"), pred(is_money_with_value)],
        |ts| Some((*ts.get(1)?).clone()),
    )
}

fn rule_interval_numeral_dash() -> Rule {
    rule(
        "<numeral> - <amount-of-money>",
        vec![pred(is_natural), regex("-"), pred(is_simple_amount_of_money)],
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
        vec![pred(is_simple_amount_of_money), regex("-"), pred(is_simple_amount_of_money)],
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

fn interval_between_numeral(re1: &str, re2: &str) -> Rule {
    rule(
        "between|from <numeral> to|and <amount-of-money>",
        vec![regex(re1), pred(is_positive), regex(re2), pred(is_simple_amount_of_money)],
        |ts| {
            let from = num(ts.get(1)?)?;
            let d = aom(ts.get(3)?)?;
            let to = d.value?;
            if from < to {
                tok(with_interval(from, to, currency_only(d.currency)))
            } else {
                None
            }
        },
    )
}

fn interval_between(re1: &str, re2: &str) -> Rule {
    rule(
        "between|from <amount-of-money> to|and <amount-of-money>",
        vec![
            regex(re1),
            pred(is_simple_amount_of_money),
            regex(re2),
            pred(is_simple_amount_of_money),
        ],
        |ts| {
            let a = aom(ts.get(1)?)?;
            let b = aom(ts.get(3)?)?;
            let (from, to) = (a.value?, b.value?);
            if from < to && a.currency == b.currency {
                tok(with_interval(from, to, currency_only(a.currency)))
            } else {
                None
            }
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "over/above/at least/more than <amount-of-money>",
        vec![regex("više od|najmanje|preko|iznad"), pred(is_simple_amount_of_money)],
        |ts| {
            let d = aom(ts.get(1)?)?;
            tok(with_min(d.value?, currency_only(d.currency)))
        },
    )
}

fn rule_interval_max() -> Rule {
    rule(
        "under/less/lower/no more than <amount-of-money>",
        vec![regex("ispod|manje od"), pred(is_simple_amount_of_money)],
        |ts| {
            let d = aom(ts.get(1)?)?;
            tok(with_max(d.value?, currency_only(d.currency)))
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_a_currency(),
        rule_unit_amount(),
        rule_dollar(),
        rule_pound(),
        rule_other_pounds(),
        rule_aed(),
        rule_cent(),
        rule_hrk(),
        rule_inr(),
        rule_intersect(),
        rule_intersect_and_number(),
        rule_intersect_ix_lipa(),
        rule_intersect_x_cents(),
        rule_kwd(),
        rule_qar(),
        rule_sar(),
        rule_precision(),
        rule_interval_numeral_dash(),
        rule_interval_dash(),
        interval_between_numeral("od|otprilike|približno", "do"),
        interval_between_numeral("izmedju", "i"),
        interval_between("od|otprilike", "do"),
        interval_between("izmedju", "i"),
        rule_interval_min(),
        rule_interval_max(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
