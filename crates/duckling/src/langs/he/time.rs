//! `Duckling.Time.HE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::DurationData;
use crate::dims::time::helpers::*;
use crate::dims::time::types::{
    Form, IntervalDirection, TimeData,
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

fn tod(t: &Token) -> Option<(i64, bool)> {
    match t {
        Token::Time(TimeData { form: Some(Form::TimeOfDay { hours: Some(h), is12h }), .. }) => {
            Some((*h, *is12h))
        }
        _ => None,
    }
}

/// `Duckling.Duration.Helpers.isGrain`.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

/// `Token Time . partOfDay . mkLatent <$> interval Open (hour False a) (hour False b)`.
fn latent_pod(a: i64, b: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, a), &hour(false, b)).map(|x| part_of_day(mk_latent(x))))
}

fn rule_next_dayofweek() -> Rule {
    rule(
        "next <day-of-week>",
        vec![pred(is_a_day_of_week), regex("(הבא(ה)?)")],
        |ts| tt(pred_nth(1, true, td(ts[0])?)),
    )
}

fn rule_namedday() -> Rule {
    rule("ב <named-day>", vec![regex("ב"), pred(is_a_day_of_week)], |ts| Some(ts[1].clone()))
}

fn rule_at_hour_timeofday() -> Rule {
    rule(
        "at hour <time-of-day>",
        vec![regex("בשעה"), pred(is_a_time_of_day)],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    )
}

fn rule_hourofday_and_integer() -> Rule {
    rule(
        "<hour-of-day> and <integer>",
        vec![pred(is_an_hour_of_day), regex("ו"), pred(is_integer_between_tok(1, 59))],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[2])?))
        },
    )
}

fn rule_hourofday_and_quarter() -> Rule {
    rule(
        "<hour-of-day> and quarter",
        vec![pred(is_an_hour_of_day), regex("ו"), regex("רבע(י)?")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 15))
        },
    )
}

fn rule_hourofday_and_half() -> Rule {
    rule(
        "<hour-of-day> and half",
        vec![pred(is_an_hour_of_day), regex("ו"), regex("חצי|מחצית")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 30))
        },
    )
}

fn rule_hourofday_integer() -> Rule {
    rule(
        "<hour-of-day> <integer>",
        vec![pred(is_an_hour_of_day), pred(is_integer_between_tok(1, 59))],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[1])?))
        },
    )
}

fn rule_hourofday_quarter() -> Rule {
    rule(
        "<hour-of-day> quarter",
        vec![pred(is_an_hour_of_day), regex("רבע(י)?")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 15))
        },
    )
}

fn rule_hourofday_half() -> Rule {
    rule(
        "<hour-of-day> half",
        vec![pred(is_an_hour_of_day), regex("חצי|מחצית")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 30))
        },
    )
}

fn rule_integer_totillbefore_integer_hourofday() -> Rule {
    rule(
        "<integer> to|till|before <integer> (hour-of-day)",
        vec![pred(is_integer_between_tok(1, 59)), regex("לפני|ל"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_before(get_int_value(ts[0])?, td(ts[2])?)),
    )
}

fn rule_quarter_totillbefore_integer_hourofday() -> Rule {
    rule(
        "quarter to|till|before <integer> (hour-of-day)",
        vec![regex("רבע(י)?"), regex("לפני|ל"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_before(15, td(ts[2])?)),
    )
}

fn rule_half_totillbefore_integer_hourofday() -> Rule {
    rule(
        "half to|till|before <integer> (hour-of-day)",
        vec![regex("חצי|מחצית"), regex("לפני|ל"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_before(30, td(ts[2])?)),
    )
}

fn rule_integer_afterpast_integer_hourofday() -> Rule {
    rule(
        "integer after|past <integer> (hour-of-day)",
        vec![pred(is_integer_between_tok(1, 59)), regex("אחרי"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_after(get_int_value(ts[0])?, td(ts[2])?)),
    )
}

fn rule_quarter_afterpast_integer_hourofday() -> Rule {
    rule(
        "quarter after|past <integer> (hour-of-day)",
        vec![regex("רבע(י)?"), regex("אחרי"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_after(15, td(ts[2])?)),
    )
}

/// Upstream production matches `_:Token Time:_` on `[regex, regex, Time]`, so it never fires.
fn rule_half_afterpast_integer_hourofday() -> Rule {
    rule(
        "half after|past <integer> (hour-of-day)",
        vec![regex("חצי|מחצית"), regex("אחרי"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_after(30, td(ts[1])?)),
    )
}

fn rule_namedday2() -> Rule {
    rule("named-day", vec![regex("(יום )?שני")], |_| tt(day_of_week(1)))
}

fn rule_since_timeofday() -> Rule {
    rule(
        "since <time-of-day>",
        vec![regex("מ"), dimension(Dim::Time)],
        |ts| tt(with_direction(IntervalDirection::After, td(ts[1])?.clone())),
    )
}

fn rule_last_time() -> Rule {
    rule(
        "last <time>",
        vec![dimension(Dim::Time), regex("שעבר|(ה)?קודם|(ה)?אחרון")],
        |ts| tt(pred_nth(-1, false, td(ts[0])?)),
    )
}

fn rule_namedday6() -> Rule {
    rule("named-day", vec![regex("(יום )?שישי")], |_| tt(day_of_week(5)))
}

fn rule_datetime_datetime_interval() -> Rule {
    rule(
        "<datetime> - <datetime> (interval)",
        vec![pred(is_not_latent), regex("\\-|ע(ד)?"), pred(is_not_latent)],
        |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_the_dayofmonth_non_ordinal() -> Rule {
    rule(
        "the <day-of-month> (non ordinal)",
        vec![regex("ה/S"), pred(is_dom_integer)],
        |ts| tt(mk_latent(day_of_month(get_int_value(ts[1])?))),
    )
}

fn rule_cycle_after_time() -> Rule {
    rule(
        "<cycle> after <time>",
        vec![dimension(Dim::TimeGrain), regex("אחרי"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, 1, td(ts[2])?)),
    )
}

fn rule_in_duration() -> Rule {
    rule("in <duration>", vec![regex("בעוד"), dimension(Dim::Duration)], |ts| {
        tt(in_duration(dur(ts[1])?))
    })
}

fn rule_in_namedmonth() -> Rule {
    rule("in <named-month>", vec![regex("ב"), pred(is_a_month)], |ts| Some(ts[1].clone()))
}

fn rule_now() -> Rule {
    rule("now", vec![regex("עכשיו|מייד")], |_| tt(now()))
}

fn rule_current_dayofweek() -> Rule {
    rule(
        "current <day-of-week>",
        vec![pred(is_a_day_of_week), regex("(הזה|הזאת|הקרוב(ה)?)")],
        |ts| tt(pred_nth(0, true, td(ts[0])?)),
    )
}

fn rule_from_datetime_datetime_interval() -> Rule {
    rule(
        "from <datetime> - <datetime> (interval)",
        vec![regex("מ|משעה"), dimension(Dim::Time), regex("\\-|עד"), dimension(Dim::Time)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_namedday4() -> Rule {
    rule("named-day", vec![regex("(יום )?רביעי")], |_| tt(day_of_week(3)))
}

fn rule_the_cycle_after_time() -> Rule {
    rule(
        "the <cycle> after <time>",
        vec![regex("ה"), dimension(Dim::TimeGrain), regex("אחרי"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, 1, td(ts[3])?)),
    )
}

fn rule_the_cycle_before_time() -> Rule {
    rule(
        "the <cycle> before <time>",
        vec![regex("ה"), dimension(Dim::TimeGrain), regex("לפני"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, -1, td(ts[3])?)),
    )
}

fn rule_last_dayofweek() -> Rule {
    rule(
        "last <day-of-week>",
        vec![pred(is_a_day_of_week), regex("(שעבר(ה)?|הקודמת|הקודם)")],
        |ts| tt(pred_nth(0, true, td(ts[0])?)),
    )
}

fn rule_the_ides_of_namedmonth() -> Rule {
    rule(
        "the ides of <named-month>",
        vec![regex("באמצע"), pred(is_a_month)],
        |ts| match td(ts[1])?.form {
            Some(Form::Month(m)) => {
                let d = if [3, 5, 7, 10].contains(&m) { 15 } else { 13 };
                ttd(intersect(td(ts[1])?, &day_of_month(d)))
            }
            _ => None,
        },
    )
}

fn rule_noon() -> Rule {
    rule("noon", vec![regex("(ב)?צהריים")], |_| tt(hour(false, 12)))
}

fn rule_today() -> Rule {
    rule("today", vec![regex("היום")], |_| tt(today()))
}

fn rule_between_timeofday_and_timeofday_interval() -> Rule {
    rule(
        "between <time-of-day> and <time-of-day> (interval)",
        vec![regex("בין"), pred(is_a_time_of_day), regex("ל"), pred(is_a_time_of_day)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_next_cycle() -> Rule {
    rule(
        "next <cycle>",
        vec![dimension(Dim::TimeGrain), regex("הבא(ה)?")],
        |ts| tt(cycle_nth(grain_of(ts[0])?, 1)),
    )
}

fn rule_for_duration() -> Rule {
    rule("for <duration>", vec![regex("תוך"), dimension(Dim::Duration)], |ts| {
        tt(in_duration(dur(ts[1])?))
    })
}

fn rule_duration_from_now() -> Rule {
    rule(
        "<duration> from now",
        vec![dimension(Dim::Duration), regex("מעכשיו")],
        |ts| tt(in_duration(dur(ts[0])?)),
    )
}

fn rule_lunch() -> Rule {
    rule("lunch", vec![regex("(ב)?צהריים")], |_| latent_pod(12, 14))
}

fn rule_last_cycle() -> Rule {
    rule(
        "last <cycle>",
        vec![dimension(Dim::TimeGrain), regex("האחרון|האחרונה|שעבר|שעברה")],
        |ts| tt(cycle_nth(grain_of(ts[0])?, -1)),
    )
}

fn rule_afternoon() -> Rule {
    rule("afternoon", vec![regex("אחה(״)?צ|אחר הצהריים")], |_| latent_pod(12, 19))
}

fn rule_namedmonth_dayofmonth_ordinal() -> Rule {
    rule(
        "<named-month> <day-of-month> (ordinal)",
        vec![pred(is_a_month), pred(is_dom_ordinal)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_namedday5() -> Rule {
    rule("named-day", vec![regex("(יום )?חמישי")], |_| tt(day_of_week(4)))
}

fn rule_dayofmonthordinal_namedmonth() -> Rule {
    rule(
        "<day-of-month>(ordinal) <named-month>",
        vec![pred(is_dom_ordinal), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
    )
}

fn rule_intersect_by() -> Rule {
    rule(
        "intersect by \",\"",
        vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_nth_time_after_time() -> Rule {
    rule(
        "nth <time> after <time>",
        vec![
            dimension(Dim::Ordinal),
            dimension(Dim::Time),
            regex("אחרי"),
            dimension(Dim::Time),
        ],
        |ts| tt(pred_nth_after(ordinal_value(ts[0])? - 1, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_mmdd() -> Rule {
    rule(
        "mm/dd",
        vec![regex("(0?[1-9]|1[0-2])/(3[01]|[12]\\d|0?[1-9])")],
        |ts| {
            let m = parse_int(&rx(ts[0], 0)?)?;
            let d = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        },
    )
}

fn rule_after_duration() -> Rule {
    rule(
        "after <duration>",
        vec![regex("אחרי"), dimension(Dim::Duration)],
        |ts| tt(with_direction(IntervalDirection::After, in_duration(dur(ts[1])?))),
    )
}

fn rule_timeofday_latent() -> Rule {
    rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
        tt(mk_latent(hour(true, get_int_value(ts[0])?)))
    })
}

fn rule_dayofmonth_ordinal_of_namedmonth() -> Rule {
    rule(
        "<day-of-month> (ordinal) of <named-month>",
        vec![pred(is_dom_ordinal), regex("של|ב|ל"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
    )
}

fn rule_this_evening() -> Rule {
    rule("this evening", vec![regex("הערב")], |_| {
        let pod = interval(Open, &hour(false, 18), &hour(false, 0))?;
        ttd(intersect(&today(), &pod).map(part_of_day))
    })
}

fn rule_between_datetime_and_datetime_interval() -> Rule {
    rule(
        "between <datetime> and <datetime> (interval)",
        vec![regex("בין"), dimension(Dim::Time), regex("ל"), dimension(Dim::Time)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_end_of_year() -> Rule {
    rule("End of year", vec![regex("סוף (ה)?שנה")], |_| tt(cycle_nth(G::Year, 1)))
}

fn rule_duration_ago() -> Rule {
    rule("<duration> ago", vec![regex("לפני"), dimension(Dim::Duration)], |ts| {
        tt(duration_ago(dur(ts[1])?))
    })
}

fn rule_last_n_cycle() -> Rule {
    rule(
        "last n <cycle>",
        vec![
            pred(is_integer_between_tok(1, 9999)),
            dimension(Dim::TimeGrain),
            regex("אחרון|אחרונות|אחרונה|אחרונים"),
        ],
        |ts| tt(cycle_n(true, grain_of(ts[1])?, -get_int_value(ts[0])?)),
    )
}

fn rule_midnighteodend_of_day() -> Rule {
    rule("midnight|EOD|end of day", vec![regex("(ב)?חצות")], |_| tt(hour(false, 0)))
}

fn rule_dayofmonth_non_ordinal_namedmonth() -> Rule {
    rule(
        "<day-of-month> (non ordinal) <named-month>",
        vec![pred(is_dom_integer), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
    )
}

fn rule_intersect() -> Rule {
    rule(
        "intersect",
        vec![pred(is_not_latent), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[1])?)),
    )
}

fn rule_until_timeofday() -> Rule {
    rule(
        "until <time-of-day>",
        vec![regex("עד"), dimension(Dim::Time)],
        |ts| tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone())),
    )
}

fn rule_at_timeofday() -> Rule {
    rule(
        "at <time-of-day>",
        vec![regex("ב"), pred(is_a_time_of_day)],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    )
}

fn rule_nth_time_of_time() -> Rule {
    rule(
        "nth <time> of <time>",
        vec![
            dimension(Dim::Time),
            dimension(Dim::Ordinal),
            regex("של|ב"),
            dimension(Dim::Time),
        ],
        |ts| {
            let n = ordinal_value(ts[1])? - 1;
            ttd(intersect(td(ts[3])?, td(ts[0])?).map(|x| pred_nth(n, false, &x)))
        },
    )
}

fn rule_time_partofday() -> Rule {
    rule(
        "<time> <part-of-day>",
        vec![dimension(Dim::Time), pred(is_a_part_of_day)],
        |ts| ttd(intersect(td(ts[1])?, td(ts[0])?)),
    )
}

fn rule_weekend() -> Rule {
    rule("week-end", vec![regex("(סופ״ש|סוף השבוע)")], |_| tt(weekend()))
}

fn rule_namedday_dayofmonth_ordinal() -> Rule {
    rule(
        "<named-day> <day-of-month> (ordinal)",
        vec![pred(is_a_day_of_week), pred(is_dom_ordinal)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_date() -> Rule {
    rule("ב <date>", vec![regex("ב"), dimension(Dim::Time)], |ts| Some(ts[1].clone()))
}

fn rule_next_time() -> Rule {
    rule(
        "next <time>",
        vec![pred(is_not_latent), regex("הבא(ה)?")],
        |ts| tt(pred_nth(0, true, td(ts[0])?)),
    )
}

fn rule_ordinal_quarter_year() -> Rule {
    rule(
        "<ordinal> quarter <year>",
        vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter)), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, G::Quarter, ordinal_value(ts[0])? - 1, td(ts[2])?)),
    )
}

fn rule_yyyymmdd() -> Rule {
    rule(
        "yyyy-mm-dd",
        vec![regex("(\\d{2,4})-(0?[1-9]|1[0-2])-(3[01]|[12]\\d|0?[1-9])")],
        |ts| {
            let y = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let d = parse_int(&rx(ts[0], 2)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn rule_the_ordinal_cycle_after_time() -> Rule {
    rule(
        "the <ordinal> <cycle> after <time>",
        vec![
            regex("ה"),
            dimension(Dim::Ordinal),
            dimension(Dim::TimeGrain),
            regex("אחרי|לאחר"),
            dimension(Dim::Time),
        ],
        |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[4])?)),
    )
}

fn rule_next_n_cycle() -> Rule {
    rule(
        "next n <cycle>",
        vec![
            pred(is_integer_between_tok(1, 9999)),
            dimension(Dim::TimeGrain),
            regex("הבא|הבאה|הבאים|הבאות"),
        ],
        |ts| tt(cycle_n(true, grain_of(ts[1])?, get_int_value(ts[0])?)),
    )
}

fn rule_morning() -> Rule {
    rule("morning", vec![regex("(ב)?בוקר")], |_| latent_pod(4, 12))
}

fn rule_this_cycle() -> Rule {
    rule(
        "this <cycle>",
        vec![dimension(Dim::TimeGrain), regex("הקרו(ב)?ה|הזה|הזאת")],
        |ts| tt(cycle_nth(grain_of(ts[0])?, 0)),
    )
}

fn rule_this_time() -> Rule {
    rule(
        "this <time>",
        vec![dimension(Dim::Time), regex("הקרוב|הזה")],
        |ts| tt(pred_nth(0, false, td(ts[0])?)),
    )
}

fn rule_dayofmonth_non_ordinal_of_namedmonth() -> Rule {
    rule(
        "<day-of-month> (non ordinal) of <named-month>",
        vec![pred(is_dom_integer), regex("של|ב|ל"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
    )
}

fn rule_end_of_month() -> Rule {
    rule("End of month", vec![regex("סוף (ה)?חודש")], |_| tt(cycle_nth(G::Month, 1)))
}

fn rule_yesterday() -> Rule {
    rule("yesterday", vec![regex("(אתמול|אמש)")], |_| tt(cycle_nth(G::Day, -1)))
}

fn rule_after_timeofday() -> Rule {
    rule(
        "after <time-of-day>",
        vec![regex("אחרי"), dimension(Dim::Time)],
        |ts| tt(with_direction(IntervalDirection::After, td(ts[1])?.clone())),
    )
}

fn rule_dayofmonth_ordinal() -> Rule {
    rule("<day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
        tt(mk_latent(day_of_month(get_int_value(ts[0])?)))
    })
}

fn rule_timeofday_ampm() -> Rule {
    rule(
        "<time-of-day> am|pm",
        vec![pred(is_a_time_of_day), regex("(in the )?([ap])(\\s|\\.)?m?\\.?")],
        |ts| {
            let ap = rx(ts[1], 1)?;
            tt(time_of_day_ampm(to_lower(&ap) == "a", td(ts[0])?))
        },
    )
}

fn rule_ordinal_cycle_after_time() -> Rule {
    rule(
        "<ordinal> <cycle> after <time>",
        vec![
            dimension(Dim::Ordinal),
            dimension(Dim::TimeGrain),
            regex("אחרי|לאחר"),
            dimension(Dim::Time),
        ],
        |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?)),
    )
}

fn rule_namedday7() -> Rule {
    rule("named-day", vec![regex("(יום )?שבת")], |_| tt(day_of_week(6)))
}

fn rule_hhmm() -> Rule {
    rule(
        "hh:mm",
        vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(true, h, m))
        },
    )
}

fn rule_time_of_partofday() -> Rule {
    rule(
        "<time> of <part-of-day>",
        vec![dimension(Dim::Time), regex("ב"), pred(is_a_part_of_day)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_year() -> Rule {
    rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
        tt(year(get_int_value(ts[0])?))
    })
}

fn rule_namedmonth_dayofmonth_non_ordinal() -> Rule {
    rule(
        "<named-month> <day-of-month> (non ordinal)",
        vec![pred(is_a_month), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_namedday8() -> Rule {
    rule("named-day", vec![regex("(יום )?ראשון")], |_| tt(day_of_week(7)))
}

fn rule_absorption_of_after_named_day() -> Rule {
    rule(
        "absorption of , after named day",
        vec![pred(is_a_day_of_week), regex(",")],
        |ts| Some(ts[0].clone()),
    )
}

fn rule_last_dayofweek_of_time() -> Rule {
    rule(
        "last <day-of-week> of <time>",
        vec![pred(is_a_day_of_week), regex("האחרון של"), dimension(Dim::Time)],
        |ts| tt(pred_last_of(td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_hhmm_military_ampm() -> Rule {
    rule(
        "hhmm (military) am|pm",
        vec![regex("((?:1[012]|0?\\d))([0-5]\\d)"), regex("([ap])\\.?m?\\.?")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let ap = rx(ts[1], 0)?;
            tt(time_of_day_ampm(to_lower(&ap) == "a", &hour_minute(true, h, m)))
        },
    )
}

fn rule_cycle_before_time() -> Rule {
    rule(
        "<cycle> before <time>",
        vec![dimension(Dim::TimeGrain), regex("לפני"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, -1, td(ts[2])?)),
    )
}

fn rule_duration_after_time() -> Rule {
    rule(
        "<duration> after <time>",
        vec![dimension(Dim::Duration), regex("אחרי|לאחר"), dimension(Dim::Time)],
        |ts| tt(duration_after(dur(ts[0])?, td(ts[2])?)),
    )
}

fn rule_eveningnight() -> Rule {
    rule("evening|night", vec![regex("(ב)?ערב")], |_| latent_pod(18, 0))
}

fn rule_ordinal_quarter() -> Rule {
    rule(
        "<ordinal> quarter",
        vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))],
        |ts| {
            tt(cycle_nth_after(
                true,
                G::Quarter,
                get_int_value(ts[0])? - 1,
                &cycle_nth(G::Year, 0),
            ))
        },
    )
}

fn rule_namedday3() -> Rule {
    rule("named-day", vec![regex("(יום )?שלישי")], |_| tt(day_of_week(2)))
}

fn rule_the_dayofmonth_ordinal() -> Rule {
    rule(
        "the <day-of-month> (ordinal)",
        vec![regex("ה"), pred(is_dom_ordinal)],
        |ts| tt(day_of_month(ordinal_value(ts[1])?)),
    )
}

fn rule_duration_before_time() -> Rule {
    rule(
        "<duration> before <time>",
        vec![dimension(Dim::Duration), regex("לפני"), dimension(Dim::Time)],
        |ts| tt(duration_before(dur(ts[0])?, td(ts[2])?)),
    )
}

fn rule_partofday_of_time() -> Rule {
    rule(
        "<part-of-day> of <time>",
        vec![regex("ב"), pred(is_a_part_of_day), regex("של"), dimension(Dim::Time)],
        |ts| ttd(intersect(td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_mmddyyyy() -> Rule {
    rule(
        "mm/dd/yyyy",
        vec![regex("(0?[1-9]|1[0-2])[/-](3[01]|[12]\\d|0?[1-9])[-/](\\d{2,4})")],
        |ts| {
            let y = parse_int(&rx(ts[0], 2)?)?;
            let m = parse_int(&rx(ts[0], 0)?)?;
            let d = parse_int(&rx(ts[0], 1)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn rule_tomorrow() -> Rule {
    rule("tomorrow", vec![regex("(מחר|למחרת)")], |_| tt(cycle_nth(G::Day, 1)))
}

fn rule_timeofday_oclock() -> Rule {
    rule(
        "<time-of-day> o'clock",
        vec![pred(is_a_time_of_day), regex("o.?clock")],
        |ts| tt(not_latent(td(ts[0])?.clone())),
    )
}

fn rule_dayofmonthordinal_namedmonth_year() -> Rule {
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

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = vec![
        rule_absorption_of_after_named_day(),
        rule_after_duration(),
        rule_after_timeofday(),
        rule_afternoon(),
        rule_at_hour_timeofday(),
        rule_at_timeofday(),
        rule_between_datetime_and_datetime_interval(),
        rule_between_timeofday_and_timeofday_interval(),
        rule_current_dayofweek(),
        rule_cycle_after_time(),
        rule_cycle_before_time(),
        rule_date(),
        rule_datetime_datetime_interval(),
        rule_dayofmonth_non_ordinal_namedmonth(),
        rule_dayofmonth_non_ordinal_of_namedmonth(),
        rule_dayofmonth_ordinal(),
        rule_dayofmonth_ordinal_of_namedmonth(),
        rule_dayofmonthordinal_namedmonth(),
        rule_dayofmonthordinal_namedmonth_year(),
        rule_duration_after_time(),
        rule_duration_ago(),
        rule_duration_before_time(),
        rule_duration_from_now(),
        rule_end_of_month(),
        rule_end_of_year(),
        rule_eveningnight(),
        rule_for_duration(),
        rule_from_datetime_datetime_interval(),
        rule_hhmm(),
        rule_hhmm_military_ampm(),
        rule_hhmmss(),
        rule_hourofday_and_integer(),
        rule_hourofday_and_quarter(),
        rule_hourofday_and_half(),
        rule_hourofday_integer(),
        rule_hourofday_quarter(),
        rule_hourofday_half(),
        rule_in_duration(),
        rule_in_namedmonth(),
        rule_intersect(),
        rule_intersect_by(),
        rule_last_cycle(),
        rule_last_dayofweek(),
        rule_last_dayofweek_of_time(),
        rule_last_n_cycle(),
        rule_last_time(),
        rule_lunch(),
        rule_midnighteodend_of_day(),
        rule_mmdd(),
        rule_mmddyyyy(),
        rule_morning(),
        rule_namedday(),
        rule_namedday2(),
        rule_namedday3(),
        rule_namedday4(),
        rule_namedday5(),
        rule_namedday6(),
        rule_namedday7(),
        rule_namedday8(),
        rule_namedday_dayofmonth_ordinal(),
        rule_namedmonth_dayofmonth_non_ordinal(),
        rule_namedmonth_dayofmonth_ordinal(),
        rule_next_cycle(),
        rule_next_dayofweek(),
        rule_next_n_cycle(),
        rule_next_time(),
        rule_noon(),
        rule_now(),
        rule_nth_time_after_time(),
        rule_nth_time_of_time(),
        rule_ordinal_cycle_after_time(),
        rule_ordinal_quarter(),
        rule_ordinal_quarter_year(),
        rule_partofday_of_time(),
        rule_integer_afterpast_integer_hourofday(),
        rule_quarter_afterpast_integer_hourofday(),
        rule_half_afterpast_integer_hourofday(),
        rule_integer_totillbefore_integer_hourofday(),
        rule_quarter_totillbefore_integer_hourofday(),
        rule_half_totillbefore_integer_hourofday(),
        rule_since_timeofday(),
        rule_the_cycle_after_time(),
        rule_the_cycle_before_time(),
        rule_the_dayofmonth_non_ordinal(),
        rule_the_dayofmonth_ordinal(),
        rule_the_ides_of_namedmonth(),
        rule_the_ordinal_cycle_after_time(),
        rule_this_cycle(),
        rule_this_evening(),
        rule_this_time(),
        rule_time_of_partofday(),
        rule_time_partofday(),
        rule_timeofday_ampm(),
        rule_timeofday_latent(),
        rule_timeofday_oclock(),
        rule_today(),
        rule_tomorrow(),
        rule_until_timeofday(),
        rule_weekend(),
        rule_year(),
        rule_yesterday(),
        rule_yyyymmdd(),
    ];
    v.extend(mk_rule_months(&[
        ("ינואר", "ינואר"),
        ("פברואר", "פברואר"),
        ("מרץ", "מרץ"),
        ("אפריל", "אפריל"),
        ("מאי", "מאי"),
        ("יוני", "יוני"),
        ("יולי", "יולי"),
        ("אוגוסט", "אוגוסט"),
        ("ספטמבר", "ספטמבר"),
        ("אוקטובר", "אוקטובר"),
        ("נובמבר", "נובמבר"),
        ("דצמבר", "דצמבר"),
    ]));
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
