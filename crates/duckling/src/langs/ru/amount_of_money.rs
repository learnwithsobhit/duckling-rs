//! `Duckling.AmountOfMoney.RU.Rules`.

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

fn rule_ruble() -> Rule {
    rule("руб", vec![regex("руб(л(ь|ями|ям|ях|я|ю|(е|ё)м|ей|е|и))?")], |_| {
        tok(currency_only(Currency::RUB))
    })
}

fn rule_pounds() -> Rule {
    rule("£", vec![regex("фунт(ами|ам|ах|а|у|ом|е|ы|ов)?")], |_| {
        tok(currency_only(Currency::Pound))
    })
}

fn rule_dollar() -> Rule {
    rule("$", vec![regex("долл?ар(ами|ам|ах|а|у|ом|е|ы|ов)?")], |_| {
        tok(currency_only(Currency::Dollar))
    })
}

fn rule_cent() -> Rule {
    rule(
        "cent",
        vec![regex("цент(ами|ам|ах|а|у|ом|е|ы|ов)?|пени|пенс(ами|ам|ах|а|у|ом|е|ы|ов)?|ц")],
        |_| tok(currency_only(Currency::Cent)),
    )
}

fn rule_eur() -> Rule {
    rule("€", vec![regex("евр(о|а)")], |_| tok(currency_only(Currency::EUR)))
}

fn rule_bucks() -> Rule {
    rule("bucks", vec![regex("бакс(ами|ам|ах|а|у|ом|ах|ы|ов)?")], |_| {
        tok(currency_only(Currency::Unnamed))
    })
}

fn rule_hryvnia() -> Rule {
    rule("Hryvnia", vec![regex("грн")], |_| tok(currency_only(Currency::UAH)))
}

fn rule_intersect_and_x_cents() -> Rule {
    rule(
        "intersect (and X cents)",
        vec![pred(is_without_cents), regex("и"), pred(is_cents)],
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
        vec![pred(is_without_cents), regex("и"), pred(is_natural)],
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
        vec![regex("точно|около|приблизительно|блико( к)?|почти"), pred(is_money_with_value)],
        |ts| Some((*ts.get(1)?).clone()),
    )
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> to|and <amount-of-money>",
        vec![regex("между|от"), pred(is_positive), regex("до|и"), pred(is_simple_amount_of_money)],
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

fn rule_interval_between_numeral2() -> Rule {
    rule(
        "between|from <amount-of-money> to|and <numeral>",
        vec![regex("между|от"), pred(is_simple_amount_of_money), regex("до|и"), pred(is_natural)],
        |ts| {
            let d = aom(ts.get(1)?)?;
            let from = d.value?;
            let to = num(ts.get(3)?)?;
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
            regex("между|от"),
            pred(is_simple_amount_of_money),
            regex("до|и"),
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

fn rule_interval_numeral_dash() -> Rule {
    rule(
        "<numeral> - <amount-of-money>",
        vec![pred(is_positive), regex("-"), pred(is_simple_amount_of_money)],
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
        vec![regex("под|((менее|меньше|ниже|не больше|не более)( чем)?)"), pred(is_simple_amount_of_money)],
        |ts| {
            let d = aom(ts.get(1)?)?;
            tok(with_max(d.value?, currency_only(d.currency)))
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "over/above/at least/more than <amount-of-money>",
        vec![regex("сверх|как минимум|((с?выше|больше|более)( чем)?)"), pred(is_simple_amount_of_money)],
        |ts| {
            let d = aom(ts.get(1)?)?;
            tok(with_min(d.value?, currency_only(d.currency)))
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_unit_amount(),
        rule_bucks(),
        rule_cent(),
        rule_dollar(),
        rule_eur(),
        rule_hryvnia(),
        rule_intersect(),
        rule_intersect_and_numeral(),
        rule_intersect_and_x_cents(),
        rule_intersect_x_cents(),
        rule_interval_between_numeral(),
        rule_interval_between_numeral2(),
        rule_interval_between(),
        rule_interval_max(),
        rule_interval_min(),
        rule_interval_numeral_dash(),
        rule_interval_dash(),
        rule_pounds(),
        rule_precision(),
        rule_ruble(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
