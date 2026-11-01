//! `Duckling.Time.ES.Rules`.

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

/// `Duckling.Duration.Helpers.isGrain`.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

/// `Token Time . mkLatent . partOfDay <$> interval Open (hour False a) (hour False b)`.
fn latent_pod(a: i64, b: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, a), &hour(false, b)).map(|x| mk_latent(part_of_day(x))))
}

fn rule_a_las_hourmintimeofday() -> Rule {
    rule(
        "a las <hour-min>(time-of-day)",
        vec![regex("((al?)( las?)?|las?)"), pred(is_a_time_of_day), regex("horas?")],
        |ts| Some(ts[1].clone()),
    )
}

fn rule_a_las_timeofday() -> Rule {
    rule(
        "a las <time-of-day>",
        vec![regex("((para)|(al?))( las?)?|las?"), pred(is_a_time_of_day)],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    )
}

fn rule_afternoon() -> Rule {
    rule("afternoon", vec![regex("tarde")], |_| latent_pod(12, 19))
}

fn rule_ano_nuevo() -> Rule {
    rule("ano nuevo", vec![regex("a(n|ñ)o nuevo")], |_| tt(month_day(1, 1)))
}

fn rule_ce_time() -> Rule {
    rule("ce <time>", vec![regex("este"), dimension(Dim::Time)], |ts| {
        tt(pred_nth(0, false, td(ts[1])?))
    })
}

fn rule_datetime_datetime_interval() -> Rule {
    rule(
        "<datetime> - <datetime> (interval)",
        vec![pred(is_not_latent), regex("\\-|al?"), pred(is_not_latent)],
        |ts| ttd(interval(Open, td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_day_of_month_st() -> Rule {
    rule("day of month (1st)", vec![regex("primero|uno|prem\\.?|1o")], |_| tt(day_of_month(1)))
}

fn rule_dayofweek_dayofmonth() -> Rule {
    rule(
        "<day-of-week> <day-of-month>",
        vec![pred(is_a_day_of_week), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_dddd_monthinterval() -> Rule {
    rule(
        "dd-dd <month>(interval)",
        vec![
            regex("(3[01]|[12]\\d|0?[1-9])"),
            regex("\\-|al?"),
            regex("(3[01]|[12]\\d|0?[1-9])"),
            regex("de"),
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

fn rule_ddmm() -> Rule {
    rule(
        "dd[/-]mm",
        vec![regex("(3[01]|[12]\\d|0?[1-9])[/-](0?[1-9]|1[0-2])")],
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

fn rule_de_datetime_datetime_interval() -> Rule {
    rule(
        "de <datetime> - <datetime> (interval)",
        vec![regex("del?"), dimension(Dim::Time), regex("\\-|al?"), dimension(Dim::Time)],
        |ts| ttd(interval(Open, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_del_medioda() -> Rule {
    rule("del mediodía", vec![regex("del mediod(i|í)a")], |_| latent_pod(12, 17))
}

fn rule_del_year() -> Rule {
    rule(
        "del <year>",
        vec![regex("del( a(ñ|n)o)?"), pred(is_integer_between_tok(1000, 2100))],
        |ts| tt(year(get_int_value(ts[1])?)),
    )
}

fn rule_dentro_de_duration() -> Rule {
    rule(
        "dentro de <duration>",
        vec![regex("dentro de"), dimension(Dim::Duration)],
        |ts| ttd(interval(Open, &now(), &in_duration(dur(ts[1])?))),
    )
}

fn rule_dim_time_de_la_manana() -> Rule {
    rule(
        "<dim time> de la manana",
        vec![pred(is_a_time_of_day), regex("(a|en|de) la ma(ñ|n)ana")],
        |ts| {
            let manana = interval(Open, &hour(false, 0), &hour(false, 12)).map(|x| mk_latent(part_of_day(x)))?;
            ttd(intersect(td(ts[0])?, &manana))
        },
    )
}

fn rule_dim_time_de_la_tarde() -> Rule {
    rule(
        "<dim time> de la tarde",
        vec![pred(is_a_time_of_day), regex("(a|en|de) la tarde")],
        |ts| {
            let tarde = interval(Open, &hour(false, 12), &hour(false, 21)).map(|x| mk_latent(part_of_day(x)))?;
            ttd(intersect(td(ts[0])?, &tarde))
        },
    )
}

fn rule_dim_time_de_la_tarde2() -> Rule {
    rule(
        "<time-of-day> de la tarde",
        vec![pred(is_an_hour_of_day), pred(is_integer_between_tok(1, 59)), regex("(a|en|de) la tarde")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            let minutes = get_int_value(ts[1])?;
            let t = hour_minute(is12h, hours, minutes);
            let tarde = interval(Open, &hour(false, 12), &hour(false, 21)).map(|x| mk_latent(part_of_day(x)))?;
            ttd(intersect(&t, &tarde))
        },
    )
}

fn rule_dom_of_month() -> Rule {
    rule(
        "<day-of-month> (ordinal or number) de <named-month>",
        vec![pred(is_dom_value), regex("de"), pred(is_a_month)],
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

fn rule_el_cycle_antes_time() -> Rule {
    rule(
        "el <cycle> antes <time>",
        vec![regex("l[ea']? ?"), dimension(Dim::TimeGrain), regex("antes de"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, -1, td(ts[3])?)),
    )
}

fn rule_el_cycle_proximoque_viene() -> Rule {
    rule(
        "el <cycle> (proximo|que viene)",
        vec![
            regex("(el|los|la|las) ?"),
            dimension(Dim::TimeGrain),
            regex("(pr(ó|o)xim(o|a)s?|que vienen?|siguientes?)"),
        ],
        |ts| tt(cycle_nth(grain_of(ts[1])?, 1)),
    )
}

fn rule_el_cycle_proximoque_viene_time() -> Rule {
    rule(
        "el <cycle> proximo|que viene <time>",
        vec![
            regex("(el|los|la|las)"),
            dimension(Dim::TimeGrain),
            regex("(pr(ó|o)xim(o|a)s?|que vienen?|siguientes?)"),
            dimension(Dim::Time),
        ],
        |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, 1, td(ts[3])?)),
    )
}

fn rule_el_dayofmonth_de_namedmonth() -> Rule {
    rule(
        "el <day-of-month> de <named-month>",
        vec![regex("el"), pred(is_dom_integer), regex("de"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[3])?, ts[1])),
    )
}

fn rule_el_dayofmonth_non_ordinal() -> Rule {
    rule(
        "el <day-of-month> (non ordinal)",
        vec![regex("el"), pred(is_integer_between_tok(1, 31))],
        |ts| tt(day_of_month(get_int_value(ts[1])?)),
    )
}

fn rule_el_proximo_cycle() -> Rule {
    rule(
        "el proximo <cycle> ",
        vec![regex("(el|los|la|las) ?"), regex("pr(ó|o)xim(o|a)s?|siguientes?"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[2])?, 1)),
    )
}

fn rule_el_time() -> Rule {
    rule("el <time>", vec![regex("d?el"), pred(is_not_latent)], |ts| Some(ts[1].clone()))
}

fn rule_en_duration() -> Rule {
    rule("en <duration>", vec![regex("en"), dimension(Dim::Duration)], |ts| {
        tt(in_duration(dur(ts[1])?))
    })
}

fn rule_entre_datetime_et_datetime_interval() -> Rule {
    rule(
        "entre <datetime> et <datetime> (interval)",
        vec![regex("entre"), dimension(Dim::Time), regex("y"), dimension(Dim::Time)],
        |ts| ttd(interval(Open, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_entre_dd_et_dd_monthinterval() -> Rule {
    rule(
        "entre dd et dd <month>(interval)",
        vec![
            regex("entre( el)?"),
            regex("(0?[1-9]|[12]\\d|3[01])"),
            regex("y( el)?"),
            regex("(0?[1-9]|[12]\\d|3[01])"),
            regex("de"),
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

fn rule_esteen_un_cycle() -> Rule {
    rule(
        "este|en un <cycle>",
        vec![regex("(est(e|a|os)|en (el|los|la|las) ?)"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
    )
}

fn rule_evening() -> Rule {
    rule("evening", vec![regex("noche")], |_| latent_pod(18, 0))
}

fn rule_hace_duration() -> Rule {
    rule("hace <duration>", vec![regex("hace"), dimension(Dim::Duration)], |ts| {
        tt(duration_ago(dur(ts[1])?))
    })
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

fn rule_hourofday_and_relative_minutes() -> Rule {
    rule(
        "<hour-of-day> and <relative minutes>",
        vec![pred(is_an_hour_of_day), regex("y"), pred(is_integer_between_tok(1, 59))],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[2])?))
        },
    )
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

fn rule_hourofday_minus_integer_as_relative_minutes() -> Rule {
    rule(
        "<hour-of-day> minus <integer> (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("menos\\s?"), pred(is_integer_between_tok(1, 59))],
        |ts| ttd(minutes_before(get_int_value(ts[2])?, td(ts[0])?)),
    )
}

fn rule_in_the_partofday() -> Rule {
    rule(
        "in the <part-of-day>",
        vec![regex("(a|en|de|por) la"), pred(is_a_part_of_day)],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    )
}

fn rule_integer_in_the_partofday() -> Rule {
    rule(
        "<integer> in the <part-of-day>",
        vec![pred(is_a_part_of_day), regex("(a|en|de|por) la"), dimension(Dim::Time)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_intersect() -> Rule {
    rule(
        "intersect",
        vec![pred(is_not_latent), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[1])?)),
    )
}

fn rule_intersect_by_de() -> Rule {
    rule(
        "intersect by `de`",
        vec![pred(is_not_latent), regex("de"), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_la_cycle_pasado() -> Rule {
    rule(
        "la <cycle> pasado",
        vec![regex("(el|los|la|las) ?"), dimension(Dim::TimeGrain), regex("pasad(a|o)s?|(u|ú)ltim[ao]s?")],
        |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
    )
}

fn rule_la_pasado_cycle() -> Rule {
    rule(
        "la pasado <cycle>",
        vec![regex("(el|los|la|las) ?"), regex("pasad(a|o)s?|(u|ú)ltim[ao]s?"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[2])?, -1)),
    )
}

fn rule_midnight() -> Rule {
    rule("midnight", vec![regex("medianoche")], |_| tt(hour(false, 0)))
}

fn rule_month_dom() -> Rule {
    rule(
        "<named-month> <day-of-month>",
        vec![pred(is_a_month), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_morning() -> Rule {
    rule("morning", vec![regex("ma(ñ|n)ana")], |_| latent_pod(4, 12))
}

fn rule_n_cycle_proximoque_viene() -> Rule {
    rule(
        "n <cycle> (proximo|que viene)",
        vec![
            pred(is_integer_between_tok(2, 9999)),
            dimension(Dim::TimeGrain),
            regex("(pr(ó|o)xim(o|a)s?|que vienen?|siguientes?)"),
        ],
        |ts| tt(cycle_n(true, grain_of(ts[1])?, get_int_value(ts[0])?)),
    )
}

fn rule_n_pasados_cycle() -> Rule {
    rule(
        "n pasados <cycle>",
        vec![pred(is_integer_between_tok(2, 9999)), regex("pasad(a|o)s?"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_n(true, grain_of(ts[2])?, -get_int_value(ts[0])?)),
    )
}

fn rule_n_proximas_cycle() -> Rule {
    rule(
        "n proximas <cycle>",
        vec![pred(is_integer_between_tok(2, 9999)), regex("pr(ó|o)xim(o|a)s?"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_n(true, grain_of(ts[2])?, get_int_value(ts[0])?)),
    )
}

fn rule_namedmonthnamedday_next() -> Rule {
    rule(
        "<named-month|named-day> next",
        vec![dimension(Dim::Time), regex("que vienen?")],
        |ts| tt(pred_nth(1, false, td(ts[0])?)),
    )
}

fn rule_namedmonthnamedday_past() -> Rule {
    rule(
        "<named-month|named-day> past",
        vec![dimension(Dim::Time), regex("pasad(o|a)")],
        |ts| tt(pred_nth(-1, false, td(ts[0])?)),
    )
}

fn rule_navidad() -> Rule {
    rule("Navidad", vec![regex("(la )?navidad")], |_| tt(month_day(12, 25)))
}

fn rule_nochevieja() -> Rule {
    rule("Nochevieja", vec![regex("nochevieja")], |_| tt(month_day(12, 31)))
}

fn rule_noon() -> Rule {
    rule("noon", vec![regex("medio(\\s*)d(í|i)a")], |_| tt(hour(false, 12)))
}

fn rule_now() -> Rule {
    rule("now", vec![regex("(hoy)|(en este momento)")], |_| tt(today()))
}

fn rule_nth_time_de_time() -> Rule {
    rule(
        "nth <time> de <time>",
        vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("de|en"), dimension(Dim::Time)],
        |ts| {
            let v = ordinal_value(ts[0])?;
            ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(v - 1, false, &x)))
        },
    )
}

fn rule_nth_time_de_time2() -> Rule {
    rule(
        "nth <time> de <time>",
        vec![
            regex("the"),
            dimension(Dim::Ordinal),
            dimension(Dim::Time),
            regex("de|en"),
            dimension(Dim::Time),
        ],
        |ts| {
            let v = ordinal_value(ts[1])?;
            ttd(intersect(td(ts[4])?, td(ts[2])?).map(|x| pred_nth(v - 1, false, &x)))
        },
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

fn rule_ordinal_quarter_year() -> Rule {
    rule(
        "<ordinal> quarter <year>",
        vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter)), regex("del? ?"), dimension(Dim::Time)],
        |ts| {
            // Haskell: (Token Ordinal ...:_:Token Time td:_) — Time is matched at index 2.
            let v = ordinal_value(ts[0])?;
            tt(cycle_nth_after(false, G::Quarter, v - 1, td(ts[2])?))
        },
    )
}

fn rule_pasados_n_cycle() -> Rule {
    rule(
        "pasados n <cycle>",
        vec![regex("pasad(a|o)s?"), pred(is_integer_between_tok(2, 9999)), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_n(true, grain_of(ts[2])?, -get_int_value(ts[1])?)),
    )
}

fn rule_proximas_n_cycle() -> Rule {
    rule(
        "proximas n <cycle>",
        vec![regex("pr(ó|o)xim(o|a)s?"), pred(is_integer_between_tok(2, 9999)), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_n(true, grain_of(ts[2])?, get_int_value(ts[1])?)),
    )
}

fn rule_right_now() -> Rule {
    rule("right now", vec![regex("ahor(it)?a|ya|en\\s?seguida|cuanto antes")], |_| tt(now()))
}

fn rule_the_day_after_tomorrow() -> Rule {
    rule("the day after tomorrow", vec![regex("pasado\\s?ma(n|ñ)ana")], |_| {
        tt(cycle_nth(G::Day, 2))
    })
}

fn rule_the_day_before_yesterday() -> Rule {
    rule(
        "the day before yesterday",
        vec![regex("anteayer|antes de (ayer|anoche)|antier")],
        |_| tt(cycle_nth(G::Day, -2)),
    )
}

fn rule_this_or_next_day_of_week() -> Rule {
    rule(
        "this|next <day-of-week>",
        vec![regex("(este|pr(o|ó)ximo)"), pred(is_a_day_of_week)],
        |ts| {
            let m = to_lower(&rx(ts[0], 0)?);
            let dow = td(ts[1])?;
            let out = match m.as_str() {
                "este" => Some(pred_nth(0, true, dow)),
                "próximo" | "proximo" => intersect(dow, &cycle_nth(G::Week, 1)),
                _ => None,
            }?;
            tt(out)
        },
    )
}

fn rule_this_partofday() -> Rule {
    rule(
        "this <part-of-day>",
        vec![regex("est(e|a)"), pred(is_a_part_of_day)],
        |ts| ttd(intersect(&today(), td(ts[1])?).map(part_of_day)),
    )
}

fn rule_timeofday_ampm() -> Rule {
    rule(
        "<time-of-day> am|pm",
        vec![pred(is_a_time_of_day), regex("([ap])\\.?m?\\.?")],
        |ts| {
            let ap = to_lower(&rx(ts[1], 0)?);
            tt(time_of_day_ampm(ap == "a", td(ts[0])?))
        },
    )
}

fn rule_timeofday_horas() -> Rule {
    rule(
        "<time-of-day> horas",
        vec![pred(is_a_time_of_day), regex("h\\.?(ora)?s?")],
        |ts| tt(mk_latent(td(ts[0])?.clone())),
    )
}

fn rule_timeofday_latent() -> Rule {
    rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
        let v = get_int_value(ts[0])?;
        tt(mk_latent(hour(v < 13, v)))
    })
}

fn rule_timeofday_partofday() -> Rule {
    rule(
        "<time-of-day> <part-of-day>",
        vec![dimension(Dim::Time), pred(is_a_part_of_day)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[1])?)),
    )
}

fn rule_tomorrow() -> Rule {
    rule("tomorrow", vec![regex("ma(n|ñ)ana")], |_| tt(cycle_nth(G::Day, 1)))
}

fn rule_two_time_tokens_separated_by() -> Rule {
    rule(
        "two time tokens separated by \",\"",
        vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_ultimo_dayofweek_de_time() -> Rule {
    rule(
        "ultimo <day-of-week> de <time>",
        vec![regex("(ú|u)ltimo"), pred(is_a_day_of_week), regex("de|en"), dimension(Dim::Time)],
        |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_weekend() -> Rule {
    rule("week-end", vec![regex("week[ -]?end|fin de semana")], |_| tt(weekend()))
}

fn rule_year() -> Rule {
    rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
        tt(year(get_int_value(ts[0])?))
    })
}

fn rule_year_latent() -> Rule {
    rule("year (latent)", vec![pred(is_integer_between_tok(25, 10000))], |ts| {
        tt(mk_latent(year(get_int_value(ts[0])?)))
    })
}

fn rule_yesterday() -> Rule {
    rule("yesterday", vec![regex("ayer")], |_| tt(cycle_nth(G::Day, -1)))
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

fn rule_hourofday_and_three_quarter() -> Rule {
    rule(
        "<hour-of-day> and 3 quarters",
        vec![pred(is_an_hour_of_day), regex("y (3|tres) cuartos?")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 45))
        },
    )
}

fn rule_hourofday_and_half() -> Rule {
    rule("<hour-of-day> and half", vec![pred(is_an_hour_of_day), regex("y media")], |ts| {
        let (hours, is12h) = tod(ts[0])?;
        tt(hour_minute(is12h, hours, 30))
    })
}

fn rule_hourofday_and_quarter() -> Rule {
    rule("<hour-of-day> and quarter", vec![pred(is_an_hour_of_day), regex("y cuarto")], |ts| {
        let (hours, is12h) = tod(ts[0])?;
        tt(hour_minute(is12h, hours, 15))
    })
}

fn rule_hourofday_and_relative_minutes2() -> Rule {
    rule(
        "<hour-of-day> and <relative minutes> minutes",
        vec![
            pred(is_an_hour_of_day),
            regex("y"),
            pred(is_integer_between_tok(1, 59)),
            regex("min\\.?(uto)?s?"),
        ],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[2])?))
        },
    )
}

fn rule_hourofday_three_quarter() -> Rule {
    rule(
        "<hour-of-day> three quarters (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("(3|tres) cuartos?")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 45))
        },
    )
}

fn rule_hourofday_half() -> Rule {
    rule(
        "<hour-of-day> half (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("media")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 30))
        },
    )
}

fn rule_hourofday_quarter() -> Rule {
    rule(
        "<hour-of-day> quarter (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("cuarto")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 15))
        },
    )
}

fn rule_hourofday_integer_as_relative_minutes2() -> Rule {
    rule(
        "<hour-of-day> <integer> (as relative minutes) minutes",
        vec![pred(is_an_hour_of_day), pred(is_integer_between_tok(1, 59)), regex("min\\.?(uto)?s?")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[1])?))
        },
    )
}

fn rule_hourofday_minus_three_quarter() -> Rule {
    rule(
        "<hour-of-day> minus three quarter (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("menos\\s? (3|tres) cuartos?")],
        |ts| ttd(minutes_before(45, td(ts[0])?)),
    )
}

fn rule_hourofday_minus_half() -> Rule {
    rule(
        "<hour-of-day> minus half (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("menos\\s? media")],
        |ts| ttd(minutes_before(30, td(ts[0])?)),
    )
}

fn rule_hourofday_minus_quarter() -> Rule {
    rule(
        "<hour-of-day> minus quarter (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("menos\\s? cuarto")],
        |ts| ttd(minutes_before(15, td(ts[0])?)),
    )
}

fn rule_hourofday_minus_integer_as_relative_minutes2() -> Rule {
    rule(
        "<hour-of-day> minus <integer> (as relative minutes) minutes",
        vec![
            pred(is_an_hour_of_day),
            regex("menos\\s?"),
            pred(is_integer_between_tok(1, 59)),
            regex("min\\.?(uto)?s?"),
        ],
        |ts| ttd(minutes_before(get_int_value(ts[2])?, td(ts[0])?)),
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

fn rule_el_dayofmonth_non_ordinal_with_dia() -> Rule {
    rule(
        "dia <day-of-month> (non ordinal)",
        vec![regex("día"), pred(is_integer_between_tok(1, 31))],
        |ts| tt(day_of_month(get_int_value(ts[1])?)),
    )
}

fn rule_next_week() -> Rule {
    rule("next week", vec![pred(is_grain(G::Week)), regex("que viene")], |_| {
        tt(cycle_nth(G::Week, 1))
    })
}

fn rule_next_week_alt() -> Rule {
    rule(
        "next week (alt)",
        vec![regex("pr(ó|o)xim(o|a)s?|siguientes?"), pred(is_grain(G::Week))],
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
        ("Lunes", "lunes|lun?\\.?"),
        ("Martes", "martes|mar?\\.?"),
        ("Miercoles", "mi(e|é)\\.?(rcoles)?|mx|mier?\\."),
        ("Jueves", "jueves|jue|jue\\."),
        ("Viernes", "viernes|vie|vie\\."),
        ("Sabado", "s(á|a)bado|s(á|a)b\\.?"),
        ("Domingo", "domingo|dom\\.?"),
    ])
}

fn rule_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("Enero", "enero|ene\\.?"),
        ("Febrero", "febrero|feb\\.?"),
        ("Marzo", "marzo|mar\\.?"),
        ("Abril", "abril|abr\\.?"),
        ("Mayo", "mayo?\\.?"),
        ("Junio", "junio|jun\\.?"),
        ("Julio", "julio|jul\\.?"),
        ("Agosto", "agosto|ago\\.?"),
        ("Septiembre", "septiembre|sept?\\.?"),
        ("Octubre", "octubre|oct\\.?"),
        ("Noviembre", "noviembre|nov\\.?"),
        ("Diciembre", "diciembre|dic\\.?"),
    ])
}

fn rule_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("verano", "verano", month_day(6, 21), month_day(9, 23)),
        ("otoño", "oto(ñ|n)o", month_day(9, 23), month_day(12, 21)),
        ("invierno", "invierno", month_day(12, 21), month_day(3, 20)),
        ("primavera", "primavera", month_day(3, 20), month_day(6, 21)),
    ])
}

fn rule_periodic_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("Día de África", "d(í|i)a de (la (libertad|liberacion) )?(á|a)frica(na)?", month_day(5, 25)),
        ("Día de la Industrialización de África", "d(í|i)a de (la )?industrializaci(ó|o)n de (á|a)frica(na)?", month_day(11, 20)),
        ("Día de Todos los Santos", "d(í|i)a de todos los santos", month_day(11, 1)),
        ("Día de las Almas", "d(í|i)a de las almas", month_day(11, 2)),
        ("Día de los Inocentes de Abril", "d(í|i)a de (los inocentes|las bromas)( de abril)?", month_day(4, 1)),
        ("Día Mundial de la Lengua Árabe", "d(í|i)a mundial de la lengua (á|a)rabe", month_day(12, 18)),
        ("Asunción de María", "asunci(ó|o)n de mar(í|i)a", month_day(8, 15)),
        ("Día del Boxeo", "(el )?d(í|i)a del (boxeo|boxeador)", month_day(12, 26)),
        ("Día de Lengua China", "d(í|i)a de lengua china", month_day(4, 20)),
        ("Nochebuena", "(la (cena de )?)?(nochebuena|noche buena)", month_day(12, 24)),
        ("Día de Conmemoración de las Víctimas de la Guerra Química", "d(í|i)a (de )?(conmemorativo|conmemoraci(ó|o)n) (en recuerdo )?de las v(í|i)ctimas de la guerra qu(í|i)mica", month_day(4, 29)),
        ("Día de Conmemoración de las Víctimas del Genocidio de Ruanda", "d(í|i)a (de )?(conmemorativo|conmemoraci(ó|o)n) (en recuerdo )?de las v(í|i)ctimas del genocidio de ruanda", month_day(4, 7)),
        ("Día de la Marina", "d(í|i)a (de la|del) marin(a|o)", month_day(6, 25)),
        ("Día de la Tierra", "d(í|i)a (internacional )?de la (madre )?tierra", month_day(4, 22)),
        ("Día de la Lengua Inglesa", "d(í|i)a de la lengua inglesa", month_day(4, 23)),
        ("Epifanía", "Epifan(í|i)a", month_day(1, 6)),
        ("Las Fiestas de San Francisco de Asís", "(las )?fiestas de san (francisco de as(í|i)s|pacho)", month_day(10, 4)),
        ("Fiesta de la Inmaculada Concepción", "(d(í|i)a|fiesta) de la inmaculada concepci(ó|o)n( de (la virgen )?mar(í|i)a)?", month_day(12, 8)),
        ("Día Mundial de las Madres y de los Padres", "d(í|i)a mundial de las madres y de los padres", month_day(6, 1)),
        ("Día de Todos los Santos", "(d(í|i)a|festividad) de todos los santos", month_day(10, 31)),
        ("Día de los Derechos Humanos", "d(í|i)a de los derechos humanos", month_day(12, 10)),
        ("Día Internacional de Sensibilización sobre el Albinismo", "d(í|i)a internacional de sensibilizaci(ó|o)n sobre el albinismo", month_day(6, 13)),
        ("Día Internacional contra la Corrupción", "d(í|i)a internacional contra la corrupci(ó|o)n", month_day(12, 9)),
        ("Día Internacional de los Asteroides", "d(í|i)a internacional de los asteroides", month_day(6, 30)),
        ("Celebra el Día Internacional de la Bisexualidad", "celebra el d(í|i)a internacional de la bisexualidad", month_day(9, 23)),
        ("Día Internacional de Recordación de Desastres de Chernóbil", "d(í|i)a internacional de recordaci(ó|o)n de desastres de chern(ó|o)bil ", month_day(4, 26)),
        ("Día Internacional de la Aviacion Civil", "d(í|i)a internacional de la aviacion civil", month_day(12, 7)),
        ("Día Internacional de la Aduana", "d(í|i)a internacional de la(s)? aduana(s)?", month_day(1, 26)),
        ("Día Internacional de la Lucha contra el Uso Indebido y el Tráfico Ilícito de Drogas", "d(í|i)a internacional de la lucha contra el uso indebido y el tr(á|a)fico il(í|i)cito de drogas", month_day(6, 26)),
        ("Día Internacional contra los Ensayos Nucleares", "d(í|i)a internacional contra los ensayos nucleares", month_day(8, 29)),
        ("Día Internacional de la Diversidad Biológica", "d(í|i)a internacional de la diversidad biol(ó|o)gica", month_day(5, 22)),
        ("Día Internacional de los Monumentos y Sitios", "d(í|i)a internacional de los monumentos y sitios", month_day(4, 18)),
        ("Día Internacional para la Prevención de la Explotación del medio ambiente en la Guerra y los Conflictos Armados", "d(í|i)a internacional para la prevenci(ó|o)n de la explotaci(ó|o)n del medio ambiente en la guerra y los conflictos armados", month_day(11, 6)),
        ("Día de las Naciones Unidas para la Cooperación Sur-Sur", "d(í|i)a de las naciones unidas para la cooperaci(ó|o)n sur(\\-|\\s)sur", month_day(9, 12)),
        ("Día Internacional para la Tolerancia", "d(í|i)a internacional (de|para) la tolerancia", month_day(11, 16)),
        ("Día Internacional para la Abolición de la Esclavitud", "d(í|i)a internacional para la abolici(ó|o)n de la esclavitud", month_day(12, 2)),
        ("Día Internacional de la Eliminación de la Discriminación Racial", "d(í|i)a internacional de la eliminaci(ó|o)n de la discriminaci(ó|o)n racial", month_day(3, 21)),
        ("Día Internacional de la Eliminación de la Violencia Sexual en los Conflictos", "d(í|i)a internacional de la eliminaci(ó|o)n de la violencia sexual en los conflictos", month_day(6, 19)),
        ("Día Internacional de la Eliminación de la Violencia contra la Mujer", "d(í|i)a internacional de la eliminaci(ó|o)n de la violencia contra la mujer.", month_day(11, 25)),
        ("Día Internacional para la Erradicación de la Pobreza", "d(í|i)a internacional para la erradicaci(ó|o)n de la pobreza", month_day(10, 17)),
        ("Día Internacional de la Preservación de la Capa de Ozono", "d(í|i)a internacional de la preservaci(ó|o)n de la capa de ozono", month_day(9, 16)),
        ("Día Internacional del Recuerdo de la Trata de Esclavos y de su Abolición", "d(í|i)a internacional del recuerdo de la trata de esclavos y de su abolici(ó|o)n", month_day(8, 23)),
        ("Día Internacional del Derecho a la Verdad en relación con Violaciones Graves de los Derechos Humanos y de la Dignidad de las Víctimas de las Naciones Unidas", "d(í|i)a internacional del derecho a la verdad en relaci(ó|o)n con violaciones graves de los derechos humanos y de la dignidad de las v(í|i)ctimas de las naciones unidas", month_day(3, 24)),
        ("Día Internacional para la Eliminación Total de las Armas Nucleares", "d(í|i)a internacional para la eliminaci(ó|o)n total de las armas nucleares", month_day(9, 26)),
        ("Día Internacional de las Naciones Unidas en Apoyo de las Víctimas de la Tortura", "d(í|i)a internacional de las naciones unidas en apoyo de las v(í|i)ctimas de la tortura", month_day(6, 26)),
        ("Día Internacional de la Beneficencia", "d(í|i)a internacional de la beneficencia", month_day(9, 5)),
        ("Día Internacional de Conmemoración anual en memoria de las Víctimas del Holocausto", "d(í|i)a internacional de conmemoraci(ó|o)n anual en memoria de las v(í|i)ctimas del holocausto", month_day(1, 27)),
        ("Día Internacional de la Democracia", "d(í|i)a (internacional )?de la democracia", month_day(9, 15)),
        ("Día Internacional de las Personas con Discapacidad", "d(í|i)a internacional de las personas con discapacidad", month_day(12, 3)),
        ("Día Internacional de la Familia", "d(í|i)a internacional de la(s)? familia(s)?", month_day(5, 15)),
        ("Día Internacional de las Remesas Familiares", "d(í|i)a internacional de las remesas familiares", month_day(6, 16)),
        ("Día Internacional de los Bosques", "d(í|i)a internacional de los bosques", month_day(3, 21)),
        ("Día Internacional de la Amistad", "d(í|i)a internacional de la amistad", month_day(7, 30)),
        ("Día Internacional de la Felicidad", "d(í|i)a (internacional|mundial) de la felicidad", month_day(3, 20)),
        ("Día Internacional de los Vuelos Espaciales", "d(í|i)a internacional (de los|del) vuelos espaciales", month_day(4, 12)),
        ("Día Internacional de los Niños Víctimas Inocentes de Agresión", "d(í|i)a internacional de los ni(ñ|n)os v(í|i)ctimas inocentes de agresi(ó|o)n", month_day(6, 4)),
        ("Día Internacional de la no Violencia", "d(í|i)a internacional de la no violencia", month_day(10, 2)),
        ("Día Internacional del Nowruz", "d(í|i)a internacional del nowruz", month_day(3, 21)),
        ("Día Internacional de las Personas Mayores", "d(í|i)a internacional de las personas mayores", month_day(10, 1)),
        ("Dia Internacional de la Paz", "d(í|i)a internacional de la paz", month_day(9, 21)),
        ("Día Internacional de las Personas con Discapacidad", "d(í|i)a internacional de las personas con discapacidad", month_day(12, 3)),
        ("Día Internacional de Recuerdo de las Víctimas de la Esclavitud y la Trata Transatlántica de Esclavos", "d(í|i)a internacional de recuerdo de las v(í|i)ctimas de la esclavitud y la trata transatl(á|a)ntica de esclavos", month_day(3, 25)),
        ("Día Internacional de las Mujeres Rurales", "d(í|i)a internacional de las mujeres rurales", month_day(10, 15)),
        ("Día Internacional de Solidaridad con los miembros del Personal Detenidos o Desaparecidos", "d(í|i)a internacional de solidaridad con los miembros del personal detenidos o desaparecidos", month_day(3, 25)),
        ("Día Internacional de Solidaridad con el Pueblo Palestino", "d(í|i)a internacional de solidaridad con el pueblo palestino", month_day(11, 29)),
        ("Día Internacional del Deporte para el Desarrollo y la Paz", "d(í|i)a internacional del deporte para el desarrollo y la paz", month_day(4, 6)),
        ("Día Internacional del Personal de Paz de las Naciones Unidas", "d(í|i)a internacional del personal de paz de las naciones unidas", month_day(5, 29)),
        ("Día Internacional de la Mujer y la Niña en la Ciencia", "d(í|i)a internacional de la mujer y la ni(ñ|n)a en la ciencia", month_day(2, 11)),
        ("Día Internacional del Yoga", "d(í|i)a Internacional del Yoga", month_day(6, 21)),
        ("Día Internacional de Tolerancia Cero con la Mutilación Genital Femenina", "d(í|i)a internacional de tolerancia cero con la mutilaci(ó|o)n genital femenina", month_day(2, 6)),
        ("Día Internacional de la Niña", "d(í|i)a internacional de la ni(ñ|n)a", month_day(10, 11)),
        ("Día Internacional de las Víctimas de Desapariciones Forzadas", "d(í|i)a internacional de las v(í|i)ctimas de desapariciones forzadas", month_day(8, 30)),
        ("Día Internacional de los Pueblos Indígenas del Mundo", "d(í|i)a internacional de los pueblos ind(í|i)genas del mundo", month_day(8, 9)),
        ("Día Internacional para poner fin a la Impunidad de los Crímenes contra Periodistas", "d(í|i)a internacional para poner fin a la impunidad de los cr(í|i)menes contra periodistas", month_day(11, 2)),
        ("Día Internacional para la Erradicación de la Fístula Obstétrica", "d(í|i)a internacional para la erradicaci(ó|o)n de la f(í|i)stula obst(é|e)trica", month_day(5, 23)),
        ("Día Internacional para la Reducción de los Desastres Naturales", "d(í|i)a internacional para la reducci(ó|o)n de los desastres (naturales)?", month_day(10, 13)),
        ("Día Internacional de la Solidaridad Humana", "d(í|i)a internacional de la solidaridad humana", month_day(12, 20)),
        ("Día Internacional del Jazz", "d(í|i)a internacional del jazz", month_day(4, 30)),
        ("Día Internacional de la Alfabetización", "d(í|i)a internacional de la alfabetizaci(ó|o)n", month_day(9, 8)),
        ("Día Internacional del Hombre", "d(í|i)a internacional de(l)? los hombre(s)?", month_day(11, 19)),
        ("Día Internacional del Migrante", "d(í|i)a internacional del migrante", month_day(12, 18)),
        ("Día Internacional de la Lengua Materna", "d(í|i)a (internacional|mundial) de la lengua materna", month_day(2, 21)),
        ("Día Internacional de las Montañas", "d(í|i)a internacional de las monta(ñ|n)as", month_day(12, 11)),
        ("Día Internacional de las Enfermeras", "d(í|i)a internacional de las enfermeras", month_day(5, 12)),
        ("Día de la Conciencia Internacional sobre la Sobredosis", "ioad|d(í|i)a de (la )?conciencia internacional (de|sobre) la sobredosis", month_day(8, 31)),
        ("Día Internacional del Voluntariado para el Desarrollo Económico y Social", "d(í|i)a internacional del voluntariado para el desarrollo econ(ó|o)mico y social", month_day(12, 5)),
        ("Día Internacional de las Viudas", "d(í|i)a internacional de las viudas", month_day(6, 23)),
        ("Día Internacional de la Mujer", "d(í|i)a internacional de la mujer", month_day(3, 8)),
        ("Día Internacional de la Juventud", "d(í|i)a internacional de la juventud", month_day(8, 12)),
        ("El Primero de Mayo", "(el )?primero de mayo", month_day(5, 1)),
        ("Día Internacional de Nelson Mandela", "d(í|i)a (internacional )?de nelson mandela", month_day(7, 18)),
        ("La Navidad Ortodoxa", "(la )?navidad ortodoxa", month_day(1, 7)),
        ("Año Nuevo Ortodoxo", "(el )?(a(ñ|n)o nuevo viejo|a(ñ|n)o viejo ortodoxo|a(ñ|n)o nuevo ortodoxo|a(ñ|n)o nuevo juliano)", month_day(1, 14)),
        ("Día Mundial de los Servicios Públicos", "d(í|i)a (mundial|nacional) de(l)? (los )?(servicios|servidor) p(ú|u)blico(s)?", month_day(6, 23)),
        ("Día de San Jorge", "d(í|i)a de san jorge|fiesta de san jorge", month_day(4, 23)),
        ("Día de San Patricio", "d(í|i)a de san patricio", month_day(3, 17)),
        ("Dia de San Esteban", "d(í|i)a de san esteban", month_day(12, 26)),
        ("Día Internacional del Tiempo del Recuerdo y la Reconciliación para Quienes Perdieron la Vida durante la Segunda Guerra Mundial", "d(í|i)a internacional del tiempo del recuerdo y la reconciliaci(ó|o)n para quienes perdieron la vida durante la segunda guerra mundial", month_day(5, 8)),
        ("Día de las Naciones Unidas", "d(í|i)a de las naciones unidas", month_day(10, 24)),
        ("Día Internacional de Información sobre el Peligro de las Minas y de Asistencia para las Actividades Relativas a las Minas", "d(í|i)a internacional de informaci(ó|o)n sobre el peligro de las minas y de asistencia para las actividades relativas a las minas", month_day(4, 4)),
        ("Día Mundial de la Salud", "d(í|i)a mundial de la salud( de las naciones unidas)?", month_day(4, 7)),
        ("Día Universal del Niño", "d(í|i)a universal del ni(ñ|n)o", month_day(11, 20)),
        ("Día de San Valentín", "(el )?d(í|i)a de san valent(í|i)n", month_day(2, 14)),
        ("Día Mundial de la lucha contra el SIDA", "d(í|i)a mundial de(l)? (la lucha contra el )?SIDA", month_day(12, 1)),
        ("Día Mundial de Concienciación sobre el Autismo", "d(í|i)a mundial de(l)? (concienciaci(ó|o)n sobre el )?autismo", month_day(4, 2)),
        ("Día Mundial de la Artritis Reumatoide", "d(í|i)a mundial de la artritis( reumatoide)?", month_day(5, 20)),
        ("Día Mundial del Donante de Sangre", "d(í|i)a mundial del donante de sangre", month_day(6, 14)),
        ("Día Internacional del Libro y del Derecho de Autor", "d(í|i)a internacional del libro y de(l)? (los )?derecho(s)? de autor", month_day(4, 23)),
        ("Día Mundial del Braille", "d(í|i)a mundial del braille", month_day(1, 4)),
        ("Día Mundial contra el Cáncer", "d(í|i)a mundial contra el c(á|a)ncer", month_day(2, 4)),
        ("Día Mundial de las Ciudades", "d(í|i)a mundial de las ciudades", month_day(10, 31)),
        ("Día Mundial de la PC", "d(í|i)a mundial (de la|del) (PC|Paralisia Cerebral)", month_day(10, 6)),
        ("Día Mundial contra el Trabajo Infantil", "d(í|i)a mundial contra el trabajo infantil", month_day(6, 12)),
        ("Día Mundial contra la Trata de Personas", "d(í|i)a mundial contra la trata de personas", month_day(7, 30)),
        ("Día Mundial del Patrimonio Audiovisual", "d(í|i)a mundial (del|sobre el) patrimonio audiovisual", month_day(10, 27)),
        ("Día Mundial de la Diversidad Cultural para el Diálogo y el Desarrollo", "d(í|i)a mundial de la diversidad cultural para el diálogo y el desarrollo", month_day(5, 21)),
        ("Día Mundial de la Seguridad y la Salud en el Trabajo", "d(í|i)a mundial de la seguridad y la salud en el trabajo", month_day(4, 28)),
        ("Día Mundial por la Abolición de la Esclavitud", "d(í|i)a (internacional|mundial) (para|por) la abolici(ó|o)n de la esclavitud", month_day(12, 2)),
        ("Día Mundial de la Justicia Social", "d(í|i)a mundial de la justicia social", month_day(2, 20)),
        ("Día Mundial del Enfermo", "d(í|i)a mundial (del|de los) enfermos", month_day(2, 11)),
        ("Día Mundial de Lucha contra la Desertificación y la Sequía", "d(í|i)a mundial (de lucha contra|para combatir) la desertificaci(ó|o)n y la sequ(í|i)a", month_day(6, 17)),
        ("Día Mundial de Información sobre el Desarrollo", "d(í|i)a mundial de informaci(ó|o)n sobre el desarrollo", month_day(10, 24)),
        ("Día Mundial de la Diabetes", "d(í|i)a mundial de la diabetes", month_day(11, 14)),
        ("Día Mundial del Síndrome de Down", "d(í|i)a mundial del s(í|i)ndrome de down", month_day(3, 21)),
        ("Día Mundial de Toma de Conciencia del Abuso y Maltrato en la Vejez", "d(í|i)a mundial de toma de conciencia del abuso y maltrato en la vejez", month_day(6, 15)),
        ("Día Mundial del Medio Ambiente", "d(í|i)a mundial del medio ambiente", month_day(6, 5)),
        ("Día Mundial de la Alimentación", "d(í|i)a mundial de la alimentaci(ó|o)n", month_day(10, 16)),
        ("Día para la Conmemoración y Dignificación de las Víctimas del Crimen de Genocidio y para su Prevención", "d(í|i)a para la conmemoraci(ó|o)n y dignificaci(ó|o)n de las v(í|i)ctimas del crimen de genocidio y para su prevenci(ó|o)n", month_day(12, 9)),
        ("Día Mundial del Corazón", "d(í|i)a mundial del coraz(ó|o)n", month_day(9, 29)),
        ("Día Mundial de la Hepatitis", "d(í|i)a mundial (contra|de) la hepatitis", month_day(7, 28)),
        ("Día Mundial de la Asistencia Humanitaria", "d(í|i)a mundial de la asistencia humanitaria", month_day(8, 19)),
        ("Día Internacional de la Sociedad de la Información", "d(í|i)a (internacional|mundial) de (las telecomunicaciones y )?la sociedad de la informaci(ó|o)n", month_day(5, 17)),
        ("Día Mundial de la Propiedad Intelectual", "d(í|i)a mundial de la propiedad intelectual", month_day(4, 26)),
        ("Día Mundial de la Malaria", "d(í|i)a mundial (de la|del) (malaria|paludismo)", month_day(4, 25)),
        ("Día Mundial de la Salud Mental", "d(í|i)a mundial de la salud mental", month_day(10, 10)),
        ("Día Mundial de la Meteorología", "d(í|i)a (mundial de (la )?meteorolog(í|i)a|meteorol(ó|o)gico mundial)", month_day(3, 23)),
        ("Día Mundial sin Tabaco", "d(í|i)a mundial sin tabaco", month_day(5, 31)),
        ("Dia Mundial de los Océanos", "d(í|i)a mundial de los oc(é|e)anos", month_day(6, 8)),
        ("Día Mundial del Cáncer de Ovario", "d(í|i)a mundial del c(á|a)ncer de ovario", month_day(5, 8)),
        ("Día Mundial contra la Neumonía", "d(í|i)a mundial (contra|de) la neumon(í|i)a", month_day(11, 12)),
        ("Dia Mundial de la Poesia", "d(í|i)a mundial de la poesia", month_day(3, 21)),
        ("Día Mundial de la Población", "d(í|i)a mundial de la poblaci(ó|o)n", month_day(7, 11)),
        ("Día Mundial del Correo", "d(í|i)a mundial del correo", month_day(10, 9)),
        ("Día de la Prematuridad Mundial", "d(í|i)a (de la prematuridad mundial|mundial del (ni(ñ|n)o )?prematuro)", month_day(11, 17)),
        ("Día Mundial de la Libertad de Prensa", "d(í|i)a mundial de la libertad de prensa", month_day(5, 3)),
        ("Día Mundial contra la Rabia", "d(í|i)a mundial (contra|de) la rabia", month_day(9, 28)),
        ("Día Mundial de la Radio", "d(í|i)a mundial de la radio", month_day(2, 13)),
        ("Día Mundial de los Refugiados", "d(í|i)a mundial (del|de los) refugiado(s)?", month_day(6, 20)),
        ("Día Mundial de la Ciencia para la Paz y el Desarrollo", "d(í|i)a mundial de la ciencia para la paz y el desarrollo", month_day(11, 10)),
        ("Día Mundial de la Salud Sexual", "dmss|d(í|i)a mundial de la salud sexual", month_day(9, 4)),
        ("Día Mundial del Suelo", "d(í|i)a mundial del suelo", month_day(12, 5)),
        ("Día Mundial del Accidente Cerebrovascular", "d(í|i)a mundial del (accidente cerebrovascular|acv)", month_day(10, 29)),
        ("Día Mundial para la Prevención del Suicidio", "d(í|i)a mundial para la prevenci(ó|o)n del suicidio", month_day(9, 10)),
        ("Día Mundial de los Docentes", "d(í|i)a mundial (de los docentes|del maestro)", month_day(10, 5)),
        ("Día Mundial de la Televisión", "d(í|i)a mundial de la televisi(ó|o)n", month_day(11, 21)),
        ("Día Mundial del Baño", "d(í|i)a mundial del (ba(ñ|n)o|retrete)", month_day(11, 19)),
        ("Día Mundial del Turismo", "dmt|d(í|i)a mundial del turismo", month_day(9, 27)),
        ("Día Mundial de la Tuberculosis", "d(í|i)a mundial de (lucha contra)?la (tuberculosis|tb)", month_day(3, 24)),
        ("Día Mundial del Atun", "d(í|i)a mundial del atun", month_day(5, 2)),
        ("Día Mundial del Veganismo", "d(í|i)a mundial del veganismo", month_day(11, 1)),
        ("Día Mundial del Vegetarianismo", "d(í|i)a mundial del vegetarianismo", month_day(10, 1)),
        ("Día Mundial del Agua", "d(í|i)a mundial del agua", month_day(3, 22)),
        ("Día Mundial de los Humedales", "d(í|i)a mundial de los humedales", month_day(2, 2)),
        ("Día Mundial de la Vida Silvestre", "d(í|i)a mundial de la vida silvestre", month_day(3, 3)),
        ("Día Mundial de las Habilidades de la Juventud", "d(í|i)a mundial de las habilidades de la juventud", month_day(7, 15)),
        ("Día de la Cero Discriminación", "d(í|i)a de la cero discriminaci(ó|o)n", month_day(3, 1)),
        ("Día de la Commonwealth", "d(í|i)a de la commonwealth", nth_dow_of_month(2, 1, 3)),
        (
            "Día Mundial en Recuerdo de las Víctimas de Accidentes de Tráfico",
            "d(í|i)a (mundial )?en recuerdo de las v(í|i)ctimas (de accidentes )?de tr(á|a)fico",
            nth_dow_of_month(3, 7, 11),
        ),
        ("Día Internacional de las Cooperativas", "d(í|i)a internacional de las cooperativas", nth_dow_of_month(1, 6, 7)),
        (
            "Día de Martin Luther King",
            "(MLK|d(í|i)a de Martin Luther King,?)( Jr\\.?| J(ú|u)nior)?|d(í|i)a de los derechos humanos en Idaho",
            nth_dow_of_month(3, 1, 1),
        ),
        (
            "Viernes negro",
            "viernes negro",
            cycle_nth_after(false, G::Day, 1, &nth_dow_of_month(4, 4, 11)),
        ),
        ("Día Mundial del Hábitat", "d(í|i)a mundial del h(á|a)bitat", nth_dow_of_month(1, 1, 10)),
        ("Día Mundial del Riñón", "d(í|i)a mundial del ri(ñ|n)(ó|o)n", nth_dow_of_month(2, 4, 3)),
        ("Día Mundial de la Lepra", "d(í|i)a mundial (contra|de) la lepra", pred_last_of(&day_of_week(7), &month(1))),
        ("Día Marítimo Mundial", "d(í|i)a mar(í|i)timo mundial", pred_last_of(&day_of_week(4), &month(9))),
        ("Día Mundial de las Aves Migratorias", "d(í|i)a mundial de las aves migratorias", nth_dow_of_month(2, 6, 5)),
        ("Día mundial de la filosofía", "d(í|i)a mundial de la filosof(í|i)a", nth_dow_of_month(3, 4, 11)),
        ("Día mundial de la religion", "d(í|i)a mundial de la(s)? religion(es)?", nth_dow_of_month(3, 7, 1)),
        ("Día mundial de la vista", "d(í|i)a mundial de la (vista|visi(ó|o)n)", nth_dow_of_month(2, 4, 10)),
        ("Día del jefe", "(el )?d(í|i)a (nacional )?del jefe", pred_nth_closest(0, &weekday(), &month_day(10, 16))),
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
        rule_dayofweek_dayofmonth(),
        rule_dddd_monthinterval(),
        rule_ddmm(),
        rule_ddmmyyyy(),
        rule_de_datetime_datetime_interval(),
        rule_del_medioda(),
        rule_del_year(),
        rule_dentro_de_duration(),
        rule_dim_time_de_la_manana(),
        rule_dim_time_de_la_tarde(),
        rule_dim_time_de_la_tarde2(),
        rule_dom_of_month(),
        rule_dom_month(),
        rule_el_cycle_antes_time(),
        rule_el_cycle_proximoque_viene(),
        rule_el_cycle_proximoque_viene_time(),
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
        rule_hourofday_and_relative_minutes(),
        rule_hourofday_integer_as_relative_minutes(),
        rule_hourofday_minus_integer_as_relative_minutes(),
        rule_in_the_partofday(),
        rule_integer_in_the_partofday(),
        rule_intersect(),
        rule_intersect_by_de(),
        rule_la_cycle_pasado(),
        rule_la_pasado_cycle(),
        rule_midnight(),
        rule_month_dom(),
        rule_morning(),
        rule_n_cycle_proximoque_viene(),
        rule_n_pasados_cycle(),
        rule_n_proximas_cycle(),
        rule_namedmonthnamedday_next(),
        rule_namedmonthnamedday_past(),
        rule_navidad(),
        rule_nochevieja(),
        rule_noon(),
        rule_now(),
        rule_nth_time_de_time(),
        rule_nth_time_de_time2(),
        rule_ordinal_quarter(),
        rule_ordinal_quarter_year(),
        rule_pasados_n_cycle(),
        rule_proximas_n_cycle(),
        rule_right_now(),
        rule_the_day_after_tomorrow(),
        rule_the_day_before_yesterday(),
        rule_this_or_next_day_of_week(),
        rule_this_partofday(),
        rule_timeofday_ampm(),
        rule_timeofday_horas(),
        rule_timeofday_latent(),
        rule_timeofday_partofday(),
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
        rule_hourofday_minus_three_quarter(),
        rule_hourofday_minus_half(),
        rule_hourofday_minus_quarter(),
        rule_hourofday_minus_integer_as_relative_minutes2(),
        rule_timezone(),
        rule_el_dayofmonth_non_ordinal_with_dia(),
        rule_next_week(),
        rule_next_week_alt(),
        rule_year_by_adding_three_numbers(),
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
