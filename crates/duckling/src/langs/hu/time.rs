//! `Duckling.Time.HU.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time::helpers::*;
use crate::dims::time::types::{TimeData, TimeIntervalType::Open};
use crate::dims::time_grain::Grain as G;
use crate::dims::util::{parse_int, to_lower};
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

fn ttd(o: Option<TimeData>) -> Option<Token> {
    o.map(Token::Time)
}

fn grain_of(t: &Token) -> Option<G> {
    match t {
        Token::TimeGrain(g) => Some(*g),
        _ => None,
    }
}

fn rule_intersect() -> Rule {
    rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
        ttd(intersect(td(ts[0])?, td(ts[1])?))
    })
}

fn rule_instants() -> Vec<Rule> {
    mk_rule_instants(&[
        ("right now", G::Second, 0, "mostani(t)?|((épp )?most)|azonnal"),
        ("today", G::Day, 0, "m(á|a)(i( nap(o(t|n)|it?)?|t)?|ma)?"),
        ("tomorrow", G::Day, 1, "holnap(i(t)?)?"),
        ("day after tomorrow", G::Day, 2, "holnapután(i(t)?)?"),
        ("yesterday", G::Day, -1, "tegnap(i(t)?)?"),
        ("day before yesterday", G::Day, -2, "tegnapelőtt(i(t)?)?"),
        ("end of month", G::Month, 1, "(a )?hónap vége|hó vég(e|i(t)?|én)"),
        ("end of year", G::Year, 1, "(az )?év vég(e|i(t)?|én)"),
    ])
}

fn rule_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Monday", "hétfő(n|t|i(t)?)?|hét\\.?" ),
        ("Tuesday", "kedd(en|et|i(t)?)?"),
        ("Wednesday", "szerda(i(t)?)?|szerdá(n|t)|szer\\.?" ),
        ("Thursday", "csütörtök(ö(n|t)|i(t)?)?|csüt\\.?" ),
        ("Friday", "péntek(e(n|t)|i(t)?)?|pén\\.?" ),
        ("Saturday", "szombat(o(n|t)|i(t)?)?|szom\\.?" ),
        ("Sunday", "vasárnap(ot|i(t)?)?|vas\\.?" ),
    ])
}

fn rule_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("January", "január(ban|i(t)?)?|jan\\.?" ),
        ("February", "február(ban|i(t)?)?|febr?\\.?" ),
        ("March", "március(ban|i(t)?)?|márc?\\.?" ),
        ("April", "április(ban|i(t)?)?|ápr\\.?" ),
        ("May", "május(ban|i(t)?)?|máj\\.?" ),
        ("June", "június(ban|i(t)?)?|jún\\.?" ),
        ("July", "július(ban|i(t)?)?|júl\\.?" ),
        ("August", "augusztus(ban|i(t)?)?|aug\\.?" ),
        ("September", "szeptember(ben|i(t)?)?|szept?\\.?" ),
        ("October", "október(ben|i(t)?)?|okt\\.?" ),
        ("November", "november(ben|i(t)?)?|nov\\.?" ),
        ("December", "december(ben|i(t)?)?|dec\\.?" ),
    ])
}

fn rule_month_dom_numeral() -> Rule {
    rule(
        "<named-month> <day-of-month> (non ordinal)",
        vec![pred(is_a_month), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_month_dom_numeral_suffix() -> Rule {
    rule(
        "<named-month> <day-of-month> (non ordinal)",
        vec![
            pred(is_a_month),
            pred(is_dom_integer),
            regex("(-?(jei|jén|án|én|ai|ei|je|ji|a|e|i))"),
        ],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_month_dom_ordinal() -> Rule {
    rule(
        "<named-month> <day-of-month> (ordinal)",
        vec![pred(is_a_month), pred(is_dom_ordinal)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_cycle_this_last_next() -> Rule {
    rule(
        "this|last|next <cycle>",
        vec![regex("(most|előző|múlt|következő|jövő)"), dimension(Dim::TimeGrain)],
        |ts| {
            let grain = grain_of(ts[1])?;
            match to_lower(&rx(ts[0], 0)?).as_str() {
                "most" => tt(cycle_nth(grain, 0)),
                "előző" | "múlt" => tt(cycle_nth(grain, -1)),
                "következő" | "jövő" => tt(cycle_nth(grain, 1)),
                _ => None,
            }
        },
    )
}

fn rule_next_dow() -> Rule {
    rule("next <day-of-week>", vec![regex("jövő"), pred(is_a_day_of_week)], |ts| {
        tt(pred_nth(1, true, td(ts[1])?))
    })
}

fn rule_hhmm() -> Rule {
    rule("hh:mm", vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)")], |ts| {
        let h = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        tt(hour_minute(true, h, m))
    })
}

fn rule_hhmmss() -> Rule {
    rule(
        "hh:mm:ss",
        vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)[:.]([0-5]\\d)")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let s = parse_int(&rx(ts[0], 2)?)?;
            tt(hour_minute_second(true, h, m, s))
        },
    )
}

fn rule_tod_latent() -> Rule {
    rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
        tt(mk_latent(hour(true, get_int_value(ts[0])?)))
    })
}

fn rule_tod_am() -> Rule {
    rule(
        "am <time-of-day>",
        vec![regex("(de\\.?|délelőtt)"), pred(is_a_time_of_day)],
        |ts| tt(time_of_day_ampm(true, td(ts[1])?)),
    )
}

fn rule_tod_pm() -> Rule {
    rule(
        "pm <time-of-day>",
        vec![regex("(du\\.?|délután)"), pred(is_a_time_of_day)],
        |ts| tt(time_of_day_ampm(false, td(ts[1])?)),
    )
}

fn rule_yyyymmdd() -> Rule {
    rule(
        "yyyy.mm.dd",
        vec![regex("(\\d{2,4})\\s?[-\\.]\\s?(0?[1-9]|1[0-2])\\s?[-\\.]\\s?(3[01]|[12]\\d|0?[1-9])")],
        |ts| {
            let y = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let d = parse_int(&rx(ts[0], 2)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn rule_mmdd() -> Rule {
    rule("mm.dd", vec![regex("(0?[1-9]|1[0-2])\\s?[-\\.]\\s?(3[01]|[12]\\d|0?[1-9])")], |ts| {
        let m = parse_int(&rx(ts[0], 0)?)?;
        let d = parse_int(&rx(ts[0], 1)?)?;
        tt(month_day(m, d))
    })
}

fn pod_bounds(match_: &str) -> (TimeData, TimeData) {
    match match_ {
        "reggel" | "reggeli" | "reggelit" => (hour(false, 6), hour(false, 10)),
        "délelőtt" | "délelőtti" | "délelőttit" => (hour(false, 8), hour(false, 12)),
        "délben" | "déli" | "délit" => (hour(false, 12), hour(false, 13)),
        "délután" | "délutáni" | "délutánit" => (hour(false, 12), hour(false, 18)),
        "este" | "esti" | "estit" => (hour(false, 16), hour(false, 20)),
        _ => (hour(false, 20), hour(false, 23)),
    }
}

fn rule_part_of_days() -> Rule {
    rule(
        "part of days",
        vec![regex(
            "(reggel(i(t)?)?|délelőtt(i(t)?)?|délben|déli(t)?|délután(i(t)?)?|est(e|i(t)?)|éjszaka(i(t)?)?)",
        )],
        |ts| {
            let (start, end) = pod_bounds(&to_lower(&rx(ts[0], 0)?));
            tt(part_of_day(interval(Open, &start, &end)?))
        },
    )
}

fn rule_time_pod() -> Rule {
    rule(
        "<time> <part-of-day>",
        vec![dimension(Dim::Time), pred(is_a_part_of_day)],
        |ts| ttd(intersect(td(ts[1])?, td(ts[0])?)),
    )
}

fn season_bounds(match_: &str) -> Option<(TimeData, TimeData)> {
    match match_ {
        "nyár" | "nyári" | "nyárit" | "nyáron" => Some((month_day(6, 21), month_day(9, 23))),
        "ősz" | "őszi" | "őszit" | "ősszel" => Some((month_day(9, 23), month_day(12, 21))),
        "tél" | "téli" | "télit" | "télen" => Some((month_day(12, 21), month_day(3, 20))),
        "tavasz" | "tavaszi" | "tavaszit" | "tavasszal" => Some((month_day(3, 20), month_day(6, 21))),
        _ => None,
    }
}

fn rule_seasons() -> Rule {
    rule(
        "seasons",
        vec![regex("(nyár(i(t)?|on)?|ősz(i(t)?)?|ősszel|tél(i(t)?|en)?|tavasz(i(t)?)?|tavasszal)")],
        |ts| {
            let (start, end) = season_bounds(&to_lower(&rx(ts[0], 0)?))?;
            ttd(interval(Open, &start, &end))
        },
    )
}

fn build_rules() -> Vec<Rule> {
    let mut v = vec![
        rule_intersect(),
        rule_month_dom_numeral(),
        rule_month_dom_numeral_suffix(),
        rule_month_dom_ordinal(),
        rule_cycle_this_last_next(),
        rule_next_dow(),
        rule_hhmm(),
        rule_hhmmss(),
        rule_tod_latent(),
        rule_tod_am(),
        rule_tod_pm(),
        rule_yyyymmdd(),
        rule_mmdd(),
        rule_part_of_days(),
        rule_time_pod(),
        rule_seasons(),
    ];
    v.extend(rule_instants());
    v.extend(rule_days_of_week());
    v.extend(rule_months());
    v
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &RULES
}
