//! `Duckling.Time.GA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time::helpers::*;
use crate::dims::time::types::TimeData;
use crate::dims::time_grain::Grain as G;
use crate::dims::util::parse_int;
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

fn td(t: &Token) -> Option<&TimeData> {
    match t {
        Token::Time(d) => Some(d),
        _ => None,
    }
}

fn rx(t: &Token, i: usize) -> Option<String> {
    match t {
        Token::RegexMatch(g) => g.0.get(i).map(|s| s.to_string()),
        _ => None,
    }
}

fn ordinal_value(t: &Token) -> Option<i64> {
    match t {
        Token::Ordinal(o) => Some(o.value),
        _ => None,
    }
}

fn grain_of(t: &Token) -> Option<G> {
    match t {
        Token::TimeGrain(g) => Some(*g),
        _ => None,
    }
}

/// `Duckling.Duration.Helpers.isGrain` — matches a TimeGrain token.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Monday", "luai?n|lu\\.?" ),
        ("Tuesday", "mh?(á|a)irt|m(á|a)?\\.?" ),
        ("Wednesday", "ch?(é|e)adaoin|c(é|e)\\.?" ),
        ("Thursday", "d(é|e)ardaoin|d(é|e)?\\.?" ),
        ("Friday", "h?aoine|ao\\.?" ),
        ("Saturday", "sathai?rn|sa\\.?" ),
        ("Sunday", "domhnai?[cg]h|do\\.?" ),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("January", "(mh?(í|i) )?(an )?t?ean(á|a)ir|ean\\.?" ),
        ("February", "(mh?(í|i) )?(na )?feabhra|fea\\.?" ),
        ("March", "(mh?(í|i) )?(an )?mh?(á|a)rta|m(á|a)r\\.?" ),
        ("April", "(mh?(í|i) )?(an )?t?aibre(á|a)i?n|abr\\.?" ),
        ("May", "(mh?(í|i) )?(na )?bh?ealtaine|bea\\.?" ),
        ("June", "(mh?(í|i) )?(an )?mh?eith(ea|i)mh|mei\\.?" ),
        ("July", "(mh?(í|i) )?i(ú|u)il|i(ú|u)i\\.?" ),
        ("August", "(mh?(í|i) )?(na )?l(ú|u)nasa|l(ú|u)n\\.?" ),
        ("September", "(mh?(í|i) )?mh?e(á|a)n f(ó|o)mhair|mef?\\.?" ),
        ("October", "(mh?(í|i) )?(na )?nollai?g|nol\\.?" ),
        ("November", "(mh?(í|i) )?(na )?samh(ain|na)|sam\\.?" ),
        ("December", "(mh?(í|i) )?(na )?nollai?g|nol\\.?" ),
    ])
}

fn rules_main() -> Vec<Rule> {
    vec![
        // ruleAbsorptionOfAfterNamedDay
        rule("absorption of , after named day", vec![pred(is_a_day_of_week), regex(",")], |ts| {
            Some(ts[0].clone())
        }),
        // ruleAmrach
        rule("amárach", vec![regex("am(á|a)rach")], |_| tt(cycle_nth(G::Day, 1))),
        // ruleAnCycleDeTime
        rule(
            "an <cycle> de <time>",
            vec![regex("an"), dimension(Dim::TimeGrain), regex("d[e']"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, 0, td(ts[3])?)),
        ),
        // ruleAnCycleINdiaidhTime
        rule(
            "an <cycle> i ndiaidh <time>",
            vec![
                regex("the"),
                dimension(Dim::TimeGrain),
                regex("(i ndiaidh|tar (é|e)is)"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, 1, td(ts[3])?)),
        ),
        // ruleAnCycleSeo
        rule(
            "an <cycle> seo",
            vec![regex("an"), dimension(Dim::TimeGrain), regex("seo")],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
        ),
        // ruleAnDayofmonthNonOrdinal
        rule(
            "an <day-of-month> (non ordinal)",
            vec![regex("an|na"), pred(is_dom_integer)],
            |ts| {
                let v = get_int_value(ts[1])?;
                tt(mk_latent(day_of_month(v)))
            },
        ),
        // ruleAnDayofmonthOrdinal
        rule("an <day-of-month> (ordinal)", vec![regex("an|na"), pred(is_dom_ordinal)], |ts| {
            let n = get_int_value(ts[1])?;
            tt(day_of_month(n))
        }),
        // ruleAnNamedday
        rule("an named-day", vec![regex("an"), pred(is_a_day_of_week)], |ts| {
            Some(ts[1].clone())
        }),
        // ruleAnNollaig
        rule("An Nollaig", vec![regex("(l(á|a) |an )?(nollai?g)")], |_| tt(month_day(12, 25))),
        // ruleAnOrdinalCycleDeTime
        rule(
            "an <ordinal> <cycle> de <time>",
            vec![
                regex("an"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("d[e']"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[4])?)),
        ),
        // ruleAnOrdinalCycleINdiaidhTime
        rule(
            "an <ordinal> <cycle> i ndiaidh <time>",
            vec![
                regex("an"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("(i ndiaidh|tar (é|e)is)"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[4])?)),
        ),
        // ruleAnnaCycleRoimhTime
        rule(
            "(an|na) <cycle> roimh <time>",
            vec![regex("the"), dimension(Dim::TimeGrain), regex("roimh"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, -1, td(ts[3])?)),
        ),
        // ruleAnois
        rule("anois", vec![regex("anois|(ag an (t-?)?am seo)")], |_| tt(now())),
        // ruleArAmrach
        rule("arú amárach", vec![regex("ar(ú|u) am(á|a)rach")], |_| tt(cycle_nth(G::Day, 2))),
        // ruleArDate
        rule("ar <date>", vec![regex("ar"), dimension(Dim::Time)], |ts| Some(ts[1].clone())),
        // ruleArInn — negative cycleNth stays negative
        rule("arú inné", vec![regex("ar(ú|u) inn(é|e)")], |_| tt(cycle_nth(G::Day, -2))),
        // ruleCycleINdiaidhTime
        rule(
            "<cycle> i ndiaidh <time>",
            vec![dimension(Dim::TimeGrain), regex("(i ndiaidh|tar (é|e)is)"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, 1, td(ts[2])?)),
        ),
        // ruleCycleInniu
        rule(
            "<cycle> ó inniu",
            vec![
                pred(is_integer_between_tok(1, 9999)),
                dimension(Dim::TimeGrain),
                regex("(ó|o)(n l(á|a) (at(á|a) )?)?inniu"),
            ],
            |ts| {
                let v = get_int_value(ts[0])?;
                tt(cycle_n(true, grain_of(ts[1])?, v))
            },
        ),
        // ruleCycleRoimhTime
        rule(
            "<cycle> roimh <time>",
            vec![dimension(Dim::TimeGrain), regex("roimhe?"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, -1, td(ts[2])?)),
        ),
        // ruleCycleShin — upstream uses +1
        rule(
            "<cycle> ó shin",
            vec![dimension(Dim::TimeGrain), regex("(ó|o) shin")],
            |ts| tt(cycle_nth(grain_of(ts[0])?, 1)),
        ),
        // ruleDNamedday
        rule("dé named-day", vec![regex("d(é|e)"), pred(is_a_day_of_week)], |ts| {
            Some(ts[1].clone())
        }),
        // ruleDayofmonthOrdinal
        rule("<day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
            let n = get_int_value(ts[0])?;
            tt(mk_latent(day_of_month(n)))
        }),
        // ruleDayofmonthordinalNamedmonth
        rule(
            "<day-of-month>(ordinal) <named-month>",
            vec![pred(is_dom_ordinal), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
        ),
        // ruleDayofmonthordinalNamedmonthYear
        rule(
            "<day-of-month>(ordinal) <named-month> year",
            vec![pred(is_dom_ordinal), pred(is_a_month), regex("(\\d{2,4})")],
            |ts| {
                let int_val = parse_int(&rx(ts[2], 0)?)?;
                let dom = intersect_dom(td(ts[1])?, ts[0])?;
                ttd(intersect(&dom, &year(int_val)))
            },
        ),
        // ruleDdmm
        rule("dd/mm", vec![regex("(3[01]|[12]\\d|0?[1-9])/(0?[1-9]|1[0-2])")], |ts| {
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        }),
        // ruleDdmmyyyy
        rule(
            "dd/mm/yyyy",
            vec![regex("(3[01]|[12]\\d|0?[1-9])[-/](0?[1-9]|1[0-2])[/-](\\d{2,4})")],
            |ts| {
                let d = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let y = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        // ruleIGceannCycle — upstream prod matches Ordinal against a Numeral pattern
        rule(
            "i gceann <cycle>",
            vec![
                regex("(i|faoi) g?ch?eann"),
                pred(is_integer_between_tok(1, 9999)),
                dimension(Dim::TimeGrain),
            ],
            |ts| match (ts.get(1)?, ts.get(2)?) {
                (Token::Ordinal(od), Token::TimeGrain(g)) => tt(cycle_n(true, *g, od.value)),
                _ => None,
            },
        ),
        // ruleInn — negative cycleNth stays negative
        rule("inné", vec![regex("inn(é|e)")], |_| tt(cycle_nth(G::Day, -1))),
        // ruleInniu
        rule("inniu", vec![regex("inniu")], |_| tt(today())),
        // ruleIntersect
        rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        // ruleIntersectBy
        rule(
            "intersect by \",\"",
            vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleLFhileBrde
        rule(
            "Lá Fhéile Bríde",
            vec![regex("(l(á|a) )?(fh(e|é)ile|'?le) bh?r(í|i)de")],
            |_| tt(month_day(2, 1)),
        ),
        // ruleLFhilePdraig
        rule(
            "Lá Fhéile Pádraig",
            vec![regex("(l(á|a) )?(fh(e|é)ile|'?le) ph?(á|a)draig")],
            |_| tt(month_day(3, 17)),
        ),
        // ruleLFhileVailintn
        rule(
            "Lá Fhéile Vailintín",
            vec![regex("(l(á|a) )?(fh(e|é)ile|'?le) vailint(í|i)n")],
            |_| tt(month_day(2, 14)),
        ),
        // ruleLNaNaithreacha
        rule("Lá na nAithreacha", vec![regex("l(á|a) na naithreacha")], |_| {
            tt(nth_dow_of_month(2, 7, 6))
        }),
        // ruleNollaigNaMban
        rule(
            "Nollaig na mBan",
            vec![regex("(l(á|a) |an )?nollaig (bheag|na mban)")],
            |_| tt(month_day(1, 6)),
        ),
        // ruleOnANamedday
        rule("on a named-day", vec![regex("ar an"), pred(is_a_day_of_week)], |ts| {
            Some(ts[1].clone())
        }),
        // ruleOrdinalCycleDeTime
        rule(
            "<ordinal> <cycle> de <time>",
            vec![
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("d[e']"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?)),
        ),
        // ruleOrdinalCycleINdiaidhTime
        rule(
            "<ordinal> <cycle> i ndiaidh <time>",
            vec![
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("(i ndiaidh|tar (é|e)is)"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?)),
        ),
        // ruleOrdinalRithe
        rule("<ordinal> ráithe", vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))], |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth_after(true, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
        }),
        // ruleOrdinalRitheYear
        rule(
            "<ordinal> ráithe <year>",
            vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter)), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, G::Quarter, ordinal_value(ts[0])? - 1, td(ts[2])?)),
        ),
        // ruleTimeSeo
        rule("<time> seo", vec![dimension(Dim::Time), regex("seo")], |ts| {
            tt(pred_nth(0, false, td(ts[0])?))
        }),
        // ruleTimeSeoChaite
        rule("<time> seo chaite", vec![dimension(Dim::Time), regex("seo ch?aite")], |ts| {
            tt(pred_nth(-1, false, td(ts[0])?))
        }),
        // ruleTimeSeoChugainn
        rule(
            "<time> seo chugainn",
            vec![pred(is_not_latent), regex("seo (chugainn|at(a|á) ag teacht)")],
            |ts| tt(pred_nth(0, true, td(ts[0])?)),
        ),
        // ruleYear
        rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
            let v = get_int_value(ts[0])?;
            tt(year(v))
        }),
        // ruleYearLatent
        rule("year (latent)", vec![pred(is_integer_between_tok(1, 999))], |ts| {
            let n = get_int_value(ts[0])?;
            tt(mk_latent(year(n)))
        }),
        // ruleYearLatent2
        rule("year (latent)", vec![pred(is_integer_between_tok(2101, 10000))], |ts| {
            let n = get_int_value(ts[0])?;
            tt(mk_latent(year(n)))
        }),
        // ruleYyyymmdd
        rule(
            "yyyy-mm-dd",
            vec![regex("(\\d{2,4})-(0?[1-9]|1[0-2])-(3[01]|[12]\\d|0?[1-9])")],
            |ts| {
                let y = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let d = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
    ]
}

fn ttd(o: Option<TimeData>) -> Option<Token> {
    o.map(Token::Time)
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = rules_main();
    v.extend(rules_days_of_week());
    v.extend(rules_months());
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
