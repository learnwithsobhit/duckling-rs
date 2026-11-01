//! `Duckling.AmountOfMoney.HE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::amount_of_money::*;
use crate::dims::numeral::is_positive;
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

fn rule_one_shekel() -> Rule {
    rule("שקל", vec![regex("שקל( אחד| בודד)?")], |_| {
        tok(with_value(1.0, currency_only(Currency::ILS)))
    })
}

fn rule_two_shekel() -> Rule {
    rule("שנקל", vec![regex("שנקל")], |_| tok(with_value(2.0, currency_only(Currency::ILS))))
}

fn rule_one_agura() -> Rule {
    rule("oneAgura", vec![regex("אגורה( אחת| בודדת)?")], |_| {
        tok(with_value(0.01, currency_only(Currency::ILS)))
    })
}

fn rule_shekel() -> Rule {
    rule("שקל", vec![regex("שקל(ים)?|ש״?ח")], |_| tok(currency_only(Currency::ILS)))
}

fn rule_thousand_shekel() -> Rule {
    rule("אש״ח", vec![pred(is_positive), regex("אש״?ח")], |ts| {
        let v = num(ts.first()?)?;
        tok(with_value(v * 1000.0, currency_only(Currency::ILS)))
    })
}

fn rule_million_shekel() -> Rule {
    rule("מש״ח", vec![pred(is_positive), regex("מש״?ח")], |ts| {
        let v = num(ts.first()?)?;
        tok(with_value(v * 1000000.0, currency_only(Currency::ILS)))
    })
}

fn rule_agura() -> Rule {
    rule("multiple agura", vec![pred(is_positive), regex("אגורות")], |ts| {
        let v = num(ts.first()?)?;
        tok(with_value(v * 0.01, currency_only(Currency::ILS)))
    })
}

fn rule_intersect() -> Rule {
    rule(
        "intersect",
        vec![pred(is_without_cents), regex("ו"), pred(is_positive)],
        |ts| {
            let d = aom(ts.first()?)?;
            let v = d.value?;
            let p = num(ts.get(2)?)?;
            tok(with_value(v + p, currency_only(d.currency)))
        },
    )
}

fn rule_intersect_and_x_cents() -> Rule {
    rule(
        "intersect (and X cents)",
        vec![pred(is_without_cents), regex("ו"), pred(is_cents)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = aom(ts.get(2)?)?.value?;
            tok(with_cents(c, fd.clone()))
        },
    )
}

fn rule_pounds() -> Rule {
    rule("£", vec![regex("פאונד(ים)?")], |_| tok(currency_only(Currency::Pound)))
}

fn rule_dollar() -> Rule {
    rule("$", vec![regex("דולר(ים)?")], |_| tok(currency_only(Currency::Dollar)))
}

fn rule_cent() -> Rule {
    rule("cent", vec![regex("סנט(ים)?")], |_| tok(currency_only(Currency::Cent)))
}

fn rule_eur() -> Rule {
    rule("€", vec![regex("אירו|יורו")], |_| tok(currency_only(Currency::EUR)))
}

fn rule_one_gbp() -> Rule {
    rule("OneGBP", vec![regex("לירה שטרלינג")], |_| {
        tok(with_value(1.0, currency_only(Currency::GBP)))
    })
}

fn rule_gbp() -> Rule {
    rule("OneGBP", vec![regex("לירות שטרלינג")], |_| tok(currency_only(Currency::GBP)))
}

fn rule_precision() -> Rule {
    rule(
        "about|exactly <amount-of-money>",
        vec![regex("בערך|בדיוק|קרוב ל"), pred(is_money_with_value)],
        |ts| Some((*ts.get(1)?).clone()),
    )
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> to|and <amount-of-money>",
        vec![
            // Upstream `מ?|בין ` also matches empty; keep the non-empty prefixes only.
            regex("מ|בין "),
            pred(is_positive),
            regex("עד |ל"),
            pred(is_simple_amount_of_money),
        ],
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

fn rule_interval_between() -> Rule {
    rule(
        "between|from <amount-of-money> to|and <amount-of-money>",
        vec![
            // Upstream `מ?|בין ` also matches empty; keep the non-empty prefixes only.
            regex("מ|בין "),
            pred(is_simple_amount_of_money),
            regex("עד |ל"),
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
        vec![regex("פחות מ|עד|לא יותר מ|מתחת ל?|לא מעל"), pred(is_simple_amount_of_money)],
        |ts| {
            let d = aom(ts.get(1)?)?;
            tok(with_max(d.value?, currency_only(d.currency)))
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "over/above/at least/more than <amount-of-money>",
        vec![regex("יותר מ|מעל|מ|לא פחות מ|לא מתחת ל"), pred(is_simple_amount_of_money)],
        |ts| {
            let d = aom(ts.get(1)?)?;
            tok(with_min(d.value?, currency_only(d.currency)))
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_unit_amount(),
        rule_cent(),
        rule_eur(),
        rule_intersect_and_x_cents(),
        rule_interval_between_numeral(),
        rule_interval_between(),
        rule_interval_max(),
        rule_interval_min(),
        rule_interval_numeral_dash(),
        rule_interval_dash(),
        rule_pounds(),
        rule_one_gbp(),
        rule_gbp(),
        rule_precision(),
        rule_intersect(),
        rule_agura(),
        rule_shekel(),
        rule_one_shekel(),
        rule_two_shekel(),
        rule_one_agura(),
        rule_thousand_shekel(),
        rule_million_shekel(),
        rule_dollar(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
