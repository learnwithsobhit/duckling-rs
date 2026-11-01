//! `Duckling.Distance.ZH.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance::{
    distance, is_simple_distance, unit_only, with_interval, with_max, with_min, with_unit, DistanceData,
    Unit,
};
use crate::dims::numeral::is_positive;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: DistanceData) -> Option<Token> {
    Some(Token::Distance(d))
}

fn unit_rule(name: &str, re: &str, unit: Unit) -> Rule {
    rule(name, vec![dimension(Dim::Distance), regex(re)], move |ts| match ts.first()? {
        Token::Distance(dd) => tok(with_unit(unit, dd.clone())),
        _ => None,
    })
}

fn rule_dist_feet_and_dist_inch() -> Rule {
    rule(
        "<dist> feet and <dist> inch ",
        vec![
            dimension(Dim::Distance),
            regex("'|f(oo|ee)?ts?|英尺|呎"),
            dimension(Dim::Distance),
            regex("''|\"|inch(es)?|英寸|英吋|吋"),
        ],
        |ts| match ts.first()? {
            Token::Distance(dd) => tok(with_unit(Unit::Foot, dd.clone())),
            _ => None,
        },
    )
}

fn rule_dist_one_meter_and() -> Rule {
    rule("one meter and <dist>", vec![regex("米"), pred(is_positive)], |ts| match ts.get(1)? {
        Token::Numeral(n) => tok(with_unit(Unit::Metre, distance(1.0 + n.value / 10.0))),
        _ => None,
    })
}

fn rule_dist_meters_and() -> Rule {
    rule(
        "<dist> meters and <dist>",
        vec![pred(is_positive), regex("米"), pred(is_positive)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(n1), Token::Numeral(n2)) => {
                tok(with_unit(Unit::Metre, distance(n1.value + n2.value / 10.0)))
            }
            _ => None,
        },
    )
}

fn rule_interval_numeral_dash() -> Rule {
    rule(
        "<numeral> - <dist>",
        vec![pred(is_positive), regex("-|~|到|至"), pred(is_simple_distance)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (
                Token::Numeral(n),
                Token::Distance(DistanceData { value: Some(to), unit: Some(u), .. }),
            ) if n.value < *to => tok(with_interval(n.value, *to, unit_only(*u))),
            _ => None,
        },
    )
}

fn rule_interval_dash() -> Rule {
    rule(
        "<dist> - <dist>",
        vec![pred(is_simple_distance), regex("-|~|到|至"), pred(is_simple_distance)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (
                Token::Distance(DistanceData { value: Some(from), unit: Some(u1), .. }),
                Token::Distance(DistanceData { value: Some(to), unit: Some(u2), .. }),
            ) if *from < *to && u1 == u2 => tok(with_interval(*from, *to, unit_only(*u1))),
            _ => None,
        },
    )
}

fn rule_interval_bound() -> Rule {
    rule(
        "under/less/lower/no more than <amount-of-money> (最多|至少|最少)",
        vec![regex("(最多|至少|最少|起碼)"), pred(is_simple_distance)],
        |ts| {
            let m = groups(ts)?.first()?.to_string();
            match ts.get(1)? {
                Token::Distance(DistanceData { value: Some(to), unit: Some(u), .. }) => match m.as_str()
                {
                    "最多" => tok(with_max(*to, unit_only(*u))),
                    "最少" | "至少" | "起碼" => tok(with_min(*to, unit_only(*u))),
                    _ => None,
                },
                _ => None,
            }
        },
    )
}

fn rule_interval_bound2() -> Rule {
    rule(
        "under/less/lower/no more than <amount-of-money> (以下|以上)",
        vec![pred(is_simple_distance), regex("(以下|以上)")],
        |ts| {
            let m = groups(&ts[1..])?.first()?.to_string();
            match ts.first()? {
                Token::Distance(DistanceData { value: Some(to), unit: Some(u), .. }) => match m.as_str()
                {
                    "以下" => tok(with_max(*to, unit_only(*u))),
                    "以上" => tok(with_min(*to, unit_only(*u))),
                    _ => None,
                },
                _ => None,
            }
        },
    )
}

fn rule_precision() -> Rule {
    rule("about <distance>", vec![dimension(Dim::Distance), regex("左右")], |ts| {
        ts.first().map(|t| (*t).clone())
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        unit_rule("<dist> centimeters", "cm|厘米|公分", Unit::Centimetre),
        unit_rule("<dist> feet", "'|f(oo|ee)?ts?|英尺|呎", Unit::Foot),
        rule_dist_feet_and_dist_inch(),
        unit_rule("<dist> inch", "''|\"|inch(es)?|英寸|英吋|吋", Unit::Inch),
        unit_rule("<dist> km", "km|千米|公(里|裏)", Unit::Kilometre),
        unit_rule("<dist> meters", "m|米|公尺", Unit::Metre),
        unit_rule("<dist> miles", "miles?|英(里|裏)", Unit::Mile),
        rule_dist_one_meter_and(),
        rule_dist_meters_and(),
        rule_dist_one_meter_and(),
        rule_dist_meters_and(),
        rule_interval_numeral_dash(),
        rule_interval_dash(),
        rule_interval_bound(),
        rule_interval_bound2(),
        rule_precision(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
