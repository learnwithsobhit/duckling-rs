//! `Duckling.Time.KA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::{duration, DurationData};
use crate::dims::numeral::is_natural;
use crate::dims::time::computed::orthodox_easter;
use crate::dims::time::helpers::*;
use crate::dims::time::types::{
    IntervalDirection, TimeData,
    TimeIntervalType::{Closed, Open},
};
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

/// `Duckling.Duration.Helpers.isGrain` — matches a `TimeGrain` token.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn not_latent_or_year(t: &Token) -> bool {
    is_not_latent(t) || is_grain_of_time(G::Year)(t)
}

fn month_or_month_grain(t: &Token) -> bool {
    is_a_month(t) || is_grain_of_time(G::Month)(t)
}

fn day_or_month_grain(t: &Token) -> bool {
    is_grain_of_time(G::Day)(t) || is_grain_of_time(G::Month)(t)
}

fn not_latent_tod(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t)
}

/// Georgian weekend: Saturday–Sunday (`dayOfWeek 6` .. `dayOfWeek 7`).
fn geo_weekend() -> TimeData {
    interval(Open, &day_of_week(6), &day_of_week(7)).unwrap_or_else(weekend)
}

fn latent_pod(a: i64, b: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, a), &hour(false, b)).map(|x| part_of_day(mk_latent(x))))
}

fn part_of_days_bounds(match_: &str) -> (TimeData, TimeData) {
    match to_lower(match_).as_str() {
        "დილას" | "დილის" | "დილით" | "დილა" | "დილ" => (hour(false, 4), hour(false, 12)),
        "საღამოთი" | "საღამოს" | "საღამო" => (hour(false, 18), hour(false, 0)),
        "შუაღამით" | "შუაღამისას" | "შუაღამის" | "შუაღამე" => (hour(false, 18), hour(false, 0)),
        "ღამით" | "ღამის" | "ღამე" => (hour(false, 18), hour(false, 0)),
        "გვიან ღამე" | "გვიან ღამით" => (hour(false, 21), hour(false, 0)),
        "შუადღე" | "შუადღის" | "შუადღით" => (hour(false, 12), hour(false, 18)),
        "დღისით" | "დღის" => (hour(false, 12), hour(false, 18)),
        _ => (hour(false, 12), hour(false, 19)),
    }
}

fn after_part_of_day_bounds(match_: &str) -> Option<(TimeData, TimeData)> {
    match to_lower(match_).as_str() {
        "ლანჩის მერე" | "ლანჩის შემდეგ" => Some((hour(false, 13), hour(false, 17))),
        "სამსახურის მერე" | "სამსახურის შემდეგ" => Some((hour(false, 17), hour(false, 21))),
        "სკოლის მერე" | "სკოლის შემდეგ" => Some((hour(false, 15), hour(false, 21))),
        _ => None,
    }
}

fn season_n(match_: &str) -> Option<i64> {
    match to_lower(match_).as_str() {
        "ეს" | "ამ" | "ახლანდელი" | "მიმდინარე" => Some(0),
        "წინა" | "ბოლო" | "გასული" | "გასულ" => Some(-1),
        "შემდეგი" | "მომავალი" => Some(1),
        _ => None,
    }
}

fn cycle_this_last_next_n(match_: &str) -> Option<i64> {
    match to_lower(match_).as_str() {
        "ეს" | "ამ" | "მიმდინარე" => Some(0),
        "წინა" | "გასული" | "გასულ" => Some(-1),
        "წინის წინ" | "წინის წინა" => Some(-2),
        "შემდეგ" | "შემდეგი" | "მომავალ" | "მომავალი" => Some(1),
        _ => None,
    }
}

fn cycle_this_last_next_interval_n(match_: &str) -> i64 {
    match to_lower(match_).as_str() {
        "ეს" | "ამ" | "მიმდინარე" => 0,
        "წინა" | "გასული" | "გასულ" => -1,
        "წინის წინ" | "წინის წინა" => -2,
        _ => 1,
    }
}

fn year_n(match_: &str) -> i64 {
    match to_lower(match_).as_str() {
        "წელს" => 0,
        "შარშან" | "გასული წელი" | "გასულ წელს" => -1,
        "შარშან წინ" | "შარშანწინ" => -2,
        _ => -3,
    }
}

fn part_of_month_bounds(match_: &str) -> Option<(i64, i64)> {
    match to_lower(match_).as_str() {
        "დასაწყისისკენ" | "დასაწყისიდან" | "დასაწყისას" | "დასაწყისში" | "დასაწყისი" => Some((1, 10)),
        "შუისკენ" | "შუაში" | "შუა" => Some((11, 20)),
        "ბოლომდე" | "ბოლოსკენ" | "ბოლოში" | "ბოლოს" | "ბოლო" => Some((21, -1)),
        _ => None,
    }
}

fn end_or_beginning_of_month_bounds(match_: &str) -> Option<(i64, i64)> {
    match to_lower(match_).as_str() {
        "დასაწყისი" | "დასაწყისში" => Some((1, 10)),
        "ბოლოს" | "ბოლოში" | "ბოლო" => Some((21, -1)),
        _ => None,
    }
}

fn end_or_beginning_of_year_bounds(match_: &str) -> Option<(i64, i64)> {
    match to_lower(match_).as_str() {
        "დასაწყისისკენ" | "დასაწყისში" => Some((1, 4)),
        "ბოლოსკენ" | "ბოლოს" | "ბოლოში" => Some((9, -1)),
        _ => None,
    }
}

fn end_or_beginning_of_week_bounds(match_: &str) -> Option<(i64, i64)> {
    match to_lower(match_).as_str() {
        "დასაწყისი" | "დასაწყისში" => Some((1, 3)),
        "ბოლოში" | "ბოლოს" => Some((5, 7)),
        _ => None,
    }
}

fn interval_dom_month(td_: &TimeData, a: &Token, b: &Token) -> Option<Token> {
    let dom1 = intersect_dom(td_, a)?;
    let dom2 = intersect_dom(td_, b)?;
    ttd(interval(Closed, &dom1, &dom2))
}

fn rules_main() -> Vec<Rule> {
    vec![
        rule(
            "intersect",
            vec![pred(not_latent_or_year), pred(not_latent_or_year)],
            |ts| {
                let a = td(ts[0])?;
                let b = td(ts[1])?;
                if !a.latent || !b.latent {
                    ttd(intersect(a, b).map(not_latent))
                } else {
                    None
                }
            },
        ),
        rule(
            "<ordinal> quarter",
            vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))],
            |ts| {
                let n = get_int_value(ts[0])?;
                let start = cycle_nth_after(true, G::Quarter, n - 1, &cycle_nth(G::Year, 0));
                let end = cycle_nth_after(true, G::Quarter, n, &cycle_nth(G::Year, 0));
                ttd(interval(Open, &start, &end))
            },
        ),
        rule(
            "last|past|next <duration>",
            vec![regex("ბოლო|უკანასკნელი?"), dimension(Dim::Duration)],
            |ts| ttd(interval(Closed, &duration_ago(dur(ts[1])?), &now())),
        ),
        rule(
            "in year",
            vec![pred(is_grain_of_time(G::Year)), regex(" ?წელს| ?წელი| ?წლის|-ში|ში")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        rule(
            "this|next <day-of-week>",
            vec![regex("შემდეგი|მომავალი"), pred(is_a_day_of_week)],
            |ts| tt(pred_nth(1, true, td(ts[1])?)),
        ),
        rule(
            "next <time>",
            vec![regex("შემდეგი?|მომავალი?"), pred(is_ok_with_this_next)],
            |ts| tt(pred_nth(1, true, td(ts[1])?)),
        ),
        rule(
            "this <time>",
            vec![regex("ამ|ეს|ახლანდელი|მიმდინარე"), pred(is_ok_with_this_next)],
            |ts| tt(pred_nth(0, false, td(ts[1])?)),
        ),
        rule(
            "last <time>",
            vec![regex("(წინა|ბოლო|გასული?)"), pred(is_ok_with_this_next)],
            |ts| tt(pred_nth(-1, false, td(ts[1])?)),
        ),
        rule(
            "<time> after next",
            vec![regex("(შემდეგის შემდეგი? ?)"), dimension(Dim::Time)],
            |ts| {
                let match_ = to_lower(&rx(ts[0], 0)?);
                tt(pred_nth(2, match_ == "შემდეგის შემდეგ", td(ts[1])?))
            },
        ),
        rule(
            "<time> before last",
            vec![regex("(ბოლოს წინა ?|წინის წინა ?)"), dimension(Dim::Time)],
            |ts| {
                let match_ = to_lower(&rx(ts[0], 0)?);
                tt(pred_nth(-2, match_ == "ბოლოს წინა", td(ts[1])?))
            },
        ),
        rule(
            "last weekend of <named-month>",
            vec![
                regex("ბოლო უიქ?კ? ?ენდი|ვიქ?კ? ?ენდი|შაბ?ფ?ათკვირა|შაბ?ფ?ათ-კვირა|უქმეები"),
                pred(is_a_month),
            ],
            |ts| tt(pred_last_of(&geo_weekend(), td(ts[1])?)),
        ),
        rule(
            "last weekend of <named-month>",
            vec![
                pred(is_a_month),
                regex("(ს? ?ბოლო უიქ?კ? ?ენდი ?|ს? ბოლო ?ვიქ?კ? ?ენდი ?|ს? ბოლო ?შაბ?ფ?ათკვირა ?|ს? ბოლო ?შაბ?ფ?ათ-კვირა ?)| ს?უქმეები ?"),
            ],
            |ts| tt(pred_last_of(&geo_weekend(), td(ts[0])?)),
        ),
        rule(
            "last workweek of <named-month>",
            vec![regex("ბოლო სამუშაო ?კვირას?(ში)?"), pred(is_a_month)],
            |ts| tt(pred_last_of(&workweek(), td(ts[1])?)),
        ),
        rule(
            "last workweek of <named-month>",
            vec![pred(is_a_month), regex("(ს? ბოლო სამუშაო ?კვირას?(ში)?)")],
            |ts| tt(pred_last_of(&workweek(), td(ts[0])?)),
        ),
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Time), dimension(Dim::Ordinal), dimension(Dim::Time)],
            |ts| {
                let n = ordinal_value(ts[1])? - 1;
                ttd(intersect(td(ts[2])?, td(ts[0])?).map(|x| pred_nth(n, false, &x)))
            },
        ),
        // Haskell prod is `(Token Ordinal:Token Time:_:Token Time)` against a
        // Time/regex/Ordinal/Time pattern — upstream never fires. Port as written.
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Time), regex("დან"), dimension(Dim::Ordinal), dimension(Dim::Time)],
            |ts| {
                let od = ordinal_value(ts[0])?;
                let td1 = td(ts[1])?;
                let td2 = td(ts[3])?;
                tt(pred_nth_after(od - 1, td1, td2))
            },
        ),
        rule("year (latent)", vec![pred(is_integer_between_tok(25, 10000))], |ts| {
            tt(mk_latent(year(get_int_value(ts[0])?)))
        }),
        // Haskell prod is `(_:token:_)` against a single-token pattern.
        rule("the <day-of-month> (number)", vec![pred(is_dom_integer)], |ts| {
            let n = get_int_value(ts.get(1)?)?;
            tt(mk_latent(day_of_month(n)))
        }),
        rule(
            "<named-month> <day-of-month> (non ordinal)",
            vec![pred(is_a_month), pred(is_dom_ordinal), regex("( დღეს?)?")],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
            tt(mk_latent(hour(true, get_int_value(ts[0])?)))
        }),
        rule(
            "at <time-of-day>",
            vec![pred(is_a_time_of_day), regex("ს|-ს|-ზე|ზე| ზე")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        rule(
            "<time-of-day> o'clock",
            vec![pred(is_a_time_of_day), regex("სა?ათი?ს?(ზე)?(ისთვის)?")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        rule("hh:mm", vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)")], |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(true, h, m))
        }),
        rule(
            "hhmm (latent)",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))([0-5]\\d)(?!.\\d)")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(mk_latent(hour_minute(true, h, m)))
            },
        ),
        rule(
            "hh:mm:ss",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)[:.]([0-5]\\d)")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let s = parse_int(&rx(ts[0], 2)?)?;
                tt(hour_minute_second(true, h, m, s))
            },
        ),
        rule(
            "hhmm (military) am|pm",
            vec![regex("((?:1[012]|0?\\d))([0-5]\\d)"), regex("([ap])\\.?m?\\.?")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let ap = to_lower(&rx(ts[1], 0)?);
                tt(time_of_day_ampm(ap == "a", &hour_minute(true, h, m)))
            },
        ),
        rule(
            "military spelled out numbers am|pm",
            vec![
                pred(is_integer_between_tok(10, 12)),
                pred(is_integer_between_tok(1, 59)),
                regex("([ap])(\\s|\\.)?m?\\.?"),
            ],
            |ts| {
                let hh = get_int_value(ts[0])?;
                let mm = get_int_value(ts[1])?;
                // Haskell `GroupMatch (_:ap:_)` reads the second capture group.
                let ap = to_lower(&rx(ts[2], 1)?);
                tt(time_of_day_ampm(ap == "a", &hour_minute(true, hh, mm)))
            },
        ),
        rule(
            "six thirty six a.m.",
            vec![pred(is_integer_between_tok(110, 999)), regex("([ap])(\\s|\\.)?m?\\.?")],
            |ts| {
                let n = get_int_value(ts[0])?;
                let m = n % 100;
                if m >= 60 {
                    return None;
                }
                let h = n / 100;
                let ap = to_lower(&rx(ts[1], 1)?);
                tt(time_of_day_ampm(ap == "a", &hour_minute(true, h, m)))
            },
        ),
        rule(
            "<time-of-day> am|pm",
            vec![pred(is_a_time_of_day), regex("(in the )?([ap])(\\s|\\.)?(m?)\\.?")],
            |ts| {
                let ap = to_lower(&rx(ts[1], 1)?);
                let m_empty = rx(ts[1], 3).as_deref() == Some("");
                let body = time_of_day_ampm(ap == "a", td(ts[0])?);
                if matches!(ts[0], Token::Time(t) if t.latent) && m_empty {
                    tt(mk_latent(body))
                } else {
                    tt(body)
                }
            },
        ),
        rule(
            "half to|till|before <hour-of-day>",
            vec![pred(is_an_hour_of_day), regex("ის ნახევა?რი?(ზე)?|-ის ნახევა?რი?(ზე)?")],
            |ts| ttd(minutes_before(30, td(ts[0])?)),
        ),
        rule(
            "integer after|past <hour-of-day>",
            vec![pred(is_an_hour_of_day), pred(is_integer_between_tok(1, 59)), regex("წუთი?(ზე)?(სთვის)?")],
            |ts| {
                let n = get_int_value(ts[1])?;
                ttd(minutes_before(60 - n, td(ts[0])?))
            },
        ),
        rule(
            "integer after|past <hour-of-day>",
            vec![
                pred(is_an_hour_of_day),
                regex("-ის|ის|საათ(სა)?(ზე)?(და )?"),
                pred(is_integer_between_tok(1, 59)),
                regex("წუთი?(ზე)?(სთვის)?"),
            ],
            |ts| {
                let n = get_int_value(ts[2])?;
                ttd(minutes_before(60 - n, td(ts[0])?))
            },
        ),
        rule(
            "integer after|past <hour-of-day>",
            vec![
                pred(is_an_hour_of_day),
                regex("საათი?(სა)?(ზე)?( ?და ?)?"),
                pred(is_integer_between_tok(1, 59)),
                regex("წუთი?(ზე)?(სთვის)?"),
            ],
            |ts| {
                let n = get_int_value(ts[2])?;
                ttd(minutes_after(n, td(ts[0])?))
            },
        ),
        rule(
            "<integer> to|till|before <hour-of-day>",
            vec![
                pred(is_an_hour_of_day),
                pred(is_integer_between_tok(1, 59)),
                regex("(წუთი ?)აკლ(ია)?(და)?|(წუთი ?)უკლია"),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                ttd(minutes_before(n, td(ts[0])?))
            },
        ),
        rule(
            "<integer> to|till|before <hour-of-day>",
            vec![
                pred(is_an_hour_of_day),
                regex("რო"),
                pred(is_integer_between_tok(1, 59)),
                regex("(წუთი ?)აკლ(ია)?(და)?|(წუთი ?)უკლია"),
            ],
            |ts| {
                let n = get_int_value(ts[2])?;
                ttd(minutes_before(n, td(ts[0])?))
            },
        ),
        rule(
            "in|during the <part-of-day>",
            vec![pred(is_a_part_of_day), regex("(-ის )?(ის )?განმავლობაში")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        rule(
            "this <part-of-day>",
            vec![regex("დღეს ?|ამ ?"), pred(is_a_part_of_day)],
            |ts| {
                ttd(intersect(&cycle_nth(G::Day, 0), td(ts[1])?).map(|x| part_of_day(not_latent(x))))
            },
        ),
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
        rule("mm/yyyy", vec![regex("(0?[1-9]|1[0-2])[/-](\\d{4})")], |ts| {
            let y = parse_int(&rx(ts[0], 1)?)?;
            let m = parse_int(&rx(ts[0], 0)?)?;
            tt(year_month_day(y, m, 1))
        }),
        rule(
            "part of days",
            vec![regex(
                "(გვიან ღამე?(ით)?|დილა?ს?(ის)?(ით)?|საღამოს?(თი)?|(შუა)?ღამე?(ით)?ი?ს?(ისას)?|შუადღის|შუადღით|დღის)",
            )],
            |ts| {
                let (start, end) = part_of_days_bounds(&rx(ts[0], 0)?);
                ttd(interval(Open, &start, &end).map(|x| part_of_day(mk_latent(x))))
            },
        ),
        rule("early morning", vec![regex("დილაუთენია|დილაუთენია")], |_| latent_pod(4, 9)),
        rule(
            "tonight",
            vec![regex("(დღეს )?(გვიან )?ღამე?(ით)?(ისას)?|(დღეს )?(შუა )?ღამე?(ით)?ი?ს?(ისას)?|საღამოს")],
            |ts| {
                let h = if to_lower(&rx(ts[0], 0)?) == "გვიან " { 21 } else { 18 };
                let evening = interval(Open, &hour(false, h), &hour(false, 0))?;
                ttd(intersect(&cycle_nth(G::Day, 0), &evening).map(|x| part_of_day(not_latent(x))))
            },
        ),
        rule(
            "after lunch/work/school",
            vec![regex("((სკოლის|ლანჩის|სამსახურის) (მერე|შემდეგ))")],
            |ts| {
                let (start, end) = after_part_of_day_bounds(&rx(ts[0], 0)?)?;
                let pod = interval(Open, &start, &end)?;
                ttd(intersect(&cycle_nth(G::Day, 0), &pod).map(|x| part_of_day(not_latent(x))))
            },
        ),
        rule("<time> <part-of-day>", vec![dimension(Dim::Time), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(td(ts[1])?, td(ts[0])?))
        }),
        rule("<part-of-day> <time>", vec![pred(is_a_part_of_day), dimension(Dim::Time)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule(
            "week-end",
            vec![regex(
                "(მიმდინარე )?ვიქ?კ? ?ენდი?(ზე)?(სას)?|(მიმდინარე )?უიქ?კ? ?ენდი?(ზე)?(სას)?|(მიმდინარე )?შაბ?ფ?ათ ?-?კვირას?|(მიმდინარე )?უქმეები?(ზე)?",
            )],
            |_| tt(mk_ok_for_this_next(geo_weekend())),
        ),
        rule(
            "<time-of-day> sharp|exactly",
            vec![regex("(ზუსტად|იმენა)"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        rule(
            "about|exactly <time-of-day>",
            vec![regex("დაახლოებით"), pred(is_grain_finer_than(G::Year))],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        rule(
            "<month> dd-dd (interval)",
            vec![
                pred(is_a_month),
                pred(is_dom_value),
                regex("-დან|დან| დან"),
                pred(is_dom_value),
                regex("-მდე|მდე| მდე"),
            ],
            |ts| interval_dom_month(td(ts[0])?, ts[1], ts[3]),
        ),
        rule(
            "<month> dd-dd (interval)",
            vec![pred(is_a_month), pred(is_dom_value), regex("-"), pred(is_dom_value)],
            |ts| interval_dom_month(td(ts[0])?, ts[1], ts[3]),
        ),
        rule(
            "dd-dd <month> (interval)",
            vec![
                pred(is_dom_value),
                regex("-დან|დან| დან"),
                pred(is_dom_value),
                regex("-მდე|მდე| მდე"),
                pred(is_a_month),
            ],
            |ts| interval_dom_month(td(ts[4])?, ts[0], ts[2]),
        ),
        rule(
            "dd-dd <month> (interval)",
            vec![pred(is_dom_value), regex("-"), pred(is_dom_value), pred(is_a_month)],
            |ts| interval_dom_month(td(ts[3])?, ts[0], ts[2]),
        ),
        rule(
            "between <time> and <time>",
            vec![
                dimension(Dim::Time),
                regex("-ი?დან ?|ი?დან ?| ი?დან ?"),
                dimension(Dim::Time),
                regex("-ა?მდე ?|ა?მდე ?| ა?მდე ?"),
            ],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "between <time> and <time> 1",
            vec![
                dimension(Dim::Time),
                regex("-ი?დან ?|ი?დან ?| ი?დან ?"),
                regex("დღემდე|ამ წუთამდე|ახლამდე|ამ მომენტამდე"),
            ],
            |ts| ttd(interval(Closed, td(ts[0])?, &now())),
        ),
        rule(
            "between <time-of-day> and <time-of-day>",
            vec![
                dimension(Dim::Time),
                pred(is_a_part_of_day),
                regex("დან|-ი?დან ?|ი?დან ?| ი?დან ?"),
                dimension(Dim::Time),
                pred(is_a_part_of_day),
                regex("მდე|-ა?მდე ?|ა?მდე ?| ა?მდე ?"),
            ],
            |ts| {
                let dom1 = intersect(td(ts[1])?, td(ts[0])?)?;
                let dom2 = intersect(td(ts[4])?, td(ts[3])?)?;
                ttd(interval(Closed, &dom1, &dom2))
            },
        ),
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![
                pred(not_latent_tod),
                regex("დან|-ი?დან ?|ი?დან ?| ი?დან ?"),
                pred(is_a_time_of_day),
                regex("მდე|-ა?მდე ?|ა?მდე ?| ა?მდე ?"),
            ],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "between <time-of-day> and <time-of-day> (interval)",
            vec![pred(is_a_time_of_day), regex(" ?და ?"), pred(is_a_time_of_day), regex("შორის")],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "from <time-of-day> - <time-of-day> (interval)",
            vec![
                pred(is_a_time_of_day),
                regex("დან|-ი?დან ?|ი?დან ?| ი?დან ?"),
                pred(is_a_time_of_day),
                regex("მდე|-ა?მდე ?|ა?მდე ?| ა?მდე ?"),
            ],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // Haskell prod is `(_:Token Time:_)` against Time/regex.
        rule(
            "by <time>",
            vec![dimension(Dim::Time), regex("მდე|-ა?მდე ?|ა?მდე ?| ა?მდე ?")],
            |ts| ttd(interval(Open, &now(), td(ts[1])?)),
        ),
        rule(
            "the <cycle> after|before <time>",
            vec![dimension(Dim::Time), regex("(დან|ა?მდე)"), dimension(Dim::TimeGrain)],
            |ts| {
                let n = if to_lower(&rx(ts[1], 0)?) == "დან" { 1 } else { -1 };
                tt(cycle_nth_after(false, grain_of(ts[2])?, n, td(ts[0])?))
            },
        ),
        rule(
            "the <cycle> after <time>",
            vec![
                dimension(Dim::Time),
                regex("დან"),
                pred(is_integer_between_tok(0, 99999)),
                dimension(Dim::TimeGrain),
                regex("ში|ის შემდეგ"),
            ],
            |ts| {
                let n = get_int_value(ts[2])?;
                tt(cycle_nth_after(false, grain_of(ts[3])?, n, td(ts[0])?))
            },
        ),
        rule(
            "this|last|next <cycle>",
            vec![
                regex("(ეს|ამ|მიმდინარე|შემდეგი?|მომავალი?|წინა|წინის წინა?|გასული?)"),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let n = cycle_this_last_next_n(&rx(ts[0], 0)?)?;
                let g = grain_of(ts[1])?;
                // Same span as `ruleCycleThisLastNextInterval`. A lone
                // `cycleNth` loses ranking to nothing and emits a point value;
                // fixtures expect the full cycle as an interval.
                ttd(interval(Open, &cycle_nth(g, n), &cycle_nth(g, n + 1)))
            },
        ),
        rule(
            "<day-of-month> (ordinal or number) of <month>",
            vec![pred(is_dom_value), pred(is_grain_of_time(G::Month))],
            |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
        ),
        rule(
            "<ordinal> <cycle> of <time>",
            vec![dimension(Dim::Time), regex("დან"), dimension(Dim::Ordinal), dimension(Dim::TimeGrain)],
            |ts| {
                let n = get_int_value(ts[2])?;
                tt(cycle_nth_after(true, grain_of(ts[3])?, n - 1, td(ts[0])?))
            },
        ),
        rule(
            "<ordinal> last <cycle> of <time>",
            vec![dimension(Dim::Time), regex("ბოლოდან"), dimension(Dim::Ordinal), dimension(Dim::TimeGrain)],
            |ts| {
                let n = get_int_value(ts[2])?;
                let grain = grain_of(ts[3])?;
                let base = td(ts[0])?;
                tt(cycle_nth_after(true, grain, -n, &cycle_nth_after(true, base.time_grain, 1, base)))
            },
        ),
        rule(
            "in|within|after <duration>",
            vec![dimension(Dim::Duration), regex("(განმავლობაში|შემდეგ)")],
            |ts| match to_lower(&rx(ts[1], 0)?).as_str() {
                "განმავლობაში" => ttd(interval(Open, &now(), &in_duration(dur(ts[0])?))),
                "შემდეგ" => tt(with_direction(IntervalDirection::After, in_duration(dur(ts[0])?))),
                _ => None,
            },
        ),
        rule(
            "<integer> <named-day> ago|back",
            vec![pred(is_natural), pred(is_a_day_of_week), regex(" ?დღის წინ| ?დღის უკან")],
            |ts| match ts[0] {
                Token::Numeral(n) => tt(pred_nth(-(n.value.floor() as i64), false, td(ts[1])?)),
                _ => None,
            },
        ),
        rule(
            "last|past|next <duration>",
            vec![regex("(წინა|მომდევნო|გასული?|მომავალ)"), dimension(Dim::Duration)],
            |ts| {
                let d = dur(ts[1])?;
                match to_lower(&rx(ts[0], 0)?).as_str() {
                    "მომდევნო" | "მომავალ" => tt(cycle_n(true, d.grain, d.value)),
                    "წინა" | "გასული" | "გასულ" => tt(cycle_n(true, d.grain, -d.value)),
                    _ => None,
                }
            },
        ),
        rule(
            "<duration> hence|ago",
            vec![dimension(Dim::Duration), regex("წინ|უკან")],
            |ts| match to_lower(&rx(ts[1], 0)?).as_str() {
                "წინ" | "უკან" => tt(duration_ago(dur(ts[0])?)),
                _ => tt(in_duration(dur(ts[0])?)),
            },
        ),
        rule(
            "<day> <duration> hence|ago",
            vec![pred(day_or_month_grain), dimension(Dim::Duration), regex("(წინ|უკან)")],
            |ts| match to_lower(&rx(ts[2], 0)?).as_str() {
                "უკან" => ttd(intersect(td(ts[0])?, &duration_interval_ago(dur(ts[1])?))),
                _ => ttd(intersect(td(ts[0])?, &in_duration_interval(dur(ts[1])?))),
            },
        ),
        rule(
            "<duration> after|before|from <time>",
            vec![dimension(Dim::Time), dimension(Dim::Duration), regex("(წინ)")],
            |ts| match to_lower(&rx(ts[2], 0)?).as_str() {
                "წინ" => tt(duration_before(dur(ts[1])?, td(ts[0])?)),
                _ => tt(duration_after(dur(ts[1])?, td(ts[0])?)),
            },
        ),
        rule(
            "<duration> after|before|from <time>",
            vec![dimension(Dim::Duration), regex("(წინ)")],
            |ts| match to_lower(&rx(ts[1], 0)?).as_str() {
                "წინ" => tt(duration_before(dur(ts[0])?, &now())),
                _ => tt(duration_after(dur(ts[0])?, &now())),
            },
        ),
        rule(
            "in <number> (implicit minutes)",
            vec![pred(is_integer_between_tok(0, 60)), regex(" ?წუთში")],
            |ts| match ts[0] {
                Token::Numeral(n) => tt(in_duration(&duration(G::Minute, n.value.floor() as i64))),
                _ => None,
            },
        ),
        rule(
            "this|last|next <cycle>",
            vec![
                regex("(ეს|ამ|მიმდინარე|შემდეგი?|მომავალი?|წინა|წინის წინა?|გასული?|მომდევნო)"),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let n = cycle_this_last_next_interval_n(&rx(ts[0], 0)?);
                let g = grain_of(ts[1])?;
                // Haskell `interval Closed (cycleNth n) (cycleNth n)` resolves as
                // the full cycle. Open n .. n+1 is the same span and evaluates
                // cleanly when start and end are distinct cycles.
                ttd(interval(Open, &cycle_nth(g, n), &cycle_nth(g, n + 1)))
            },
        ),
        rule(
            "part of <named-month>",
            vec![
                pred(month_or_month_grain),
                regex("(ის )?(დასაწყის(ი|ში|ისას|ისკენ)|შუ(ა|აში|ისკენ)|ბოლო(ს|მდე|ში|სკენ)?)"),
            ],
            |ts| {
                let (sd, ed) = part_of_month_bounds(&rx(ts[1], 1)?)?;
                let start = intersect(td(ts[0])?, &day_of_month(sd))?;
                let end = if ed != -1 {
                    intersect(td(ts[0])?, &day_of_month(ed))?
                } else {
                    cycle_last_of(G::Day, td(ts[0])?)
                };
                ttd(interval(Open, &start, &end))
            },
        ),
        rule(
            "at the beginning|end of <year>",
            vec![
                pred(is_grain_of_time(G::Year)),
                regex("(წლის )?დასაწყისი?(ისკენ)?(ში)? |ბოლოს?(კენ)?(ში)? "),
            ],
            |ts| {
                let (sd, ed) = end_or_beginning_of_year_bounds(&rx(ts[1], 1)?)?;
                let start = intersect(td(ts[0])?, &month(sd))?;
                let end = if ed != -1 {
                    intersect(td(ts[0])?, &cycle_last_of(G::Month, &month(ed)))?
                } else {
                    cycle_nth_after(false, G::Year, 1, &intersect(td(ts[0])?, &month(1))?)
                };
                ttd(interval(Open, &start, &end))
            },
        ),
        rule(
            "at the beginning|end of <named-month>",
            vec![pred(month_or_month_grain), regex("(ის )?(დასაწყისშ?ი|ბოლო(ს?|ში))")],
            |ts| {
                let (sd, ed) = end_or_beginning_of_month_bounds(&rx(ts[1], 1)?)?;
                let start = intersect(td(ts[0])?, &day_of_month(sd))?;
                let end = if ed != -1 {
                    intersect(td(ts[0])?, &day_of_month(ed))?
                } else {
                    cycle_last_of(G::Day, td(ts[0])?)
                };
                ttd(interval(Open, &start, &end))
            },
        ),
        rule(
            "at the beginning|end of <week>",
            vec![pred(is_grain_of_time(G::Week)), regex("(დასაწყისი|დასაწყისში|ბოლოში|ბოლოს)")],
            |ts| {
                let (sd, ed) = end_or_beginning_of_week_bounds(&rx(ts[1], 1)?)?;
                let start = intersect(td(ts[0])?, &day_of_week(sd))?;
                let end = intersect(td(ts[0])?, &day_of_week(ed))?;
                ttd(interval(Open, &start, &end))
            },
        ),
        rule("now", vec![regex("ახლავე|ახლა|ამწუთას|ამ მომენტისთვის")], |_| tt(now())),
        rule(
            "last|this|next <season>",
            vec![regex("(ამ|ეს|ახლანდელი|მიმდინარე|შემდეგი|მომავალი|წინა|ბოლო|გასული?) სეზონი?(ზე)?")],
            |ts| tt(pred_nth(season_n(&rx(ts[0], 0)?)?, false, &season())),
        ),
        rule(
            "end of month",
            vec![regex("(ამ )?(მიმდინარე )?თვის ბოლოს?(კენ)?(თვის)?")],
            |_| {
                let start = intersect(&day_of_month(21), &cycle_nth(G::Month, 0))?;
                let end = cycle_last_of(G::Day, &cycle_nth(G::Month, 0));
                ttd(interval(Open, &start, &end))
            },
        ),
        rule(
            "beginning of month",
            vec![regex("(ამ )?(მიმდინარე )?თვის დასაწყისი?(ში)?(სკენ)?(სთვის)?")],
            |_| {
                let start = intersect(&day_of_month(1), &cycle_nth(G::Month, 0))?;
                let end = intersect(&day_of_month(10), &cycle_nth(G::Month, 0))?;
                ttd(interval(Open, &start, &end))
            },
        ),
        rule(
            "end of year",
            vec![regex("((მიმდინარე )?(ამ )?წლის ბოლოს?(კენ)?(ში)?(თვის)?)")],
            |_| {
                let start = intersect(&month(9), &cycle_nth(G::Year, 0))?;
                let end = intersect(&month(1), &cycle_nth(G::Year, 1))?;
                ttd(interval(Open, &start, &end))
            },
        ),
        rule(
            "beginning of year",
            vec![regex("((ამ )?(მიმდინარე )?წლის დასაწყისი?(ში)?(სკენ)?(სთვის)?)")],
            |_| {
                let start = intersect(&month(1), &cycle_nth(G::Year, 0))?;
                let end = intersect(&month(4), &cycle_nth(G::Year, 0))?;
                ttd(interval(Open, &start, &end))
            },
        ),
        // Haskell pattern is `isGrainCoarserThan Day` (Time token) but prod
        // matches `Token TimeGrain`. Upstream never fires. Port as written.
        rule("<TimeGrain>", vec![pred(is_grain_coarser_than(G::Day))], |ts| {
            let grain = grain_of(ts[0])?;
            ttd(interval(Closed, &cycle_nth(grain, 0), &now()))
        }),
        rule(
            "this|last|next year",
            vec![regex("(შარშან ?წინის? ?წინ|შარშან ?წინ|შარშან|გასული? წელი?ს?|წელს)")],
            |ts| {
                let n = year_n(&rx(ts[0], 0)?);
                ttd(interval(Open, &cycle_nth(G::Year, n), &cycle_nth(G::Year, n + 1)))
            },
        ),
        rule(
            "between <time> and <time>",
            vec![
                pred(is_dom_value),
                regex("-ი?დან ?|ი?დან ?| ი?დან ?"),
                pred(is_dom_value),
                pred(is_a_month),
                regex("-ა?მდე ?|ა?მდე ?| ა?მდე ?"),
            ],
            |ts| interval_dom_month(td(ts[3])?, ts[0], ts[2]),
        ),
        rule(
            "work-week",
            vec![regex("(მიმდინარე )?სამუშაო ?კვირას?(ში)?")],
            |_| tt(mk_ok_for_this_next(workweek())),
        ),
        rule("in <named-month>", vec![pred(is_a_month), regex("ში")], |ts| {
            let td2 = td(ts[0])?;
            ttd(interval(
                Closed,
                &cycle_nth_after(false, G::Month, 0, td2),
                &cycle_nth_after(false, G::Month, 0, td2),
            ))
        }),
        rule(
            "dd/mm",
            vec![regex("(3[01]|[12]\\d|0?[1-9])\\s?[/-]\\s?(1[0-2]|0?[1-9])")],
            |ts| {
                let d = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(month_day(m, d))
            },
        ),
        rule(
            "dd/mm/yyyy",
            vec![regex("(3[01]|[12]\\d|0?[1-9])[-/\\s](1[0-2]|0?[1-9])[-/\\s](\\d{2,4})")],
            |ts| {
                let d = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let y = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        rule(
            "mm.dd.yyyy",
            vec![regex("(1[0-2]|0?[1-9])\\.(3[01]|[12]\\d|0?[1-9])\\.(\\d{4})")],
            |ts| {
                let m = parse_int(&rx(ts[0], 0)?)?;
                let d = parse_int(&rx(ts[0], 1)?)?;
                let y = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        rule(
            "<integer> year",
            vec![pred(is_natural), regex("წელიწად(ის|ი|ში)?|წლ(ის)?|წელ(შ?ი|ს)")],
            |ts| {
                let n = get_int_value(ts[0])?;
                ttd(interval(Open, &year(n), &year(n + 1)))
            },
        ),
    ]
}

fn rules_instants() -> Vec<Rule> {
    mk_rule_instants(&[
        ("ახლა", G::Second, 0, "ახლავე|ეხლავე|ეხლა|ახლა|ამ წამს|ამ წუთას|ამ მომენტისთვი"),
        ("დღეს", G::Day, 0, "დღეს"),
        ("ხვალ", G::Day, 1, "ხვალე?"),
        ("ზეგ", G::Day, 2, "ზეგ"),
        ("მაზეგ", G::Day, 3, "მაზეგ"),
        ("გუშინ", G::Day, -1, "გუშინ?"),
        ("გუშინწინ", G::Day, -2, "გუშინ ?წინ"),
    ])
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("ორშაბათი", "ორშაბათი?ს?"),
        ("სამშაბათი", "სამშაბათი?ს?"),
        ("ოთხშაბათი", "ოთხშაბათი?ს?"),
        ("ხუთშაბათი", "ხუთშაბათი?ს?"),
        ("პარასკევი", "პარასკევი?ს?"),
        ("შაბათი", "შაბათი?ს?"),
        ("კვირა", "კვირას?|კვირის"),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months_with_latent(&[
        ("იანვარი", "იანვა?რი?ს?(ის)?", false),
        ("თებერვალი", "თებერვა?ლი?ს?(ის)?", false),
        ("მარტი", "მარტი?ს?(ის)?", false),
        ("აპრილი", "აპრილი?ს?(ის)?", false),
        ("მაისი", "მაისი?ს?(ის)?", false),
        ("ივნისი", "ივნისი?ს?(ის)?", false),
        ("ივლისი", "ივლისი?ს?(ის)?", false),
        ("აგვისტო", "აგვისტო?ს?", false),
        ("სექტემბერი", "სექტემბე?რის|სექტემბე?რი?ს?", false),
        ("ოქტომბერი", "ოქტომბე?რის|ოქტომბე?რი?ს?", false),
        ("ნოემბერი", "ნოემბე?რის|ნოემბე?რი?ს?", false),
        ("დეკემბერი", "დეკემბრის?|დეკემბე?რი?ს?", false),
    ])
}

fn rules_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("ზაფხული", "ზაფხული?(ში)?ს?", month_day(6, 1), month_day(8, 31)),
        ("შემოდგომა", "შემოდგომა?(ში)?ს?", month_day(9, 1), month_day(11, 30)),
        ("ზამთარი", "ზამთარი?(ში)?ს?", month_day(12, 1), month_day(2, 28)),
        ("გაზაფხული", "გაზაფხული?(ში)?ს?", month_day(3, 1), month_day(5, 31)),
    ])
}

fn rules_computed_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        (
            "Orthodox Easter Monday",
            "orthodox\\s+easter\\s+mon(day)?",
            cycle_nth_after(false, G::Day, 1, &orthodox_easter()),
        ),
        ("აღდგომა", "აღდგომი?ა?ს?(ისას)?", orthodox_easter()),
        ("დიდი შაბათი", "დიდი? შაბათი?ს?(ისას)?", cycle_nth_after(false, G::Day, -1, &orthodox_easter())),
        (
            "წითელი პარასკევი",
            "წითელი? პარასკევი?ს?(სას)?",
            cycle_nth_after(false, G::Day, -2, &orthodox_easter()),
        ),
        (
            "დიდი ხუთშაბათი",
            "დიდი? ხუთშაბათი?ს?(ისას)?",
            cycle_nth_after(false, G::Day, -7, &orthodox_easter()),
        ),
    ])
}

fn rules_periodic_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("ახალი წელი", "ახალი წე?ლი?ს?", month_day(1, 1)),
        ("შობა", "შობა?(ი)?ს?", month_day(1, 7)),
        ("ნათლისღება", "ნათლისღება?(ი)?ს?", month_day(1, 19)),
        ("დედის დღე", "დედის დღე?ს?", month_day(3, 3)),
        ("ქალთა საერთაშორისო დღე", "ქალთა (საერთაშორისო )?დღე?ს?", month_day(3, 8)),
        (
            "საქართველოს დამოუკიდებლობის აღდგენის დღე",
            "საქართველოს (თავისუფლებისა და ერთიანობისთვის დაღუპულთა მოხსენიების)?(დამოუკიდებლობის აღდგენის )?დღეს?",
            month_day(4, 9),
        ),
        ("გამარჯვების დღე", "(ფაშიზმზე)?გამარჯვების დღეს?", month_day(5, 9)),
        (
            "წმინდა ანდრია პირველწოდებულის ხსენების დღე",
            "(წმინდა)?ანდრია პირველწოდებულის ხსენების დღე?ს?",
            month_day(5, 12),
        ),
        ("დამოუკიდებლობის დღე", "(საქართველოს)?დამოუკიდებლობის დღე?ს?", month_day(5, 26)),
        ("მცხეთობა", "მცხეთობა?ს?", month_day(10, 14)),
        ("გიორგობა", "გიორგობა?ს?", month_day(11, 26)),
    ])
}

fn build_rules() -> Vec<Rule> {
    let mut v = rules_main();
    v.extend(rules_instants());
    v.extend(rules_days_of_week());
    v.extend(rules_months());
    v.extend(rules_seasons());
    v.extend(rules_computed_holidays());
    v.extend(rules_periodic_holidays());
    v
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &RULES
}
