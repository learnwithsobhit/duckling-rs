//! `Duckling.Time.CA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::DurationData;
use crate::dims::time::helpers::*;
use crate::dims::time::types::{
    Form, TimeData,
    TimeIntervalType::{Closed, Open},
};
use crate::dims::time_grain::Grain as G;
use crate::dims::util::{parse_int, to_lower, to_upper};
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

/// `TimeData {form = Just (TimeOfDay (Just hours) is12H)}`.
fn tod(t: &Token) -> Option<(i64, bool)> {
    match t {
        Token::Time(TimeData { form: Some(Form::TimeOfDay { hours: Some(h), is12h }), .. }) => {
            Some((*h, *is12h))
        }
        _ => None,
    }
}

/// `Duckling.Duration.Helpers.isGrain` — CA uses this as a TimeGrain token check.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

/// `Token Time . mkLatent . partOfDay <$> interval Open (hour False a) (hour False b)`.
fn latent_pod(a: i64, b: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, a), &hour(false, b)).map(|x| mk_latent(part_of_day(x))))
}

fn rule_the_day_after_tomorrow() -> Rule {
    rule(
        "the day after tomorrow",
        vec![regex("dem(a|à) passat|l'endem(a|à)|passat dem(a|à)")],
        |_| tt(cycle_nth(G::Day, 2)),
    )
}

fn rule_hace_duration() -> Rule {
    rule("fa <duration>", vec![regex("fa"), dimension(Dim::Duration)], |ts| {
        tt(duration_ago(dur(ts[1])?))
    })
}

fn rule_ce_time() -> Rule {
    rule("ce <time>", vec![regex("aquest"), dimension(Dim::Time)], |ts| {
        tt(pred_nth(0, false, td(ts[1])?))
    })
}

fn rule_this_dayofweek() -> Rule {
    rule(
        "this <day-of-week>",
        vec![regex("aquest"), pred(is_a_day_of_week)],
        |ts| tt(pred_nth(0, true, td(ts[1])?)),
    )
}

fn rule_datetime_datetime_interval() -> Rule {
    rule(
        "<datetime> - <datetime> (interval)",
        vec![pred(is_not_latent), regex("\\s?-\\s?|\\sal?\\s"), pred(is_not_latent)],
        |ts| ttd(interval(Open, td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_evening() -> Rule {
    rule("evening", vec![regex("vespre")], |_| latent_pod(19, 22))
}

fn rule_matinada() -> Rule {
    rule("matinada", vec![regex("matinada")], |_| latent_pod(1, 6))
}

fn rule_nit() -> Rule {
    rule("nit", vec![regex("nit")], |_| latent_pod(22, 1))
}

fn rule_migdia() -> Rule {
    rule("migdia", vec![regex("migdia|mig dia")], |_| latent_pod(11, 14))
}

fn rule_day_of_month_st() -> Rule {
    rule("day of month (1st)", vec![regex("primer|u|1er( de)?")], |_| tt(day_of_month(1)))
}

fn rule_en_duration() -> Rule {
    rule("en <duration>", vec![regex("\\sen\\s"), dimension(Dim::Duration)], |ts| {
        tt(in_duration(dur(ts[1])?))
    })
}

fn rule_today() -> Rule {
    rule("today", vec![regex("avui")], |_| tt(today()))
}

fn rule_ultimo_dayofweek_de_time() -> Rule {
    rule(
        "ultimo <day-of-week> de <time>",
        vec![regex("darrer"), pred(is_a_day_of_week), regex("de|d'"), dimension(Dim::Time)],
        |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_entre_datetime_et_datetime_interval() -> Rule {
    rule(
        "entre <datetime> et <datetime> (interval)",
        vec![regex("(entre|entre les|entre la)"), dimension(Dim::Time), regex("(i|i les|i la)"), dimension(Dim::Time)],
        |ts| ttd(interval(Open, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_hhhmm_timeofday() -> Rule {
    rule(
        "hh(:|.|h)mm (time-of-day)",
        vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:h\\.]([0-5]\\d)")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(true, h, m))
        },
    )
}

fn rule_el_dayofmonth_de_namedmonth() -> Rule {
    rule(
        "el <day-of-month> de <named-month>",
        vec![regex("el|l'"), pred(is_dom_integer), regex("de|d'"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[3])?, ts[1])),
    )
}

fn rule_n_pasados_cycle() -> Rule {
    rule(
        "n pasados <cycle>",
        vec![pred(is_integer_between_tok(2, 9999)), regex("passa(ts?|da|des)|darrer(s|a|es)?"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_n(true, grain_of(ts[2])?, -get_int_value(ts[0])?)),
    )
}

fn rule_el_proximo_cycle() -> Rule {
    rule(
        "el proximo <cycle> ",
        // Upstream `((el|els|la|les) )?` also matches empty; keep the non-empty article only.
        vec![regex("((el|els|la|les) )"), regex("proper(a|s|es)?"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[2])?, 1)),
    )
}

fn rule_pasados_n_cycle() -> Rule {
    rule(
        "pasados n <cycle>",
        vec![regex("passa(ts?|da|des)|darrer(s|a|es)?"), pred(is_integer_between_tok(2, 9999)), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_n(true, grain_of(ts[2])?, -get_int_value(ts[1])?)),
    )
}

fn rule_el_dayofmonth_non_ordinal() -> Rule {
    rule(
        "el <day-of-month> (non ordinal)",
        vec![regex("el|l'"), pred(is_integer_between_tok(1, 31))],
        |ts| tt(day_of_month(get_int_value(ts[1])?)),
    )
}

fn rule_noon() -> Rule {
    rule("noon", vec![regex("migdia|mig dia")], |_| tt(hour(false, 12)))
}

fn rule_proximas_n_cycle() -> Rule {
    rule(
        "proximas n <cycle>",
        vec![regex("proper(a|s|es)?"), pred(is_integer_between_tok(2, 9999)), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_n(true, grain_of(ts[2])?, get_int_value(ts[1])?)),
    )
}

fn rule_nochevieja() -> Rule {
    rule("Nochevieja", vec![regex("(nit de cap d'any|darrer dia de l'any)")], |_| {
        tt(month_day(12, 31))
    })
}

fn rule_the_day_before_yesterday() -> Rule {
    rule("the day before yesterday", vec![regex("abans d'ahir")], |_| tt(cycle_nth(G::Day, -2)))
}

fn rule_duration_in() -> Rule {
    rule("in <duration>", vec![regex("en"), dimension(Dim::Duration)], |ts| {
        tt(in_duration(dur(ts[1])?))
    })
}

fn rule_duration_ago() -> Rule {
    rule("<duration> ago", vec![regex("fa"), dimension(Dim::Duration)], |ts| {
        tt(duration_ago(dur(ts[1])?))
    })
}

fn rule_hourofday_integer_as_relative_minutes() -> Rule {
    rule(
        "<hour-of-day> <integer> (as relative minutes)",
        vec![pred(is_an_hour_of_day), pred(is_integer_between_tok(1, 59))],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[1])?))
        },
    )
}

/// Upstream binds `(Token Time td:_)` as the first token (regex), so these campanar rules never fire.
fn campanar(name: &'static str, re: &'static str, minutes: i64) -> Rule {
    rule(name, vec![regex(re), pred(is_an_hour_of_day)], move |ts| ttd(minutes_before(minutes, td(ts[0])?)))
}

fn rule_hourofday_integer_as_relative_minutes2() -> Rule {
    rule(
        "<hour-of-day> <integer> (as relative minutes) minutes",
        vec![pred(is_an_hour_of_day), pred(is_integer_between_tok(1, 59)), regex("min\\.?(ut)?s?")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[1])?))
        },
    )
}

fn rule_hourofday_quarter() -> Rule {
    rule(
        "<hour-of-day> quarter (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("quart")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 15))
        },
    )
}

fn rule_hourofday_half() -> Rule {
    rule(
        "<hour-of-day> half (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("mitja|(2|dos)(\\squarts|\\/4)")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 30))
        },
    )
}

fn rule_hourofday_three_quarter() -> Rule {
    rule(
        "<hour-of-day> three quarters (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("(3|tres)(\\squarts|\\/4)")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 45))
        },
    )
}

fn rule_hourofday_and_relative_minutes() -> Rule {
    rule(
        "<hour-of-day> and <relative minutes>",
        vec![pred(is_an_hour_of_day), regex("i"), pred(is_integer_between_tok(1, 59))],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[2])?))
        },
    )
}

fn rule_hourofday_and_relative_minutes2() -> Rule {
    rule(
        "<hour-of-day> and <relative minutes> minutes",
        vec![
            pred(is_an_hour_of_day),
            regex("i"),
            pred(is_integer_between_tok(1, 59)),
            regex("min\\.?(ut)?s?"),
        ],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[2])?))
        },
    )
}

fn rule_hourofday_and_quarter() -> Rule {
    rule(
        "<hour-of-day> and quarter",
        vec![pred(is_an_hour_of_day), regex("i quart")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 15))
        },
    )
}

fn rule_hourofday_and_half() -> Rule {
    rule(
        "<hour-of-day> and half",
        vec![pred(is_an_hour_of_day), regex("i (mitja|(2|dos)(\\squarts|\\/4))")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 30))
        },
    )
}

fn rule_hourofday_and_three_quarter() -> Rule {
    rule(
        "<hour-of-day> and 3 quarters",
        vec![pred(is_an_hour_of_day), regex("i (tres|3)(\\squarts|\\/4)")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 45))
        },
    )
}

fn rule_del_year() -> Rule {
    rule(
        "del <year>",
        vec![regex("de(l| l'any)? "), pred(is_integer_between_tok(1000, 2100))],
        |ts| tt(year(get_int_value(ts[1])?)),
    )
}

fn rule_ddmm() -> Rule {
    rule(
        "dd[/-]mm",
        vec![regex("(3[01]|[12]\\d|0?[1-9])[/-](0?[1-9]|1[0-2])( |$)")],
        |ts| {
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        },
    )
}

fn rule_ddmmyyyy() -> Rule {
    rule(
        "dd[/-.]mm[/-.]yyyy",
        vec![regex("(3[01]|[12]\\d|0?[1-9])[\\./-](0?[1-9]|1[0-2])[\\./-](\\d{2,4})")],
        |ts| {
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let y = parse_int(&rx(ts[0], 2)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn rule_afternoon() -> Rule {
    rule("afternoon", vec![regex("tarda")], |_| latent_pod(14, 19))
}

fn rule_midnight() -> Rule {
    rule("midnight", vec![regex("mitjanit|mitja nit")], |_| tt(hour(false, 0)))
}

fn rule_pod_in() -> Rule {
    rule(
        "in the <part-of-day>",
        vec![regex("(a la|al|per la|pel|del)"), pred(is_a_part_of_day)],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    )
}

fn rule_pod_this() -> Rule {
    rule(
        "this <part-of-day>",
        vec![regex("aquesta?"), pred(is_a_part_of_day)],
        |ts| ttd(intersect(&today(), td(ts[1])?).map(|x| part_of_day(not_latent(x)))),
    )
}

fn rule_tod_pod() -> Rule {
    rule(
        "<time-of-day> <part-of-day>",
        vec![pred(is_a_time_of_day), pred(is_a_part_of_day)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[1])?).map(not_latent)),
    )
}

fn rule_ano_nuevo() -> Rule {
    rule("ano nuevo", vec![regex("cap d'any")], |_| tt(month_day(1, 1)))
}

fn rule_dddd_monthinterval() -> Rule {
    rule(
        "dd-dd <month>(interval)",
        vec![
            regex("(3[01]|[12]\\d|0?[1-9])"),
            regex("\\-|al?"),
            regex("(3[01]|[12]\\d|0?[1-9])"),
            regex("de|d'"),
            pred(is_a_month),
        ],
        |ts| {
            let d1 = parse_int(&rx(ts[0], 0)?)?;
            let d2 = parse_int(&rx(ts[2], 0)?)?;
            let t = td(ts[4])?;
            let from = intersect(&day_of_month(d1), t)?;
            let to = intersect(&day_of_month(d2), t)?;
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn rule_timeofday_latent() -> Rule {
    rule(
        "time-of-day (latent)",
        vec![pred(is_integer_between_tok(0, 23))],
        |ts| {
            let v = get_int_value(ts[0])?;
            tt(mk_latent(hour(v < 13, v)))
        },
    )
}

fn rule_namedmonthnamedday_past() -> Rule {
    rule(
        "<named-month|named-day> past",
        vec![dimension(Dim::Time), regex("passat")],
        |ts| tt(pred_nth(-1, false, td(ts[0])?)),
    )
}

/// Upstream binds `(Token Time td:_)` as the first token (regex), so this never fires.
fn rule_namedmonthnamedday_past2() -> Rule {
    rule(
        "<named-month|named-day> past",
        vec![regex("passat|darrer|l'ultim"), dimension(Dim::Time)],
        |ts| tt(pred_nth(-1, false, td(ts[0])?)),
    )
}

fn rule_right_now() -> Rule {
    rule(
        "right now",
        vec![regex("ara( mateix)?|ja|en aquests? moments?|tant aviat com puguis?")],
        |_| tt(now()),
    )
}

fn rule_n_cycle_vinent() -> Rule {
    rule(
        "n <cycle> (proximo|que viene)",
        vec![pred(is_integer_between_tok(2, 9999)), dimension(Dim::TimeGrain), regex("(vinents?)")],
        |ts| tt(cycle_n(true, grain_of(ts[1])?, get_int_value(ts[0])?)),
    )
}

fn rule_namedmonthnamedday_next() -> Rule {
    rule(
        "<named-month|named-day> next",
        vec![dimension(Dim::Time), regex("vinents?")],
        |ts| tt(pred_nth(1, false, td(ts[0])?)),
    )
}

fn rule_intersect() -> Rule {
    rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
        ttd(intersect(td(ts[0])?, td(ts[1])?))
    })
}

fn rule_de_datetime_datetime_interval() -> Rule {
    rule(
        "de <datetime> - <datetime> (interval)",
        vec![regex("(des )?del?"), dimension(Dim::Time), regex("\\-|(fins )?al?"), dimension(Dim::Time)],
        |ts| ttd(interval(Open, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_nth_time_de_time2() -> Rule {
    rule(
        "nth <time> de <time>",
        vec![regex("(el|la)"), dimension(Dim::Ordinal), dimension(Dim::Time), regex("de|d'"), dimension(Dim::Time)],
        |ts| {
            let v = ordinal_value(ts[1])?;
            ttd(intersect(td(ts[4])?, td(ts[2])?).map(|x| pred_nth(v - 1, false, &x)))
        },
    )
}

fn rule_dentro_de_duration() -> Rule {
    rule(
        "dentro de <duration>",
        vec![regex("dintre de"), dimension(Dim::Duration)],
        |ts| ttd(interval(Open, &now(), &in_duration(dur(ts[1])?))),
    )
}

fn rule_weekend() -> Rule {
    rule("week-end", vec![regex("week[ -]?end|cap de setmana")], |_| tt(weekend()))
}

fn rule_ordinal_quarter_year() -> Rule {
    rule(
        "<ordinal> quarter <year>",
        vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter)), regex("del? ?"), dimension(Dim::Time)],
        |ts| {
            let v = ordinal_value(ts[0])?;
            tt(cycle_nth_after(false, G::Quarter, v - 1, td(ts[3])?))
        },
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

/// Upstream binds `(Token Time td:_)` as the first token (regex), so this never fires.
fn rule_timeofday_horas() -> Rule {
    rule(
        "<time-of-day> horas",
        // Upstream `(la | les )?` also matches empty; keep the non-empty article only.
        vec![regex("(la | les )"), pred(is_a_time_of_day), regex("h?\\.?(or(e|a))?s?")],
        |ts| tt(mk_latent(td(ts[0])?.clone())),
    )
}

fn rule_el_cycle_antes_time() -> Rule {
    rule(
        "el <cycle> antes <time>",
        // Upstream `(el|la|l')? ?` also matches empty; keep every non-empty variant.
        vec![regex("(el|la|l') ?|(el|la|l')| "), dimension(Dim::TimeGrain), regex("abans de"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, -1, td(ts[3])?)),
    )
}

fn rule_two_time_tokens_separated_by() -> Rule {
    rule(
        "two time tokens separated by \",\"",
        vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_morning() -> Rule {
    rule("morning", vec![regex("mat(i|í)")], |_| latent_pod(6, 12))
}

fn rule_a_las_hourmintimeofday() -> Rule {
    rule(
        "a las <hour-min>(time-of-day)",
        vec![regex("a (l'|les)"), pred(is_a_time_of_day), regex("horas?")],
        |ts| Some(ts[1].clone()),
    )
}

fn rule_la_cycle_pasado() -> Rule {
    rule(
        "la <cycle> pasado",
        // Upstream `(el|els|la|les|l')?` also matches empty; keep the non-empty article only.
        vec![regex("(el|els|la|les|l')"), dimension(Dim::TimeGrain), regex("passa(t|da|ts|des)")],
        |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
    )
}

fn rule_year_latent() -> Rule {
    rule("year (latent)", vec![pred(is_integer_between_tok(25, 10000))], |ts| {
        tt(mk_latent(year(get_int_value(ts[0])?)))
    })
}

fn rule_yesterday() -> Rule {
    rule("yesterday", vec![regex("ahir")], |_| tt(cycle_nth(G::Day, -1)))
}

fn rule_dayofweek_dayofmonth() -> Rule {
    rule(
        "<day-of-week> <day-of-month>",
        vec![pred(is_a_day_of_week), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_timeofday_ampm() -> Rule {
    rule(
        "<time-of-day> am|pm",
        vec![pred(is_a_time_of_day), regex("([ap])\\.?m?\\.?")],
        |ts| tt(time_of_day_ampm(to_lower(&rx(ts[1], 0)?) == "a", td(ts[0])?)),
    )
}

fn rule_dayofmonth_de_namedmonth() -> Rule {
    rule(
        "<day-of-month> de <named-month>",
        vec![pred(is_dom_integer), regex("de|d'"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
    )
}

fn rule_entre_dd_et_dd_monthinterval() -> Rule {
    rule(
        "entre dd et dd <month>(interval)",
        vec![
            regex("entre( el)?|des del?"),
            regex("(0?[1-9]|[12]\\d|3[01])"),
            regex("i( el)?|fins al?"),
            regex("(0?[1-9]|[12]\\d|3[01])"),
            regex("de|d'"),
            pred(is_a_month),
        ],
        |ts| {
            let v1 = parse_int(&rx(ts[1], 0)?)?;
            let v2 = parse_int(&rx(ts[3], 0)?)?;
            let t = td(ts[5])?;
            let from = intersect(&day_of_month(v1), t)?;
            let to = intersect(&day_of_month(v2), t)?;
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn rule_namedmonth_dayofmonth() -> Rule {
    rule(
        "<named-month> <day-of-month>",
        vec![pred(is_a_month), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_el_time() -> Rule {
    rule("el <time>", vec![regex("d?el"), pred(is_not_latent)], |ts| Some(ts[1].clone()))
}

fn rule_year() -> Rule {
    rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
        tt(year(get_int_value(ts[0])?))
    })
}

fn rule_esteen_un_cycle() -> Rule {
    rule(
        "este|en un <cycle>",
        vec![regex("(aquesta?|dins (els?|la|l'|les)?)"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
    )
}

fn rule_n_proximas_cycle() -> Rule {
    rule(
        "n proximas <cycle>",
        vec![pred(is_integer_between_tok(2, 9999)), regex("proper(a|es|s)?"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_n(true, grain_of(ts[2])?, get_int_value(ts[0])?)),
    )
}

fn rule_la_pasado_cycle() -> Rule {
    rule(
        "la pasado <cycle>",
        vec![regex("(el|els|la|les) ?"), regex("passa(ts?|da|des)|darrer(a|s|es)?"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[2])?, -1)),
    )
}

fn rule_a_las_timeofday() -> Rule {
    rule(
        "a las <time-of-day>",
        // Upstream `a?( las?)?|las?` also matches empty; reject the empty match (engine panics
        // on a zero-width hit at end-of-string). Non-empty matches are unchanged.
        vec![regex("a( las?)?| ?las?"), pred(is_a_time_of_day)],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    )
}

fn rule_ordinal_quarter() -> Rule {
    rule(
        "<ordinal> quarter",
        vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))],
        |ts| {
            let v = ordinal_value(ts[0])?;
            tt(cycle_nth_after(false, G::Quarter, v - 1, &cycle_nth(G::Year, 0)))
        },
    )
}

fn rule_el_cycle_vinent() -> Rule {
    rule(
        "el <cycle> (proximo|que viene)",
        vec![regex("(el|els|la|les|l') ?"), dimension(Dim::TimeGrain), regex("(vinents?)")],
        |ts| tt(cycle_nth(grain_of(ts[1])?, 1)),
    )
}

fn rule_el_cycle_vinent_time() -> Rule {
    rule(
        "el <cycle> proximo|que viene <time>",
        vec![regex("(el|els|la|les|l') ?"), dimension(Dim::TimeGrain), regex("(vinents?)"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, 1, td(ts[3])?)),
    )
}

fn rule_del_medioda() -> Rule {
    rule("del mediodía", vec![regex("del migdia")], |_| latent_pod(12, 17))
}

fn rule_intersect_by_de() -> Rule {
    rule(
        "intersect by `de`",
        vec![pred(is_not_latent), regex("de|d'"), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_tomorrow() -> Rule {
    rule("tomorrow", vec![regex("dem(a|à)")], |_| tt(cycle_nth(G::Day, 1)))
}

fn rule_nth_time_de_time() -> Rule {
    rule(
        "nth <time> de <time>",
        vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("de|d'"), dimension(Dim::Time)],
        |ts| {
            let v = ordinal_value(ts[0])?;
            ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(v - 1, false, &x)))
        },
    )
}

fn rule_timezone() -> Rule {
    rule(
        "<time> timezone",
        vec![
            pred(|t| is_a_time_of_day(t) && is_not_latent(t)),
            regex("\\b(YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT)\\b"),
        ],
        |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
    )
}

fn rule_next_week() -> Rule {
    rule("next week", vec![pred(is_grain(G::Week)), regex("vinent")], |_| {
        tt(cycle_nth(G::Week, 1))
    })
}

fn rule_el_dayofmonth_non_ordinal_with_dia() -> Rule {
    rule(
        "dia <day-of-month> (non ordinal)",
        vec![regex("dia"), pred(is_integer_between_tok(1, 31))],
        |ts| tt(day_of_month(get_int_value(ts[1])?)),
    )
}

fn rule_next_week_alt() -> Rule {
    rule(
        "next week (alt)",
        vec![regex("proper(a|s|es)|següents?"), pred(is_grain(G::Week))],
        |_| tt(cycle_nth(G::Week, 1)),
    )
}

fn rule_year_by_adding_three_numbers() -> Rule {
    rule(
        "year (value by adding three composing numbers together)",
        vec![regex("mil"), pred(is_integer_between_tok(100, 1000)), pred(is_integer_between_tok(1, 100))],
        |ts| {
            let v1 = get_int_value(ts[1])?;
            let v2 = get_int_value(ts[2])?;
            tt(year(1000 + v1 + v2))
        },
    )
}

fn rule_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Dilluns", "dilluns|dl\\.?" ),
        ("Dimarts", "dimarts|dm\\.?" ),
        ("Dimecres", "dimecres|dc\\.?" ),
        ("Dijous", "dijous|dj\\.?" ),
        ("Divendres", "divendres|dv\\.?" ),
        ("Dissabte", "dissabte|ds\\.?" ),
        ("Diumenge", "diumenge|dg\\.?" ),
    ])
}

fn rule_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("Gener", "gener|gen\\.?"),
        ("Febrer", "febrer|feb\\.?"),
        ("Març", "març|mar\\.?"),
        ("Abril", "abril|abr\\.?"),
        ("Maig", "maig?\\.?"),
        ("Juny", "juny?\\.?"),
        ("Juliol", "juliol|jul\\.?"),
        ("Agost", "agost|ago\\.?"),
        ("Setembre", "setembre|set\\.?"),
        ("Octubre", "octubre|oct\\.?"),
        ("Novembre", "novembre|nov\\.?"),
        ("Desembre", "desembre|des\\.?"),
    ])
}

fn rule_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("estiu", "estiu", month_day(6, 21), month_day(9, 23)),
        ("tardor", "tardor", month_day(9, 23), month_day(12, 21)),
        ("hivern", "hivern", month_day(12, 21), month_day(3, 20)),
        ("primavera", "primavera", month_day(3, 20), month_day(6, 21)),
    ])
}

fn rule_periodic_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("Navidad", "(el )?nadal", month_day(12, 25)),
        ("Dia de África", "dia de (la (libertad|liberacion) )?(á|a)frica(na)?", month_day(5, 25)),
        ("Dia de la Industrialización de África", "dia de (la )?industrializaci(ó|o)n de (á|a)frica(na)?", month_day(11, 20)),
        ("Diada de Tots Sants", "((dia|diada|festivitat) de )?tots sants", month_day(11, 1)),
        ("Dia de difunts", "(dia|diada) de(ls)? difunts", month_day(11, 2)),
        ("Epifanía", "Epifania|((dia|diada) dels)? reis( mags)?( d'orient)?", month_day(1, 6)),
        ("Festivitat de la Purissima", "(dia|diada|festivitat) de la (purissima|immaculada) (concepci(ó|o)( de (maria|la mare de déu))?)?", month_day(12, 8)),
        ("Assumpció de Maria", "assumpci(ó|o) de (mar(í|i)a|la mare de déu)", month_day(8, 15)),
        ("Diada de Sant Jordi", "((dia|diada|festivitat) de )?sant jordi", month_day(4, 23)),
        ("Diada de Sant Esteve", "((dia|diada|festivitat) de )?sant esteve", month_day(12, 26)),
        ("Diada de Sant Joan", "((dia|diada|festivitat) de )?sant joan", month_day(6, 24)),
        ("Revetlla de Sant Joan", "(revetlla|nit) de sant joan", month_day(6, 23)),
        ("Diada de Sant Narcís", "((dia|diada|festivitat) de )?sant narc(i|í)s", month_day(10, 29)),
        ("Diada de Sant Josep", "((dia|diada|festivitat) de )?sant? jos(e|é)p?", month_day(3, 19)),
        ("Diada de la Mercè", "((dia|diada|festivitat) de )?(la mare de déu de)? la merc(è|é)", month_day(9, 24)),
        ("Diada de Santa Tecla", "((dia|diada|festivitat) de )?santa tecla", month_day(9, 23)),
        ("Diada de Sant Anastasi", "((dia|diada|festivitat) de )?sant anastasi", month_day(5, 11)),
        ("Diada de Montserrat", "((dia|diada|festivitat) de )?(la mare de déu de)? montserrat", month_day(4, 27)),
        ("Dia dels Innocents d'Abril", "dia (dels innocents|les bromes)( d'abril)?", month_day(4, 1)),
        ("Dia Mundial de la Lengua Àrab", "dia mundial de la lengua (á|a)rabe", month_day(12, 18)),
        ("Dia de la Boxa", "(el )?dia de (la boxa|l'(B|b)oxeador)", month_day(12, 26)),
        ("Dia de Llengua Xinesa", "dia de llengua xinesa", month_day(4, 20)),
        ("Dia de Commemoració de les Víctimes de la Guerra Química", "dia (de )?(commemoratiu|conmemoraci(ó|o)) (en record )?de las v(í|i)ctimes de la guerra qu(í|i)mica", month_day(4, 29)),
        ("Dia de Commemoració de les Víctimes de l'Genocidi de Rwanda", "dia (de )?(commemoratiu|conmemoraci(ó|o)) (en record )?de las v(í|i)ctimes de l'genocidi de (Rw|Ru|rw|ru)anda", month_day(4, 7)),
        ("Dia de la Marina", "dia (de la marina|l' mar(í|i))", month_day(6, 25)),
        ("Dia de la Terra", "dia (internacional )?de la (mare )?terra", month_day(4, 22)),
        ("Dia de la Llengua Anglesa", "dia de la llengua anglesa", month_day(4, 23)),
        ("Les Festes de Sant Francesc d'Assís", "(les )?festes de sant (francesc d'as(í|i)s|pacho)", month_day(10, 4)),
        ("Dia Mundial de les Mares i dels Pares", "dia mundial de les mades y dels pares", month_day(6, 1)),
        ("Dia dels Drets Humans", "dia dels drets humans", month_day(12, 10)),
        ("Dia Internacional de Sensibilització sobre el Albinismo", "dia internacional de sensibilitzaci(ó|o) sobre l'albinisme", month_day(6, 13)),
        ("Dia Internacional contra la Corrupció", "dia internacional contra la corrupci(ó|o)", month_day(12, 9)),
        ("Dia Internacional dels Asteroides", "dia internacional dels asteroides", month_day(6, 30)),
        ("Celebra el Dia Internacional de la Bisexualitat", "celebra el dia internacional de la bisexualitat", month_day(9, 23)),
        ("Dia Internacional de Recordación de Desastres de Txernòbil", "dia internacional de record de desastres de Txern(ò|o)bil", month_day(4, 26)),
        ("Dia Internacional de l'Aviació Civil", "dia internacional de l'aviaci(ó|o) civil", month_day(12, 7)),
        ("Dia Internacional de la Duana", "dia internacional de l(a|es) duan(a|es)", month_day(1, 26)),
        ("Dia Internacional de la Lluita contra l'Ús Indegut i el Tràfic Il·lícit de Drogues", "dia internacional de la lluita contra l'(ú|u)s indegut i el tr(à|a)fic il·l(í|i)cit de drogues", month_day(6, 26)),
        ("Dia Internacional contra els Assajos Nuclears", "dia internacional contra els assajos nuclears", month_day(8, 29)),
        ("Dia Internacional de la Diversitat Biològica", "dia internacional de la diversitat biol(ò|o)gica", month_day(5, 22)),
        ("Dia Internacional dels Monuments i Llocs", "dia internacional dels monuments i llocs", month_day(4, 18)),
        ("Dia Internacional per a la Prevenció de l'Explotació de l'entorn en la Guerra i els Conflictes Armats", "dia internacional per a la prevenci(ó|o) de l'explotaci(ó|o) de l'entorn a la guerra i els conflictes armats", month_day(11, 6)),
        ("Dia de les Nacions Unides per a la Cooperació Sud-Sud", "dia de les nacions unides per a la cooperació sud(\\-|\\s)sud", month_day(9, 12)),
        ("Dia Internacional per la Tolerància", "dia internacional (de|per) la toler(à|a)ncia", month_day(11, 16)),
        ("Dia Internacional per a l'Abolició de l'Esclavitud", "dia internacional per a l'abolici(ó|o) de l'esclavitud", month_day(12, 2)),
        ("Dia Internacional de l'Eliminació de la Discriminació Racial", "dia internacional de l'eliminaci(ó|o) de la discriminaci(ó|o) racial", month_day(3, 21)),
        ("Dia Internacional de l'Eliminació de la Violència Sexual en els Conflictes", "dia internacional de l'eliminaci(ó|o) de la viol(è|e)ncia sexual en els conflictes", month_day(6, 19)),
        ("Dia Internacional de l'Eliminació de la Violència contra la Dona", "dia internacional de l'eliminaci(ó|o) de la viol(è|e)ncia contra (la dona|les dones)", month_day(11, 25)),
        ("Dia Internacional per a l'Eradicació de la Pobresa", "dia internacional per a l'eradicaci(ó|o) de la pobresa", month_day(10, 17)),
        ("Dia Internacional de la Preservació de la Capa d'Ozó", "dia internacional de la preservaci(ó|o) de la capa d'oz(ó|o)", month_day(9, 16)),
        ("Dia Internacional de l'Record del Tràfic d'Esclaus i la seva Abolició", "dia internacional de l'record del tr(à|a)fic d'esclaus i de la seva abolici(ó|o)", month_day(8, 23)),
        ("Dia Internacional del Derecho a la Verdad en relación con Violaciones Graves de los Derechos Humanos y de la Dignidad de las Víctimas de las Naciones Unidas", "dia internacional del derecho a la verdad en relaci(ó|o)n con violaciones graves de los derechos humanos y de la dignidad de las v(í|i)ctimas de las naciones unidas", month_day(3, 24)),
        ("Dia Internacional per a l'Eliminació Total de les Armes Nuclears", "dia internacional per a l'eliminaci(ó|o) total de les armes nuclears", month_day(9, 26)),
        ("Dia Internacional de les Nacions Unides a Support de les Víctimes de la Tortura", "dia internacional de les Nacions Unides en support de les v(í|i)ctimes de la tortura", month_day(6, 26)),
        ("Dia Internacional de la Beneficència", "dia internacional de la benefic(è|e)ncia", month_day(9, 5)),
        ("Dia Internacional de Commemoració anual en memòria de les Víctimes de l'Holocaust", "dia internacional de commemoraci(ó|o) anual en mem(ò|o)ria de les v(í|i)ctimes de l'holocaust", month_day(1, 27)),
        ("Dia Internacional de la Democràcia", "dia (internacional)?de la democr(à|a)cia", month_day(9, 15)),
        ("Dia Internacional de les Persones amb Discapacitat", "dia internacional de les persones amb discapacitat", month_day(12, 3)),
        ("Dia Internacional de la Família", "dia internacional de l(a|es)? fam(í|i)li(a|es)?", month_day(5, 15)),
        ("Dia Internacional de les Remeses Familiars", "dia internacional de les remeses familiars", month_day(6, 16)),
        ("Dia Internacional dels Boscos", "dia internacional dels boscos", month_day(3, 21)),
        ("Dia Internacional de l'Amistat", "dia internacional de l'amistat", month_day(7, 30)),
        ("Dia Internacional de la Felicitat", "dia (internacional|mundial) de la felicitat", month_day(3, 20)),
        ("Dia Internacional dels Vols Espacials", "dia internacional (dels|del) vol(s)? espacial(s)?", month_day(4, 12)),
        ("Dia Internacional dels Nens Víctimes Innocents d'Agressió", "dia internacional dels nens v(í|i)ctimes innocents d'agressi(ó|o)", month_day(6, 4)),
        ("Dia Internacional de la no Violència", "dia internacional de la no viol(è|e)ncia", month_day(10, 2)),
        ("Dia Internacional de l'Nowruz", "dia internacional de l'Nowruz", month_day(3, 21)),
        ("Dia Internacional de la Gent Gran", "dia internacional de la gent gran", month_day(10, 1)),
        ("Dia Internacional de la Pau", "dia internacional de la pau", month_day(9, 21)),
        ("Dia Internacional de les Persones amb Discapacitat", "dia internacional de les persones amb discapacitat", month_day(12, 3)),
        ("Dia Internacional de Record de les Víctimes de l'Esclavitud i el Tràfic Transatlàntica d'Esclaus", "dia internacional de record de les v(í|i)ctimes de l'esclavitud i la tracta transatl(à|a)ntica d'esclaus", month_day(3, 25)),
        ("Dia Internacional de les Dones Rurals", "dia internacional de les dones rurals", month_day(10, 15)),
        ("Dia Internacional de Solidaritat amb els membres de l'Personal Detinguts o Desapareguts", "dia internacional de solidaritat amb els membres de l'personal detinguts o desapareguts", month_day(3, 25)),
        ("Dia Internacional de Solidaritat amb el Poble Palestí", "dia internacional de solidaritat amb el poble palestí", month_day(11, 29)),
        ("Dia Internacional de l'Esport per al Desenvolupament i la Pau", "dia internacional de l'esport per al desenvolupament i la pau", month_day(4, 6)),
        ("Dia Internacional de l'Personal de Pau de les Nacions Unides", "dia internacional de el personal de pau de les Nacions Unides", month_day(5, 29)),
        ("Dia Internacional de la Dona i la Nena a la Ciència", "dia internacional de la dona i la nena en la ci(è|e)ncia", month_day(2, 11)),
        ("Dia Internacional de l'Ioga", "dia Internacional de l'(I|i|Y|i)oga", month_day(6, 21)),
        ("Dia Internacional de Tolerància Zero amb la Mutilació Genital Femenina", "dia internacional de toler(à|a)ncia zero amb la mutilaci(ó|o) genital femenina", month_day(2, 6)),
        ("Dia Internacional de la Nena", "dia internacional de la nena", month_day(10, 11)),
        ("Dia Internacional de les Víctimes de Desaparicions Forçades", "dia internacional de les v(í|i)ctimes de desaparicions forçades", month_day(8, 30)),
        ("Dia Internacional dels Pobles Indígenes de l'Món", "dia internacional dels pobles ind(í|i)genes de l'm(ó|o)n", month_day(8, 9)),
        ("Dia Internacional per posar fi a la impunitat dels Crims contra Periodistes", "dia internacional per posar fi a la impunitat dels crims contra periodistes", month_day(11, 2)),
        ("Dia Internacional per a l'Eradicació de la Fístula Obstètrica", "dia internacional per a l'eradicaci(ó|o) de la f(í|i)stula obst(è|e)trica", month_day(5, 23)),
        ("Dia Internacional per a la Reducció dels Desastres Naturals", "dia internacional per a la reducci(ó|o) dels desastres (naturals)?", month_day(10, 13)),
        ("Dia Internacional de la Solidaritat Humana", "dia internacional de la solidaritat humana", month_day(12, 20)),
        ("Dia Internacional de l'Jazz", "dia internacional de jazz", month_day(4, 30)),
        ("Dia Internacional de l'Alfabetització", "dia internacional de l'alfabetitzaci(ó|o)", month_day(9, 8)),
        ("Dia Internacional de l'Home", "dia internacional de(l)? els home(s)?", month_day(11, 19)),
        ("Dia Internacional de l'Migrant", "dia internacional de l'migrant", month_day(12, 18)),
        ("Dia Internacional de la Llengua Materna", "dia (internacional|mundial) de la llengua materna", month_day(2, 21)),
        ("Dia Internacional de les Muntanyes", "dia internacional de les muntanyes", month_day(12, 11)),
        ("Dia Internacional de les Infermeres", "dia internacional de les infermeres", month_day(5, 12)),
        ("Dia de la Consciència Internacional sobre la Sobredosi", "dia de (la)? consciència internacional (de|sobre) la sobredosi", month_day(8, 31)),
        ("Dia Internacional de l'Voluntariat per al Desenvolupament Econòmic i Social", "dia internacional de l'voluntariat per al desenvolupament econ(ò|o)mic i social", month_day(12, 5)),
        ("Dia Internacional de les Vídues", "dia internacional de les v(í|i)dues", month_day(6, 23)),
        ("Dia Internacional de la Dona", "dia internacional de la dona", month_day(3, 8)),
        ("Dia Internacional de la Joventut", "dia internacional de la joventut", month_day(8, 12)),
        ("El Primer de Maig", "(el )?primer de maig", month_day(5, 1)),
        ("Dia Internacional de Nelson Mandela", "dia (internacional)? de nelson mandela", month_day(7, 18)),
        ("El Nadal Ortodoxa", "(la )?nadal ortodoxa", month_day(1, 7)),
        ("Any Nou Ortodox", "(l')?any (nou vell|vell ortodox|nou ortodox|nou juli(à|a))", month_day(1, 14)),
        ("Dia Mundial dels Serveis Públics", "dia (mundial|nacional) de(ls )?(serveis|servidor) p(ú|u)blic(s)?", month_day(6, 23)),
        ("Dia de Sant Patrici", "dia de sant patrici", month_day(3, 17)),
        ("Dia Internacional de l'Temps de l'Record i la Reconciliació per qui Van perdre la Vida durant la Segona Guerra Mundial", "dia internacional de el temps de l'record i la reconciliaci(ó|o) per als que van perdre la vida durant la segona guerra mundial", month_day(5, 8)),
        ("Dia de les Nacions Unides", "dia de les nacions unides", month_day(10, 24)),
        ("Dia Internacional d'Informació sobre el Perill de les Mines i d'Assistència per a les Activitats Relatives a les Mines", "dia internacional d'informaci(ó|o) sobre el perill de les mines i d'assist(è|e)ncia per a les activitats relatives a les mines", month_day(4, 4)),
        ("Dia Mundial de la Salut", "dia mundial de la salut (de les nacions unides)?", month_day(4, 7)),
        ("Dia Universal del Nen", "dia universal de( l'infant|l nen)", month_day(11, 20)),
        ("Dia de Sant Valentí", "(el )?dia de sant valent(í|i)", month_day(2, 14)),
        ("Dia Mundial de la lluita contra la SIDA", "dia mundial de( la lluita contra)?( la)?SIDA", month_day(12, 1)),
        ("Dia Mundial de Conscienciació sobre l'Autisme", "dia mundial de (la concienciaci(ó|o) sobre )?l'autismo", month_day(4, 2)),
        ("Dia Mundial de la Artritis Reumatoide", "dia mundial de l'artritis (reumatoide)?", month_day(5, 20)),
        ("Dia Mundial de l'Donant de Sang", "dia mundial de l'donant de sang", month_day(6, 14)),
        ("Dia Internacional de l'Llibre i de el Dret d'Autor", "dia internacional de l'libro i de( el|ls) dret(s)? d'autor", month_day(4, 23)),
        ("Dia Mundial de l'Braille", "dia mundial de l'braille", month_day(1, 4)),
        ("Dia Mundial contra el Càncer", "dia mundial contra el c(à|a)ncer", month_day(2, 4)),
        ("Dia Mundial de les Ciutats", "dia mundial de les ciutats", month_day(10, 31)),
        ("Dia Mundial de la PC", "dia mundial de la (pc|par(à|a)lisi cerebral)", month_day(10, 6)),
        ("Dia Mundial contra el Treball Infantil", "dia mundial contra el treball infantil", month_day(6, 12)),
        ("Dia Mundial contra el Tràfic de Persones", "dia mundial contra el tr(à|a)fic de persones", month_day(7, 30)),
        ("Dia Mundial de l'Patrimoni Audiovisual", "dia mundial (de l'|sobre el) patrimoni audiovisual", month_day(10, 27)),
        ("Dia Mundial de la Diversitat Cultural per al Diàleg i el Desenvolupament", "dia mundial de la diversitat cultural per al di(a|à)leg i el desenvolupament", month_day(5, 21)),
        ("Dia Mundial de la Seguretat i la Salut en el Treball", "dia mundial de la seguretat i la salut en el treball", month_day(4, 28)),
        ("Dia Mundial per l'Abolició de l'Esclavitud", "dia mundial per l'abolici(ó|o) de l'esclavitud", month_day(12, 2)),
        ("Dia Mundial de la Justícia Social", "dia mundial de la just(í|i)cia social", month_day(2, 20)),
        ("Dia Mundial de l'Malalt", "dia mundial (de l'|dels) malalts", month_day(2, 11)),
        ("Dia Mundial de Lluita contra la Desertificació i la Sequera", "dia mundial (de lluita contra|per combatre) la desertificaci(ó|o) i la sequera", month_day(6, 17)),
        ("Dia Mundial d'Informació sobre el Desenvolupament", "dia mundial d'informaci(ó|o) sobre el desenvolupament", month_day(10, 24)),
        ("Dia Mundial de la Diabetis", "dia mundial de la diabetis", month_day(11, 14)),
        ("Dia Mundial de la Síndrome de Down", "dia mundial de la s (í|i)ndrome de Down", month_day(3, 21)),
        ("Dia Mundial de Presa de Consciència de l'Abús i Maltractament en la Vellesa", "dia mundial de presa de consci(è|e)ncia de l'ab(ú|u)s i maltractament en la vellesa", month_day(6, 15)),
        ("Dia Mundial de l'Medi Ambient", "dia mundial de l'(entorn|medi ambient)", month_day(6, 5)),
        ("Dia Mundial de l'Alimentació", "dia mundial de l'alimentaci(ó|o)", month_day(10, 16)),
        ("Dia per a la Commemoració i Dignificació de les Víctimes de l'Crim de Genocidi i per a la seva prevenció", "dia per a la commemoraci(ó|o) i dignificaci(ó|o) de les v(í|i)ctimes de l'crim de genocidi i per a la seva prevenci(ó|o)", month_day(12, 9)),
        ("Dia Mundial de l'Cor", "dia mundial de l'cor", month_day(9, 29)),
        ("Dia Mundial de l'Hepatitis", "dia mundial (contra|de) l'hepatitis", month_day(7, 28)),
        ("Dia Mundial de l'Assistència Humanitària", "dia mundial de l'assist(è|e)ncia humanit(à|a)ria", month_day(8, 19)),
        ("Dia Internacional de la Societat de la Informació", "dia (internacional|mundial) de (les telecomunicacions i)? la societat de la informaci(ó|o)", month_day(5, 17)),
        ("Dia Mundial de la Propietat Intel·lectual", "dia mundial de la propietat intel·lectual", month_day(4, 26)),
        ("Dia Mundial de la Malària", "dia mundial de la mal(à|a)ria", month_day(4, 25)),
        ("Dia Mundial de la Salut Mental", "dia mundial de la salut mental", month_day(10, 10)),
        ("Dia Mundial de la Meteorologia", "dia (mundial de (la )?meteorologia|meteorol(ò|o)gic mundial)", month_day(3, 23)),
        ("Dia Mundial sense Tabac", "dia mundial sense tabac", month_day(5, 31)),
        ("Dia Mundial dels Oceans", "dia mundial dels oceans", month_day(6, 8)),
        ("Dia Mundial de l'Càncer de Ovari", "dia mundial de l'c(à|a)ncer d'ovari", month_day(5, 8)),
        ("Dia Mundial contra la Pneumònia", "dia mundial contra la pneum(ò|o)nia", month_day(11, 12)),
        ("Dia Mundial de la Poesia", "dia mundial de la poesia", month_day(3, 21)),
        ("Dia Mundial de la Població", "dia mundial de la poblaci(ó|o)", month_day(7, 11)),
        ("Dia Mundial del Correo", "Dia Mundial de l'Correu", month_day(10, 9)),
        ("Dia de la Prematuridad Mundial", "dia (de la prematuritat mundial|mundial de l'infant prematur)", month_day(11, 17)),
        ("Dia Mundial de la Llibertat de Premsa", "dia mundial de la llibertat de premsa", month_day(5, 3)),
        ("Dia Mundial contra la Ràbia", "dia mundial (contra | de) la r(à|a)bia", month_day(9, 28)),
        ("Dia Mundial de la Ràdio", "dia mundial de la r(à|a)dio", month_day(2, 13)),
        ("Dia Mundial dels Refugiats", "dia mundial (de l'|dels) refugiat(s)?", month_day(6, 20)),
        ("Dia Mundial de la Ciència per la Pau i el Desenvolupament", "dia mundial de la ci(è|e)ncia per a la pau i el desenvolupament", month_day(11, 10)),
        ("Dia Mundial de la Salut Sexual", "dmss|dia mundial de la salut sexual", month_day(9, 4)),
        ("Dia Mundial de l'Sòl", "dia mundial de terra", month_day(12, 5)),
        ("Dia Mundial de l'Accident Cerebrovascular", "dia mundial del (accident cerebrovascular|acv)", month_day(10, 29)),
        ("Dia Mundial per a la Prevenció de l'Suïcidi", "dia mundial per a la prevenci(ó|o)de l'su(ï|i)cidi", month_day(9, 10)),
        ("Dia Mundial dels Docents", "dia mundial (dels docents|de l'mestre)", month_day(10, 5)),
        ("Dia Mundial de la Televisió", "dia mundial de la televisi(ó|o)", month_day(11, 21)),
        ("Dia Mundial de l'bany", "dia mundial del (lavabo|v(à|a)ter)", month_day(11, 19)),
        ("Dia Mundial de l'Turisme", "dmt|dia mundial de l'turisme", month_day(9, 27)),
        ("Dia Mundial de la Tuberculosi", "dia mundial de (lluita contra)?la (tuberculosi|tb)", month_day(3, 24)),
        ("Dia Mundial de la Tonyina", "dia mundial de la tonyina", month_day(5, 2)),
        ("Dia Mundial del Veganismo", "dia mundial del veganismo", month_day(11, 1)),
        ("Dia Mundial del Vegetarianismo", "dia mundial del vegetarianismo", month_day(10, 1)),
        ("Dia Mundial del Agua", "dia mundial del agua", month_day(3, 22)),
        ("Dia Mundial de les Zones Humides", "dia mundial de les zones humides", month_day(2, 2)),
        ("Dia Mundial de la Vida Silvestre", "dia mundial de la vida silvestre", month_day(3, 3)),
        ("Dia Mundial de les Habilitats de la Joventut", "dia mundial de les habilitats de la joventut", month_day(7, 15)),
        ("Dia de la Zero Discriminació", "dia de la zero discriminació", month_day(3, 1)),
        ("Dia de la Commonwealth", "dia de la commonwealth", nth_dow_of_month(2, 1, 3)),
        (
            "Dia Mundial en Record de les Víctimes d'Accidents de Trànsit",
            "dia (mundial )?en recor de las v(í|i)ctimes (d'accidents )?de tr(à|a)nsit",
            nth_dow_of_month(3, 7, 11),
        ),
        ("Dia Internacional de les Cooperatives", "dia internacional de les cooperatives", nth_dow_of_month(1, 6, 7)),
        (
            "Dia de Martin Luther King",
            "(MLK|dia de Martin Luther King,?)( Jr\\.?| J(ú|u)nior)?|dia dels drets humans a Idaho",
            nth_dow_of_month(3, 1, 1),
        ),
        (
            "Divendres negre",
            "divendres negre|black( |-)?friday",
            cycle_nth_after(false, G::Day, 1, &nth_dow_of_month(4, 4, 11)),
        ),
        ("Dia Mundial de l'Hàbitat", "dia mundial de l'h(à|a)bitat", nth_dow_of_month(1, 1, 10)),
        ("Dia Mundial de l'Ronyó", "dia mundial de l'rony(ó|o)", nth_dow_of_month(2, 4, 3)),
        ("Dia Mundial de la Lepra", "dia mundial (contra|de) la lepra", pred_last_of(&day_of_week(7), &month(1))),
        ("Dia Marítim Mundial", "dia mar(í|i)tim mundial", pred_last_of(&day_of_week(4), &month(9))),
        ("Dia Mundial de les Aus Migratòries", "dia mundial de les aus migrat(ò|o)ries", nth_dow_of_month(2, 6, 5)),
        ("Dia mundial de la filosofia", "Dia mundial de la filosofia", nth_dow_of_month(3, 4, 11)),
        ("Dia mundial de la religió", "dia mundial de (la|les)? religi(ó|ons)", nth_dow_of_month(3, 7, 1)),
        ("Dia mundial de la vista", "dia mundial de la (vista|visi(ó|o))", nth_dow_of_month(2, 4, 10)),
        ("Dia de el cap", "(el )?dia (nacional )?de el cap", pred_nth_closest(0, &weekday(), &month_day(10, 16))),
    ])
}

fn build_rules() -> Vec<Rule> {
    let mut v = vec![
        rule_a_las_hourmintimeofday(),
        rule_a_las_timeofday(),
        rule_afternoon(),
        rule_ano_nuevo(),
        rule_ce_time(),
        rule_datetime_datetime_interval(),
        rule_day_of_month_st(),
        rule_dayofmonth_de_namedmonth(),
        rule_dayofweek_dayofmonth(),
        rule_dddd_monthinterval(),
        rule_ddmm(),
        rule_ddmmyyyy(),
        rule_de_datetime_datetime_interval(),
        rule_el_cycle_vinent_time(),
        rule_del_medioda(),
        rule_del_year(),
        rule_dentro_de_duration(),
        rule_el_cycle_antes_time(),
        rule_el_cycle_vinent(),
        rule_el_cycle_vinent(),
        rule_el_dayofmonth_de_namedmonth(),
        rule_el_dayofmonth_non_ordinal(),
        rule_el_proximo_cycle(),
        rule_el_time(),
        rule_en_duration(),
        rule_entre_datetime_et_datetime_interval(),
        rule_entre_dd_et_dd_monthinterval(),
        rule_esteen_un_cycle(),
        rule_evening(),
        rule_hace_duration(),
        rule_hhhmm_timeofday(),
        campanar("mig quart de <hour-of-day>", "mig quart (de |d')", 53),
        campanar("(un|1)(\\squart|\\/4) de <hour-of-day>", "un quart (de |d')", 45),
        campanar("un quart i cinc de <hour-of-day>", "((un quart|1\\/4) i cinc|(dos|2)(\\squarts|\\/4) menys deu) (de |d')", 40),
        campanar("un quart i mig de <hour-of-day>", "(un|1)(\\squart|\\/4) i mig (de |d')", 38),
        campanar("un quart i deu de <hour-of-day>", "((un|1)(\\squart|\\/4) i deu|(dos|2)(\\squarts|\\/4) menys cinc) (de |d')", 35),
        campanar("dos quarts de <hour-of-day>", "((dos|2)(\\squarts|\\/4)|quarts) (de |d')", 30),
        campanar("dos quarts i cinc de <hour-of-day>", "((dos|2)(\\squarts|\\/4) i cinc|(tres|3)(\\squarts|\\/4) menys deu) (de |d')", 25),
        campanar("dos quarts i mig de <hour-of-day>", "(dos|2)(\\squarts|\\/4) i mig (de |d')", 23),
        campanar("Dos quarts i deu de <hour-of-day>", "((dos|2)(\\squarts|\\/4) i deu|(tres|3)(\\squarts|\\/4) menys cinc) (de |d')", 20),
        campanar("tres quarts de <hour-of-day>", "(tres|3)(\\squarts|\\/4) (de |d')", 15),
        campanar("Tres quarts i cinc de <hour-of-day>", "(tres|3)(\\squarts|\\/4) i cinc (de |d')", 10),
        campanar("Tres quarts i mig de <hour-of-day>", "(tres|3)(\\squarts|\\/4) i mig (de |d')", 8),
        campanar("Tres quarts i deu de <hour-of-day>", "(tres|3)(\\squarts|\\/4) i deu (de |d')", 20),
        rule_hourofday_and_relative_minutes(),
        rule_hourofday_integer_as_relative_minutes(),
        rule_intersect(),
        rule_intersect_by_de(),
        rule_la_cycle_pasado(),
        rule_la_pasado_cycle(),
        rule_matinada(),
        rule_nit(),
        rule_migdia(),
        rule_midnight(),
        rule_morning(),
        rule_pod_in(),
        rule_pod_this(),
        rule_tod_pod(),
        rule_n_cycle_vinent(),
        rule_n_pasados_cycle(),
        rule_n_proximas_cycle(),
        rule_namedmonth_dayofmonth(),
        rule_namedmonthnamedday_next(),
        rule_namedmonthnamedday_past(),
        rule_namedmonthnamedday_past2(),
        rule_nochevieja(),
        rule_noon(),
        rule_today(),
        rule_nth_time_de_time(),
        rule_nth_time_de_time2(),
        rule_ordinal_quarter(),
        rule_ordinal_quarter_year(),
        rule_pasados_n_cycle(),
        rule_proximas_n_cycle(),
        rule_right_now(),
        rule_the_day_after_tomorrow(),
        rule_the_day_before_yesterday(),
        rule_this_dayofweek(),
        rule_timeofday_ampm(),
        rule_timeofday_horas(),
        rule_timeofday_latent(),
        rule_tomorrow(),
        rule_two_time_tokens_separated_by(),
        rule_ultimo_dayofweek_de_time(),
        rule_weekend(),
        rule_year(),
        rule_year_latent(),
        rule_yesterday(),
        rule_yyyymmdd(),
        rule_hourofday_and_three_quarter(),
        rule_hourofday_and_half(),
        rule_hourofday_and_quarter(),
        rule_hourofday_and_relative_minutes2(),
        rule_hourofday_three_quarter(),
        rule_hourofday_half(),
        rule_hourofday_quarter(),
        rule_hourofday_integer_as_relative_minutes2(),
        rule_timezone(),
        rule_el_dayofmonth_non_ordinal_with_dia(),
        rule_next_week(),
        rule_next_week_alt(),
        rule_year_by_adding_three_numbers(),
        rule_duration_in(),
        rule_duration_ago(),
    ];
    v.extend(rule_days_of_week());
    v.extend(rule_months());
    v.extend(rule_seasons());
    v.extend(rule_periodic_holidays());
    v
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &RULES
}
