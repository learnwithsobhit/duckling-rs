//! `Duckling.AmountOfMoney.NB.Rules`.

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

fn currency_rule(name: &str, re: &str, c: Currency) -> Rule {
    rule(name, vec![regex(re)], move |_| tok(currency_only(c)))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleUnitAmount
        rule("<unit> <amount>", vec![pred(is_currency_only), pred(is_positive)], |ts| {
            let c = aom(ts.first()?)?.currency;
            tok(with_value(num(ts.get(1)?)?, currency_only(c)))
        }),
        // ruleAboutAmountofmoney
        rule(
            "about <amount-of-money>",
            vec![regex("omtrent|cirka|rundt|ca"), pred(is_money_with_value)],
            |ts| Some((*ts.get(1)?).clone()),
        ),
        // ruleCent
        currency_rule(
            "cent",
            "cent(?:\\b|s)|\\bc\\b|pen(?:ce|ny|nies)|\\bp\\b|(?:ø|ö)rer?|fen|haleru|groszy|pais(?:e|a)|centesim(?:o|i)|centimes?|\\bct?\\b|\\brp?\\b|rap(?:pen)?s?|satang",
            Currency::Cent,
        ),
        // ruleDirham
        currency_rule("AED", "dirhams?", Currency::AED),
        // ruleIntersect
        rule("intersect", vec![pred(is_without_cents), pred(is_natural)], |ts| {
            tok(with_cents(num(ts.get(1)?)?, aom(ts.first()?)?.clone()))
        }),
        // ruleIntersectAndNumeral
        rule(
            "intersect (and number)",
            vec![pred(is_without_cents), regex("og"), pred(is_natural)],
            |ts| tok(with_cents(num(ts.get(2)?)?, aom(ts.first()?)?.clone())),
        ),
        // ruleIntersectAndXCents
        rule(
            "intersect (and X cents)",
            vec![pred(is_without_cents), regex("og"), pred(is_cents)],
            |ts| tok(with_cents(aom(ts.get(2)?)?.value?, aom(ts.first()?)?.clone())),
        ),
        // ruleIntersectXCents
        rule("intersect (X cents)", vec![pred(is_without_cents), pred(is_cents)], |ts| {
            tok(with_cents(aom(ts.get(1)?)?.value?, aom(ts.first()?)?.clone()))
        }),
        // ruleNok
        currency_rule("NOK", "(?:norske? ?)?kr(?:oner?)?", Currency::NOK),
        // rulePound
        currency_rule("£", "po?und?s?", Currency::Pound),
        // ruleSEK
        currency_rule("SEK", "(?:svenske? )?kron(?:a|or)|svenske? kr(?:oner?)?", Currency::SEK),
        // ruleDKK
        currency_rule("DKK", "danske? kr(?:oner?)?", Currency::DKK),
        // ruleUSD
        currency_rule("USD", "amerikanske? dollars?", Currency::USD),
        // ruleAUD
        currency_rule("AUD", "australske? dollars?", Currency::AUD),
        // ruleCAD
        currency_rule("CAD", "(?:k|c)anadiske? dollars?", Currency::CAD),
        // ruleGBP
        currency_rule("GBP", "(?:britiske?|engelske?) po?unds?", Currency::GBP),
        // ruleCHF
        currency_rule("CHF", "sveitsiske? francs?", Currency::CHF),
        // ruleCNY
        currency_rule("CNY", "(?:kinesiske? )?(?:yuan|renminbi)", Currency::CNY),
        // ruleCZK
        currency_rule("CZK", "(?:tsjekkiske? )?koruna", Currency::CZK),
        // ruleHKD
        currency_rule("HKD", "hong kong dollars?", Currency::HKD),
        // ruleINR
        currency_rule("INR", "(?:indiske? )?rup(?:i(?:er)?|ees?)", Currency::INR),
        // ruleJPY
        currency_rule("JPY", "japanske? yen", Currency::JPY),
        // ruleNZD
        currency_rule("NZD", "(?:nz|new zealand(?:ske)?) dollars?", Currency::NZD),
        // rulePKR
        currency_rule("PKR", "pakistanske? rup(?:i(?:er)?|ees?)", Currency::PKR),
        // rulePLN
        currency_rule("PLN", "(?:polske? )?(?:z|s)?lotys?", Currency::PLN),
        // ruleSGD
        currency_rule("SGD", "singapor(?:e|ske?) dollars?", Currency::SGD),
        // ruleTHB
        currency_rule("THB", "(?:thai(?:land(?:ske?)?)? )?b(?:ah|ha)t", Currency::THB),
        // ruleZAR
        currency_rule("ZAR", "(?:sør(?: |-)?afrika(?:nske?)? )?rand", Currency::ZAR),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
