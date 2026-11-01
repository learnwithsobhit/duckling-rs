//! `Duckling.Time.EN.Rules` (generic English time rules).

use once_cell::sync::Lazy;

use super::helpers::*;
use super::types::{
    Form, IntervalDirection, TimeData, TimeIntervalType::{Closed, Open},
};
use crate::dims::duration::{duration, DurationData};
use crate::dims::numeral::{is_natural, NumeralData};
use crate::dims::time_grain::Grain as G;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

/// The `TimeData` of a `Token Time`.
fn td(t: &Token) -> Option<&TimeData> {
    match t {
        Token::Time(d) => Some(d),
        _ => None,
    }
}

/// Group `i` of a regex-match token.
fn rx(t: &Token, i: usize) -> Option<String> {
    match t {
        Token::RegexMatch(g) => g.0.get(i).map(|s| s.to_string()),
        _ => None,
    }
}

fn ttd(o: Option<TimeData>) -> Option<Token> {
    o.map(Token::Time)
}

fn numeral(t: &Token) -> Option<&NumeralData> {
    match t {
        Token::Numeral(n) => Some(n),
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

fn dur(t: &Token) -> Option<&DurationData> {
    match t {
        Token::Duration(d) => Some(d),
        _ => None,
    }
}

fn floor_i(v: f64) -> i64 {
    v.floor() as i64
}

fn either(
    a: impl Fn(&Token) -> bool + Send + Sync + 'static,
    b: impl Fn(&Token) -> bool + Send + Sync + 'static,
) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| a(t) || b(t)
}

fn rule_intersect() -> Rule {
    rule(
        "intersect",
        vec![
            pred(is_grain_finer_than(G::Year)),
            pred(either(is_not_latent, is_grain_of_time(G::Year))),
        ],
        |ts| {
            let (a, b) = (td(ts[0])?, td(ts[1])?);
            if !a.latent || !b.latent {
                ttd(intersect(a, b).map(not_latent))
            } else {
                None
            }
        },
    )
}

fn rule_intersect_of() -> Rule {
    rule(
        "intersect by \",\", \"of\", \"from\", \"'s\"",
        vec![pred(is_not_latent), regex("of|from|for|'s|,"), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?).map(not_latent)),
    )
}

fn rule_intersect_year() -> Rule {
    rule(
        "intersect by \",\", \"of\", \"from\" for year",
        vec![pred(is_not_latent), regex("of|from|,"), pred(is_grain_of_time(G::Year))],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?).map(not_latent)),
    )
}

fn rule_absorb_on_day() -> Rule {
    rule("on <day>", vec![regex("on"), pred(is_grain_of_time(G::Day))], |ts| Some(ts[1].clone()))
}

fn rule_absorb_on_adow() -> Rule {
    rule("on a <named-day>", vec![regex("on a"), pred(is_a_day_of_week)], |ts| Some(ts[1].clone()))
}

fn rule_absorb_in_month_year() -> Rule {
    rule(
        "in|during <named-month>|year",
        vec![regex("in|during"), pred(either(is_a_month, is_grain_of_time(G::Year)))],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    )
}

fn rule_absorb_comma_tod() -> Rule {
    rule("absorption of , after named day", vec![pred(is_a_day_of_week), regex(",")], |ts| Some(ts[0].clone()))
}

fn rules_instants() -> Vec<Rule> {
    mk_rule_instants(&[
        ("right now", G::Second, 0, "((just|right)\\s*)now|immediately"),
        ("today", G::Day, 0, "todays?|(at this time)"),
        ("tomorrow", G::Day, 1, "(tmrw?|tomm?or?rows?)"),
        ("yesterday", G::Day, -1, "yesterdays?"),
    ])
}

fn rule_now() -> Rule {
    rule("now", vec![regex("now|at\\sthe\\smoment|atm")], |_| tt(now()))
}

fn rule_asap() -> Rule {
    rule("as soon as possible", vec![regex("asap|as\\ssoon\\sas\\spossible")], |_| {
        tt(with_direction(IntervalDirection::After, now()))
    })
}

fn rule_next_dow() -> Rule {
    rule("this|next <day-of-week>", vec![regex("(this|next)"), pred(is_a_day_of_week)], |ts| {
        let m = to_lower(&rx(ts[0], 0)?);
        let dow = td(ts[1])?;
        let t = match m.as_str() {
            "this" => Some(pred_nth(0, true, dow)),
            "next" => intersect(dow, &cycle_nth(G::Week, 1)),
            _ => None,
        };
        ttd(t)
    })
}

fn rule_this_time() -> Rule {
    rule("this <time>", vec![regex("this|current|coming"), pred(is_ok_with_this_next)], |ts| {
        tt(pred_nth(0, false, td(ts[1])?))
    })
}

fn rule_next_time() -> Rule {
    rule("next <time>", vec![regex("next"), pred(is_ok_with_this_next)], |ts| tt(pred_nth(0, true, td(ts[1])?)))
}

fn rule_last_time() -> Rule {
    rule("last <time>", vec![regex("(this past|last|previous)"), pred(is_ok_with_this_next)], |ts| {
        tt(pred_nth(-1, false, td(ts[1])?))
    })
}

fn rule_last_weekend_of_month() -> Rule {
    rule(
        "last weekend of <named-month>",
        vec![regex("last\\s(week(\\s|-)?end|wkend)\\s(of|in)"), pred(is_a_month)],
        |ts| tt(pred_last_of(&weekend(), td(ts[1])?)),
    )
}

fn rule_time_before_last_after_next() -> Rule {
    rule(
        "<time> before last|after next",
        vec![dimension(Dim::Time), regex("(before last|after next)")],
        |ts| {
            let m = to_lower(&rx(ts[1], 0)?);
            tt(pred_nth(1, m == "after next", td(ts[0])?))
        },
    )
}

fn rule_ordinal_dow_of_time() -> Rule {
    rule(
        "first|second|third|fourth|fifth <day-of-week> of <time>",
        vec![pred(is_ordinal_between(1, 5)), pred(is_a_day_of_week), regex("(of|in)"), dimension(Dim::Time)],
        |ts| {
            let ord = get_int_value(ts[0])?;
            tt(pred_nth_after(ord - 1, td(ts[1])?, td(ts[3])?))
        },
    )
}

fn rule_last_dow_of_time() -> Rule {
    rule(
        "last <day-of-week> of <time>",
        vec![regex("last"), pred(is_a_day_of_week), regex("(of|in)"), dimension(Dim::Time)],
        |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_last_cycle_of_time() -> Rule {
    rule(
        "last <cycle> of <time>",
        vec![regex("last"), dimension(Dim::TimeGrain), regex("of|in"), dimension(Dim::Time)],
        |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
    )
}

fn rule_last_night() -> Rule {
    rule("last night", vec![regex("(late )?last night")], |ts| {
        let m = to_lower(&rx(ts[0], 0)?);
        let hours = if m == "late " { 3 } else { 6 };
        let start = duration_before(&DurationData { value: hours, grain: G::Hour }, &today());
        ttd(interval(Open, &start, &today()).map(|t| part_of_day(not_latent(t))))
    })
}

fn rule_nth_time_of_time() -> Rule {
    rule(
        "nth <day-of-week> of <month-or-greater>",
        vec![
            dimension(Dim::Ordinal),
            pred(is_a_day_of_week),
            regex("of|in"),
            pred(|t| !is_grain_finer_than(G::Month)(t)),
        ],
        |ts| {
            let od = ordinal_value(ts[0])?;
            ttd(intersect(td(ts[3])?, td(ts[1])?).map(|t| pred_nth(od - 1, false, &t)))
        },
    )
}

fn rule_the_nth_time_of_time() -> Rule {
    rule(
        "the nth <day-of-week> of <month-or-greater>",
        vec![
            regex("the"),
            dimension(Dim::Ordinal),
            pred(is_a_day_of_week),
            regex("of|in"),
            pred(|t| !is_grain_finer_than(G::Month)(t)),
        ],
        |ts| {
            let od = ordinal_value(ts[1])?;
            ttd(intersect(td(ts[4])?, td(ts[2])?).map(|t| pred_nth(od - 1, false, &t)))
        },
    )
}

fn rule_nth_time_after_time() -> Rule {
    rule(
        "nth <time> after <time>",
        vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("after"), dimension(Dim::Time)],
        |ts| tt(pred_nth_after(ordinal_value(ts[0])? - 1, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_the_nth_time_after_time() -> Rule {
    rule(
        "the nth <time> after <time>",
        vec![regex("the"), dimension(Dim::Ordinal), dimension(Dim::Time), regex("after"), dimension(Dim::Time)],
        |ts| tt(pred_nth_after(ordinal_value(ts[1])? - 1, td(ts[2])?, td(ts[4])?)),
    )
}

fn rule_ndow_from_time() -> Rule {
    rule(
        "<integer> <day-of-week> from <time>",
        vec![dimension(Dim::Numeral), pred(is_a_day_of_week), regex("from"), dimension(Dim::Time)],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(pred_nth_after(n - 1, td(ts[1])?, td(ts[3])?))
        },
    )
}

fn rule_year_latent() -> Rule {
    rule("year (latent)", vec![pred(is_integer_between_tok(25, 10000))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(mk_latent(year(n)))
    })
}

fn rule_year_adbc() -> Rule {
    rule(
        "<year> (bc|ad)",
        vec![pred(is_integer_between_tok(1, 10000)), regex("(a\\.?d\\.?|b\\.?c\\.?)")],
        |ts| {
            let y = get_int_value(ts[0])?;
            let ab = to_lower(&rx(ts[1], 0)?);
            tt(year_adbc(if ab.starts_with('b') { -y } else { y }))
        },
    )
}

fn rule_dom_latent() -> Rule {
    rule("<day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
        let n = get_int_value(ts[0])?;
        tt(mk_latent(day_of_month(n)))
    })
}

fn rule_the_dom_numeral() -> Rule {
    rule("the <day-of-month> (number)", vec![regex("the"), pred(is_dom_integer)], |ts| {
        let n = get_int_value(ts[1])?;
        tt(mk_latent(day_of_month(n)))
    })
}

fn rule_the_dom_ordinal() -> Rule {
    rule("the <day-of-month> (ordinal)", vec![regex("the"), pred(is_dom_ordinal)], |ts| {
        tt(day_of_month(ordinal_value(ts[1])?))
    })
}

fn rule_named_dom_ordinal() -> Rule {
    rule(
        "<named-month>|<named-day> <day-of-month> (ordinal)",
        vec![pred(either(is_a_month, is_a_day_of_week)), pred(is_dom_ordinal)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_month_dom_numeral() -> Rule {
    rule(
        "<named-month> <day-of-month> (non ordinal)",
        vec![pred(is_a_month), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_the_dom_of_month() -> Rule {
    rule(
        "the <day-of-month> (ordinal or number) of <named-month>",
        vec![regex("the"), pred(is_dom_value), regex("of|in"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[3])?, ts[1])),
    )
}

fn rule_dom_of_month() -> Rule {
    rule(
        "<day-of-month> (ordinal or number) of <named-month>",
        vec![pred(is_dom_value), regex("of|in"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
    )
}

fn rule_dom_month() -> Rule {
    rule(
        "<day-of-month> (ordinal or number) <named-month>",
        vec![pred(is_dom_value), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
    )
}

fn rule_dom_month_year() -> Rule {
    rule(
        "<day-of-month>(ordinal or number)/<named-month>/year",
        vec![pred(is_dom_value), regex("[-/\\s]"), pred(is_a_month), regex("[-/\\s]"), regex("(\\d{2,4})")],
        |ts| {
            let int_val = parse_int(&rx(ts[4], 0)?)?;
            let dom = intersect_dom(td(ts[2])?, ts[0])?;
            ttd(intersect(&dom, &year(int_val)))
        },
    )
}

fn rule_dom_ordinal_month_year() -> Rule {
    rule(
        "<day-of-month>(ordinal) <named-month> year",
        vec![pred(is_dom_ordinal), pred(is_a_month), regex("(\\d{2,4})")],
        |ts| {
            let int_val = parse_int(&rx(ts[2], 0)?)?;
            let dom = intersect_dom(td(ts[1])?, ts[0])?;
            ttd(intersect(&dom, &year(int_val)))
        },
    )
}

fn rule_ides_of_month() -> Rule {
    rule("the ides of <named-month>", vec![regex("the ides? of"), pred(is_a_month)], |ts| {
        let t = td(ts[1])?;
        match t.form {
            Some(Form::Month(m)) => {
                ttd(intersect(t, &day_of_month(if [3, 5, 7, 10].contains(&m) { 15 } else { 13 })))
            }
            _ => None,
        }
    })
}

fn rule_tod_latent() -> Rule {
    rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(mk_latent(hour(n < 13, n)))
    })
}

fn rule_at_tod() -> Rule {
    rule("at <time-of-day>", vec![regex("at|@"), pred(is_a_time_of_day)], |ts| tt(not_latent(td(ts[1])?.clone())))
}

fn rule_tod_oclock() -> Rule {
    rule("<time-of-day> o'clock", vec![pred(is_a_time_of_day), regex("o.?clock")], |ts| {
        tt(not_latent(td(ts[0])?.clone()))
    })
}

/// Start hour of an interval predicate whose first end is an hour-only `TimeDatePredicate`.
fn interval_start_hour(p: &super::types::Predicate) -> Option<i64> {
    use super::types::Predicate as P;
    match p {
        P::TimeIntervals(_, a, _) => match &**a {
            P::TimeDate(d) => match d.hour {
                Some((false, start)) => Some(start),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn rule_pod_at_tod() -> Rule {
    use super::types::Predicate as P;
    rule(
        "<part-of-day> at <time-of-day>",
        vec![pred(is_a_part_of_day), regex("at|@"), pred(is_a_time_of_day)],
        |ts| {
            let pod = td(ts[0])?;
            let tod = td(ts[2])?;
            let start = match &pod.time_pred {
                P::Intersect(a, _) => interval_start_hour(a),
                p @ P::TimeIntervals(..) => interval_start_hour(p),
                _ => None,
            }?;
            match tod.form {
                Some(Form::TimeOfDay { hours: Some(hours), is12h: true }) => {
                    tt(time_of_day_ampm(start < 12 || hours == 12, tod))
                }
                _ => None,
            }
        },
    )
}

fn rule_pod_intersect_tod_latent() -> Rule {
    rule(
        "<part-of-day> <latent-time-of-day> (latent)",
        vec![pred(is_a_part_of_day), pred(is_a_time_of_day)],
        |ts| {
            let (a, b) = (td(ts[0])?, td(ts[1])?);
            if !a.latent || b.latent {
                ttd(intersect(a, b).map(mk_latent))
            } else {
                None
            }
        },
    )
}

fn rule_hhmm() -> Rule {
    rule("hh:mm", vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)")], |ts| {
        let h = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        tt(hour_minute(h != 0 && h < 12, h, m))
    })
}

fn rule_hhhmm() -> Rule {
    rule("hhhmm", vec![regex("(?<!/)((?:[01]?\\d)|(?:2[0-3]))h(([0-5]\\d)|(?!\\d))")], |ts| {
        let h = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?).unwrap_or(0);
        tt(hour_minute(false, h, m))
    })
}

fn rule_hhmm_latent() -> Rule {
    rule("hhmm (latent)", vec![regex("((?:[01]?\\d)|(?:2[0-3]))([0-5]\\d)(?!.\\d)")], |ts| {
        let h = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        tt(mk_latent(hour_minute(h < 12, h, m)))
    })
}

fn rule_hhmmss() -> Rule {
    rule("hh:mm:ss", vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)[:.]([0-5]\\d)")], |ts| {
        let h = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        let s = parse_int(&rx(ts[0], 2)?)?;
        tt(hour_minute_second(h < 12, h, m, s))
    })
}

fn rule_military_ampm() -> Rule {
    rule(
        "hhmm (military) am|pm",
        vec![regex("((?:1[012]|0?\\d))([0-5]\\d)"), regex("([ap])\\.?m?\\.?")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let ap = to_lower(&rx(ts[1], 0)?);
            tt(time_of_day_ampm(ap == "a", &hour_minute(true, h, m)))
        },
    )
}

fn rule_tod_ampm() -> Rule {
    rule(
        "<time-of-day> am|pm",
        vec![pred(is_a_time_of_day), regex("(in the )?([ap])(\\s|\\.)?(m?)\\.?")],
        |ts| {
            let t = td(ts[0])?;
            let g = match ts[1] {
                Token::RegexMatch(g) => g,
                _ => return None,
            };
            if t.latent && g.0.get(3).map(|s| s.is_empty()) == Some(true) {
                let ap = to_lower(g.0.get(1)?);
                return tt(mk_latent(time_of_day_ampm(ap == "a", t)));
            }
            match t.form {
                Some(Form::TimeOfDay { hours: Some(hours), .. }) if hours < 13 => {
                    let ap = to_lower(g.0.get(1)?);
                    tt(time_of_day_ampm(ap == "a", t))
                }
                _ => None,
            }
        },
    )
}

fn hour_of_day_minute(ts: &[&Token], tok_idx: usize) -> Option<Token> {
    let t = td(ts[0])?;
    match t.form {
        Some(Form::TimeOfDay { hours: Some(hours), is12h }) => {
            let n = get_int_value(ts[tok_idx])?;
            let r = hour_minute(is12h, hours, n);
            tt(if t.latent { mk_latent(r) } else { r })
        }
        _ => None,
    }
}

fn rule_ho_numeral() -> Rule {
    rule(
        "<hour-of-day> <integer>",
        vec![pred(is_an_hour_of_day), pred(is_integer_between_tok(10, 59))],
        |ts| hour_of_day_minute(ts, 1),
    )
}

fn rule_ho_numeral_dash() -> Rule {
    rule(
        "<hour-of-day> - <integer-as-word>",
        vec![pred(is_an_hour_of_day), regex("-(?!\\d)"), pred(is_integer_between_tok(10, 59))],
        |ts| hour_of_day_minute(ts, 2),
    )
}

fn rule_ho_numeral_alt() -> Rule {
    rule(
        "<hour-of-day> zero <integer>",
        vec![pred(is_an_hour_of_day), regex("(zero|o(h|u)?)"), pred(is_integer_between_tok(1, 9))],
        |ts| hour_of_day_minute(ts, 2),
    )
}

fn rule_ho_numeral_alt_dash() -> Rule {
    rule(
        "<hour-of-day> - zero - <integer>",
        vec![
            pred(is_an_hour_of_day),
            regex("-"),
            regex("(zero|o(h|u)?)"),
            regex("-(?!\\d)"),
            pred(is_integer_between_tok(1, 9)),
        ],
        |ts| hour_of_day_minute(ts, 4),
    )
}

fn hod_fixed(ts: &[&Token], idx: usize, f: impl Fn(i64, bool) -> Option<TimeData>) -> Option<Token> {
    match td(ts[idx])?.form {
        Some(Form::TimeOfDay { hours: Some(hours), is12h }) => ttd(f(hours, is12h)),
        _ => None,
    }
}

fn rule_hod_half() -> Rule {
    rule("<hour-of-day> half", vec![pred(is_an_hour_of_day), regex("half")], |ts| {
        hod_fixed(ts, 0, |h, i| Some(hour_minute(i, h, 30)))
    })
}

fn rule_hod_quarter() -> Rule {
    rule("<hour-of-day> quarter", vec![pred(is_an_hour_of_day), regex("(a|one)? ?quarter")], |ts| {
        hod_fixed(ts, 0, |h, i| Some(hour_minute(i, h, 15)))
    })
}

fn rule_numeral_to_hod() -> Rule {
    rule(
        "<integer> to|till|before <hour-of-day>",
        vec![pred(is_integer_between_tok(1, 59)), regex("to|till|before|of"), pred(is_an_hour_of_day)],
        |ts| {
            let n = get_int_value(ts[0])?;
            ttd(minutes_before(n, td(ts[2])?))
        },
    )
}

fn rule_half_to_hod() -> Rule {
    rule("half to|till|before <hour-of-day>", vec![regex("half (to|till|before|of)"), pred(is_an_hour_of_day)], |ts| {
        ttd(minutes_before(30, td(ts[1])?))
    })
}

fn rule_quarter_to_hod() -> Rule {
    rule(
        "quarter to|till|before <hour-of-day>",
        vec![regex("(a|one)? ?quarter (to|till|before|of)"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_before(15, td(ts[1])?)),
    )
}

fn rule_numeral_after_hod() -> Rule {
    rule(
        "integer after|past <hour-of-day>",
        vec![pred(is_integer_between_tok(1, 59)), regex("after|past"), pred(is_an_hour_of_day)],
        |ts| {
            let n = get_int_value(ts[0])?;
            ttd(minutes_after(n, td(ts[2])?))
        },
    )
}

fn rule_half_after_hod() -> Rule {
    rule("half after|past <hour-of-day>", vec![regex("half (after|past)"), pred(is_an_hour_of_day)], |ts| {
        ttd(minutes_after(30, td(ts[1])?))
    })
}

fn rule_quarter_after_hod() -> Rule {
    rule(
        "quarter after|past <hour-of-day>",
        vec![regex("(for )?((a|one) )?quarter (after|past)"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_after(15, td(ts[1])?)),
    )
}

fn rule_half_hod() -> Rule {
    rule("half <integer> (UK style hour-of-day)", vec![regex("half"), pred(is_an_hour_of_day)], |ts| {
        ttd(minutes_after(30, td(ts[1])?))
    })
}

fn rule_mmyyyy() -> Rule {
    rule("mm/yyyy", vec![regex("(0?[1-9]|1[0-2])[/-](\\d{4})")], |ts| {
        let y = parse_int(&rx(ts[0], 1)?)?;
        let m = parse_int(&rx(ts[0], 0)?)?;
        tt(year_month(y, m))
    })
}

fn rule_yyyymm() -> Rule {
    rule("yyyy-mm", vec![regex("(\\d{4})\\s*[/-]\\s*(1[0-2]|0?[1-9])")], |ts| {
        let y = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        tt(year_month(y, m))
    })
}

fn rule_yyyymmdd() -> Rule {
    rule("yyyy-mm-dd", vec![regex("(\\d{2,4})-(0?[1-9]|1[0-2])-(3[01]|[12]\\d|0?[1-9])")], |ts| {
        let y = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        let d = parse_int(&rx(ts[0], 2)?)?;
        tt(year_month_day(y, m, d))
    })
}

fn rule_yyyyqq() -> Rule {
    rule("yyyyqq", vec![regex("(\\d{2,4})q([1-4])")], |ts| {
        let y = parse_int(&rx(ts[0], 0)?)?;
        let q = parse_int(&rx(ts[0], 1)?)?;
        tt(cycle_nth_after(true, G::Quarter, q - 1, &year(y)))
    })
}

fn rule_noon_midnight_eod() -> Rule {
    rule(
        "noon|midnight|EOD|end of day",
        vec![regex("(noon|midni(ght|te)|(the )?(EOD|end of (the )?day))")],
        |ts| tt(hour(false, if to_lower(&rx(ts[0], 0)?) == "noon" { 12 } else { 0 })),
    )
}

fn rule_part_of_days() -> Rule {
    rule("part of days", vec![regex("(morning|after ?noo?n(ish)?|evening|night|(at )?lunch)")], |ts| {
        let (start, end) = match to_lower(&rx(ts[0], 0)?).as_str() {
            "morning" => (hour(false, 0), hour(false, 12)),
            "evening" => (hour(false, 18), hour(false, 0)),
            "night" => (hour(false, 18), hour(false, 0)),
            "lunch" => (hour(false, 12), hour(false, 14)),
            "at lunch" => (hour(false, 12), hour(false, 14)),
            _ => (hour(false, 12), hour(false, 19)),
        };
        let t = interval(Open, &start, &end)?;
        tt(part_of_day(mk_latent(t)))
    })
}

fn rule_early_morning() -> Rule {
    rule("early morning", vec![regex("early ((in|hours of) the )?morning")], |_| {
        ttd(interval(Open, &hour(false, 0), &hour(false, 9)).map(|t| part_of_day(mk_latent(t))))
    })
}

fn rule_pod_in() -> Rule {
    rule("in|during the <part-of-day>", vec![regex("(in|during)( the)?"), pred(is_a_part_of_day)], |ts| {
        tt(not_latent(td(ts[1])?.clone()))
    })
}

fn rule_pod_this() -> Rule {
    rule("this <part-of-day>", vec![regex("this"), pred(is_a_part_of_day)], |ts| {
        ttd(intersect(&today(), td(ts[1])?).map(|t| part_of_day(not_latent(t))))
    })
}

fn rule_tonight() -> Rule {
    rule("tonight", vec![regex("(late )?toni(ght|gth|te)s?")], |ts| {
        let h = if to_lower(&rx(ts[0], 0)?) == "late " { 21 } else { 18 };
        let evening = interval(Open, &hour(false, h), &hour(false, 0))?;
        ttd(intersect(&today(), &evening).map(|t| part_of_day(not_latent(t))))
    })
}

fn rule_after_partofday() -> Rule {
    rule("after lunch/work/school", vec![regex("after[\\s-]?(lunch|work|school)")], |ts| {
        let (start, end) = match to_lower(&rx(ts[0], 0)?).as_str() {
            "lunch" => (hour(false, 13), hour(false, 17)),
            "work" => (hour(false, 17), hour(false, 21)),
            "school" => (hour(false, 15), hour(false, 21)),
            _ => return None,
        };
        let t = interval(Open, &start, &end)?;
        ttd(intersect(&today(), &t).map(|t| part_of_day(not_latent(t))))
    })
}

fn rule_time_pod() -> Rule {
    rule("<time> <part-of-day>", vec![dimension(Dim::Time), pred(is_a_part_of_day)], |ts| {
        ttd(intersect(td(ts[1])?, td(ts[0])?))
    })
}

fn rule_pod_of_time() -> Rule {
    rule("<part-of-day> of <time>", vec![pred(is_a_part_of_day), regex("of"), dimension(Dim::Time)], |ts| {
        ttd(intersect(td(ts[0])?, td(ts[2])?))
    })
}

fn rule_weekend() -> Rule {
    rule("week-end", vec![regex("(week(\\s|-)?end|wkend)s?")], |_| tt(mk_ok_for_this_next(weekend())))
}

fn rule_week() -> Rule {
    rule("week", vec![regex("(all|rest of the|the) week")], |ts| {
        let m = to_lower(&rx(ts[0], 0)?);
        let end = cycle_nth_after(true, G::Day, -2, &cycle_nth(G::Week, 1));
        let period = match m.as_str() {
            "all" => interval(Closed, &cycle_nth(G::Week, 0), &end),
            "rest of the" => interval(Open, &today(), &end),
            "the" => interval(Open, &today(), &end),
            _ => None,
        };
        match m.as_str() {
            "the" => ttd(period.map(mk_latent)),
            _ => ttd(period),
        }
    })
}

fn rule_season() -> Rule {
    rule("last|this|next <season>", vec![regex("(this|current|next|last|past|previous) seasons?")], |ts| {
        let n = match to_lower(&rx(ts[0], 0)?).as_str() {
            "this" | "current" => 0,
            "last" | "past" | "previous" => -1,
            "next" => 1,
            _ => return None,
        };
        tt(pred_nth(n, false, &season()))
    })
}

fn rules_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("summer", "summer", month_day(6, 21), month_day(9, 23)),
        ("fall", "fall|autumn", month_day(9, 23), month_day(12, 21)),
        ("winter", "winter", month_day(12, 21), month_day(3, 20)),
        ("spring", "spring", month_day(3, 20), month_day(6, 21)),
    ])
}

fn rule_tod_precision() -> Rule {
    rule(
        "<time-of-day> sharp|exactly",
        vec![pred(is_a_time_of_day), regex("(sharp|exactly|-?ish|approximately)")],
        |ts| tt(not_latent(td(ts[0])?.clone())),
    )
}

fn rule_tod_pod() -> Rule {
    rule("<time-of-day> <part-of-day>", vec![pred(is_a_time_of_day), pred(is_a_part_of_day)], |ts| {
        ttd(intersect(td(ts[0])?, td(ts[1])?).map(not_latent))
    })
}

fn rule_precision_tod() -> Rule {
    rule(
        "about|exactly <time-of-day>",
        vec![regex("(about|around|approximately|exactly)"), pred(is_grain_finer_than(G::Year))],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    )
}

/// Shared shape of the "<month> dd-dd" interval rules.
fn dom_interval(name: &str, pattern: Vec<crate::types::PatternItem>, td_idx: usize, a: usize, b: usize) -> Rule {
    rule(name, pattern, move |ts| {
        let t = td(ts[td_idx])?;
        let dom1 = intersect_dom(t, ts[a])?;
        let dom2 = intersect_dom(t, ts[b])?;
        ttd(interval(Closed, &dom1, &dom2))
    })
}

fn rule_interval_month_dddd() -> Rule {
    dom_interval(
        "<month> dd-dd (interval)",
        vec![pred(is_a_month), pred(is_dom_value), regex("\\-|to|th?ru|through|(un)?til(l)?"), pred(is_dom_value)],
        0,
        1,
        3,
    )
}

fn rule_interval_dddd_month() -> Rule {
    dom_interval(
        "dd-dd <month> (interval)",
        vec![pred(is_dom_value), regex("\\-|to|th?ru|through|(un)?til(l)?"), pred(is_dom_value), pred(is_a_month)],
        3,
        0,
        2,
    )
}

fn rule_interval_from_month_dddd() -> Rule {
    dom_interval(
        "from <month> dd-dd (interval)",
        vec![
            regex("from"),
            pred(is_a_month),
            pred(is_dom_value),
            regex("\\-|to|th?ru|through|(un)?til(l)?"),
            pred(is_dom_value),
        ],
        1,
        2,
        4,
    )
}

fn rule_interval_from_dddd_month() -> Rule {
    dom_interval(
        "from the <day-of-month> (ordinal or number) to the <day-of-month> (ordinal or number) <named-month> (interval)",
        vec![
            regex("from( the)?"),
            pred(is_dom_value),
            regex("\\-|to( the)?|th?ru|through|(un)?til(l)?"),
            pred(is_dom_value),
            pred(is_a_month),
        ],
        4,
        1,
        3,
    )
}

fn rule_interval_from_dddd_of_month() -> Rule {
    dom_interval(
        "from the <day-of-month> (ordinal or number) to the <day-of-month> (ordinal or number) of <named-month> (interval)",
        vec![
            regex("from( the)?"),
            pred(is_dom_value),
            regex("\\-|to( the)?|th?ru|through|(un)?til(l)?"),
            pred(is_dom_value),
            regex("of"),
            pred(is_a_month),
        ],
        5,
        1,
        3,
    )
}

fn rule_interval_year_latent() -> Rule {
    rule(
        "<year> (latent) - <year> (latent) (interval)",
        vec![
            pred(is_integer_between_tok(1000, 10000)),
            regex("\\-|to|th?ru|through|(un)?til(l)?"),
            pred(is_integer_between_tok(1000, 10000)),
        ],
        |ts| {
            let y1 = get_int_value(ts[0])?;
            let y2 = get_int_value(ts[2])?;
            if y1 < y2 {
                ttd(interval(Closed, &year(y1), &year(y2)))
            } else {
                None
            }
        },
    )
}

fn rule_interval_dash() -> Rule {
    rule(
        "<datetime> - <datetime> (interval)",
        vec![pred(is_not_latent), regex("\\-|to|th?ru|through|(un)?til(l)?"), pred(is_not_latent)],
        |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_interval_slash() -> Rule {
    rule("<datetime>/<datetime> (interval)", vec![pred(is_not_latent), regex("/"), pred(is_not_latent)], |ts| {
        let (a, b) = (td(ts[0])?, td(ts[2])?);
        if same_grain(a, b) {
            ttd(interval(Closed, a, b))
        } else {
            None
        }
    })
}

fn rule_interval_from() -> Rule {
    rule(
        "from <datetime> - <datetime> (interval)",
        vec![regex("from"), dimension(Dim::Time), regex("\\-|to|th?ru|through|(un)?til(l)?"), dimension(Dim::Time)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_interval_between() -> Rule {
    rule(
        "between <time> and <time>",
        vec![regex("between"), dimension(Dim::Time), regex("and"), dimension(Dim::Time)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_interval_tod_dash() -> Rule {
    rule(
        "<time-of-day> - <time-of-day> (interval)",
        vec![
            pred(|t| is_not_latent(t) && is_a_time_of_day(t)),
            regex("\\-|:|to|th?ru|through|(un)?til(l)?"),
            pred(is_a_time_of_day),
        ],
        |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_interval_tod_from() -> Rule {
    rule(
        "from <time-of-day> - <time-of-day> (interval)",
        vec![
            regex("(later than|from|(in[\\s-])?between)"),
            pred(is_a_time_of_day),
            regex("((but )?before)|\\-|to|th?ru|through|(un)?til(l)?"),
            pred(is_a_time_of_day),
        ],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_interval_tod_ampm() -> Rule {
    rule(
        "hh(:mm) - <time-of-day> am|pm",
        vec![
            regex("(?:from )?((?:[01]?\\d)|(?:2[0-3]))([:.]([0-5]\\d))?"),
            regex("\\-|to|th?ru|through|(un)?til(l)?"),
            pred(is_a_time_of_day),
            regex("(in the )?([ap])(\\s|\\.)?m?\\.?"),
        ],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let td2 = td(ts[2])?;
            let ampm = to_lower(&rx(ts[3], 1)?) == "a";
            let td1 = match parse_int(&rx(ts[0], 2)?) {
                Some(m) => hour_minute(true, h, m),
                None => hour(true, h),
            };
            ttd(interval(Closed, &time_of_day_ampm(ampm, &td1), &time_of_day_ampm(ampm, td2)))
        },
    )
}

fn rule_interval_tod_between() -> Rule {
    rule(
        "between <time-of-day> and <time-of-day> (interval)",
        vec![regex("between"), pred(is_a_time_of_day), regex("and"), pred(is_a_time_of_day)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_interval_by() -> Rule {
    rule("by <time>", vec![regex("by"), dimension(Dim::Time)], |ts| ttd(interval(Open, &now(), td(ts[1])?)))
}

fn rule_interval_by_the_end_of() -> Rule {
    rule("by the end of <time>", vec![regex("by (the )?end of"), dimension(Dim::Time)], |ts| {
        ttd(interval(Closed, &now(), td(ts[1])?))
    })
}

fn rule_interval_until_time() -> Rule {
    rule(
        "until <time>",
        vec![regex("(anytime |sometimes? )?(before|(un)?til(l)?|through|up to)"), dimension(Dim::Time)],
        |ts| tt(with_direction(IntervalDirection::Before, not_latent(td(ts[1])?.clone()))),
    )
}

fn rule_interval_after_from_since_time() -> Rule {
    rule(
        "from|since|after <time>",
        vec![regex("from|since|(anytime |sometimes? )?after"), dimension(Dim::Time)],
        |ts| tt(with_direction(IntervalDirection::After, not_latent(td(ts[1])?.clone()))),
    )
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Monday", "mondays?|mon\\.?"),
        ("Tuesday", "tuesdays?|tues?\\.?"),
        ("Wednesday", "wed?nesdays?|wed\\.?"),
        ("Thursday", "thursdays?|thu(rs?)?\\.?"),
        ("Friday", "fridays?|fri\\.?"),
        ("Saturday", "saturdays?|sat\\.?"),
        ("Sunday", "sundays?|sun\\.?"),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months_with_latent(&[
        ("January", "january|jan\\.?", false),
        ("February", "february|feb\\.?", false),
        ("March", "march|mar\\.?", false),
        ("April", "april|apr\\.?", false),
        ("May", "may", true),
        ("June", "june|jun\\.?", false),
        ("July", "july|jul\\.?", false),
        ("August", "august|aug\\.?", false),
        ("September", "september|sept?\\.?", false),
        ("October", "october|oct\\.?", false),
        ("November", "november|nov\\.?", false),
        ("December", "december|dec\\.?", false),
    ])
}

fn rule_part_of_month() -> Rule {
    rule("part of <named-month>", vec![regex("(early|mid|late)-?( of)?"), pred(is_a_month)], |ts| {
        let t = td(ts[1])?;
        let (sd, ed) = match to_lower(&rx(ts[0], 0)?).as_str() {
            "early" => (1, 10),
            "mid" => (11, 20),
            "late" => (21, -1),
            _ => return None,
        };
        let start = intersect(t, &day_of_month(sd))?;
        let end = if ed != -1 { intersect(t, &day_of_month(ed))? } else { cycle_last_of(G::Day, t) };
        ttd(interval(Open, &start, &end))
    })
}

fn rule_end_or_beginning_of_month() -> Rule {
    rule(
        "at the beginning|end of <named-month>",
        vec![regex("(at the )?(beginning|end) of"), pred(is_a_month)],
        |ts| {
            let t = td(ts[1])?;
            let (sd, ed) = match to_lower(&rx(ts[0], 1)?).as_str() {
                "beginning" => (1, 10),
                "end" => (21, -1),
                _ => return None,
            };
            let start = intersect(t, &day_of_month(sd))?;
            let end = if ed != -1 { intersect(t, &day_of_month(ed))? } else { cycle_last_of(G::Day, t) };
            ttd(interval(Open, &start, &end))
        },
    )
}

fn rule_end_of_month() -> Rule {
    rule("end of month", vec![regex("(by (the )?|(at )?the )?(EOM|end of (the )?month)")], |ts| {
        let m = to_lower(&rx(ts[0], 0)?);
        let (start, end) = if m.starts_with("by") {
            (Some(now()), intersect(&day_of_month(1), &cycle_nth(G::Month, 1)))
        } else {
            (
                intersect(&day_of_month(21), &cycle_nth(G::Month, 0)),
                Some(cycle_last_of(G::Day, &cycle_nth(G::Month, 0))),
            )
        };
        match (start, end) {
            (Some(s), Some(e)) => ttd(interval(Open, &s, &e)),
            _ => None,
        }
    })
}

fn rule_beginning_of_month() -> Rule {
    rule("beginning of month", vec![regex("((at )?the )?(BOM|beginning of (the )?month)")], |_| {
        let start = intersect(&day_of_month(1), &cycle_nth(G::Month, 0))?;
        let end = intersect(&day_of_month(10), &cycle_nth(G::Month, 0))?;
        ttd(interval(Open, &start, &end))
    })
}

fn rule_end_or_beginning_of_year() -> Rule {
    rule(
        "at the beginning|end of <year>",
        vec![regex("(at the )?(beginning|end) of"), pred(is_grain_of_time(G::Year))],
        |ts| {
            let t = td(ts[1])?;
            let (sd, ed) = match to_lower(&rx(ts[0], 1)?).as_str() {
                "beginning" => (1, 4),
                "end" => (9, -1),
                _ => return None,
            };
            let start = intersect(t, &month(sd))?;
            let end = if ed != -1 {
                intersect(t, &cycle_last_of(G::Month, &month(ed)))?
            } else {
                cycle_nth_after(false, G::Year, 1, &intersect(t, &month(1))?)
            };
            ttd(interval(Open, &start, &end))
        },
    )
}

fn rule_end_of_year() -> Rule {
    rule("end of year", vec![regex("(by (the )?|(at )?the )?(EOY|end of (the )?year)")], |ts| {
        let m = to_lower(&rx(ts[0], 0)?);
        let start = if m.starts_with("by") { now() } else { intersect(&month(9), &cycle_nth(G::Year, 0))? };
        let end = intersect(&month(1), &cycle_nth(G::Year, 1))?;
        ttd(interval(Open, &start, &end))
    })
}

fn rule_beginning_of_year() -> Rule {
    rule("beginning of year", vec![regex("((at )?the )?(BOY|beginning of (the )?year)")], |_| {
        let start = intersect(&month(1), &cycle_nth(G::Year, 0))?;
        let end = intersect(&month(4), &cycle_nth(G::Year, 0))?;
        ttd(interval(Open, &start, &end))
    })
}

fn rule_end_or_beginning_of_week() -> Rule {
    rule(
        "at the beginning|end of <week>",
        vec![regex("(at the )?(beginning|end) of"), pred(is_grain_of_time(G::Week))],
        |ts| {
            let t = td(ts[1])?;
            let (sd, ed) = match to_lower(&rx(ts[0], 1)?).as_str() {
                "beginning" => (1, 3),
                "end" => (5, 7),
                _ => return None,
            };
            let start = intersect(t, &day_of_week(sd))?;
            let end = intersect(t, &day_of_week(ed))?;
            ttd(interval(Open, &start, &end))
        },
    )
}

fn rule_closest() -> Rule {
    rule(
        "the closest <day> to <time>",
        vec![regex("the\\s+closest"), pred(is_grain_of_time(G::Day)), regex("to"), dimension(Dim::Time)],
        |ts| tt(pred_nth_closest(0, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_nth_closest() -> Rule {
    rule(
        "the <ordinal> closest <day> to <time>",
        vec![
            regex("the"),
            dimension(Dim::Ordinal),
            regex("closest"),
            pred(is_grain_of_time(G::Day)),
            regex("to"),
            dimension(Dim::Time),
        ],
        |ts| {
            let n = get_int_value(ts[1])?;
            tt(pred_nth_closest(n - 1, td(ts[3])?, td(ts[5])?))
        },
    )
}

fn is_grain(g: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(x) if *x == g)
}

fn rule_cycle_this_last_next() -> Rule {
    rule(
        "this|last|next <cycle>",
        vec![
            regex("(this|current|coming|next|(the( following)?)|last|past|previous|upcoming)"),
            dimension(Dim::TimeGrain),
        ],
        |ts| {
            let grain = grain_of(ts[1])?;
            let n = match to_lower(&rx(ts[0], 0)?).as_str() {
                "this" | "current" | "the" => 0,
                "coming" | "next" | "upcoming" | "the following" => 1,
                "last" | "past" | "previous" => -1,
                _ => return None,
            };
            tt(cycle_nth(grain, n))
        },
    )
}

fn rule_dom_of_time_month() -> Rule {
    rule(
        "<day-of-month> (ordinal or number) of <month>",
        vec![pred(is_dom_value), regex("of( the)?"), pred(is_grain_of_time(G::Month))],
        |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
    )
}

fn rule_cycle_the_after_before_time() -> Rule {
    rule(
        "the <cycle> after|before <time>",
        vec![regex("the"), dimension(Dim::TimeGrain), regex("(after|before)"), dimension(Dim::Time)],
        |ts| {
            let n = if to_lower(&rx(ts[2], 0)?) == "after" { 1 } else { -1 };
            tt(cycle_nth_after(false, grain_of(ts[1])?, n, td(ts[3])?))
        },
    )
}

fn rule_cycle_after_before_time() -> Rule {
    rule(
        "<cycle> after|before <time>",
        vec![dimension(Dim::TimeGrain), regex("(after|before)"), dimension(Dim::Time)],
        |ts| {
            let n = if to_lower(&rx(ts[1], 0)?) == "after" { 1 } else { -1 };
            tt(cycle_nth_after(false, grain_of(ts[0])?, n, td(ts[2])?))
        },
    )
}

fn rule_cycle_ordinal_of_time() -> Rule {
    rule(
        "<ordinal> <cycle> of <time>",
        vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("of|in|from"), dimension(Dim::Time)],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth_after(true, grain_of(ts[1])?, n - 1, td(ts[3])?))
        },
    )
}

fn last_ordinal_of(n: i64, grain: G, t: &TimeData) -> Option<Token> {
    tt(cycle_nth_after(true, grain, -n, &cycle_nth_after(true, t.time_grain, 1, t)))
}

fn rule_cycle_last_ordinal_of_time() -> Rule {
    rule(
        "<ordinal> last <cycle> of <time>",
        vec![
            dimension(Dim::Ordinal),
            regex("last"),
            dimension(Dim::TimeGrain),
            regex("of|in|from"),
            dimension(Dim::Time),
        ],
        |ts| last_ordinal_of(get_int_value(ts[0])?, grain_of(ts[2])?, td(ts[4])?),
    )
}

fn rule_cycle_the_ordinal_of_time() -> Rule {
    rule(
        "the <ordinal> <cycle> of <time>",
        vec![
            regex("the"),
            dimension(Dim::Ordinal),
            dimension(Dim::TimeGrain),
            regex("of|in|from"),
            dimension(Dim::Time),
        ],
        |ts| {
            let n = get_int_value(ts[1])?;
            tt(cycle_nth_after(true, grain_of(ts[2])?, n - 1, td(ts[4])?))
        },
    )
}

fn rule_cycle_the_last_ordinal_of_time() -> Rule {
    rule(
        "the <ordinal> last <cycle> of <time>",
        vec![
            regex("the"),
            dimension(Dim::Ordinal),
            regex("last"),
            dimension(Dim::TimeGrain),
            regex("of|in|from"),
            dimension(Dim::Time),
        ],
        |ts| last_ordinal_of(get_int_value(ts[1])?, grain_of(ts[3])?, td(ts[5])?),
    )
}

fn rule_cycle_the_of_time_grain() -> Rule {
    rule(
        "the <cycle> of the <time grain>",
        vec![regex("the"), dimension(Dim::TimeGrain), regex("of( the)?"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, 0, &cycle_nth(grain_of(ts[3])?, 0))),
    )
}

fn rule_cycle_the_of_time() -> Rule {
    rule(
        "the <cycle> of <time>",
        vec![regex("the"), dimension(Dim::TimeGrain), regex("of"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, 0, td(ts[3])?)),
    )
}

fn rule_cycle_ordinal_after_time() -> Rule {
    rule(
        "<ordinal> <cycle> after <time>",
        vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("after"), dimension(Dim::Time)],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth_after(true, grain_of(ts[1])?, n - 1, td(ts[3])?))
        },
    )
}

fn rule_cycle_the_ordinal_after_time() -> Rule {
    rule(
        "<ordinal> <cycle> after <time>",
        vec![regex("the"), dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("after"), dimension(Dim::Time)],
        |ts| {
            let n = get_int_value(ts[1])?;
            tt(cycle_nth_after(true, grain_of(ts[2])?, n - 1, td(ts[4])?))
        },
    )
}

fn rule_cycle_ordinal_quarter() -> Rule {
    rule("<ordinal> quarter", vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(cycle_nth_after(true, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
    })
}

fn rule_cycle_the_ordinal_quarter() -> Rule {
    rule("the <ordinal> quarter", vec![regex("the"), dimension(Dim::Ordinal), pred(is_grain(G::Quarter))], |ts| {
        let n = get_int_value(ts[1])?;
        tt(cycle_nth_after(true, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
    })
}

fn rule_cycle_ordinal_quarter_year() -> Rule {
    rule(
        "<ordinal> quarter <year>",
        vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter)), dimension(Dim::Time)],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth_after(false, G::Quarter, n - 1, td(ts[2])?))
        },
    )
}

fn rule_duration_in_within_after() -> Rule {
    rule("in|within|after <duration>", vec![regex("(in|within|after)"), dimension(Dim::Duration)], |ts| {
        let dd = dur(ts[1])?;
        match to_lower(&rx(ts[0], 0)?).as_str() {
            "within" => ttd(interval(Open, &now(), &in_duration(dd))),
            "after" => tt(with_direction(IntervalDirection::After, in_duration(dd))),
            "in" => tt(in_duration(dd)),
            _ => None,
        }
    })
}

fn rule_duration_last_next() -> Rule {
    rule("last|past|next <duration>", vec![regex("([lp]ast|next)"), dimension(Dim::Duration)], |ts| {
        let dd = dur(ts[1])?;
        match to_lower(&rx(ts[0], 0)?).as_str() {
            "next" => tt(cycle_n(true, dd.grain, dd.value)),
            "last" | "past" => tt(cycle_n(true, dd.grain, -dd.value)),
            _ => None,
        }
    })
}

fn rule_ndow_ago() -> Rule {
    rule(
        "<integer> <named-day> ago|back",
        vec![pred(is_natural), pred(is_a_day_of_week), regex("ago|back")],
        |ts| {
            let v = numeral(ts[0])?.value;
            tt(pred_nth(-floor_i(v), false, td(ts[1])?))
        },
    )
}

fn rule_duration_hence_ago() -> Rule {
    rule("<duration> hence|ago", vec![dimension(Dim::Duration), regex("(hence|ago)")], |ts| {
        let dd = dur(ts[0])?;
        match to_lower(&rx(ts[1], 0)?).as_str() {
            "ago" => tt(duration_ago(dd)),
            _ => tt(in_duration(dd)),
        }
    })
}

fn day_or_month(t: &Token) -> bool {
    is_grain_of_time(G::Day)(t) || is_grain_of_time(G::Month)(t)
}

fn rule_day_duration_hence_ago() -> Rule {
    rule(
        "<day> <duration> hence|ago",
        vec![pred(day_or_month), dimension(Dim::Duration), regex("(from now|hence|ago)")],
        |ts| {
            let (t, dd) = (td(ts[0])?, dur(ts[1])?);
            match to_lower(&rx(ts[2], 0)?).as_str() {
                "ago" => ttd(intersect(t, &duration_interval_ago(dd))),
                _ => ttd(intersect(t, &in_duration_interval(dd))),
            }
        },
    )
}

fn rule_day_in_duration() -> Rule {
    rule(
        "<day> in <duration>",
        vec![pred(day_or_month), regex("in"), pred(is_duration_greater_than(G::Hour))],
        |ts| ttd(intersect(td(ts[0])?, &in_duration_interval(dur(ts[2])?))),
    )
}

fn rule_in_duration_at_time() -> Rule {
    rule(
        "in <duration> at <time-of-day>",
        vec![regex("in"), pred(is_duration_greater_than(G::Hour)), regex("at"), pred(is_a_time_of_day)],
        |ts| ttd(intersect(td(ts[3])?, &in_duration_interval(dur(ts[1])?))),
    )
}

fn rule_in_numeral() -> Rule {
    rule(
        "in <number> (implicit minutes)",
        vec![regex("in"), pred(is_integer_between_tok(0, 60))],
        |ts| {
            let v = numeral(ts[1])?.value;
            tt(in_duration(&duration(G::Minute, floor_i(v))))
        },
    )
}

fn rule_duration_after_before_time() -> Rule {
    rule(
        "<duration> after|before|from|past <time>",
        vec![dimension(Dim::Duration), regex("(after|before|from|past)"), dimension(Dim::Time)],
        |ts| {
            let (dd, t) = (dur(ts[0])?, td(ts[2])?);
            match to_lower(&rx(ts[1], 0)?).as_str() {
                "before" => tt(duration_before(dd, t)),
                _ => tt(duration_after(dd, t)),
            }
        },
    )
}

fn rule_interval_for_duration_from() -> Rule {
    rule(
        "for <duration> from <time>",
        vec![
            regex("for"),
            dimension(Dim::Duration),
            regex("(from|starting|beginning|after|starting from)"),
            dimension(Dim::Time),
        ],
        |ts| {
            let (dd, t1) = (dur(ts[1])?, td(ts[3])?);
            ttd(interval(Closed, t1, &duration_after(dd, t1)))
        },
    )
}

fn rule_interval_time_for_duration() -> Rule {
    rule(
        "<time> for <duration>",
        vec![pred(is_not_latent), regex("for"), dimension(Dim::Duration)],
        |ts| {
            let (t1, dd) = (td(ts[0])?, dur(ts[2])?);
            ttd(interval(Closed, t1, &duration_after(dd, t1)))
        },
    )
}

fn rule_interval_from_time_for_duration() -> Rule {
    rule(
        "from <time> for <duration>",
        vec![
            regex("(from|starting|beginning|after|starting from)"),
            pred(is_not_latent),
            regex("for"),
            dimension(Dim::Duration),
        ],
        |ts| {
            let (t1, dd) = (td(ts[1])?, dur(ts[3])?);
            ttd(interval(Closed, t1, &duration_after(dd, t1)))
        },
    )
}

const TIMEZONE_NAME: &str = "YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT";

fn tod_no_tz(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t) && has_no_timezone(t)
}

fn tod_no_tz_any(t: &Token) -> bool {
    is_a_time_of_day(t) && has_no_timezone(t)
}

fn rule_timezone() -> Rule {
    rule(
        "<time> timezone",
        vec![pred(tod_no_tz), regex(&format!("\\b({})\\b", TIMEZONE_NAME))],
        |ts| ttd(in_timezone(&rx(ts[1], 0)?.to_uppercase(), td(ts[0])?)),
    )
}

fn rule_timezone_bracket() -> Rule {
    rule(
        "<time> (timezone)",
        vec![pred(tod_no_tz), regex(&format!("\\(({})\\)", TIMEZONE_NAME))],
        |ts| ttd(in_timezone(&rx(ts[1], 0)?.to_uppercase(), td(ts[0])?)),
    )
}

fn rule_interval_dash_timezone() -> Rule {
    rule(
        "<datetime> - <datetime> (interval) timezone",
        vec![
            pred(tod_no_tz_any),
            regex("\\-|to|th?ru|through|(un)?til(l)?"),
            pred(tod_no_tz_any),
            regex(&format!("\\b({})\\b", TIMEZONE_NAME)),
        ],
        |ts| {
            let tz = rx(ts[3], 0)?.to_uppercase();
            let tdz1 = in_timezone(&tz, td(ts[0])?)?;
            let tdz2 = in_timezone(&tz, td(ts[2])?)?;
            ttd(interval(Closed, &tdz1, &tdz2))
        },
    )
}

fn rule_mid_day() -> Rule {
    rule("Mid-day", vec![regex("(the )?mid(\\s)?day")], |_| tt(hour(false, 12)))
}

fn rule_upcoming_grain() -> Rule {
    rule("upcoming <integer> <cycle>", vec![regex("upcoming"), pred(is_natural), dimension(Dim::TimeGrain)], |ts| {
        tt(cycle_nth(grain_of(ts[2])?, floor_i(numeral(ts[1])?.value)))
    })
}

fn rule_upcoming_grain_alt() -> Rule {
    rule("<integer> upcoming <cycle>", vec![pred(is_natural), regex("upcoming"), dimension(Dim::TimeGrain)], |ts| {
        tt(cycle_nth(grain_of(ts[2])?, floor_i(numeral(ts[0])?.value)))
    })
}

fn rule_minutes_to_hod() -> Rule {
    rule(
        "<integer> minutes to|till|before <hour-of-day>",
        vec![
            pred(is_integer_between_tok(1, 59)),
            pred(is_grain(G::Minute)),
            regex("to|till|before|of"),
            pred(is_an_hour_of_day),
        ],
        |ts| {
            let n = get_int_value(ts[0])?;
            ttd(minutes_before(n, td(ts[3])?))
        },
    )
}

/// `Duckling.Time.EN.Rules.rules` (in upstream order).
pub fn build_rules() -> Vec<Rule> {
    let mut v = vec![
        rule_intersect(),
        rule_intersect_of(),
        rule_intersect_year(),
        rule_absorb_on_day(),
        rule_absorb_on_adow(),
        rule_absorb_in_month_year(),
        rule_absorb_comma_tod(),
        rule_next_dow(),
        rule_next_time(),
        rule_this_time(),
        rule_last_time(),
        rule_time_before_last_after_next(),
        rule_ordinal_dow_of_time(),
        rule_last_dow_of_time(),
        rule_last_cycle_of_time(),
        rule_last_night(),
        rule_last_weekend_of_month(),
        rule_nth_time_of_time(),
        rule_the_nth_time_of_time(),
        rule_nth_time_after_time(),
        rule_the_nth_time_after_time(),
        rule_ndow_from_time(),
        rule_year_latent(),
        rule_year_adbc(),
        rule_the_dom_numeral(),
        rule_the_dom_ordinal(),
        rule_dom_latent(),
        rule_named_dom_ordinal(),
        rule_month_dom_numeral(),
        rule_dom_month(),
        rule_dom_of_month(),
        rule_the_dom_of_month(),
        rule_dom_ordinal_month_year(),
        rule_dom_month_year(),
        rule_ides_of_month(),
        rule_tod_latent(),
        rule_at_tod(),
        rule_tod_oclock(),
        rule_pod_at_tod(),
        rule_pod_intersect_tod_latent(),
        rule_hhmm(),
        rule_hhhmm(),
        rule_hhmm_latent(),
        rule_hhmmss(),
        rule_military_ampm(),
        rule_tod_ampm(),
        rule_ho_numeral(),
        rule_ho_numeral_dash(),
        rule_ho_numeral_alt(),
        rule_ho_numeral_alt_dash(),
        rule_hod_half(),
        rule_hod_quarter(),
        rule_numeral_to_hod(),
        rule_half_to_hod(),
        rule_quarter_to_hod(),
        rule_numeral_after_hod(),
        rule_half_after_hod(),
        rule_quarter_after_hod(),
        rule_half_hod(),
        rule_yyyyqq(),
        rule_yyyymm(),
        rule_yyyymmdd(),
        rule_mmyyyy(),
        rule_noon_midnight_eod(),
        rule_part_of_days(),
        rule_early_morning(),
        rule_pod_in(),
        rule_pod_this(),
        rule_tonight(),
        rule_after_partofday(),
        rule_time_pod(),
        rule_pod_of_time(),
        rule_weekend(),
        rule_week(),
        rule_tod_precision(),
        rule_tod_pod(),
        rule_precision_tod(),
        rule_interval_from_month_dddd(),
        rule_interval_from_dddd_month(),
        rule_interval_from_dddd_of_month(),
        rule_interval_month_dddd(),
        rule_interval_dddd_month(),
        rule_interval_year_latent(),
        rule_interval_dash(),
        rule_interval_slash(),
        rule_interval_from(),
        rule_interval_between(),
        rule_interval_tod_dash(),
        rule_interval_tod_from(),
        rule_interval_tod_ampm(),
        rule_interval_tod_between(),
        rule_interval_by(),
        rule_interval_by_the_end_of(),
        rule_interval_until_time(),
        rule_interval_after_from_since_time(),
        rule_cycle_the_after_before_time(),
        rule_cycle_this_last_next(),
        rule_dom_of_time_month(),
        rule_cycle_after_before_time(),
        rule_cycle_ordinal_of_time(),
        rule_cycle_last_ordinal_of_time(),
        rule_cycle_the_ordinal_of_time(),
        rule_cycle_the_last_ordinal_of_time(),
        rule_cycle_the_of_time_grain(),
        rule_cycle_the_of_time(),
        rule_cycle_ordinal_after_time(),
        rule_cycle_the_ordinal_after_time(),
        rule_cycle_ordinal_quarter(),
        rule_cycle_the_ordinal_quarter(),
        rule_cycle_ordinal_quarter_year(),
        rule_duration_in_within_after(),
        rule_duration_last_next(),
        rule_ndow_ago(),
        rule_duration_hence_ago(),
        rule_day_duration_hence_ago(),
        rule_day_in_duration(),
        rule_in_duration_at_time(),
        rule_duration_after_before_time(),
        rule_interval_for_duration_from(),
        rule_interval_from_time_for_duration(),
        rule_interval_time_for_duration(),
        rule_in_numeral(),
        rule_timezone(),
        rule_timezone_bracket(),
        rule_interval_dash_timezone(),
        rule_part_of_month(),
        rule_end_or_beginning_of_month(),
        rule_end_or_beginning_of_year(),
        rule_end_or_beginning_of_week(),
        rule_now(),
        rule_asap(),
        rule_season(),
        rule_end_of_month(),
        rule_beginning_of_month(),
        rule_end_of_year(),
        rule_beginning_of_year(),
        rule_closest(),
        rule_nth_closest(),
        rule_mid_day(),
        rule_upcoming_grain(),
        rule_upcoming_grain_alt(),
        rule_minutes_to_hod(),
    ];
    v.extend(rules_instants());
    v.extend(rules_days_of_week());
    v.extend(rules_months());
    v.extend(rules_seasons());
    v.extend(super::holidays::en::computed_holidays());
    v.extend(super::holidays::en::computed_holidays_prime());
    v.extend(super::holidays::en::periodic_holidays());
    v
}

static EN_RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &EN_RULES
}
