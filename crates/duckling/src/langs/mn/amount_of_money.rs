//! `Duckling.AmountOfMoney.MN.Rules`.

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

fn rule_tugriks() -> Rule {
    rule("төг", vec![regex("төг(рөг(ийн)?)?")], |_| tok(currency_only(Currency::MNT)))
}

fn rule_pounds() -> Rule {
    rule("£", vec![regex("фунт(аар|тай|аас)?")], |_| tok(currency_only(Currency::Pound)))
}

fn rule_gbp() -> Rule {
    rule("Mongolian GBP", vec![regex("Английн\\s+фунт")], |_| tok(currency_only(Currency::GBP)))
}

fn rule_dollar() -> Rule {
    rule("$", vec![regex("доллар(ын|оор|оос|той)?")], |_| tok(currency_only(Currency::Dollar)))
}

fn rule_cent() -> Rule {
    rule(
        "cent",
        vec![regex("цент(ийн|ээс|ээр|тэй)?|пени|пенс(ээр|гээр|тэй|ээс|гээс)?|ц")],
        |_| tok(currency_only(Currency::Cent)),
    )
}

fn rule_eur() -> Rule {
    rule("€", vec![regex("евро")], |_| tok(currency_only(Currency::EUR)))
}

fn rule_bucks() -> Rule {
    rule("bucks", vec![regex("бакс(аар|тай|аас)?")], |_| tok(currency_only(Currency::Unnamed)))
}

fn rule_precision() -> Rule {
    rule(
        "about|exactly <amount-of-money>",
        vec![regex("яг|ойролцоогоор|бараг"), pred(is_money_with_value)],
        |ts| Some((*ts.get(1)?).clone()),
    )
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> to|and <amount-of-money>",
        vec![
            pred(is_positive),
            regex("-c"),
            pred(is_simple_amount_of_money),
            regex("(-н\\s+)?(хооронд|хүртэл)"),
        ],
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

fn rule_interval_between_numeral2() -> Rule {
    rule(
        "between|from <amount-of-money> to|and <numeral>",
        vec![
            pred(is_simple_amount_of_money),
            regex("-c"),
            pred(is_natural),
            regex("(-н\\s+)?(хооронд|хүртэл)"),
        ],
        |ts| {
            let d = aom(ts.first()?)?;
            let from = d.value?;
            let to = num(ts.get(2)?)?;
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
            regex("-c"),
            pred(is_simple_amount_of_money),
            regex("(-н\\s+)?(хооронд|хүртэл)"),
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
        vec![pred(is_simple_amount_of_money), regex("-c\\s+(бага|доогуур|ихгүй)")],
        |ts| {
            let d = aom(ts.first()?)?;
            tok(with_max(d.value?, currency_only(d.currency)))
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "over/above/at least/more than <amount-of-money>",
        vec![pred(is_simple_amount_of_money), regex("-c\\s+(их|дээгүүр|илүү)")],
        |ts| {
            let d = aom(ts.first()?)?;
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
        rule_gbp(),
        rule_interval_between_numeral(),
        rule_interval_between_numeral2(),
        rule_interval_between(),
        rule_interval_max(),
        rule_interval_min(),
        rule_interval_numeral_dash(),
        rule_interval_dash(),
        rule_pounds(),
        rule_precision(),
        rule_tugriks(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
