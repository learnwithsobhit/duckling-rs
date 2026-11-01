//! `Duckling.Time.HR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::DurationData;
use crate::dims::time::helpers::*;
use crate::dims::time::types::{
    Form, IntervalDirection, TimeData,
    TimeIntervalType::{Closed, Open},
};
use crate::dims::time_grain::Grain as G;
use crate::dims::util::{parse_int, to_upper};
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

/// `Duckling.Duration.Helpers.isGrain` — a TimeGrain token, not a Time grain.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn not_latent_hour(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

fn not_latent_ok_this_next(t: &Token) -> bool {
    is_not_latent(t) && is_ok_with_this_next(t)
}

fn not_latent_tod(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t)
}

fn not_latent_finer_than_day(t: &Token) -> bool {
    is_not_latent(t) && is_grain_finer_than(G::Day)(t)
}

fn latent_pod(a: i64, b: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, a), &hour(false, b)).map(|x| part_of_day(mk_latent(x))))
}

fn rule_half_integer_hr_style_hourofday() -> Rule {
    rule(
        "half <integer> (HR style hour-of-day)",
        vec![regex("pol?a?"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_before(30, td(ts[1])?)),
    )
}

fn rule_numeral_totillbefore_integer_hourofday() -> Rule {
    rule(
        "numeral to|till|before <integer> (hour-of-day)",
        vec![pred(is_integer_between_tok(1, 59)), regex("do"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_before(get_int_value(ts[0])?, td(ts[2])?)),
    )
}

fn rule_quarter_totillbefore_integer_hourofday() -> Rule {
    rule(
        "quarter to|till|before <integer> (hour-of-day)",
        vec![regex(r"(kvarata?|(c|č)etvrt|frtalj)\s+do"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_before(15, td(ts[1])?)),
    )
}

fn rule_half_totillbefore_integer_hourofday() -> Rule {
    rule(
        "half to|till|before <integer> (hour-of-day)",
        vec![regex(r"di pol?a?\s+do"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_before(30, td(ts[1])?)),
    )
}

fn rule_numeral_afterpast_hourofday() -> Rule {
    rule(
        "numeral after|past (hour-of-day)",
        vec![pred(is_integer_between_tok(1, 59)), regex("poslije|nakon"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_after(get_int_value(ts[0])?, td(ts[2])?)),
    )
}

fn rule_quarter_afterpast_hourofday() -> Rule {
    rule(
        "quarter after|past (hour-of-day)",
        vec![regex(r"(kvarata?|(c|č)etvrt|frtalj)\s+(poslije|nakon)"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_after(15, td(ts[1])?)),
    )
}

fn rule_half_afterpast_hourofday() -> Rule {
    rule(
        "half after|past (hour-of-day)",
        vec![regex(r"(i pol?a?)\s+(poslije|nakon)"), pred(is_an_hour_of_day)],
        |ts| ttd(minutes_after(30, td(ts[1])?)),
    )
}

fn rule_hourofday_numeral() -> Rule {
    rule(
        "<hour-of-day> <integer> (as relative minutes)",
        vec![pred(not_latent_hour), pred(is_integer_between_tok(1, 59))],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[1])?))
        },
    )
}

fn rule_hourofday_quarter() -> Rule {
    rule("<hour-of-day> quarter", vec![pred(is_an_hour_of_day), regex("kvarata?|(c|č)etvrt|frtalj")], |ts| {
        let (hours, is12h) = tod(ts[0])?;
        tt(hour_minute(is12h, hours, 15))
    })
}

fn rule_hourofday_half() -> Rule {
    rule("<hour-of-day> half", vec![pred(is_an_hour_of_day), regex("i pol?a?")], |ts| {
        let (hours, is12h) = tod(ts[0])?;
        tt(hour_minute(is12h, hours, 30))
    })
}

fn rule_za_numeral_hourofday() -> Rule {
    rule(
        "za <integer> (hour-of-day)",
        vec![regex("za"), pred(is_integer_between_tok(1, 59)), pred(is_an_hour_of_day)],
        |ts| {
            let (hours, is12h) = tod(ts[2])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[1])?))
        },
    )
}

fn rule_za_quarter_hourofday() -> Rule {
    rule(
        "za quarter (hour-of-day)",
        vec![regex(r"za\s+(kvarata?|(c|č)etvrt|frtalj)"), pred(is_an_hour_of_day)],
        |ts| {
            let (hours, is12h) = tod(ts[1])?;
            tt(hour_minute(is12h, hours, 15))
        },
    )
}

fn rule_za_half_hourofday() -> Rule {
    rule(
        "za half (hour-of-day)",
        vec![regex(r"za\s+(i pol?a?)"), pred(is_an_hour_of_day)],
        |ts| {
            let (hours, is12h) = tod(ts[1])?;
            tt(hour_minute(is12h, hours, 30))
        },
    )
}

fn rule_since_timeofday() -> Rule {
    rule("since <time-of-day>", vec![regex("od"), dimension(Dim::Time)], |ts| {
        tt(with_direction(IntervalDirection::After, td(ts[1])?.clone()))
    })
}

fn rule_last_time() -> Rule {
    rule(
        "last <time>",
        vec![
            regex(r"(prethodn(i|u|a|e|o(ga?)?)|pro(s|š)l(ih?|u|a|e|o(ga?)?))"),
            pred(is_ok_with_this_next),
        ],
        |ts| tt(pred_nth(-1, false, td(ts[1])?)),
    )
}

fn rule_datetime_datetime_interval() -> Rule {
    rule(
        "<datetime> - <datetime> (interval)",
        vec![pred(is_not_latent), regex("\\-"), pred(is_not_latent)],
        |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_cycle_after_time() -> Rule {
    rule(
        "<cycle> after <time>",
        vec![dimension(Dim::TimeGrain), regex("nakon"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, 1, td(ts[2])?)),
    )
}

fn rule_in_duration() -> Rule {
    rule("in <duration>", vec![regex("za"), dimension(Dim::Duration)], |ts| {
        tt(in_duration(dur(ts[1])?))
    })
}

fn rule_now() -> Rule {
    rule(
        "now",
        vec![regex(r"((upravo|ov(aj?|og?|e|i))? ?sada?|trenutak|upravo|trenutno)|(ov(aj|og) trena?)")],
        |_| tt(now()),
    )
}

fn rule_last_cycle_of_time() -> Rule {
    rule(
        "last <cycle> of <time>",
        vec![regex("zadnj(ih?|a|e(ga?)?)"), dimension(Dim::TimeGrain), regex("u"), dimension(Dim::Time)],
        |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
    )
}

fn rule_from_datetime_datetime_interval() -> Rule {
    rule(
        "from <datetime> - <datetime> (interval)",
        vec![regex("od|izme(dj|đ)u"), dimension(Dim::Time), regex("\\-"), dimension(Dim::Time)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_month_dddd_interval() -> Rule {
    rule(
        "<month> dd-dd (interval)",
        vec![
            pred(is_a_month),
            regex(r"(3[01]|[12]\d|0?[1-9])"),
            regex("\\-|do"),
            regex(r"(3[01]|[12]\d|0?[1-9])"),
        ],
        |ts| {
            let d1 = parse_int(&rx(ts[1], 0)?)?;
            let d2 = parse_int(&rx(ts[3], 0)?)?;
            let from = intersect(&day_of_month(d1), td(ts[0])?)?;
            let to = intersect(&day_of_month(d2), td(ts[0])?)?;
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn rule_year_latent2() -> Rule {
    rule("year (latent)", vec![pred(is_integer_between_tok(2101, 10000))], |ts| {
        tt(mk_latent(year(get_int_value(ts[0])?)))
    })
}

fn rule_time_after_next() -> Rule {
    rule(
        "<time> after next",
        vec![dimension(Dim::Time), regex("nakon sljede(ć|c)(i|e|a)(ga?)?")],
        |ts| tt(pred_nth(1, true, td(ts[0])?)),
    )
}

fn rule_noon() -> Rule {
    rule("noon", vec![regex(r"(u )?podne(va)?")], |_| tt(hour(false, 12)))
}

fn rule_early_morning() -> Rule {
    rule(
        "early morning",
        vec![regex(r"(rano( (u )?u?jutros?)?)|(u ran(im|e) jutarnj(im|e) sat(ima|e))")],
        |_| latent_pod(3, 9),
    )
}

fn rule_today() -> Rule {
    rule("today", vec![regex(r"danas?|(dana(s|š)nj(i|eg) dana?) ")], |_| tt(today()))
}

fn rule_thisnext_dayofweek() -> Rule {
    rule(
        "this|next <day-of-week>",
        vec![regex("ov(aj?|og?|e)|sljede(c|ć)(i|u|a|e(ga?)?)"), pred(is_a_day_of_week)],
        |ts| tt(pred_nth(0, true, td(ts[1])?)),
    )
}

fn rule_between_timeofday_and_timeofday_interval() -> Rule {
    rule(
        "between <time-of-day> and <time-of-day> (interval)",
        vec![regex("od|izme(dj|đ)u"), pred(is_a_time_of_day), regex("do|i"), pred(is_a_time_of_day)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_next_cycle() -> Rule {
    rule("next <cycle>", vec![regex("sljede(c|ć)(i|a|u|e(ga?)?)"), dimension(Dim::TimeGrain)], |ts| {
        tt(cycle_nth(grain_of(ts[1])?, 1))
    })
}

fn rule_timeofday_approximately() -> Rule {
    rule("<time-of-day> approximately", vec![pred(is_a_time_of_day), regex("(-?ak)")], |ts| {
        tt(not_latent(td(ts[0])?.clone()))
    })
}

fn rule_for_duration() -> Rule {
    rule("for <duration>", vec![regex("za( jo(s|š))?|u"), dimension(Dim::Duration)], |ts| {
        tt(in_duration(dur(ts[1])?))
    })
}

fn rule_last_dayofweek_time() -> Rule {
    rule(
        "last <day-of-week> <time>",
        vec![regex("zadnj(ih?|a|e(ga?)?)"), pred(is_a_day_of_week), regex("u"), dimension(Dim::Time)],
        |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_duration_from_now() -> Rule {
    rule(
        "<duration> from now",
        vec![dimension(Dim::Duration), regex("od (sada?|ovog trenutka|dana(s|š)nj(i|eg) dana?)")],
        |ts| tt(in_duration(dur(ts[0])?)),
    )
}

fn rule_lunch() -> Rule {
    rule("lunch", vec![regex(r"(((za )|(u vrijeme )) )?ru(c|č)a?k(a|om)?")], |_| latent_pod(12, 14))
}

fn rule_last_cycle() -> Rule {
    rule(
        "last <cycle>",
        vec![regex("prethodn(i|a|e|u)|pro(s|š)l(i|a|e|u|o(ga?)?)"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
    )
}

fn rule_ddmm() -> Rule {
    rule("dd.mm", vec![regex(r"(3[01]|[12]\d|0?[1-9])\/(0?[1-9]|1[0-2])")], |ts| {
        let d = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        tt(month_day(m, d))
    })
}

fn rule_afternoon() -> Rule {
    rule("afternoon", vec![regex("po(slije)?podne")], |_| latent_pod(12, 20))
}

fn rule_time_before_last() -> Rule {
    rule(
        "<time> before last",
        vec![
            dimension(Dim::Time),
            regex("prije (prethodn(e|o(ga?)?)|pro(s|š)l(ih?|u|a|e|o(ga?)?))"),
        ],
        |ts| tt(pred_nth(-2, false, td(ts[0])?)),
    )
}

fn rule_late_night() -> Rule {
    rule("late night", vec![regex(r"(((u|po)\s)?no(c|ć)(i|as|u)?|u?jutros?)")], |_| {
        latent_pod(0, 4)
    })
}

fn rule_namedmonth_dayofmonth_ordinal() -> Rule {
    rule(
        "<named-month> <day-of-month> (ordinal)",
        vec![pred(is_a_month), pred(is_dom_ordinal)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_induring_the_partofday() -> Rule {
    rule("in|during the <part-of-day>", vec![regex("(u|tokom)"), pred(is_a_part_of_day)], |ts| {
        tt(not_latent(td(ts[1])?.clone()))
    })
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
        vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("poslije|nakon|iza"), dimension(Dim::Time)],
        |ts| tt(pred_nth_after(ordinal_value(ts[0])? - 1, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_after_duration() -> Rule {
    rule(
        "after <duration>",
        vec![regex(r"(nakon|poslije)( jo(s|š))?"), dimension(Dim::Duration)],
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
        vec![pred(is_dom_ordinal), regex("of|in"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
    )
}

fn rule_from_timeofday_timeofday_interval() -> Rule {
    rule(
        "from <time-of-day> - <time-of-day> (interval)",
        vec![regex("od"), pred(is_a_time_of_day), regex("\\-"), pred(is_a_time_of_day)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_exactly_timeofday() -> Rule {
    rule("exactly <time-of-day>", vec![regex("to(c|č)no( u)?"), pred(is_a_time_of_day)], |ts| {
        tt(not_latent(td(ts[1])?.clone()))
    })
}

fn rule_u_namedday() -> Rule {
    rule("u <named-day>", vec![regex("u"), pred(is_a_day_of_week)], |ts| Some(ts[1].clone()))
}

fn rule_between_datetime_and_datetime_interval() -> Rule {
    rule(
        "between <datetime> and <datetime> (interval)",
        vec![regex("od|izme(dj|đ)u"), dimension(Dim::Time), regex("do|i"), dimension(Dim::Time)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_by_the_end_of_time() -> Rule {
    rule(
        "by the end of <time>",
        vec![regex(r"(do )(kraja|isteka)? "), dimension(Dim::Time)],
        |ts| ttd(interval(Closed, &now(), td(ts[1])?)),
    )
}

fn rule_after_work() -> Rule {
    rule("after work", vec![regex("poslije (posla|kraja radnog vremena)")], |_| {
        let part = interval(Open, &hour(false, 17), &hour(false, 21))?;
        ttd(intersect(&today(), &part).map(|x| part_of_day(not_latent(x))))
    })
}

fn rule_last_dayofweek_time2() -> Rule {
    rule(
        "last <day-of-week> <time>",
        vec![regex("zadnj(ih?|a|e(ga?)?)"), pred(is_a_day_of_week), dimension(Dim::Time)],
        |ts| tt(pred_last_of(td(ts[1])?, td(ts[2])?)),
    )
}

fn rule_last_n_cycle() -> Rule {
    rule(
        "last n <cycle>",
        vec![
            regex("prethodn(ih?|a|e)|pro(s|š)l(a|e|ih?)"),
            pred(is_integer_between_tok(1, 9999)),
            dimension(Dim::TimeGrain),
        ],
        |ts| tt(cycle_n(true, grain_of(ts[2])?, -get_int_value(ts[1])?)),
    )
}

fn rule_within_duration() -> Rule {
    rule("within <duration>", vec![regex(r"(u|za )vrijeme"), dimension(Dim::Duration)], |ts| {
        ttd(interval(Open, &now(), &in_duration(dur(ts[1])?)))
    })
}

fn rule_midnighteodend_of_day() -> Rule {
    rule(
        "midnight|EOD|end of day",
        vec![regex(r"(u )?pono(c|ć)i?|(the )?(EOD|((do )? kraja? dana))")],
        |_| tt(hour(false, 0)),
    )
}

fn rule_dayofmonth_non_ordinal_namedmonth() -> Rule {
    rule(
        "<day-of-month> (non ordinal) <named-month>",
        vec![pred(is_dom_integer), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
    )
}

fn rule_intersect() -> Rule {
    rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
        ttd(intersect(td(ts[0])?, td(ts[1])?))
    })
}

fn rule_about_timeofday() -> Rule {
    rule("about <time-of-day>", vec![regex("(oko|cca|otprilike)"), pred(is_a_time_of_day)], |ts| {
        tt(not_latent(td(ts[1])?.clone()))
    })
}

fn rule_until_timeofday() -> Rule {
    rule(
        "until <time-of-day>",
        vec![regex(r"((nekad|najkasnije) )?(prije|do)"), pred(is_a_time_of_day)],
        |ts| tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone())),
    )
}

fn rule_at_timeofday() -> Rule {
    rule("at <time-of-day>", vec![regex("u|@"), pred(is_a_time_of_day)], |ts| {
        tt(not_latent(td(ts[1])?.clone()))
    })
}

fn rule_nth_time_of_time() -> Rule {
    rule(
        "nth <time> of <time>",
        vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("u"), dimension(Dim::Time)],
        |ts| {
            let n = ordinal_value(ts[0])? - 1;
            ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(n, false, &x)))
        },
    )
}

fn rule_time_partofday() -> Rule {
    rule("<time> <part-of-day>", vec![dimension(Dim::Time), pred(is_a_part_of_day)], |ts| {
        ttd(intersect(td(ts[1])?, td(ts[0])?))
    })
}

fn rule_weekend() -> Rule {
    rule("week-end", vec![regex(r"(za )?vikenda?")], |_| tt(mk_ok_for_this_next(weekend())))
}

fn rule_prije_duration() -> Rule {
    rule("prije <duration>", vec![regex("prije"), dimension(Dim::Duration)], |ts| {
        tt(duration_ago(dur(ts[1])?))
    })
}

fn rule_about_duration() -> Rule {
    rule("about <duration>", vec![regex("oko"), dimension(Dim::Duration)], |ts| {
        tt(in_duration(dur(ts[1])?))
    })
}

fn rule_eomend_of_month() -> Rule {
    rule("EOM|End of month", vec![regex(r"(the )?(EOM|(do )? kraja mjeseca)")], |_| {
        tt(cycle_nth(G::Month, 1))
    })
}

fn rule_namedday_dayofmonth_ordinal() -> Rule {
    rule(
        "<named-day> <day-of-month> (ordinal)",
        vec![pred(is_a_day_of_week), pred(is_dom_ordinal)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_next_time() -> Rule {
    rule(
        "next <time>",
        vec![regex("sljede(c|ć)(i|u|a|e(ga?)?)"), pred(not_latent_ok_this_next)],
        |ts| tt(pred_nth(0, true, td(ts[1])?)),
    )
}

fn rule_ordinal_quarter_year() -> Rule {
    rule(
        "<ordinal> quarter <year>",
        vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter)), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, G::Quarter, get_int_value(ts[0])? - 1, td(ts[2])?)),
    )
}

fn rule_yyyymmdd() -> Rule {
    rule(
        "yyyy-mm-dd",
        vec![regex(r"(\d{2,4})-(0?[1-9]|1[0-2])-(3[01]|[12]\d|0?[1-9])")],
        |ts| {
            let y = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let d = parse_int(&rx(ts[0], 2)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn rule_next_n_cycle() -> Rule {
    rule(
        "next n <cycle>",
        vec![
            regex(r"(u )?(sljede(c|ć)(ih?|a|eg?))"),
            pred(is_integer_between_tok(1, 9999)),
            dimension(Dim::TimeGrain),
        ],
        |ts| tt(cycle_n(true, grain_of(ts[2])?, get_int_value(ts[1])?)),
    )
}

fn rule_morning() -> Rule {
    rule("morning", vec![regex(r"(u )?u?jutros?")], |_| latent_pod(4, 12))
}

fn rule_this_partofday() -> Rule {
    rule("this <part-of-day>", vec![regex("ov(aj?|og?|i|e)"), pred(is_a_part_of_day)], |ts| {
        ttd(intersect(&today(), td(ts[1])?).map(|x| part_of_day(not_latent(x))))
    })
}

fn rule_this_cycle() -> Rule {
    rule("this <cycle>", vec![regex("ov(aj?|og?|e|i)"), dimension(Dim::TimeGrain)], |ts| {
        tt(cycle_nth(grain_of(ts[1])?, 0))
    })
}

fn rule_u_namedmonth() -> Rule {
    rule("u <named-month>", vec![regex("u"), pred(is_a_month)], |ts| Some(ts[1].clone()))
}

fn rule_this_time() -> Rule {
    rule("this <time>", vec![regex("ov(aj?|og?|e)(sad)?"), pred(is_ok_with_this_next)], |ts| {
        tt(pred_nth(0, false, td(ts[1])?))
    })
}

fn rule_dayofmonth_non_ordinal_of_namedmonth() -> Rule {
    rule(
        "<day-of-month> (non ordinal) of <named-month>",
        vec![pred(is_dom_integer), regex("of|in"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
    )
}

fn rule_hhmm_military() -> Rule {
    rule(
        "hhmm (military, prefixed with m to avoid ambiguity with years)",
        vec![regex(r"m((?:[01]?\d)|(?:2[0-3]))([0-5]\d)")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(time_of_day_ampm(false, &hour_minute(false, h, m)))
        },
    )
}

fn rule_day_before_yesterday() -> Rule {
    rule("day before yesterday", vec![regex(r"(prekju(c|č)er)")], |_| tt(cycle_nth(G::Day, -2)))
}

fn rule_after_lunch() -> Rule {
    rule("after lunch", vec![regex("poslije ru(c|č)ka")], |_| {
        let part = interval(Open, &hour(false, 13), &hour(false, 17))?;
        ttd(intersect(&today(), &part).map(|x| part_of_day(not_latent(x))))
    })
}

fn rule_year_latent() -> Rule {
    rule("year (latent)", vec![pred(is_integer_between_tok(25, 999))], |ts| {
        tt(mk_latent(year(get_int_value(ts[0])?)))
    })
}

fn rule_yesterday() -> Rule {
    rule("yesterday", vec![regex(r"(ju(c|č)er)")], |_| tt(cycle_nth(G::Day, -1)))
}

fn rule_after_timeofday() -> Rule {
    rule(
        "after <time-of-day>",
        vec![regex(r"(nekad |najranije )?(prije|od)"), dimension(Dim::Time)],
        |ts| tt(with_direction(IntervalDirection::After, td(ts[1])?.clone())),
    )
}

fn rule_dayofmonth_ordinal() -> Rule {
    rule("<day-of-month>. (ordinal)", vec![pred(is_dom_ordinal), regex("\\.|i|og")], |ts| {
        tt(day_of_month(get_int_value(ts[0])?))
    })
}

fn rule_ordinal_cycle_after_time() -> Rule {
    rule(
        "<ordinal> <cycle> after <time>",
        vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("nakon"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?)),
    )
}

fn rule_ordinal_cycle_of_time() -> Rule {
    rule(
        "<ordinal> <cycle> of <time>",
        vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("od|u"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?)),
    )
}

fn rule_after_next_time() -> Rule {
    rule(
        "after next <time>",
        vec![regex("nakon sljede(ć|c)(i|e|a)(ga?)?"), dimension(Dim::Time)],
        |ts| tt(pred_nth(1, true, td(ts[1])?)),
    )
}

fn rule_day_after_tomorrow() -> Rule {
    rule("day after tomorrow", vec![regex(r"(preko?sutra)")], |_| tt(cycle_nth(G::Day, 2)))
}

fn rule_hhmm() -> Rule {
    rule("hh:mm", vec![regex(r"((?:[01]?\d)|(?:2[0-3]))[(:.\si]+([0-5]\d)")], |ts| {
        let h = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        tt(hour_minute(true, h, m))
    })
}

fn rule_tonight() -> Rule {
    rule("tonight", vec![regex(r"(na)?ve(c|č)er(as)?")], |_| {
        let evening = interval(Open, &hour(false, 18), &hour(false, 0))?;
        ttd(intersect(&today(), &evening).map(|x| part_of_day(not_latent(x))))
    })
}

fn rule_before_lasttime() -> Rule {
    rule(
        "before last<time>",
        vec![
            regex("prije (prethodn(i|u|a|e|o(ga?)?)|pro(s|š)l(ih?|a|e|o(ga?)?))"),
            dimension(Dim::Time),
        ],
        |ts| tt(pred_nth(-2, false, td(ts[1])?)),
    )
}

fn rule_timezone() -> Rule {
    rule(
        "<time> timezone",
        vec![
            pred(not_latent_finer_than_day),
            regex(
                r"\b(YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT)\b",
            ),
        ],
        |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
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

fn rule_absorption_of_after_named_day() -> Rule {
    rule("absorption of , after named day", vec![pred(is_a_day_of_week), regex(",")], |ts| {
        Some(ts[0].clone())
    })
}

fn rule_by_time() -> Rule {
    rule("by <time>", vec![regex("do"), dimension(Dim::Time)], |ts| {
        ttd(interval(Open, &now(), td(ts[1])?))
    })
}

fn rule_cycle_before_time() -> Rule {
    rule(
        "<cycle> before <time>",
        vec![dimension(Dim::TimeGrain), regex("prije"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, -1, td(ts[2])?)),
    )
}

fn rule_ddmmyyyy() -> Rule {
    rule(
        "dd/mm/yyyy",
        vec![regex(r"(3[01]|[12]\d|0?[1-9])[\/](0?[1-9]|1[0-2])[\/](\d{2,4})")],
        |ts| {
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let y = parse_int(&rx(ts[0], 2)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn rule_timeofday_timeofday_interval() -> Rule {
    rule(
        "<time-of-day> - <time-of-day> (interval)",
        vec![pred(not_latent_tod), regex("\\-|:"), pred(is_a_time_of_day)],
        |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_duration_after_time() -> Rule {
    rule(
        "<duration> after <time>",
        vec![dimension(Dim::Duration), regex("nakon|poslije|od"), dimension(Dim::Time)],
        |ts| tt(duration_after(dur(ts[0])?, td(ts[2])?)),
    )
}

fn rule_eveningnight() -> Rule {
    rule("evening|night", vec![regex(r"(na)?ve(c|č)er(i|as)?")], |_| {
        ttd(interval(Open, &hour(false, 18), &hour(false, 0))
            .map(|x| mk_ok_for_this_next(part_of_day(mk_latent(x)))))
    })
}

fn rule_ordinal_quarter() -> Rule {
    rule("<ordinal> quarter", vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))], |ts| {
        tt(cycle_nth_after(true, G::Quarter, get_int_value(ts[0])? - 1, &cycle_nth(G::Year, 0)))
    })
}

fn rule_day_before_day_before_yesterday() -> Rule {
    rule("day before day before yesterday", vec![regex(r"(prek\s?prekju(c|č)er)")], |_| {
        tt(cycle_nth(G::Day, -3))
    })
}

fn rule_duration_before_time() -> Rule {
    rule(
        "<duration> before <time>",
        vec![dimension(Dim::Duration), regex("prije"), dimension(Dim::Time)],
        |ts| tt(duration_before(dur(ts[0])?, td(ts[2])?)),
    )
}

fn rule_eoyend_of_year() -> Rule {
    rule("EOY|End of year", vec![regex(r"(the )?(EOY|(do )? kraja? godine)")], |_| {
        tt(cycle_nth(G::Year, 1))
    })
}

fn rule_tomorrow() -> Rule {
    rule("tomorrow", vec![regex(r"(sutra(dan)?)")], |_| tt(cycle_nth(G::Day, 1)))
}

fn rule_timeofday_oclock() -> Rule {
    rule("<time-of-day> o'clock", vec![pred(is_a_time_of_day), regex("sat(i|a)?")], |ts| {
        tt(not_latent(td(ts[0])?.clone()))
    })
}

fn rule_year2() -> Rule {
    rule("year", vec![pred(is_ordinal_between(1000, 2100)), regex("\\.|e")], |ts| {
        tt(year(get_int_value(ts[0])?))
    })
}

fn rule_dayofmonthordinal_namedmonth_year() -> Rule {
    rule(
        "<day-of-month>(ordinal) <named-month> year",
        vec![pred(is_dom_ordinal), pred(is_a_month), regex(r"(\d{2,4})")],
        |ts| {
            let y = parse_int(&rx(ts[2], 0)?)?;
            let dom = intersect_dom(td(ts[1])?, ts[0])?;
            ttd(intersect(&dom, &year(y)))
        },
    )
}

fn rule_hhmmss() -> Rule {
    rule(
        "hh:mm:ss",
        vec![regex(r"((?:[01]?\d)|(?:2[0-3]))[:.]([0-5]\d)[:.]([0-5]\d)")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let s = parse_int(&rx(ts[0], 2)?)?;
            tt(hour_minute_second(true, h, m, s))
        },
    )
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Monday", r"ponedjelja?ka?|pon\.?"),
        ("Tuesday", r"utora?ka?|uto?\.?"),
        ("Wednesday", r"srijed(a|e|u)|sri\.?"),
        ("Thursday", r"(č|c)etvrta?ka?|(č|c)et\.?"),
        ("Friday", r"peta?ka?|pet\.?"),
        ("Saturday", r"subot(a|e|u)|sub?\.?"),
        ("Sunday", r"nedjelj(a|e|u)|ned\.?"),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("January", r"sije(c|č)a?nj(a|u)?|januar(a|u)?|jan\.?|sij?\.?|prv(i|a|o(ga?)?)"),
        ("February", r"(ve)?lja(c|č)(a|e|i)|februar(a|u)?|feb\.?|ve(lj)?\.?|drug(i|a|o(ga?)?)"),
        ("March", r"o(z|ž)uja?k(a|u)?|mart(a|u)?|mar\.?|o(z|ž)u?\.?|tre(c|ć)(i|a|e(ga?)?)"),
        ("April", r"trava?nj(a|u)?|april(a|u)?|apr\.?|tra\.?|(č|c)etvrt(i|a|o(ga?)?)"),
        ("May", r"sviba?nj(a|u)?|maj|svi\.?|pet(i|a|o(ga?)?)"),
        ("June", r"lipa?nj(a|u)?|jun(i|u|a)?|jun\.?|lip?\.?|(š|s)est(i|a|o(ga?)?)"),
        ("July", r"srpa?nj(a|u)?|jul(i|u|a)?|jul\.?|srp\.?|sedm(i|a|o(ga?)?)"),
        ("August", r"kolovoz(a|u)?|august(a|u)?|aug\.?|kol\.?|osm(i|a|o(ga?)?)"),
        ("September", r"ruja?n(a|u)?|septemba?r(a|u)?|sept?\.?|ruj\.?|devet(i|a|o(ga?)?)"),
        ("October", r"listopad(a|u)?|oktobar(a|u)?|okt\.?|lis\.?|deset(i|a|o(ga?)?)"),
        ("November", r"studen(i|oga?|om)|novemba?r(a|u)?|nov\.?|stu\.?|jedanaest(i|a|o(ga?)?)"),
        ("December", r"prosina?c(a|u)?|decemba?r(a|u)?|dec\.?|pros?\.?|dvanaest(i|a|o(ga?)?)"),
    ])
}

fn rules_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("new year's eve", "star(a|u|e) godin(a|e|u)", month_day(12, 31)),
        ("new year's day", "nov(a|u|e) godin(a|e|u)", month_day(1, 1)),
        ("christmas eve", "badnjaka?", month_day(12, 24)),
        ("christmas", r"(zi(c|ć)bo|bo(z|ž)i(c|ć))(a|u|ni|na)?", month_day(12, 25)),
        ("halloween day", "no(c|ć) vje(s|š)tica", month_day(10, 31)),
        ("Mother's Day", "maj(c|č)in dan", nth_dow_of_month(2, 7, 5)),
        ("Father's Day", "dan (o(c|č)eva|tata)", nth_dow_of_month(3, 7, 6)),
        ("valentine's day", "valentinov(og?|a|)", month_day(2, 14)),
    ])
}

fn rules_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("summer", r"ljet(os?|a)", month_day(6, 21), month_day(9, 23)),
        ("fall", r"jesen(i|as)?", month_day(9, 23), month_day(12, 21)),
        ("winter", r"zim(a|e|us)?", month_day(12, 21), month_day(3, 20)),
        ("spring", r"prolje(c|ć)(e|a)", month_day(3, 20), month_day(6, 21)),
    ])
}

fn build_rules() -> Vec<Rule> {
    let mut v = vec![
        rule_about_duration(),
        rule_about_timeofday(),
        rule_absorption_of_after_named_day(),
        rule_after_duration(),
        rule_after_lunch(),
        rule_after_next_time(),
        rule_after_timeofday(),
        rule_after_work(),
        rule_afternoon(),
        rule_at_timeofday(),
        rule_before_lasttime(),
        rule_between_datetime_and_datetime_interval(),
        rule_between_timeofday_and_timeofday_interval(),
        rule_by_the_end_of_time(),
        rule_by_time(),
        rule_cycle_after_time(),
        rule_cycle_before_time(),
        rule_datetime_datetime_interval(),
        rule_day_after_tomorrow(),
        rule_day_before_day_before_yesterday(),
        rule_day_before_yesterday(),
        rule_dayofmonth_non_ordinal_namedmonth(),
        rule_dayofmonth_non_ordinal_of_namedmonth(),
        rule_dayofmonth_ordinal(),
        rule_dayofmonth_ordinal_of_namedmonth(),
        rule_dayofmonthordinal_namedmonth(),
        rule_dayofmonthordinal_namedmonth_year(),
        rule_ddmm(),
        rule_ddmmyyyy(),
        rule_duration_after_time(),
        rule_duration_before_time(),
        rule_duration_from_now(),
        rule_early_morning(),
        rule_eomend_of_month(),
        rule_eoyend_of_year(),
        rule_eveningnight(),
        rule_exactly_timeofday(),
        rule_for_duration(),
        rule_from_datetime_datetime_interval(),
        rule_from_timeofday_timeofday_interval(),
        rule_half_integer_hr_style_hourofday(),
        rule_hhmm(),
        rule_hhmm_military(),
        rule_hhmmss(),
        rule_in_duration(),
        rule_induring_the_partofday(),
        rule_intersect(),
        rule_intersect_by(),
        rule_last_cycle(),
        rule_last_cycle_of_time(),
        rule_last_dayofweek_time(),
        rule_last_dayofweek_time2(),
        rule_last_n_cycle(),
        rule_last_time(),
        rule_late_night(),
        rule_lunch(),
        rule_midnighteodend_of_day(),
        rule_month_dddd_interval(),
        rule_morning(),
        rule_namedday_dayofmonth_ordinal(),
        rule_namedmonth_dayofmonth_non_ordinal(),
        rule_namedmonth_dayofmonth_ordinal(),
        rule_next_cycle(),
        rule_next_n_cycle(),
        rule_next_time(),
        rule_noon(),
        rule_now(),
        rule_nth_time_after_time(),
        rule_nth_time_of_time(),
        rule_ordinal_cycle_after_time(),
        rule_ordinal_cycle_of_time(),
        rule_ordinal_quarter(),
        rule_ordinal_quarter_year(),
        rule_prije_duration(),
        rule_numeral_afterpast_hourofday(),
        rule_quarter_afterpast_hourofday(),
        rule_half_afterpast_hourofday(),
        rule_numeral_totillbefore_integer_hourofday(),
        rule_quarter_totillbefore_integer_hourofday(),
        rule_half_totillbefore_integer_hourofday(),
        rule_hourofday_numeral(),
        rule_hourofday_quarter(),
        rule_hourofday_half(),
        rule_since_timeofday(),
        rule_this_cycle(),
        rule_this_partofday(),
        rule_this_time(),
        rule_thisnext_dayofweek(),
        rule_time_after_next(),
        rule_time_before_last(),
        rule_time_partofday(),
        rule_timezone(),
        rule_timeofday_approximately(),
        rule_timeofday_latent(),
        rule_timeofday_oclock(),
        rule_timeofday_timeofday_interval(),
        rule_today(),
        rule_tomorrow(),
        rule_tonight(),
        rule_u_namedday(),
        rule_u_namedmonth(),
        rule_until_timeofday(),
        rule_weekend(),
        rule_within_duration(),
        rule_year(),
        rule_year2(),
        rule_year_latent(),
        rule_year_latent2(),
        rule_yesterday(),
        rule_yyyymmdd(),
        rule_za_numeral_hourofday(),
        rule_za_quarter_hourofday(),
        rule_za_half_hourofday(),
    ];
    v.extend(rules_days_of_week());
    v.extend(rules_months());
    v.extend(rules_holidays());
    v.extend(rules_seasons());
    v
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &RULES
}
