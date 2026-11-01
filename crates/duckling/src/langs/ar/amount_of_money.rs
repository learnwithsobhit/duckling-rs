//! `Duckling.AmountOfMoney.AR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::amount_of_money::*;
use crate::dims::numeral::{is_natural, is_positive};
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

fn rx(t: &Token, i: usize) -> Option<String> {
    match t {
        Token::RegexMatch(g) => g.0.get(i).map(|s| s.to_string()),
        _ => None,
    }
}

fn const_cur(name: &str, re: &str, c: Currency) -> Rule {
    rule(name, vec![regex(re)], move |_| tok(currency_only(c)))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
            let c = aom(ts.first()?)?.currency;
            let v = numeral_value(ts.get(1)?)?;
            tok(with_value(v, currency_only(c)))
        }),
        const_cur("cent", "سي?نت(ات)?", Currency::Cent),
        const_cur("£", "جنيه(ات)?", Currency::Pound),
        const_cur("EGP abbreviation", "ج[.]م[.]?", Currency::EGP),
        const_cur("dinars", "دينار|دنانير", Currency::Dinar),
        const_cur("Dirhams", "درا?هم", Currency::Dirham),
        const_cur("Riyals", "ريال(ات)?", Currency::Riyal),
        const_cur("Dollars", "دولار(ات)?", Currency::Dollar),
        const_cur("Shekel", "شي[كق]ل|شوا[كق]ل", Currency::ILS),
        const_cur("Euro", "[أاي]ورو", Currency::EUR),
        rule("other pounds", vec![regex("جنيه(ات)? (مصر|استرلين)ي?[ةه]?")], |ts| {
            match to_lower(&rx(ts.first()?, 1)?).as_str() {
                "مصر" => tok(currency_only(Currency::EGP)),
                "استرلين" => tok(currency_only(Currency::GBP)),
                _ => None,
            }
        }),
        rule("dinars", vec![regex("(دينار|دنانير) ([أا]ردن|كويت|عراق)ي?[ةه]?")], |ts| {
            match to_lower(&rx(ts.first()?, 1)?).as_str() {
                "كويت" => tok(currency_only(Currency::KWD)),
                "اردن" | "أردن" => tok(currency_only(Currency::JOD)),
                "عراق" => tok(currency_only(Currency::IQD)),
                _ => None,
            }
        }),
        rule("dirham", vec![regex("(درهم|دراهم) (اردن|أردن|مغرب)ي?[ةه]?")], |ts| {
            match to_lower(&rx(ts.first()?, 1)?).as_str() {
                "إمارات" | "امارات" => tok(currency_only(Currency::AED)),
                "مغرب" => tok(currency_only(Currency::MAD)),
                _ => None,
            }
        }),
        rule("riyals", vec![regex("(ريال|ريالات) (سعود|قطر)ي?[ةه]?")], |ts| {
            match to_lower(&rx(ts.first()?, 1)?).as_str() {
                "قطر" => tok(currency_only(Currency::QAR)),
                "سعود" => tok(currency_only(Currency::SAR)),
                _ => None,
            }
        }),
        const_cur("LBP", "لير([ةه]|ات) لبناني[ةه]?", Currency::LBP),
        rule("intersect", vec![pred(is_without_cents), pred(is_natural)], |ts| {
            let fd = aom(ts.first()?)?;
            let c = numeral_value(ts.get(1)?)?;
            tok(with_cents(c, fd.clone()))
        }),
        rule(
            "intersect (and number)",
            vec![pred(is_without_cents), regex("و"), pred(is_natural)],
            |ts| {
                let fd = aom(ts.first()?)?;
                let c = numeral_value(ts.get(2)?)?;
                tok(with_cents(c, fd.clone()))
            },
        ),
        rule(
            "intersect (and X cents)",
            vec![pred(is_without_cents), regex("و"), pred(is_cents)],
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
            vec![
                regex("(من|(ما )?بين)( ال)?"),
                pred(is_positive),
                regex("(الى|حتى|و|ل)( ا?ل)?"),
                pred(is_simple_amount_of_money),
            ],
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
                regex("(من|(ما )?بين)( ال)?"),
                pred(is_simple_amount_of_money),
                regex("(الى|حتى|و|ل)( ا?ل)?"),
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
            vec![regex("(حتى|[أا]قل من|تحت|(ما )?دون)|على ال[أا]كثر"), pred(is_simple_amount_of_money)],
            |ts| {
                let (to, c) = vc(ts.get(1)?)?;
                tok(with_max(to, currency_only(c)))
            },
        ),
        rule(
            "under/less/lower/no more than <amount-of-money>",
            vec![regex("على ال[أا]كثر"), pred(is_simple_amount_of_money)],
            |ts| {
                let (to, c) = vc(ts.get(1)?)?;
                tok(with_max(to, currency_only(c)))
            },
        ),
        rule(
            "at least/over/above/more than <amount-of-money>",
            vec![regex("فوق||[أا]كثر من|على ال[اأ]قل"), pred(is_simple_amount_of_money)],
            |ts| {
                let (to, c) = vc(ts.get(1)?)?;
                tok(with_min(to, currency_only(c)))
            },
        ),
        rule(
            "<amount-of-money> at least",
            vec![pred(is_simple_amount_of_money), regex("على ال[اأ]قل")],
            |ts| {
                let (to, c) = vc(ts.first()?)?;
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
        rule(
            "about|exactly <amount-of-money>",
            vec![regex("حوال[ي|ى]|تقريبا|بحدود"), pred(is_money_with_value)],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
