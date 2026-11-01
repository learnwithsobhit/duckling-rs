//! `Duckling.AmountOfMoney.KA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::amount_of_money::*;
use crate::dims::numeral::{is_natural, is_positive};
use crate::dims::util::to_lower;
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

fn rx0(t: &Token) -> Option<String> {
    match t {
        Token::RegexMatch(g) => g.0.first().map(|s| s.to_string()),
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
    rule(
        "£",
        vec![regex("(გირვანქა )?სტერლინგი?ს?(ად)?თ?|(გირვანქა )?ფუნ?ტი?ს?(ად)?თ?")],
        |_| tok(currency_only(Currency::Pound)),
    )
}

fn rule_other_pounds() -> Rule {
    rule("other pounds", vec![regex("(ეგვიპტური|ლიბანური) ?ფუნ?ტი?ს?(ად)?თ?")], |ts| {
        match to_lower(&rx0(ts.first()?)?).as_str() {
            "ეგვიპტური" => tok(currency_only(Currency::EGP)),
            "ლიბანური" => tok(currency_only(Currency::LBP)),
            _ => None,
        }
    })
}

fn rule_riyals() -> Rule {
    rule("riyals", vec![regex("(კატარული|საუდის არაბული) ?რიალი?")], |ts| {
        match to_lower(&rx0(ts.first()?)?).as_str() {
            "კატარული" => tok(currency_only(Currency::QAR)),
            "საუდის არაბული" => tok(currency_only(Currency::SAR)),
            _ => None,
        }
    })
}

fn rule_dinars() -> Rule {
    rule("dinars", vec![regex("(ქუვეითური)? ?დინარი?")], |ts| {
        match to_lower(&rx0(ts.first()?)?).as_str() {
            "ქუვეითური" => tok(currency_only(Currency::KWD)),
            _ => None,
        }
    })
}

fn rule_dirham() -> Rule {
    rule("დირჰემი", vec![regex("დირჰემი?")], |_| tok(currency_only(Currency::AED)))
}

fn rule_ringgit() -> Rule {
    rule("ringgit", vec![regex("(მალაიზი?ური? )?რინგიტი?")], |_| {
        tok(currency_only(Currency::MYR))
    })
}

fn rule_cent() -> Rule {
    rule(
        "cent",
        vec![regex("ცენტი?ს?(ად)?თ?|თეთრი?ს?(ად)?თ?|პენსი?ს?(ად)?თ?|ევროცენტი?ს?(ად)?თ?")],
        |_| tok(currency_only(Currency::Cent)),
    )
}

fn rule_lari() -> Rule {
    rule("Lari", vec![regex("(ლარი?ს?(ად)?თ?)|GEL")], |_| tok(currency_only(Currency::GEL)))
}

fn rule_dollars() -> Rule {
    rule("Dollars", vec![regex("დოლარი?ს?(ად)?თ?")], |_| tok(currency_only(Currency::Dollar)))
}

fn rule_euros() -> Rule {
    rule("Euros", vec![regex("ევრო?ს?დ?თ?")], |_| tok(currency_only(Currency::EUR)))
}

fn rule_a_currency() -> Rule {
    rule("a <currency>", vec![pred(is_currency_only)], |ts| {
        let mut fd = aom(ts.first()?)?.clone();
        fd.value = Some(1.0);
        tok(fd)
    })
}

fn rule_intersect_and_x_cents() -> Rule {
    rule(
        "intersect (and X cents)",
        vec![pred(is_without_cents), regex("და"), pred(is_cents)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = aom(ts.get(2)?)?.value?;
            tok(with_cents(c, fd.clone()))
        },
    )
}

fn rule_intersect_and_numeral() -> Rule {
    rule(
        "intersect (and number)",
        vec![pred(is_without_cents), regex("და"), pred(is_natural)],
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

fn rule_precision() -> Rule {
    rule(
        "about|exactly <amount-of-money>",
        vec![regex("ზუსტად|იმენა|დაახლოებით|გძეტა"), pred(is_money_with_value)],
        |ts| Some((*ts.get(1)?).clone()),
    )
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> to|and <amount-of-money>",
        vec![pred(is_positive), regex("-დან|დან"), pred(is_simple_amount_of_money), regex("-ა?მდე|ა?მდე")],
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

fn rule_interval_between() -> Rule {
    rule(
        "between|from <amount-of-money> to|and <amount-of-money>",
        vec![
            pred(is_simple_amount_of_money),
            regex("-დან|დან"),
            pred(is_simple_amount_of_money),
            regex("-ა?მდე|ა?მდე"),
        ],
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

fn rule_interval_max() -> Rule {
    rule(
        "under/less/lower/no more than <amount-of-money>",
        vec![pred(is_simple_amount_of_money), regex("-ა?მდე|ა?მდე|(-ზე|ზე) ნაკლები")],
        |ts| {
            let d = aom(ts.first()?)?;
            tok(with_max(d.value?, currency_only(d.currency)))
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "over/above/at least/more than <amount-of-money>",
        vec![pred(is_simple_amount_of_money), regex("-დან|დან|(-ზე|ზე) მეტი")],
        |ts| {
            let d = aom(ts.first()?)?;
            tok(with_min(d.value?, currency_only(d.currency)))
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_unit_amount(),
        rule_a_currency(),
        rule_dollars(),
        rule_cent(),
        rule_euros(),
        rule_dinars(),
        rule_dirham(),
        rule_intersect_and_numeral(),
        rule_intersect_and_x_cents(),
        rule_intersect_x_cents(),
        rule_interval_between_numeral(),
        rule_interval_between(),
        rule_interval_max(),
        rule_interval_min(),
        rule_interval_numeral_dash(),
        rule_interval_dash(),
        rule_other_pounds(),
        rule_pounds(),
        rule_precision(),
        rule_ringgit(),
        rule_riyals(),
        rule_lari(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
