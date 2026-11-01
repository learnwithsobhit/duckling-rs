//! `Duckling.AmountOfMoney.RO.Rules`.

use once_cell::sync::Lazy;

use crate::dims::amount_of_money::*;
use crate::dims::numeral::{is_natural, is_positive, number_with};
use crate::dims::util::to_lower;
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

fn const_currency(name: &str, re: &str, c: Currency) -> Rule {
    rule(name, vec![regex(re)], move |_| tok(currency_only(c)))
}

fn rule_unit_amount() -> Rule {
    rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
        let c = aom(ts.first()?)?.currency;
        let v = numeral_value(ts.get(1)?)?;
        tok(with_value(v, currency_only(c)))
    })
}

fn rule_amount_de_unit() -> Rule {
    rule(
        "<amount >= 20> de <unit>",
        vec![number_with(|n| n.value, |v| v >= 20.0), regex("de"), pred(is_currency_only)],
        |ts| {
            let v = numeral_value(ts.first()?)?;
            let c = aom(ts.get(2)?)?.currency;
            tok(with_value(v, currency_only(c)))
        },
    )
}

fn rule_intersect_and_numeral() -> Rule {
    rule(
        "intersect (and number)",
        vec![pred(is_without_cents), regex("(s|ș)i"), pred(is_natural)],
        |ts| {
            let fd = aom(ts.first()?)?;
            let c = numeral_value(ts.get(2)?)?;
            tok(with_cents(c, fd.clone()))
        },
    )
}

fn rule_riyals() -> Rule {
    rule("riyals", vec![regex("rial (saudit?|qatarian?)")], |ts| {
        match to_lower(match ts.first()? {
            Token::RegexMatch(g) => g.0.first()?,
            _ => return None,
        })
        .as_str()
        {
            "saudi" | "saudit" => tok(currency_only(Currency::SAR)),
            "qataria" | "qatarian" => tok(currency_only(Currency::QAR)),
            _ => None,
        }
    })
}

fn rule_precision_amount_of_money() -> Rule {
    rule(
        "about/exactly <amount-of-money>",
        vec![
            regex("exact|cam|aprox(\\.|imativ)?|(aproape|(i|î)n jur)( de)?"),
            pred(is_money_with_value),
        ],
        |ts| ts.get(1).map(|t| (*t).clone()),
    )
}

fn rule_intersect_and_x_cents() -> Rule {
    rule(
        "intersect (and X cents)",
        vec![pred(is_without_cents), regex("(s|ș)i"), pred(is_cents)],
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

fn rule_other_pounds() -> Rule {
    rule("other pounds", vec![regex("lir(a|ă) (egiptian|libanez)(a|ă)")], |ts| {
        let m = match ts.first()? {
            Token::RegexMatch(g) => to_lower(g.0.get(1)?),
            _ => return None,
        };
        match m.as_str() {
            "egiptian" => tok(currency_only(Currency::EGP)),
            "libanez" => tok(currency_only(Currency::LBP)),
            _ => None,
        }
    })
}

fn rule_intersect() -> Rule {
    rule("intersect", vec![pred(is_without_cents), pred(is_natural)], |ts| {
        let fd = aom(ts.first()?)?;
        let c = numeral_value(ts.get(1)?)?;
        tok(with_cents(c, fd.clone()))
    })
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> to|and <amount-of-money>",
        vec![regex("intre|de la"), pred(is_positive), regex("[sș]i|la"), pred(is_simple_amount_of_money)],
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
        "between|from <amount-of-money> to|and <amount-of-money>",
        vec![
            regex("intre|de la"),
            pred(is_simple_amount_of_money),
            regex("[sș]i|la"),
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
        vec![regex("sub|mai (pu[tț]|ieft)in de|nu chiar|nici macar|cel mult"), pred(is_simple_amount_of_money)],
        |ts| {
            let (to, c) = vc(ts.get(1)?)?;
            tok(with_max(to, currency_only(c)))
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "over/above/at least/more than <amount-of-money>",
        vec![regex("peste|mai (mult|scump) de|cel pu[tț]in"), pred(is_simple_amount_of_money)],
        |ts| {
            let (to, c) = vc(ts.get(1)?)?;
            tok(with_min(to, currency_only(c)))
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_unit_amount(),
        rule_amount_de_unit(),
        const_currency("AED", "dirhami?", Currency::AED),
        const_currency("cent|bani", "bani?|cen(t|ț)i?|c|¢", Currency::Cent),
        const_currency("$", "dolari?", Currency::Dollar),
        const_currency("INR", "rupii?", Currency::INR),
        rule_intersect(),
        rule_intersect_and_numeral(),
        rule_intersect_and_x_cents(),
        rule_intersect_x_cents(),
        const_currency("KWD", "dinar kuweitian", Currency::KWD),
        rule_other_pounds(),
        const_currency("£", "lire?", Currency::Pound),
        rule_precision_amount_of_money(),
        rule_riyals(),
        const_currency("RON", "roni|lei", Currency::RON),
        rule_interval_between(),
        rule_interval_between_numeral(),
        rule_interval_dash(),
        rule_interval_max(),
        rule_interval_min(),
        rule_interval_numeral_dash(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
