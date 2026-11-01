//! `Duckling.Time.RO.Rules`.

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

/// `Duckling.Duration.Helpers.isGrain`.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn tod(t: &Token) -> Option<(i64, bool)> {
    match t {
        Token::Time(TimeData { form: Some(Form::TimeOfDay { hours: Some(h), is12h }), .. }) => {
            Some((*h, *is12h))
        }
        _ => None,
    }
}

fn latent_pod(a: i64, b: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, a), &hour(false, b)).map(|x| mk_latent(part_of_day(x))))
}

fn not_latent_and_hour_of_day(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

fn rule_the_day_after_tomorrow() -> Rule {
    rule("the day after tomorrow", vec![regex("poim(a|â)ine")], |_| tt(cycle_nth(G::Day, 2)))
}

fn rule_acum() -> Rule {
    rule("acum", vec![regex("(chiar)? ?acum|imediat")], |_| tt(now()))
}

fn rule_dupamiaza() -> Rule {
    rule("dupamiaza", vec![regex("dupamiaz(a|ă)|dup(a|ă) amiaz(a|ă)")], |_| latent_pod(12, 19))
}

fn rule_valentines_day() -> Rule {
    rule("valentine's day", vec![regex("sf\\.?((a|â)ntul)? Valentin")], |_| tt(month_day(2, 14)))
}

fn rule_since_timeofday() -> Rule {
    rule("since <time-of-day>", vec![regex("de|din"), dimension(Dim::Time)], |ts| {
        tt(with_direction(IntervalDirection::After, td(ts[1])?.clone()))
    })
}

fn rule_new_years_day() -> Rule {
    rule("new year's day", vec![regex("(siua de )? an(ul)? nou")], |_| tt(month_day(1, 1)))
}

fn rule_ordinal_trimestru_year() -> Rule {
    rule(
        "<ordinal> trimestru <year>",
        vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter)), dimension(Dim::Time)],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth_after(false, G::Quarter, n - 1, td(ts[2])?))
        },
    )
}

fn rule_in_namedmonth() -> Rule {
    rule("in <named-month>", vec![regex("(i|î)n"), pred(is_a_month)], |ts| Some(ts[1].clone()))
}

fn rule_last_cycle_of_time() -> Rule {
    rule(
        "last <cycle> of <time>",
        vec![regex("ultim(ul|a)"), dimension(Dim::TimeGrain), regex("din"), dimension(Dim::Time)],
        |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
    )
}

fn rule_intro_namedday() -> Rule {
    rule(
        "intr-o <named-day>",
        vec![regex("((i|î)n(tr)?(\\-?o)?)"), pred(is_a_day_of_week)],
        |ts| Some(ts[1].clone()),
    )
}

fn rule_month_dddd_interval() -> Rule {
    rule(
        "<month> dd-dd (interval)",
        vec![
            pred(is_a_month),
            regex("(3[01]|[12]\\d|0?[1-9])"),
            regex("\\-"),
            regex("(3[01]|[12]\\d|0?[1-9])"),
        ],
        |ts| {
            let d1 = parse_int(&rx(ts[1], 0)?)?;
            let d2 = parse_int(&rx(ts[3], 0)?)?;
            let dom1 = intersect(&day_of_month(d1), td(ts[0])?)?;
            let dom2 = intersect(&day_of_month(d2), td(ts[0])?)?;
            ttd(interval(Closed, &dom1, &dom2))
        },
    )
}

fn rule_in_duration() -> Rule {
    rule("in <duration>", vec![regex("(i|î)n"), dimension(Dim::Duration)], |ts| {
        tt(in_duration(dur(ts[1])?))
    })
}

fn rule_ieri() -> Rule {
    rule("ieri", vec![regex("ieri")], |_| tt(cycle_nth(G::Day, -1)))
}

fn rule_cycle_acesta() -> Rule {
    rule(
        "<cycle> acesta",
        vec![regex("aceasta|acest|(a|ă)sta"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
    )
}

fn rule_year_latent2() -> Rule {
    rule("year (latent)", vec![pred(is_integer_between_tok(2101, 10000))], |ts| {
        tt(mk_latent(year(get_int_value(ts[0])?)))
    })
}

fn rule_azi() -> Rule {
    rule("azi", vec![regex("a(st(a|ă))?zi")], |_| tt(today()))
}

fn rule_noon() -> Rule {
    rule("noon", vec![regex("noon")], |_| tt(hour(false, 12)))
}

fn rule_time_trecuta() -> Rule {
    // Upstream production looks at the second token as Time; the pattern is Time then regex.
    rule("<time> trecut[aă]?", vec![dimension(Dim::Time), regex("(trecut(a|ă)?)")], |ts| {
        tt(pred_nth(-1, false, td(ts.get(1)?)?))
    })
}

fn rule_thisnext_dayofweek() -> Rule {
    rule(
        "this|next <day-of-week>",
        vec![pred(is_a_day_of_week), regex("aceasta|(a|ă)sta|urm(a|ă)toare")],
        |ts| tt(pred_nth(0, true, td(ts[0])?)),
    )
}

fn rule_between_timeofday_and_timeofday_interval() -> Rule {
    rule(
        "between <time-of-day> and <time-of-day> (interval)",
        vec![regex("(i|î)ntre"), pred(is_a_time_of_day), regex("(s|ș)i"), pred(is_a_time_of_day)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_urmatoarea_cycle() -> Rule {
    rule(
        "urmatoarea <cycle>",
        vec![regex("(urm(a|ă)to(area|rul)|viito(are|r))"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[1])?, 1)),
    )
}

fn rule_cycle_acesta2() -> Rule {
    rule(
        "<cycle> acesta",
        vec![dimension(Dim::TimeGrain), regex("aceasta|acest|(a|ă)sta|curent(a|ă)")],
        |ts| tt(cycle_nth(grain_of(ts[0])?, 0)),
    )
}

fn rule_craciun() -> Rule {
    rule("craciun", vec![regex("(ziua de )?cr(a|ă)ciun")], |_| tt(month_day(12, 25)))
}

fn rule_trimestru_numeral_year() -> Rule {
    rule(
        "trimestru <number> <year>",
        vec![pred(is_grain(G::Quarter)), dimension(Dim::Numeral), dimension(Dim::Time)],
        |ts| {
            let v = get_int_value(ts[1])?;
            tt(cycle_nth_after(false, G::Quarter, v - 1, td(ts[2])?))
        },
    )
}

fn rule_ddmm() -> Rule {
    rule("dd/mm", vec![regex("(3[01]|[12]\\d|0?[1-9])/(0?[1-9]|1[0-2])")], |ts| {
        let d = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        tt(month_day(m, d))
    })
}

fn rule_la_timeofday() -> Rule {
    rule("la <time-of-day>", vec![regex("la|@ (ora)?"), pred(is_a_time_of_day)], |ts| {
        tt(not_latent(td(ts[1])?.clone()))
    })
}

fn rule_black_friday() -> Rule {
    rule("black friday", vec![regex("black frid?day")], |_| tt(nth_dow_of_month(4, 5, 11)))
}

fn rule_christmas_eve() -> Rule {
    rule("christmas eve", vec![regex("ajun(ul)? (de )?cr(a|ă)ciun")], |_| tt(month_day(12, 24)))
}

fn rule_pentru_duration() -> Rule {
    rule("pentru <duration>", vec![regex("pentru"), dimension(Dim::Duration)], |ts| {
        tt(in_duration(dur(ts[1])?))
    })
}

fn rule_hourofday_integer_as_relative_minutes() -> Rule {
    rule(
        "<hour-of-day> <integer> (as relative minutes)",
        vec![pred(not_latent_and_hour_of_day), pred(is_integer_between_tok(1, 59))],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            let n = get_int_value(ts[1])?;
            tt(hour_minute(is12h, hours, n))
        },
    )
}

fn rule_hourofday_sfert() -> Rule {
    rule("<hour-of-day> sfert", vec![pred(is_an_hour_of_day), regex("((s|ș)i )?(un )?sfert")], |ts| {
        let (hours, is12h) = tod(ts[0])?;
        tt(hour_minute(is12h, hours, 15))
    })
}

fn rule_hourofday_jumatate() -> Rule {
    rule(
        "<hour-of-day> sfert",
        vec![pred(is_an_hour_of_day), regex("((s|ș)i )?jum(a|ă)tate|jumate")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 30))
        },
    )
}

fn rule_diseara() -> Rule {
    rule("diseara", vec![regex("disear(a|ă)|((i|î)n aceas(a|ă) )?sear(a|ă)")], |_| {
        let td2 = interval(Open, &hour(false, 18), &hour(false, 0))?;
        ttd(intersect(&today(), &td2).map(part_of_day))
    })
}

fn rule_intersect_by() -> Rule {
    rule(
        "intersect by \",\"",
        vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_dayofmonth_numeral() -> Rule {
    rule("<day-of-month> (number)", vec![pred(is_dom_integer)], |ts| {
        tt(mk_latent(day_of_month(get_int_value(ts[0])?)))
    })
}

fn rule_nth_time_after_time() -> Rule {
    rule(
        "nth <time> after <time>",
        vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("dup(a|ă)"), dimension(Dim::Time)],
        |ts| {
            let v = get_int_value(ts[0])?;
            tt(pred_nth_after(v - 1, td(ts[1])?, td(ts[3])?))
        },
    )
}

fn rule_mmdd() -> Rule {
    rule("mm/dd", vec![regex("(0?[1-9]|1[0-2])/(3[01]|[12]\\d|0?[1-9])")], |ts| {
        let m = parse_int(&rx(ts[0], 0)?)?;
        let d = parse_int(&rx(ts[0], 1)?)?;
        tt(month_day(m, d))
    })
}

fn rule_timeofday_latent() -> Rule {
    rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
        tt(mk_latent(hour(true, get_int_value(ts[0])?)))
    })
}

fn rule_from_timeofday_timeofday_interval() -> Rule {
    rule(
        "from <time-of-day> - <time-of-day> (interval)",
        vec![
            regex("(dup(a|ă)|(i|î)ncep(a|â)nd cu)"),
            pred(is_a_time_of_day),
            regex("(dar |(s|ș)i )?((i|î)nainte|p(a|â)n(a|ă) la( de)?)"),
            pred(is_a_time_of_day),
        ],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_season3() -> Rule {
    rule("season", vec![regex("primavar(a|ă)")], |_| {
        ttd(interval(Open, &month_day(3, 20), &month_day(6, 21)))
    })
}

fn rule_urmatoarele_n_cycle() -> Rule {
    rule(
        "urmatoarele n <cycle>",
        vec![
            regex("urm(a|ă)to(arele|rii|area)"),
            pred(is_integer_between_tok(1, 9999)),
            dimension(Dim::TimeGrain),
        ],
        |ts| {
            let v = get_int_value(ts[1])?;
            tt(cycle_n(true, grain_of(ts[2])?, v))
        },
    )
}

fn rule_season() -> Rule {
    rule("season", vec![regex("toamn(a|ă)")], |_| {
        ttd(interval(Open, &month_day(9, 23), &month_day(12, 21)))
    })
}

fn rule_dupa_duration() -> Rule {
    rule("dupa <duration>", vec![regex("dup(a|ă)"), dimension(Dim::Duration)], |ts| {
        tt(with_direction(IntervalDirection::After, in_duration(dur(ts[1])?)))
    })
}

fn rule_new_years_eve() -> Rule {
    rule("new year's eve", vec![regex("(ajun(ul)? )?(de )?an(ul)? nou")], |_| tt(month_day(12, 31)))
}

fn rule_by_the_end_of_time() -> Rule {
    rule(
        "by the end of <time>",
        vec![regex("p(a|î)n(a|ă) ((i|î)n|la)"), dimension(Dim::Time)],
        |ts| ttd(interval(Closed, &now(), td(ts[1])?)),
    )
}

fn rule_namedday_pe_dayofmonth_numeral() -> Rule {
    rule(
        "<named-day> pe <day-of-month> (number)",
        vec![pred(is_a_day_of_week), regex("pe"), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[2])),
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
    rule(
        "about <time-of-day>",
        vec![regex("(cam|aproximativ|(i|î)n jur de)"), pred(is_a_time_of_day)],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    )
}

fn rule_until_timeofday() -> Rule {
    rule(
        "until <time-of-day>",
        vec![regex("p(a|î)n(a|ă) ((i|î)n|la)"), dimension(Dim::Time)],
        |ts| tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone())),
    )
}

fn rule_dayofmonthnumber_namedmonth() -> Rule {
    rule(
        "<day-of-month>(number) <named-month>",
        vec![pred(is_dom_integer), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
    )
}

fn rule_intre_datetime_si_datetime_interval() -> Rule {
    rule(
        "intre <datetime> si <datetime> (interval)",
        vec![regex("(i|î)nre"), dimension(Dim::Time), regex("(s|ș)i"), dimension(Dim::Time)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_nth_time_of_time() -> Rule {
    rule(
        "nth <time> of <time>",
        vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("din"), dimension(Dim::Time)],
        |ts| {
            let v = get_int_value(ts[0])?;
            ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(v - 1, false, &x)))
        },
    )
}

fn rule_time_partofday() -> Rule {
    rule("<time> <part-of-day>", vec![dimension(Dim::Time), pred(is_a_part_of_day)], |ts| {
        ttd(intersect(td(ts[0])?, td(ts[1])?))
    })
}

fn rule_weekend() -> Rule {
    rule("week-end", vec![regex("(week(\\s|\\-)?end|wkend)")], |_| tt(weekend()))
}

fn rule_pe_dayofmonth_non_ordinal() -> Rule {
    rule("pe <day-of-month> (non ordinal)", vec![regex("pe"), pred(is_dom_integer)], |ts| {
        tt(mk_latent(day_of_month(get_int_value(ts[1])?)))
    })
}

fn rule_time_aceastaacestaasta() -> Rule {
    rule(
        "<time> (aceasta|acesta|[aă]sta)",
        vec![dimension(Dim::Time), regex("aceasta|(a|ă)sta|urm(a|ă)toare")],
        |ts| tt(pred_nth(0, false, td(ts[0])?)),
    )
}

fn rule_eomend_of_month() -> Rule {
    rule("EOM|End of month", vec![regex("sf(a|â)r(s|ș)itul lunii")], |_| tt(cycle_nth(G::Month, 1)))
}

fn rule_partofday_asta() -> Rule {
    rule("<part-of-day> asta", vec![pred(is_a_part_of_day), regex("asta")], |ts| {
        ttd(intersect(&today(), td(ts[0])?).map(part_of_day))
    })
}

fn rule_ultima_cycle() -> Rule {
    rule("ultima <cycle>", vec![regex("ultim(a|ul)"), dimension(Dim::TimeGrain)], |ts| {
        tt(cycle_nth(grain_of(ts[1])?, -1))
    })
}

fn rule_cycle_urmatoare() -> Rule {
    // Upstream production looks at the second token as TimeGrain; the pattern is grain then regex.
    rule(
        "<cycle> urmatoare",
        vec![dimension(Dim::TimeGrain), regex("(urm(a|ă)to(are|r)|viito(are|r))")],
        |ts| tt(cycle_nth(grain_of(ts.get(1)?)?, 1)),
    )
}

fn rule_the_dayofmonth_numeral() -> Rule {
    rule("the <day-of-month> (number)", vec![regex("pe"), pred(is_dom_integer)], |ts| {
        tt(day_of_month(get_int_value(ts[1])?))
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

fn rule_cycle_trecut() -> Rule {
    rule("<cycle> trecut", vec![dimension(Dim::TimeGrain), regex("trecut(a|ă)?")], |ts| {
        tt(cycle_nth(grain_of(ts[0])?, -1))
    })
}

fn rule_dayofmonth_numeral_of_namedmonth() -> Rule {
    rule(
        "<day-of-month> (number) of <named-month>",
        vec![pred(is_dom_integer), regex("din"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
    )
}

fn rule_duration_inainte_de_time() -> Rule {
    rule(
        "<duration> inainte de <time>",
        vec![dimension(Dim::Duration), regex("(i|î)nainte de"), dimension(Dim::Time)],
        |ts| tt(duration_before(dur(ts[0])?, td(ts[2])?)),
    )
}

fn rule_dayofmonth_non_ordinal_of_namedmonth() -> Rule {
    rule(
        "<day-of-month> (non ordinal) of <named-month>",
        vec![pred(is_dom_integer), regex("din"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
    )
}

fn rule_duration_in_urma() -> Rule {
    rule("<duration> in urma", vec![dimension(Dim::Duration), regex("(i|î)n urm(a|ă)")], |ts| {
        tt(duration_ago(dur(ts[0])?))
    })
}

fn rule_duration_de_acum() -> Rule {
    rule("<duration> de acum", vec![dimension(Dim::Duration), regex("de (acum|azi)")], |ts| {
        tt(in_duration(dur(ts[0])?))
    })
}

fn rule_sezon_anotimp() -> Rule {
    rule("sezon anotimp", vec![regex("var(a|ă)")], |_| {
        ttd(interval(Open, &month_day(6, 21), &month_day(9, 23)))
    })
}

fn rule_seara_noapte() -> Rule {
    rule("sear[aă] noapte", vec![regex("sear(a|ă)|noapte")], |_| latent_pod(18, 0))
}

fn rule_year_latent() -> Rule {
    rule("year (latent)", vec![pred(is_integer_between_tok(1, 999))], |ts| {
        tt(mk_latent(year(get_int_value(ts[0])?)))
    })
}

fn rule_season2() -> Rule {
    rule("season", vec![regex("iarn(a|ă)")], |_| {
        ttd(interval(Open, &month_day(12, 21), &month_day(3, 20)))
    })
}

fn rule_ultimele_n_cycle() -> Rule {
    rule(
        "ultimele n <cycle>",
        vec![regex("ultim(ele|ii|a)"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
        |ts| {
            let v = get_int_value(ts[1])?;
            tt(cycle_n(true, grain_of(ts[2])?, -v))
        },
    )
}

fn rule_after_timeofday() -> Rule {
    rule("after <time-of-day>", vec![regex("dup(a|ă)"), dimension(Dim::Time)], |ts| {
        tt(with_direction(IntervalDirection::After, td(ts[1])?.clone()))
    })
}

fn rule_dimineata() -> Rule {
    rule("diminea[tț][aă]", vec![regex("diminea(t|ț)(a|ă)")], |_| latent_pod(4, 12))
}

fn rule_time_urmatoarer() -> Rule {
    rule("<time> urm[aă]to(are|r)", vec![regex("urm(a|ă)to(are|r)"), pred(is_not_latent)], |ts| {
        tt(pred_nth(0, true, td(ts[1])?))
    })
}

fn rule_timeofday_fix() -> Rule {
    rule("<time-of-day> fix", vec![pred(is_a_time_of_day), regex("(fix|exact)")], |ts| {
        tt(not_latent(td(ts[0])?.clone()))
    })
}

fn rule_hhmm() -> Rule {
    rule("hh:mm", vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)")], |ts| {
        let h = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        tt(hour_minute(true, h, m))
    })
}

fn rule_maine() -> Rule {
    rule("maine", vec![regex("m(a|â)ine")], |_| tt(cycle_nth(G::Day, 1)))
}

fn rule_timezone() -> Rule {
    rule(
        "<time> timezone",
        vec![
            pred(|t| is_not_latent(t) && is_a_time_of_day(t)),
            regex("\\b(YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT)\\b"),
        ],
        |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
    )
}

fn rule_year() -> Rule {
    rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
        tt(year(get_int_value(ts[0])?))
    })
}

fn rule_halloween_day() -> Rule {
    rule("halloween day", vec![regex("hall?owe?en")], |_| tt(month_day(10, 31)))
}

fn rule_namedmonth_dayofmonth_non_ordinal() -> Rule {
    rule(
        "<named-month> <day-of-month> (non ordinal)",
        vec![pred(is_a_month), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_pe_date() -> Rule {
    rule("pe <date>", vec![regex("pe"), dimension(Dim::Time)], |ts| Some(ts[1].clone()))
}

fn rule_dayofmonthnumber_namedmonth_year() -> Rule {
    rule(
        "<day-of-month>(number) <named-month> year",
        vec![pred(is_dom_integer), pred(is_a_month), regex("(\\d{2,4})")],
        |ts| {
            let v = parse_int(&rx(ts[2], 0)?)?;
            let dom = intersect_dom(td(ts[1])?, ts[0])?;
            ttd(intersect(&dom, &year(v)))
        },
    )
}

fn rule_absorption_of_after_named_day() -> Rule {
    rule("absorption of , after named day", vec![pred(is_a_day_of_week), regex(",")], |ts| {
        Some(ts[0].clone())
    })
}

fn rule_last_dayofweek_of_time() -> Rule {
    rule(
        "last <day-of-week> of <time>",
        vec![regex("ultima"), pred(is_a_day_of_week), regex("din"), dimension(Dim::Time)],
        |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
    )
}

fn rule_ordinal_trimestru() -> Rule {
    rule("<ordinal> trimestru", vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(cycle_nth_after(true, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
    })
}

fn rule_by_time() -> Rule {
    rule("by <time>", vec![regex("p(a|â)n(a|ă) (la|(i|î)n)"), dimension(Dim::Time)], |ts| {
        ttd(interval(Open, &now(), td(ts[1])?))
    })
}

fn rule_ddmmyyyy() -> Rule {
    rule(
        "dd/mm/yyyy",
        vec![regex("(3[01]|[12]\\d|0?[1-9])[-/](0?[1-9]|1[0-2])[/-](\\d{2,4})")],
        |ts| {
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let y = parse_int(&rx(ts[0], 2)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn rule_mmddyyyy() -> Rule {
    rule(
        "mm/dd/yyyy",
        vec![regex("(0?[1-9]|1[0-2])[/-](3[01]|[12]\\d|0?[1-9])[-/](\\d{2,4})")],
        |ts| {
            let m = parse_int(&rx(ts[0], 0)?)?;
            let d = parse_int(&rx(ts[0], 1)?)?;
            let y = parse_int(&rx(ts[0], 2)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn rule_eoyend_of_year() -> Rule {
    rule("EOY|End of year", vec![regex("sf(a|â)r(s|ș)itul anului")], |_| tt(cycle_nth(G::Year, 1)))
}

fn rule_mothers_day() -> Rule {
    rule("Mother's Day", vec![regex("ziua (mamei|memeii)")], |_| tt(nth_dow_of_month(1, 7, 5)))
}

fn rule_namedday_dayofmonth_numeral() -> Rule {
    rule(
        "<named-day> <day-of-month> (number)",
        vec![pred(is_a_day_of_week), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
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

fn rule_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("luni", "lu(n(ea|i)?)?"),
        ("marti", "ma(r((t|ț)(ea|i))?)?"),
        ("miercuri", "mi(e(rcur(ea|i))?)?"),
        ("joi", "jo(ia?)?"),
        ("vineri", "vi(n(er(ea|i))?)?"),
        ("sambata", "s(a|â)mb(a|ă)t(a|ă)|s(a|â)m"),
        ("duminica", "du(m(inic(a|ă))?)?"),
    ])
}

fn rule_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("ianuarie", "ian(uarie)?"),
        ("februarie", "feb(ruarie)?"),
        ("martie", "martie|mar"),
        ("aprilie", "apr(ilie)?"),
        ("mai", "mai"),
        ("iunie", "iun(ie)?"),
        ("iulie", "iul(ie)?"),
        ("august", "aug(ust)?"),
        ("septembrie", "sept(embrie)?"),
        ("octombrie", "oct(ombrie)?"),
        ("noiembrie", "noi(embrie)?"),
        ("decembrie", "dec(embrie)?"),
    ])
}

fn build_rules() -> Vec<Rule> {
    let mut v = vec![
        rule_about_timeofday(),
        rule_absorption_of_after_named_day(),
        rule_acum(),
        rule_after_timeofday(),
        rule_azi(),
        rule_between_timeofday_and_timeofday_interval(),
        rule_black_friday(),
        rule_by_the_end_of_time(),
        rule_by_time(),
        rule_christmas_eve(),
        rule_craciun(),
        rule_cycle_acesta(),
        rule_cycle_acesta2(),
        rule_cycle_trecut(),
        rule_cycle_urmatoare(),
        rule_dayofmonth_non_ordinal_namedmonth(),
        rule_dayofmonth_non_ordinal_of_namedmonth(),
        rule_dayofmonth_numeral(),
        rule_dayofmonth_numeral_of_namedmonth(),
        rule_dayofmonthnumber_namedmonth(),
        rule_dayofmonthnumber_namedmonth_year(),
        rule_ddmm(),
        rule_ddmmyyyy(),
        rule_dimineata(),
        rule_diseara(),
        rule_dupa_duration(),
        rule_dupamiaza(),
        rule_duration_de_acum(),
        rule_duration_in_urma(),
        rule_duration_inainte_de_time(),
        rule_eomend_of_month(),
        rule_eoyend_of_year(),
        rule_from_timeofday_timeofday_interval(),
        rule_halloween_day(),
        rule_hhmm(),
        rule_hhmmss(),
        rule_hourofday_integer_as_relative_minutes(),
        rule_hourofday_jumatate(),
        rule_hourofday_sfert(),
        rule_ieri(),
        rule_in_duration(),
        rule_in_namedmonth(),
        rule_intersect(),
        rule_intersect_by(),
        rule_intre_datetime_si_datetime_interval(),
        rule_intro_namedday(),
        rule_la_timeofday(),
        rule_last_cycle_of_time(),
        rule_last_dayofweek_of_time(),
        rule_maine(),
        rule_mmdd(),
        rule_mmddyyyy(),
        rule_month_dddd_interval(),
        rule_mothers_day(),
        rule_namedday_dayofmonth_numeral(),
        rule_namedday_pe_dayofmonth_numeral(),
        rule_namedmonth_dayofmonth_non_ordinal(),
        rule_new_years_day(),
        rule_new_years_eve(),
        rule_noon(),
        rule_nth_time_after_time(),
        rule_nth_time_of_time(),
        rule_ordinal_trimestru(),
        rule_ordinal_trimestru_year(),
        rule_partofday_asta(),
        rule_pe_date(),
        rule_pe_dayofmonth_non_ordinal(),
        rule_pentru_duration(),
        rule_seara_noapte(),
        rule_season(),
        rule_season2(),
        rule_season3(),
        rule_sezon_anotimp(),
        rule_since_timeofday(),
        rule_the_dayofmonth_numeral(),
        rule_thisnext_dayofweek(),
        rule_time_aceastaacestaasta(),
        rule_time_partofday(),
        rule_timezone(),
        rule_time_trecuta(),
        rule_time_urmatoarer(),
        rule_timeofday_fix(),
        rule_timeofday_latent(),
        rule_trimestru_numeral_year(),
        rule_ultima_cycle(),
        rule_ultimele_n_cycle(),
        rule_until_timeofday(),
        rule_urmatoarea_cycle(),
        rule_urmatoarele_n_cycle(),
        rule_valentines_day(),
        rule_weekend(),
        rule_year(),
        rule_year_latent(),
        rule_year_latent2(),
        rule_yyyymmdd(),
        rule_the_day_after_tomorrow(),
    ];
    v.extend(rule_days_of_week());
    v.extend(rule_months());
    v
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &RULES
}
