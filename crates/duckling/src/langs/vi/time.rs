//! `Duckling.Time.VI.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time::helpers::*;
use crate::dims::time::types::{
    Form, TimeData,
    TimeIntervalType::Open,
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

fn latent_pod(a: i64, b: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, a), &hour(false, b)).map(|x| part_of_day(mk_latent(x))))
}

fn rule_tt_dng() -> Rule {
    rule("tết dương", vec![regex("(t(ế)t d(ư)(ơ)ng)(l(ị)ch)?")], |_| tt(month_day(1, 1)))
}

fn rule_dayofmonth_namedmonth() -> Rule {
    rule(
        "<day-of-month> (numeric with day symbol) <named-month>",
        vec![regex("(ng(à)y|m(ồ)ng)( m(ồ)ng)?"), pred(is_dom_integer), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[2])?, ts[1])),
    )
}

fn rule_dayofweek_cui_cng_ca_time() -> Rule {
    rule(
        "<day-of-week> cuối cùng của <time>",
        vec![
            pred(is_a_day_of_week),
            regex("cu(ố)i c(ù)ng|cu(ố)i"),
            regex("c(ủ)a|trong|v(à)o"),
            dimension(Dim::Time),
        ],
        |ts| tt(pred_last_of(td(ts[0])?, td(ts[3])?)),
    )
}

fn rule_time_ti() -> Rule {
    rule("<time> tới", vec![pred(is_not_latent), regex("t(ớ)i")], |ts| {
        tt(pred_nth(0, true, td(ts[0])?))
    })
}

fn rule_cycle_cui_cng_ca_time() -> Rule {
    rule(
        "<cycle> cuối cùng của <time>",
        vec![
            dimension(Dim::TimeGrain),
            regex("cu(ố)i c(ù)ng|cu(ố)i"),
            regex("c(ủ)a|trong|v(à)o"),
            dimension(Dim::Time),
        ],
        |ts| tt(cycle_last_of(grain_of(ts[0])?, td(ts[3])?)),
    )
}

fn rule_timeofday_latent2() -> Rule {
    rule(
        "time-of-day (latent)",
        vec![regex("(l(ú)c|v(à)o)( l(ú)c)?"), pred(is_integer_between_tok(0, 23))],
        |ts| tt(mk_latent(hour(false, get_int_value(ts[1])?))),
    )
}

fn rule_cch_mng_thng() -> Rule {
    rule("cách mạng tháng 8", vec![regex("c(á)ch m(ạ)ng th(á)ng (8|t(á)m)")], |_| {
        tt(month_day(8, 19))
    })
}

fn rule_timeofday_gi_ng() -> Rule {
    rule("<time-of-day> giờ đúng", vec![pred(is_a_time_of_day), regex("gi(ờ) (đ)(ú)ng")], |ts| {
        tt(not_latent(td(ts[0])?.clone()))
    })
}

fn rule_partofday_hm_nay() -> Rule {
    rule("<part-of-day> (hôm )?nay", vec![pred(is_a_part_of_day), regex("(h(ô)m )?nay")], |ts| {
        ttd(intersect(&today(), td(ts[0])?).map(part_of_day))
    })
}

fn rule_ngy_ddmmyyyy() -> Rule {
    rule(
        "ngày dd/mm/yyyy",
        vec![
            regex("ng(à)y"),
            regex(r"(3[01]|[12]\d|0?[1-9])[-/](1[0-2]|0?[1-9])[/-](\d{2,4})"),
        ],
        |ts| {
            let y = parse_int(&rx(ts[1], 2)?)?;
            let m = parse_int(&rx(ts[1], 1)?)?;
            let d = parse_int(&rx(ts[1], 0)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn rule_partofday_time() -> Rule {
    rule("<part-of-day> <time>", vec![pred(is_a_part_of_day), dimension(Dim::Time)], |ts| {
        ttd(intersect(td(ts[0])?, td(ts[1])?))
    })
}

fn rule_quarter_number() -> Rule {
    rule("quarter <number>", vec![pred(is_grain(G::Quarter)), dimension(Dim::Numeral)], |ts| {
        let n = get_int_value(ts[1])?;
        tt(cycle_nth_after(true, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
    })
}

fn rule_season4() -> Rule {
    rule("season", vec![regex("m(ù)a? xu(â)n")], |_| {
        ttd(interval(Open, &month_day(3, 20), &month_day(6, 21)))
    })
}

fn rule_noon() -> Rule {
    rule("noon", vec![regex("(bu(ổ)i )?(t(ố)i|(đ)(ê)m)")], |_| tt(hour(false, 12)))
}

fn rule_quc_t_lao_ng() -> Rule {
    rule("quốc tế lao động", vec![regex("(ng(à)y )?qu(ố)c t(ế) lao (đ)(ô)ng")], |_| {
        tt(month_day(5, 1))
    })
}

fn rule_next_cycle() -> Rule {
    rule(
        "next <cycle>",
        vec![dimension(Dim::TimeGrain), regex("(t(ớ)i|k(ế)|sau|ti(ế)p)( theo)?( ti(ế)p)?")],
        |ts| tt(cycle_nth(grain_of(ts[0])?, 1)),
    )
}

fn rule_timeofday_approximately() -> Rule {
    rule("<time-of-day> approximately", vec![pred(is_a_time_of_day), regex("g(ầ)n")], |ts| {
        tt(not_latent(td(ts[0])?.clone()))
    })
}

fn rule_quarter_number_of_year() -> Rule {
    rule(
        "quarter <number> of <year>",
        vec![pred(is_grain(G::Quarter)), dimension(Dim::Numeral), regex("c(ủ)a|trong"), dimension(Dim::Time)],
        |ts| {
            let n = get_int_value(ts[1])?;
            tt(cycle_nth_after(false, G::Quarter, n - 1, td(ts[3])?))
        },
    )
}

fn rule_lunch() -> Rule {
    rule("lunch", vec![regex("(bu(ổ)i )?tr(ư)a")], |_| latent_pod(12, 14))
}

fn rule_last_cycle() -> Rule {
    rule(
        "last <cycle>",
        vec![dimension(Dim::TimeGrain), regex("tr(ư)(ớ)c|qua|v(ừ)a r(ồ)i|ngo(á)i")],
        |ts| tt(cycle_nth(grain_of(ts[0])?, -1)),
    )
}

fn rule_ddmm() -> Rule {
    rule("dd/mm", vec![regex(r"(3[01]|[12]\d|0?[1-9])/(0?[1-9]|1[0-2])")], |ts| {
        let d = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        tt(month_day(m, d))
    })
}

fn rule_afternoon() -> Rule {
    rule("afternoon", vec![regex("(bu(ổ)i )?chi(ề)u")], |_| latent_pod(12, 19))
}

fn rule_hm_nay() -> Rule {
    rule("hôm nay", vec![regex("((ngay )?h(ô)m|b(ữ)a) nay")], |_| tt(today()))
}

fn rule_at_hhmm() -> Rule {
    rule(
        "at hh:mm",
        vec![
            regex("(l(ú)c|v(à)o)( l(ú)c)?"),
            regex(r"((?:[01]?\d)|(?:2[0-3]))[:.hg]([0-5]\d)"),
        ],
        |ts| {
            let h = parse_int(&rx(ts[1], 0)?)?;
            let m = parse_int(&rx(ts[1], 1)?)?;
            tt(hour_minute(true, h, m))
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
        "(hour-of-day) quarter",
        vec![pred(is_an_hour_of_day), regex("k(é)m"), pred(is_integer_between_tok(1, 59))],
        |ts| ttd(minutes_before(get_int_value(ts[2])?, td(ts[0])?)),
    )
}

fn rule_hourofday_half() -> Rule {
    rule("(hour-of-day) half", vec![pred(is_an_hour_of_day), regex("r(ư)(ỡ)i")], |ts| {
        let (hours, is12h) = tod(ts[0])?;
        tt(hour_minute(is12h, hours, 30))
    })
}

fn rule_hourofday_integer_pht() -> Rule {
    rule(
        "<hour-of-day> <integer> phút",
        vec![pred(is_an_hour_of_day), pred(is_integer_between_tok(1, 59)), regex("ph(ú)t")],
        |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, get_int_value(ts[1])?))
        },
    )
}

fn rule_cui_nm() -> Rule {
    rule("cuối năm", vec![regex("cu(ố)i n(ă)m")], |_| tt(cycle_nth(G::Year, 1)))
}

fn rule_quarter_number_year() -> Rule {
    rule(
        "quarter <number> <year>",
        vec![pred(is_grain(G::Quarter)), dimension(Dim::Numeral), dimension(Dim::Time)],
        |ts| {
            let v = get_int_value(ts[1])?;
            tt(cycle_nth_after(false, G::Quarter, v - 1, td(ts[2])?))
        },
    )
}

fn rule_timeofday_latent() -> Rule {
    rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
        tt(mk_latent(hour(true, get_int_value(ts[0])?)))
    })
}

fn rule_exactly_timeofday() -> Rule {
    rule("exactly <time-of-day>", vec![regex("(v(à)o )?(đ)(ú)ng"), pred(is_a_time_of_day)], |ts| {
        tt(not_latent(td(ts[1])?.clone()))
    })
}

fn rule_ngy_hm_qua() -> Rule {
    rule("ngày hôm qua", vec![regex("(ng(à)y )?(h(ô)m )?qua")], |_| tt(cycle_nth(G::Day, -1)))
}

fn rule_season3() -> Rule {
    rule("season", vec![regex("m(ù)a? (đ)(ô)ng")], |_| {
        ttd(interval(Open, &month_day(12, 21), &month_day(3, 20)))
    })
}

fn rule_season() -> Rule {
    rule("season", vec![regex("m(ù)a? (h(è)|h(ạ))")], |_| {
        ttd(interval(Open, &month_day(6, 21), &month_day(9, 23)))
    })
}

fn rule_dayofmonth_namedmonth2() -> Rule {
    rule("<day-of-month> <named-month>", vec![pred(is_dom_integer), pred(is_a_month)], |ts| {
        ttd(intersect_dom(td(ts[1])?, ts[0]))
    })
}

fn rule_year_numeric_with_year_symbol() -> Rule {
    rule(
        "year (numeric with year symbol)",
        vec![regex("n(ă)m"), pred(is_integer_between_tok(1000, 9999))],
        |ts| tt(year(get_int_value(ts[1])?)),
    )
}

fn rule_after_work() -> Rule {
    rule("after work", vec![regex("(sau gi(ờ) l(à)m|sau gi(ờ) tan t(ầ)m|l(ú)c tan t(ầ)m)")], |_| {
        let td = interval(Open, &hour(false, 17), &hour(false, 21))?;
        ttd(intersect(&today(), &td).map(|x| part_of_day(not_latent(x))))
    })
}

fn rule_last_n_cycle() -> Rule {
    rule(
        "last n <cycle>",
        vec![
            pred(is_integer_between_tok(1, 9999)),
            dimension(Dim::TimeGrain),
            regex("tr(ư)(ớ)c|qua|v(ừ)a r(ồ)i|ngo(á)i|v(ừ)a qua"),
        ],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_n(true, grain_of(ts[1])?, -n))
        },
    )
}

fn rule_timeofday_sharp() -> Rule {
    rule("<time-of-day> sharp", vec![pred(is_a_time_of_day), regex("(đ)(ú)ng")], |ts| {
        tt(not_latent(td(ts[0])?.clone()))
    })
}

fn rule_intersect() -> Rule {
    rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
        ttd(intersect(td(ts[0])?, td(ts[1])?))
    })
}

fn rule_about_timeofday() -> Rule {
    rule("about <time-of-day>", vec![regex("(v(à)o )?kho(ả)ng"), pred(is_a_time_of_day)], |ts| {
        tt(not_latent(td(ts[1])?.clone()))
    })
}

fn rule_l_tnh_nhn() -> Rule {
    rule("lễ tình nhân", vec![regex("(ng(à)y )?(l(ễ))? t(ì)nh nh(â)n")], |_| tt(month_day(2, 14)))
}

fn rule_cui_thng() -> Rule {
    rule("cuối tháng", vec![regex("cu(ố)i th(á)ng")], |_| tt(cycle_nth(G::Month, 1)))
}

fn rule_weekend() -> Rule {
    rule("week-end", vec![regex("(cu(ố)i|h(ế)t) tu(ầ)n")], |_| tt(weekend()))
}

fn rule_timeofday_sngchiuti() -> Rule {
    rule(
        "<time-of-day> sáng|chiều|tối",
        vec![pred(is_a_time_of_day), regex("(s(á)ng|chi(ề)u|t(ố)i)")],
        |ts| tt(time_of_day_ampm(to_lower(&rx(ts[1], 0)?) == "sáng", td(ts[0])?)),
    )
}

fn rule_time_trc() -> Rule {
    rule(
        "<time> trước",
        vec![pred(|t| !is_grain_finer_than(G::Day)(t)), regex("tr(ư)(ớ)c|v(ừ)a r(ồ)i")],
        |ts| tt(pred_nth(-1, false, td(ts[0])?)),
    )
}

fn rule_timeofday_gi() -> Rule {
    rule("time-of-day giờ", vec![pred(is_a_time_of_day), regex("gi(ờ)")], |ts| {
        tt(not_latent(td(ts[0])?.clone()))
    })
}

fn rule_quc_khnh() -> Rule {
    rule("quốc khánh", vec![regex("qu(ố)c kh(á)nh")], |_| tt(month_day(9, 3)))
}

fn rule_yyyymmdd() -> Rule {
    rule("yyyy-mm-dd", vec![regex(r"(\d{2,4})-(0?[1-9]|1[0-2])-(3[01]|[12]\d|0?[1-9])")], |ts| {
        let y = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        let d = parse_int(&rx(ts[0], 2)?)?;
        tt(year_month_day(y, m, d))
    })
}

fn rule_intersect_by_of_from_s() -> Rule {
    rule(
        "intersect by \"of\", \"from\", \"'s\"",
        vec![pred(is_not_latent), regex("c(ủ)a|trong"), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_next_n_cycle() -> Rule {
    rule(
        "next n <cycle>",
        vec![
            pred(is_integer_between_tok(1, 9999)),
            dimension(Dim::TimeGrain),
            regex("(t(ớ)i|k(ế)|sau|ti(ế)p)( theo)?( ti(ế)p)?"),
        ],
        |ts| tt(cycle_n(true, grain_of(ts[1])?, get_int_value(ts[0])?)),
    )
}

fn rule_morning() -> Rule {
    rule("morning", vec![regex("(bu(ổ)i )?s(á)ng")], |_| latent_pod(4, 12))
}

fn rule_this_cycle() -> Rule {
    rule(
        "this <cycle>",
        vec![dimension(Dim::TimeGrain), regex("nay|n(à)y|hi(ệ)n t(ạ)i|h(ô)m nay|n(ă)m nay")],
        |ts| tt(cycle_nth(grain_of(ts[0])?, 0)),
    )
}

fn rule_noon2() -> Rule {
    rule("noon", vec![regex("(bu(ổ)i )?(t(ố)i|(đ)(ê)m)")], |_| latent_pod(18, 0))
}

fn rule_after_lunch() -> Rule {
    rule("after lunch", vec![regex("(sau|qua) (bu(ổ)i |b(ữ)a )?tr(ư)a")], |_| {
        let td = interval(Open, &hour(false, 13), &hour(false, 17))?;
        ttd(intersect(&today(), &td).map(|x| part_of_day(not_latent(x))))
    })
}

fn rule_season2() -> Rule {
    rule("season", vec![regex("m(ù)a? thu")], |_| {
        ttd(interval(Open, &month_day(9, 23), &month_day(12, 21)))
    })
}

fn rule_time_ny() -> Rule {
    rule("<time> này", vec![dimension(Dim::Time), regex("n(à)y|nay")], |ts| {
        tt(pred_nth(0, false, td(ts[0])?))
    })
}

fn rule_timeofday_ampm() -> Rule {
    rule(
        "<time-of-day> am|pm",
        vec![pred(is_a_time_of_day), regex(r"(in the )?([ap])(\s|\.)?m?\.?")],
        |ts| tt(time_of_day_ampm(to_lower(&rx(ts[1], 1)?) == "a", td(ts[0])?)),
    )
}

fn rule_ngy_mai() -> Rule {
    rule("ngày mai", vec![regex("(ng(à)y )?mai")], |_| tt(cycle_nth(G::Day, 1)))
}

fn rule_ordinal_cycle_of_time() -> Rule {
    rule(
        "<ordinal> <cycle> of <time>",
        vec![dimension(Dim::TimeGrain), dimension(Dim::Ordinal), regex("c(ủ)a|trong"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(true, grain_of(ts[0])?, ordinal_value(ts[1])? - 1, td(ts[3])?)),
    )
}

fn rule_month_numeric_with_month_symbol() -> Rule {
    rule(
        "month (numeric with month symbol)",
        vec![regex("th(á)ng"), pred(is_integer_between_tok(1, 12))],
        |ts| tt(month(get_int_value(ts[1])?)),
    )
}

fn rule_hhmm() -> Rule {
    rule("hh:mm", vec![regex(r"((?:[01]?\d)|(?:2[0-3]))[:.hg]([0-5]\d)")], |ts| {
        let h = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        tt(hour_minute(true, h, m))
    })
}

fn rule_tonight() -> Rule {
    rule("tonight", vec![regex("(t(ố)i|(đ)(ê)m)( h(ô)m)? nay")], |_| {
        let evening = interval(Open, &hour(false, 18), &hour(false, 0))?;
        ttd(intersect(&today(), &evening).map(|x| part_of_day(not_latent(x))))
    })
}

fn rule_timezone() -> Rule {
    rule(
        "<time> timezone",
        vec![
            pred(|t| is_not_latent(t) && is_grain_finer_than(G::Day)(t)),
            regex(r"\b(YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT)\b"),
        ],
        |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
    )
}

fn rule_by_gi() -> Rule {
    rule("bây giờ", vec![regex("(ngay )?(b(â)y gi(ờ)|l(ú)c n(à)y)")], |_| tt(now()))
}

fn rule_ngy_ddmm() -> Rule {
    rule(
        "ngày dd/mm",
        vec![regex("ng(à)y"), regex(r"(3[01]|[12]\d|0?[1-9])/(1[0-2]|0?[1-9])")],
        |ts| {
            let d = parse_int(&rx(ts[1], 0)?)?;
            let m = parse_int(&rx(ts[1], 1)?)?;
            tt(month_day(m, d))
        },
    )
}

fn rule_hhmm_military_ampm() -> Rule {
    rule(
        "hhmm (military) am|pm",
        vec![regex(r"((?:1[012]|0?\d))([0-5]\d)"), regex(r"([ap])\.?m?\.?")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(time_of_day_ampm(to_lower(&rx(ts[1], 0)?) == "a", &hour_minute(true, h, m)))
        },
    )
}

fn rule_hhmm_military_sngchiuti() -> Rule {
    rule(
        "hhmm (military) sáng|chiều|tối",
        vec![regex(r"((?:1[012]|0?\d))([0-5]\d)"), regex("(s(á)ng|chi(ề)u|t(ố)i)")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(time_of_day_ampm(to_lower(&rx(ts[1], 0)?) == "sáng", &hour_minute(true, h, m)))
        },
    )
}

fn rule_ddmmyyyy() -> Rule {
    rule(
        "dd/mm/yyyy",
        vec![regex(r"(3[01]|[12]\d|0?[1-9])[-/](0?[1-9]|1[0-2])[/-](\d{2,4})")],
        |ts| {
            let y = parse_int(&rx(ts[0], 2)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let d = parse_int(&rx(ts[0], 0)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn rule_ging_sinh() -> Rule {
    rule("giáng sinh", vec![regex("(ngày )?(xmas|christmas|giáng sinh)")], |_| tt(month_day(12, 25)))
}

fn rule_ngy_hm_kia() -> Rule {
    rule("ngày hôm kia", vec![regex("(ngày )?hôm kia")], |_| tt(cycle_nth(G::Day, -2)))
}

fn rule_dayofweek_ti() -> Rule {
    rule("<day-of-week> tới", vec![pred(is_a_day_of_week), regex("t(ớ)i")], |ts| {
        tt(pred_nth(0, true, td(ts[0])?))
    })
}

fn rule_hhmmss() -> Rule {
    rule("hh:mm:ss", vec![regex(r"((?:[01]?\d)|(?:2[0-3]))[:.]([0-5]\d)[:.]([0-5]\d)")], |ts| {
        let h = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        let s = parse_int(&rx(ts[0], 2)?)?;
        tt(hour_minute_second(true, h, m, s))
    })
}

fn days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Monday", "th(ứ) (2|hai)"),
        ("Tuesday", "th(ứ) (3|ba)"),
        ("Wednesday", "th(ứ) 4|th(ứ) b(ố)n|th(ứ) t(ư)"),
        ("Thursday", "th(ứ) (5|n(ă)m)"),
        ("Friday", "th(ứ) 6|th(ứ) s(á)u"),
        ("Saturday", "th(ứ) (7|b((ả)|(ẩ))y)"),
        ("Sunday", "ch((ủ)|(ú)a) nh(ậ)t"),
    ])
}

fn months() -> Vec<Rule> {
    mk_rule_months(&[
        ("January", "th(á)ng (gi(ê)ng|m(ộ)t)"),
        ("February", "th(á)ng hai"),
        ("March", "th(á)ng ba"),
        ("April", "th(á)ng t(ư)|th(á)ng b(ố)n"),
        ("May", "th(á)ng n(ă)m"),
        ("June", "th(á)ng s(á)u"),
        ("July", "th(á)ng b(ả)y"),
        ("August", "th(á)ng t(á)m"),
        ("September", "th(á)ng ch(í)n"),
        ("October", "th(á)ng m(ư)(ờ)i"),
        ("November", "th(á)ng m(ư)(ờ)i m(ộ)t"),
        ("December", "th(á)ng m(ư)(ờ)i hai"),
    ])
}

fn build_rules() -> Vec<Rule> {
    let mut v = vec![
        rule_about_timeofday(),
        rule_after_lunch(),
        rule_after_work(),
        rule_afternoon(),
        rule_at_hhmm(),
        rule_by_gi(),
        rule_cch_mng_thng(),
        rule_cui_nm(),
        rule_cui_thng(),
        rule_cycle_cui_cng_ca_time(),
        rule_dayofmonth_namedmonth(),
        rule_dayofmonth_namedmonth2(),
        rule_dayofweek_cui_cng_ca_time(),
        rule_dayofweek_ti(),
        rule_ddmm(),
        rule_ddmmyyyy(),
        rule_exactly_timeofday(),
        rule_ging_sinh(),
        rule_hhmm(),
        rule_hhmm_military_ampm(),
        rule_hhmm_military_sngchiuti(),
        rule_hhmmss(),
        rule_hm_nay(),
        rule_hourofday_half(),
        rule_hourofday_integer(),
        rule_hourofday_integer_pht(),
        rule_hourofday_quarter(),
        rule_intersect(),
        rule_intersect_by_of_from_s(),
        rule_l_tnh_nhn(),
        rule_last_cycle(),
        rule_last_n_cycle(),
        rule_lunch(),
        rule_month_numeric_with_month_symbol(),
        rule_morning(),
        rule_next_cycle(),
        rule_next_n_cycle(),
        rule_ngy_ddmm(),
        rule_ngy_ddmmyyyy(),
        rule_ngy_hm_kia(),
        rule_ngy_hm_qua(),
        rule_ngy_mai(),
        rule_noon(),
        rule_noon2(),
        rule_ordinal_cycle_of_time(),
        rule_partofday_hm_nay(),
        rule_partofday_time(),
        rule_quarter_number(),
        rule_quarter_number_of_year(),
        rule_quarter_number_year(),
        rule_quc_khnh(),
        rule_quc_t_lao_ng(),
        rule_season(),
        rule_season2(),
        rule_season3(),
        rule_season4(),
        rule_this_cycle(),
        rule_time_ny(),
        rule_time_ti(),
        rule_time_trc(),
        rule_timeofday_ampm(),
        rule_timeofday_approximately(),
        rule_timeofday_gi(),
        rule_timeofday_gi_ng(),
        rule_timeofday_latent(),
        rule_timeofday_latent2(),
        rule_timeofday_sharp(),
        rule_timeofday_sngchiuti(),
        rule_timezone(),
        rule_tonight(),
        rule_tt_dng(),
        rule_weekend(),
        rule_year_numeric_with_year_symbol(),
        rule_yyyymmdd(),
    ];
    v.extend(days_of_week());
    v.extend(months());
    v
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &RULES
}
