//! `Duckling.Time.IT.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::DurationData;
use crate::dims::time::helpers::*;
use crate::dims::time::types::{
    Form, IntervalDirection, TimeData,
    TimeIntervalType::{Closed, Open},
};
use crate::dims::time_grain::Grain as G;
use crate::dims::util::{parse_int, to_lower, to_upper};
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

use super::holidays;

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

fn tod(t: &Token) -> Option<(i64, bool)> {
    match t {
        Token::Time(TimeData { form: Some(Form::TimeOfDay { hours: Some(h), is12h }), .. }) => {
            Some((*h, *is12h))
        }
        _ => None,
    }
}

fn month_of(t: &Token) -> Option<i64> {
    match t {
        Token::Time(TimeData { form: Some(Form::Month(m)), .. }) => Some(*m),
        _ => None,
    }
}

fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn hod_not_latent(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

/// `if hours > 12 then hours else hours + 12`
fn pm_h(hours: i64) -> i64 {
    if hours > 12 {
        hours
    } else {
        hours + 12
    }
}

fn latent_pod(from: i64, to: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, from), &hour(false, to)).map(|x| mk_latent(part_of_day(x))))
}

fn season_iv(a_m: i64, a_d: i64, b_m: i64, b_d: i64) -> Option<Token> {
    ttd(interval(Open, &month_day(a_m, a_d), &month_day(b_m, b_d)))
}

fn intersect_night(base: &TimeData) -> Option<Token> {
    let td1 = cycle_nth_after(false, G::Day, 1, base);
    let td2 = interval(Open, &hour(false, 0), &hour(false, 4))?;
    ttd(intersect(&td1, &td2))
}

const TZ: &str = "YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT";

fn rule_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Lunedi", "luned(i|ì)|lun?\\.?" ),
        ("Martedi", "marted(i|ì)|mar\\.?" ),
        ("Mercoledi", "mercoled(i|ì)|mer\\.?" ),
        ("Giovedi", "gioved(i|ì)|gio\\.?" ),
        ("Venerdi", "venerd(i|ì)|ven\\.?" ),
        ("Sabato", "sabato|sab\\.?" ),
        ("Domenica", "domenica|dom\\.?" ),
    ])
}

fn rule_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("Gennaio", "gennaio|genn?\\.?" ),
        ("Febbraio", "febbraio|febb?\\.?" ),
        ("Marzo", "marzo|mar\\.?" ),
        ("Aprile", "aprile|apr\\.?" ),
        ("Maggio", "maggio|magg?\\.?" ),
        ("Giugno", "giugno|giu\\.?" ),
        ("Luglio", "luglio|lug\\.?" ),
        ("Agosto", "agosto|ago\\.?" ),
        ("Settembre", "settembre|sett?\\.?" ),
        ("Ottobre", "ottobre|ott\\.?" ),
        ("Novembre", "novembre|nov\\.?" ),
        ("Dicembre", "dicembre|dic\\.?" ),
    ])
}

fn build_rules() -> Vec<Rule> {
    let mut v = vec![
        // ruleAfternoon
        rule("afternoon", vec![regex("pomeriggio?")], |_| latent_pod(12, 19)),
        // ruleAtTimeofday
        rule(
            "at <time-of-day>",
            vec![regex("(al)?l[e']|a"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleChristmas
        rule("christmas", vec![regex("((il )?giorno di )?natale")], |_| tt(month_day(12, 25))),
        // ruleChristmasEve
        rule("christmas eve", vec![regex("((al)?la )?vigig?lia( di natale)?")], |_| {
            tt(month_day(12, 24))
        }),
        // ruleCircaPerLeTimeofday
        rule(
            "circa per le <time-of-day>",
            vec![regex("(circa )?per( le)?|circa( alle)?|verso( le)?"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleCommemorazioneDeiDefunti
        rule(
            "commemorazione dei defunti",
            vec![regex("(((giorno|commemorazione) dei|ai) )?(morti|defunti)")],
            |_| tt(month_day(11, 2)),
        ),
        // ruleDalDatetimeAlDatetimeInterval
        rule(
            "dal <datetime> al <datetime> (interval)",
            vec![
                regex("da(l(l')?)?"),
                pred(is_not_latent),
                regex("([fs]ino )?a(l(l')?)?"),
                pred(is_not_latent),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleDalleTimeofdayAlleTimeofdayInterval
        rule(
            "dalle <time-of-day> alle <time-of-day> (interval)",
            vec![
                regex("da(ll[ae'])?"),
                pred(is_a_time_of_day),
                regex("\\-|([fs]ino )?a(ll[ae'])?"),
                pred(is_a_time_of_day),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleDatetimeDatetimeInterval
        rule(
            "<datetime> - <datetime> (interval)",
            vec![pred(is_not_latent), regex("\\-|[fs]ino a(l(l[e'])?)?"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleDayOfMonthSt
        rule("day of month (1st)", vec![regex("(primo|1o|1º|1°)")], |_| tt(day_of_month(1))),
        // ruleDayofmonthNamedmonth
        rule(
            "<day-of-month> <named-month>",
            vec![pred(is_dom_integer), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
        ),
        // ruleDdddMonthInterval
        rule(
            "dd-dd <month> (interval)",
            vec![
                regex("(3[01]|[12]\\d|0?[1-9])"),
                regex("\\-"),
                regex("(3[01]|[12]\\d|0?[1-9])"),
                pred(is_a_month),
            ],
            |ts| {
                let v1 = parse_int(&rx(ts[0], 0)?)?;
                let v2 = parse_int(&rx(ts[2], 0)?)?;
                let t = td(ts[3])?;
                let from = intersect(&day_of_month(v1), t)?;
                let to = intersect(&day_of_month(v2), t)?;
                ttd(interval(Closed, &from, &to))
            },
        ),
        // ruleDdmm
        rule(
            "dd[/-]mm",
            vec![regex("(3[01]|[12]\\d|0?[1-9])[/-](0?[1-9]|1[0-2])")],
            |ts| {
                let d = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(month_day(m, d))
            },
        ),
        // ruleDdmmyyyy
        rule(
            "dd[/-]mm[/-]yyyy",
            vec![regex("(3[01]|[12]\\d|0?[1-9])[/-](0?[1-9]|1[0-2])[/-](\\d{2,4})")],
            |ts| {
                let d = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let y = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        // ruleDimTimeAlPartofday
        rule(
            "<dim time> al <part-of-day>",
            vec![dimension(Dim::Time), regex("al|nel(la)?|in|d(i|el(la)?)"), pred(is_a_part_of_day)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleDimTimeDelMattino
        rule(
            "<dim time> del mattino",
            vec![pred(is_a_time_of_day), regex("del mattino")],
            |ts| {
                let td2 = mk_latent(part_of_day(interval(Open, &hour(false, 0), &hour(false, 12))?));
                ttd(intersect(td(ts[0])?, &td2))
            },
        ),
        // ruleDimTimePartofday
        rule(
            "<dim time> <part-of-day>",
            vec![dimension(Dim::Time), pred(is_a_part_of_day)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[1])?)),
        ),
        // ruleDopoLeTimeofday
        rule(
            "dopo le <time-of-day>",
            vec![regex("dopo( l['ea'])?|dal(l['ea'])?"), pred(is_a_time_of_day)],
            |ts| tt(with_direction(IntervalDirection::After, not_latent(td(ts[1])?.clone()))),
        ),
        // ruleDurationAgo
        rule("<duration> ago", vec![dimension(Dim::Duration), regex("fa")], |ts| {
            tt(duration_ago(dur(ts[0])?))
        }),
        // ruleEntroIlDuration
        rule(
            "entro il <duration>",
            vec![
                regex("(entro|durante|per( tutt[ao])?) (il|l[a'])|in|nel(l[a'])?"),
                dimension(Dim::TimeGrain),
            ],
            |ts| ttd(interval(Open, &now(), &cycle_nth(grain_of(ts[1])?, 1))),
        ),
        // ruleEntroLeTimeofday
        rule(
            "entro le <time-of-day>",
            vec![regex("entro( l[ea'])?|prima d(i|ell['e])"), dimension(Dim::Time)],
            |ts| tt(with_direction(IntervalDirection::Before, not_latent(td(ts[1])?.clone()))),
        ),
        // ruleEomendOfMonth
        rule("EOM|End of month", vec![regex("fine del mese")], |_| tt(cycle_nth(G::Month, 1))),
        // ruleEoyendOfYear
        rule("EOY|End of year", vec![regex("fine dell' ?anno")], |_| tt(cycle_nth(G::Year, 1))),
        // ruleEpifania
        rule("epifania", vec![regex("(epifania|befana)")], |_| tt(month_day(1, 6))),
        // ruleEvening
        rule("evening", vec![regex("ser(ata|[ae])")], |_| latent_pod(18, 0)),
        // ruleFerragosto
        rule("ferragosto", vec![regex("ferragosto|assunzione")], |_| tt(month_day(8, 15))),
        // ruleFestaDelLavoro
        rule("festa del lavoro", vec![regex("festa del lavoro|(festa|giorno) dei lavoratori")], |_| {
            tt(month_day(5, 1))
        }),
        // ruleFestaDelPap
        rule(
            "festa del papà",
            vec![regex("festa del pap(a|à)|(festa di )?s(an|\\.) giuseppe")],
            |_| tt(month_day(3, 19)),
        ),
        // ruleFestaDellaLiberazione
        rule(
            "festa della liberazione",
            vec![regex("((festa|anniversario) della|(al)?la) liberazione")],
            |_| tt(month_day(4, 25)),
        ),
        // ruleFestaDellaRepubblica
        rule("festa della repubblica", vec![regex("((festa del)?la )?repubblica")], |_| {
            tt(month_day(6, 2))
        }),
        // ruleFinoAlDatetimeInterval
        rule(
            "fino al <datetime> (interval)",
            vec![regex("[fs]ino a(l(l[ae'])?)?"), pred(is_not_latent)],
            |ts| ttd(interval(Open, &now(), td(ts[1])?)),
        ),
        // ruleGliNUltimiCycle
        rule(
            "gli <n> ultimi <cycle>",
            vec![
                regex("([nd]e)?i|([nd]el)?le"),
                pred(is_integer_between_tok(2, 9999)),
                regex("(scors|ultim)[ei]"),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[3])?, -n))
            },
        ),
        // ruleGliUltimiNCycle
        rule(
            "gli ultimi <n> <cycle>",
            vec![
                regex("((([nd]el)?le|([nd]e)?gli) )?(scors|ultim)[ei]"),
                pred(is_integer_between_tok(2, 9999)),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -n))
            },
        ),
        // ruleHalloweenDay
        rule("halloween day", vec![regex("hall?owe?en")], |_| tt(month_day(10, 31))),
        // ruleHhRelativeminutesDelPomeriggiotimeofday
        rule(
            "hh <relative-minutes> del pomeriggio(time-of-day)",
            vec![
                pred(hod_not_latent),
                pred(is_integer_between_tok(1, 59)),
                regex("d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))"),
            ],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                let n = get_int_value(ts[1])?;
                tt(hour_minute(false, pm_h(hours), n))
            },
        ),
        // ruleHhRelativeminutesDelPomeriggiotimeofday2
        rule(
            "hh <relative-minutes> del pomeriggio(time-of-day)",
            vec![
                pred(is_an_hour_of_day),
                regex("e"),
                pred(is_integer_between_tok(1, 59)),
                regex("d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))"),
            ],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                let n = get_int_value(ts[2])?;
                tt(hour_minute(false, pm_h(hours), n))
            },
        ),
        // ruleHhhmmTimeofday
        rule(
            "hh(:|h)mm (time-of-day)",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:h]([0-5]\\d)")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(hour_minute(false, h, m))
            },
        ),
        // ruleHhhmmTimeofday2
        rule(
            "hh(:|h)mm (time-of-day)",
            vec![regex("((?:0?\\d)|(?:1[0-2]))[:h]([0-5]\\d) d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))")],
            |ts| {
                let v1 = parse_int(&rx(ts[0], 0)?)?;
                let v2 = parse_int(&rx(ts[0], 1)?)?;
                tt(hour_minute(false, v1 + 12, v2))
            },
        ),
        // ruleHhmmMilitaryTimeofday
        rule(
            "hhmm (military time-of-day)",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))([0-5]\\d)")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(mk_latent(hour_minute(false, h, m)))
            },
        ),
        // ruleHourofdayAndRelativeMinutes
        rule(
            "<hour-of-day> and <relative minutes>",
            vec![pred(is_an_hour_of_day), regex("e"), pred(is_integer_between_tok(1, 59))],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                let n = get_int_value(ts[2])?;
                tt(hour_minute(is12h, h, n))
            },
        ),
        // ruleHourofdayIntegerAsRelativeMinutes
        rule(
            "<hour-of-day> <integer> (as relative minutes)",
            vec![pred(hod_not_latent), pred(is_integer_between_tok(1, 59))],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                let n = get_int_value(ts[1])?;
                tt(hour_minute(is12h, h, n))
            },
        ),
        // ruleHourofdayMinusIntegerAsRelativeMinutes
        rule(
            "<hour-of-day> minus <integer> (as relative minutes)",
            vec![pred(is_an_hour_of_day), regex("meno"), pred(is_integer_between_tok(1, 59))],
            |ts| {
                let n = get_int_value(ts[2])?;
                ttd(minutes_before(n, td(ts[0])?))
            },
        ),
        // ruleIlCycleDopoTime
        rule(
            "il <cycle> dopo <time>",
            vec![regex("l[a']|il|[nd]el"), dimension(Dim::TimeGrain), regex("dopo"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, 1, td(ts[3])?)),
        ),
        // ruleIlDayofmonthDeNamedmonth
        rule(
            "il <day-of-month> <named-month>",
            vec![regex("il|l'"), pred(is_dom_integer), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[2])?, ts[1])),
        ),
        // ruleIlTime
        rule("il <time>", vec![regex("il|l'"), pred(is_not_latent)], |ts| Some(ts[1].clone())),
        // ruleIleNCyclePassatipassate
        rule(
            "i|le n <cycle> passati|passate",
            vec![
                regex("([nd]e)?i|([nd]el)?le"),
                pred(is_integer_between_tok(2, 9999)),
                dimension(Dim::TimeGrain),
                regex("(scors|passat)[ie]"),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -n))
            },
        ),
        // ruleInDuration
        rule(
            "in|entro <duration>",
            vec![regex("(in|entro)"), dimension(Dim::Duration)],
            |ts| {
                let m = to_lower(&rx(ts[0], 0)?);
                let dd = dur(ts[1])?;
                match m.as_str() {
                    "entro" => ttd(interval(Open, &now(), &in_duration(dd))),
                    "in" => tt(in_duration(dd)),
                    _ => None,
                }
            },
        ),
        // ruleInNamedmonth
        rule(
            "in <named-month>",
            vec![regex("in|del mese( di)?"), pred(is_a_month)],
            |ts| Some(ts[1].clone()),
        ),
        // ruleInThePartofdayOfDimTime
        rule(
            "in the <part-of-day> of <dim time>",
            vec![regex("nel(la)?"), pred(is_a_part_of_day), regex("d(i|el)"), dimension(Dim::Time)],
            |ts| ttd(intersect(td(ts[1])?, td(ts[3])?)),
        ),
        // ruleInafterDuration
        rule(
            "in/after <duration>",
            vec![regex("[tf]ra|in|dopo"), dimension(Dim::Duration)],
            |ts| tt(in_duration(dur(ts[1])?)),
        ),
        // ruleIntegerDelPartOfDay
        rule(
            "<integer 0 12> del <part of day>",
            vec![
                pred(is_integer_between_tok(0, 12)),
                regex("d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))"),
            ],
            |ts| {
                let Token::Numeral(n) = ts[0] else { return None };
                tt(hour(false, 12 + n.value.floor() as i64))
            },
        ),
        // ruleIntegerLatentTimeofday
        rule(
            "<integer> (latent time-of-day)",
            vec![pred(is_integer_between_tok(0, 24))],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(mk_latent(hour(false, n)))
            },
        ),
        // ruleIntersectByDiDellaDel
        rule(
            "intersect by \"di\", \"della\", \"del\"",
            vec![pred(is_not_latent), regex("di|del(la)?"), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleLastCycleOfTime
        rule(
            "last <cycle> of <time>",
            vec![
                regex("(l')ultim[oa]"),
                dimension(Dim::TimeGrain),
                regex("di|del(l[oa'])"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastDayofweekOfTime
        rule(
            "last <day-of-week> of <time>",
            vec![
                regex("(([nd]el)?l')?ultim[oa]"),
                pred(is_a_day_of_week),
                regex("di|del(l[a'])?"),
                dimension(Dim::Time),
            ],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastDayofweekOfTime2
        rule(
            "last <day-of-week> of <time>",
            vec![regex("(l')ultim[oa]"), pred(is_a_day_of_week), regex("di"), dimension(Dim::Time)],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastTime
        rule(
            "last <time> ",
            vec![regex("(ultim|scors|passat)[oaie]"), dimension(Dim::Time)],
            |ts| tt(pred_nth(-1, false, td(ts[1])?)),
        ),
        // ruleLeIdiDiNamedmonth
        rule(
            "le idi di <named-month>",
            vec![regex("(le )?idi di"), pred(is_a_month)],
            |ts| {
                let m = month_of(ts[1])?;
                let d = if matches!(m, 3 | 5 | 7 | 10) { 15 } else { 13 };
                ttd(intersect(&day_of_month(d), td(ts[1])?))
            },
        ),
        // ruleLunch
        rule("lunch", vec![regex("(a )?pranzo")], |_| latent_pod(12, 14)),
        // ruleMidnight
        rule("midnight", vec![regex("mez?zanott?e")], |_| tt(hour(false, 0))),
        // ruleMorning
        rule("morning", vec![regex("mattin(ata|[aoe])")], |_| latent_pod(4, 12)),
        // ruleMothersDay
        rule("Mother's Day", vec![regex("festa della mamma")], |_| tt(nth_dow_of_month(2, 7, 5))),
        // ruleNameddayDayofmonth
        rule(
            "<named-day> <day-of-month>",
            vec![pred(is_a_day_of_week), pred(is_dom_integer)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        // ruleNewYearsDay
        rule("new year's day", vec![regex("(capodanno|primo dell' ?anno)")], |_| tt(month_day(1, 1))),
        // ruleNewYearsEve
        rule("new year's eve", vec![regex("((la )?vigig?lia di capodanno|san silvestro)")], |_| {
            tt(month_day(12, 31))
        }),
        // ruleNextCycle
        rule(
            "next <cycle> ",
            vec![regex("((il|la|[nd]el(la)?) )?prossim[oa]"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 1)),
        ),
        // ruleNextNCycle
        rule(
            "next n <cycle>",
            vec![
                regex("((([nd]e)?i|([nd]el)?le) )?prossim[ei]"),
                pred(is_integer_between_tok(2, 9999)),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, n))
            },
        ),
        // ruleNextNCycle2
        rule(
            "next n <cycle>",
            vec![
                regex("([nd]e)?i|([nd]el)?le"),
                pred(is_integer_between_tok(2, 9999)),
                regex("prossim[ei]"),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[3])?, n))
            },
        ),
        // ruleNextTime
        rule(
            "next <time>",
            vec![regex("prossim[ao]"), dimension(Dim::Time)],
            |ts| tt(pred_nth(1, false, td(ts[1])?)),
        ),
        // ruleNextTime2
        rule(
            "next <time>",
            vec![dimension(Dim::Time), regex("dopo|prossim[ao]")],
            |ts| tt(pred_nth(1, false, td(ts[0])?)),
        ),
        // ruleNight
        rule("night", vec![regex("nott(e|ata)")], |_| {
            let td1 = cycle_nth(G::Day, 1);
            let td2 = interval(Open, &hour(false, 0), &hour(false, 4))?;
            ttd(intersect(&td1, &td2).map(|x| part_of_day(mk_latent(x))))
        }),
        // ruleNoon
        rule("noon", vec![regex("mezz?ogiorno")], |_| tt(hour(false, 12))),
        // ruleNthTimeAfterTime
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("dopo"), dimension(Dim::Time)],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(pred_nth_after(n - 1, td(ts[1])?, td(ts[3])?))
            },
        ),
        // ruleNthTimeAfterTime2
        rule(
            "nth <time> after <time>",
            vec![
                regex("il|l'"),
                dimension(Dim::Ordinal),
                dimension(Dim::Time),
                regex("dopo"),
                dimension(Dim::Time),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(pred_nth_after(n - 1, td(ts[2])?, td(ts[4])?))
            },
        ),
        // ruleNthTimeOfTime
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("di|del(l[a'])?"), dimension(Dim::Time)],
            |ts| {
                let n = get_int_value(ts[0])?;
                ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(n - 1, false, &x)))
            },
        ),
        // ruleNthTimeOfTime2
        rule(
            "nth <time> of <time>",
            vec![
                dimension(Dim::Ordinal),
                dimension(Dim::Time),
                regex("di|del(l[oa'])|in"),
                dimension(Dim::Time),
            ],
            |ts| {
                let n = get_int_value(ts[0])?;
                ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(n - 1, false, &x)))
            },
        ),
        // ruleNthTimeOfTime3
        rule(
            "nth <time> of <time>",
            vec![
                regex("il|l'"),
                dimension(Dim::Ordinal),
                dimension(Dim::Time),
                regex("di|del(l[oa'])|in"),
                dimension(Dim::Time),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                ttd(intersect(td(ts[4])?, td(ts[2])?).map(|x| pred_nth(n - 1, false, &x)))
            },
        ),
        // ruleOgnissanti
        rule("ognissanti", vec![regex("(tutti i |ognis|festa dei |([ia]l )?giorno dei )santi")], |_| {
            tt(month_day(11, 1))
        }),
        // ruleOrdinalCycleOfTime
        rule(
            "<ordinal> <cycle> of <time>",
            vec![
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("di|del(l[a'])?"),
                dimension(Dim::Time),
            ],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(cycle_nth_after(true, grain_of(ts[1])?, n - 1, td(ts[3])?))
            },
        ),
        // rulePartofdayOfDimTime
        rule(
            "<part-of-day> of <dim time>",
            vec![pred(is_a_part_of_day), regex("d(i|el)"), dimension(Dim::Time)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleProssimiUnitofduration
        rule(
            "prossimi <unit-of-duration>",
            vec![regex("((([nd]e)?i|([nd]el)?le) )?prossim[ie]"), dimension(Dim::TimeGrain)],
            |ts| {
                let g = grain_of(ts[1])?;
                ttd(interval(Closed, &cycle_nth(g, 1), &cycle_nth(g, 3)))
            },
        ),
        // ruleRelativeMinutesToIntegerAsHourofday
        rule(
            "relative minutes to <integer> (as hour-of-day)",
            vec![pred(is_integer_between_tok(1, 59)), regex("a"), pred(is_an_hour_of_day)],
            |ts| {
                let n = get_int_value(ts[0])?;
                ttd(minutes_before(n, td(ts[2])?))
            },
        ),
        // ruleRightNow
        rule(
            "right now",
            vec![regex("(subito|(immediata|attual)mente|(proprio )?adesso|(in questo|al) momento|ora)")],
            |_| tt(now()),
        ),
        // ruleSantoStefano
        rule("santo stefano", vec![regex("s(anto|\\.) stefano")], |_| tt(month_day(12, 26))),
        // ruleSeason — estate
        rule("season", vec![regex("(in )?estate")], |_| season_iv(6, 21, 9, 23)),
        // ruleSeason2 — autunno
        rule("season", vec![regex("(in )?autunno")], |_| season_iv(9, 23, 12, 21)),
        // ruleSeason3 — inverno
        rule("season", vec![regex("(in )?inverno")], |_| season_iv(12, 21, 3, 20)),
        // ruleSeason4 — primavera
        rule("season", vec![regex("(in )?primavera")], |_| season_iv(3, 20, 6, 21)),
        // ruleStamattina
        rule("stamattina", vec![regex("stamattina")], |_| {
            let td2 = interval(Open, &hour(false, 4), &hour(false, 12))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleStanotte
        rule("stanotte", vec![regex("(sta|nella )notte|(in|nella) nottata")], |_| {
            let td1 = cycle_nth(G::Day, 1);
            let td2 = interval(Open, &hour(false, 0), &hour(false, 4))?;
            ttd(intersect(&td1, &td2).map(part_of_day))
        }),
        // ruleStasera
        rule("stasera", vec![regex("stasera")], |_| {
            let td2 = interval(Open, &hour(false, 18), &hour(false, 0))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleTheCycleLast
        rule(
            "the <cycle> last",
            vec![regex("l'|il|la|[nd]el(la)?"), dimension(Dim::TimeGrain), regex("(scors|passat)[oa]")],
            |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
        ),
        // ruleTheCycleNext
        rule(
            "the <cycle> next",
            vec![regex("l'|il|la|[nd]el(la)?"), dimension(Dim::TimeGrain), regex("prossim[oa]")],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 1)),
        ),
        // ruleTheCycleOfTime
        rule(
            "the <cycle> of <time>",
            vec![regex("il|la"), dimension(Dim::TimeGrain), regex("del"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, 0, td(ts[3])?)),
        ),
        // ruleTheDayAfterTomorrow
        rule("the day after tomorrow", vec![regex("(il giorno )?dopo\\s?domani")], |_| {
            tt(cycle_nth(G::Day, 2))
        }),
        // ruleTheDayBeforeYesterday
        rule("the day before yesterday", vec![regex("(l')?altro\\s?ieri")], |_| {
            tt(cycle_nth(G::Day, -2))
        }),
        // ruleTheDayofmonth
        rule(
            "il <day-of-month>",
            vec![regex("il|l'"), pred(is_dom_integer)],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(day_of_month(n))
            },
        ),
        // ruleTheLastCycle
        rule(
            "the last <cycle>",
            vec![
                regex("(([nd]el)?l' ?ultim|(il|la) passat|([nd]el)?l[ao] scors)[oa]"),
                dimension(Dim::TimeGrain),
            ],
            |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
        ),
        // ruleTheNthTimeOfTime
        rule(
            "the nth <time> of <time>",
            vec![
                regex("il|l[a']|[nd]el(l[a'])?"),
                dimension(Dim::Ordinal),
                dimension(Dim::Time),
                regex("di|del(l[a'])?"),
                dimension(Dim::Time),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                ttd(intersect(td(ts[4])?, td(ts[2])?).map(|x| pred_nth(n - 1, false, &x)))
            },
        ),
        // ruleTheOrdinalCycleOfTime
        rule(
            "the <ordinal> <cycle> of <time>",
            vec![
                regex("il|l[a']|[nd]el(l[a'])?"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("di|del(l[a'])?"),
                dimension(Dim::Time),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_nth_after(true, grain_of(ts[2])?, n - 1, td(ts[4])?))
            },
        ),
        // ruleThisCycle
        rule(
            "this <cycle>",
            vec![regex("(in )?quest['oa]|per (il|l['a])"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
        ),
        // ruleThisDayofweek
        rule(
            "this <day-of-week>",
            vec![regex("quest[oaie]"), pred(is_a_day_of_week)],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        // ruleThisPartofday
        rule(
            "this <part-of-day>",
            vec![regex("(que)?st[oa]|i[nl]|(al|nel)(la)?|la"), pred(is_a_part_of_day)],
            |ts| ttd(intersect(&today(), td(ts[1])?).map(part_of_day)),
        ),
        // ruleThisTime
        rule(
            "this <time>",
            vec![regex("quest[oaie']"), dimension(Dim::Time)],
            |ts| tt(pred_nth(0, false, td(ts[1])?)),
        ),
        // ruleTimeLast
        rule(
            "<time> last",
            vec![dimension(Dim::Time), regex("(ultim|scors|passat)[oaie]")],
            |ts| tt(pred_nth(-1, false, td(ts[0])?)),
        ),
        // ruleTimeNotte
        rule(
            "<time> notte",
            vec![dimension(Dim::Time), regex("((in|nella|alla) )?nott(e|ata)")],
            |ts| intersect_night(td(ts[0])?),
        ),
        // ruleTimeNotte2
        rule(
            "<time> notte",
            vec![regex("((nella|alla) )?nott(e|ata)( d(i|el))"), dimension(Dim::Time)],
            |ts| intersect_night(td(ts[1])?),
        ),
        // ruleTimeofdayCirca
        rule(
            "<time-of-day> circa",
            vec![pred(is_a_time_of_day), regex("circa")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        // ruleTimeofdayOra
        rule(
            "<time-of-day> ora",
            vec![regex("or[ea]"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleTimeofdayPrecise
        rule(
            "<time-of-day> precise",
            vec![pred(is_a_time_of_day), regex("precise")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        // ruleTimeofdayTimeofdayDayofmonthInterval
        rule(
            "<time-of-day> - <time-of-day> <day-of-month> (interval)",
            vec![pred(is_a_time_of_day), regex("\\-"), pred(is_a_time_of_day), pred(is_not_latent)],
            |ts| {
                let from = intersect(td(ts[0])?, td(ts[3])?)?;
                let to = intersect(td(ts[2])?, td(ts[3])?)?;
                ttd(interval(Closed, &from, &to))
            },
        ),
        // ruleTomorrow
        rule("tomorrow", vec![regex("domani")], |_| tt(cycle_nth(G::Day, 1))),
        // ruleTraIlDatetimeEIlDatetimeInterval
        rule(
            "tra il <datetime> e il <datetime> (interval)",
            vec![
                regex("tra( il| l')?"),
                pred(is_a_time_of_day),
                regex("e( il| l')?"),
                pred(is_a_time_of_day),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleTwoTimeTokensInARow
        rule(
            "two time tokens in a row",
            vec![pred(is_not_latent), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[1])?)),
        ),
        // ruleTwoTimeTokensSeparatedByDi
        rule(
            "two time tokens separated by `di`",
            vec![pred(is_not_latent), regex("di"), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleUna
        rule("una", vec![regex("una")], |_| tt(mk_latent(hour(true, 1)))),
        // ruleValentinesDay
        rule("valentine's day", vec![regex("san valentino|festa degli innamorati")], |_| {
            tt(month_day(2, 14))
        }),
        // ruleWeekend
        rule("week-end", vec![regex("week[ -]?end|fine ?settimana|we")], |_| tt(weekend())),
        // ruleYearLatent
        rule(
            "year (latent)",
            vec![pred(is_integer_between_tok(2101, 10000))],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(mk_latent(year(n)))
            },
        ),
        // ruleYearNotLatent
        rule(
            "year (1000-2100 not latent)",
            vec![pred(is_integer_between_tok(1000, 2100))],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(year(n))
            },
        ),
        // ruleYesterday
        rule("yesterday", vec![regex("ieri")], |_| tt(cycle_nth(G::Day, -1))),
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
        // ruleHhThreeQuarterDelPomeriggiotimeofday
        rule(
            "hh three quarter del pomeriggio(time-of-day)",
            vec![
                pred(is_an_hour_of_day),
                regex("(3|tre) quarti? d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))"),
            ],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                tt(hour_minute(false, pm_h(hours), 45))
            },
        ),
        // ruleHhThreeQuarterDelPomeriggiotimeofday2
        rule(
            "hh three quarters del pomeriggio(time-of-day)",
            vec![
                pred(is_an_hour_of_day),
                regex("e (3|tre) quarti? d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))"),
            ],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                tt(hour_minute(false, pm_h(hours), 45))
            },
        ),
        // ruleHourofdayMinusThreeQuarter
        rule(
            "<hour-of-day> minus three quarter",
            vec![pred(is_an_hour_of_day), regex("meno (3|tre) quarti?")],
            |ts| ttd(minutes_before(45, td(ts[0])?)),
        ),
        // ruleThreeQuarterToIntegerAsHourofday
        rule(
            "three quarter to <integer> (as hour-of-day)",
            vec![regex("(3|tre) quarti? a"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(45, td(ts[1])?)),
        ),
        // ruleHourofdayAndThreeQuarter
        rule(
            "<hour-of-day> and three quarters",
            vec![pred(is_an_hour_of_day), regex("e (3|tre) quarti?")],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                tt(hour_minute(is12h, h, 45))
            },
        ),
        // ruleHourofdayThreeQuarters
        rule(
            "<hour-of-day> three quarters",
            vec![pred(is_an_hour_of_day), regex("(3|tre) quarti?")],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                tt(hour_minute(is12h, h, 45))
            },
        ),
        // ruleHhQuartDelPomeriggiotimeofday
        rule(
            "hh quart del pomeriggio(time-of-day)",
            vec![
                pred(is_an_hour_of_day),
                regex("un quarto d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))"),
            ],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                tt(hour_minute(false, pm_h(hours), 15))
            },
        ),
        // ruleHhQuartDelPomeriggiotimeofday2
        rule(
            "hh quart del pomeriggio(time-of-day)",
            vec![
                pred(is_an_hour_of_day),
                regex("e un quarto d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))"),
            ],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                tt(hour_minute(false, pm_h(hours), 15))
            },
        ),
        // ruleHourofdayMinusQuart
        rule(
            "<hour-of-day> minus quart",
            vec![pred(is_an_hour_of_day), regex("meno un quarto")],
            |ts| ttd(minutes_before(15, td(ts[0])?)),
        ),
        // ruleQuartAsHourofday
        rule(
            "a quart to <integer> (as hour-of-day)",
            vec![regex("un quarto a"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(15, td(ts[1])?)),
        ),
        // ruleHourofdayAndAQuart
        rule(
            "<hour-of-day> and a quart",
            vec![pred(is_an_hour_of_day), regex("e un quarto")],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                tt(hour_minute(is12h, h, 15))
            },
        ),
        // ruleHourofdayQuart
        rule(
            "<hour-of-day> quart",
            vec![pred(is_an_hour_of_day), regex("un quarto")],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                tt(hour_minute(is12h, h, 15))
            },
        ),
        // ruleHhHalfDelPomeriggiotimeofday
        rule(
            "hh half del pomeriggio(time-of-day)",
            vec![pred(is_an_hour_of_day), regex("mezzo d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))")],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                tt(hour_minute(false, pm_h(hours), 30))
            },
        ),
        // ruleHhHalfDelPomeriggiotimeofday2
        rule(
            "hh half del pomeriggio(time-of-day)",
            vec![
                pred(is_an_hour_of_day),
                regex("e mezzo d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))"),
            ],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                tt(hour_minute(false, pm_h(hours), 30))
            },
        ),
        // ruleHourofdayMinusHalf
        rule(
            "<hour-of-day> minus half",
            vec![pred(is_an_hour_of_day), regex("meno mezzo")],
            |ts| ttd(minutes_before(30, td(ts[0])?)),
        ),
        // ruleHalfToIntegerAsHourofday
        rule(
            "half to <integer> (as hour-of-day)",
            vec![regex("mezzo a"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(30, td(ts[1])?)),
        ),
        // ruleHourofdayAndHalf
        rule(
            "<hour-of-day> and half",
            vec![pred(is_an_hour_of_day), regex("e mezzo")],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                tt(hour_minute(is12h, h, 30))
            },
        ),
        // ruleHourofdayHalf
        rule(
            "<hour-of-day> half",
            vec![pred(is_an_hour_of_day), regex("mezzo")],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                tt(hour_minute(is12h, h, 30))
            },
        ),
        // ruleHhIntegerminutesDelPomeriggiotimeofday
        rule(
            "hh <integer> minutes del pomeriggio(time-of-day)",
            vec![
                pred(is_an_hour_of_day),
                pred(is_integer_between_tok(1, 59)),
                regex("min(ut[oi]|\\.)? d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))"),
            ],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                let n = get_int_value(ts[1])?;
                tt(hour_minute(false, pm_h(hours), n))
            },
        ),
        // ruleHhIntegerminutesDelPomeriggiotimeofday2
        rule(
            "hh <relative-minutes> minutes del pomeriggio(time-of-day)",
            vec![
                pred(is_an_hour_of_day),
                regex("e"),
                pred(is_integer_between_tok(1, 59)),
                regex("min(ut[oi]|\\.)? d(i|el(la)?) (pomeriggio|(sta)?(sera|notte))"),
            ],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                let n = get_int_value(ts[2])?;
                tt(hour_minute(false, pm_h(hours), n))
            },
        ),
        // ruleHourofdayMinusIntegerMinutes
        rule(
            "<hour-of-day> minus <integer> minutes",
            vec![
                pred(is_an_hour_of_day),
                regex("meno"),
                pred(is_integer_between_tok(1, 59)),
                regex("min(ut[oi]|\\.)?"),
            ],
            |ts| {
                let n = get_int_value(ts[2])?;
                ttd(minutes_before(n, td(ts[0])?))
            },
        ),
        // ruleMinutesToIntegerAsHourofday
        rule(
            "minutes to <integer> (as hour-of-day)",
            vec![pred(is_integer_between_tok(1, 59)), regex("min(ut[oi]|\\.)? a"), pred(is_an_hour_of_day)],
            |ts| {
                let n = get_int_value(ts[0])?;
                ttd(minutes_before(n, td(ts[2])?))
            },
        ),
        // ruleHourofdayAndIntegerMinutes
        rule(
            "<hour-of-day> and <integer> minutes",
            vec![
                pred(is_an_hour_of_day),
                regex("e"),
                pred(is_integer_between_tok(1, 59)),
                regex("min(ut[oi]|\\.)?"),
            ],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                let n = get_int_value(ts[2])?;
                tt(hour_minute(is12h, h, n))
            },
        ),
        // ruleHourofdayIntegerMinutes
        rule(
            "<hour-of-day> <integer> minutes",
            vec![pred(hod_not_latent), pred(is_integer_between_tok(1, 59)), regex("min(ut[oi]|\\.)?")],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                let n = get_int_value(ts[1])?;
                tt(hour_minute(is12h, h, n))
            },
        ),
        // ruleTimezone
        rule(
            "<time> timezone",
            vec![
                pred(|t| is_not_latent(t) && is_a_time_of_day(t)),
                regex(&format!("\\b({TZ})\\b")),
            ],
            |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
        ),
        // ruleTimeEntroLeTime
        rule(
            "<time> entro le <time>",
            vec![
                dimension(Dim::Time),
                regex("entro( l[e'])?|prima d(i|ell['ea])"),
                pred(is_a_time_of_day),
            ],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?).map(|x| with_direction(IntervalDirection::Before, x))),
        ),
        // ruleImmacolataConcezione
        rule("immacolata concezione", vec![regex("(all')?immacolata( concezione)?")], |_| {
            tt(month_day(12, 8))
        }),
        // ruleOrdinalQuarter
        rule(
            "<ordinal> quarter",
            vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(cycle_nth_after(true, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
            },
        ),
        // ruleTheOrdinalQuarter
        rule(
            "the <ordinal> quarter",
            vec![regex("il|[nd]el(l')?|l'"), dimension(Dim::Ordinal), pred(is_grain(G::Quarter))],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_nth_after(true, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
            },
        ),
        // ruleCycleOrdinalQuarterYear
        rule(
            "<ordinal> quarter <year>",
            vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter)), dimension(Dim::Time)],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(cycle_nth_after(false, G::Quarter, n - 1, td(ts[2])?))
            },
        ),
        // ruleCycleTheOrdinalTime
        rule(
            "the <ordinal> <cycle> <time>",
            vec![
                regex("il|[nd]el(l')?|l'"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                dimension(Dim::Time),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_nth_after(true, grain_of(ts[2])?, n - 1, td(ts[3])?))
            },
        ),
        // ruleCycleNext
        rule(
            "<cycle> next",
            vec![dimension(Dim::TimeGrain), regex("prossim[oa]")],
            |ts| tt(cycle_nth(grain_of(ts[0])?, 1)),
        ),
        // ruleDomattina
        rule("domattina", vec![regex("domattina")], |_| {
            let td1 = cycle_nth(G::Day, 1);
            let td2 = interval(Open, &hour(false, 4), &hour(false, 12))?;
            ttd(intersect(&td1, &td2).map(part_of_day))
        }),
        // ruleVersoPartOfDay
        rule(
            "verso <part-of-day>",
            vec![regex("verso( la| il)?"), pred(is_a_part_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleEntroIlInt
        rule(
            "entro il <integer>",
            vec![regex("entro il|prima del"), pred(is_dom_integer)],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(with_direction(IntervalDirection::Before, day_of_month(n)))
            },
        ),
        // ruleEntroTime
        rule(
            "entro <time>",
            vec![regex("entro( la)?|prima d(i|ella)"), dimension(Dim::Time)],
            |ts| tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone())),
        ),
        // ruleDalInt
        rule(
            "dal <integer",
            vec![regex("dal"), pred(is_dom_integer)],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(with_direction(IntervalDirection::After, day_of_month(n)))
            },
        ),
        // ruleDopoTime
        rule(
            "dopo <time>",
            vec![regex("dopo|dal?"), dimension(Dim::Time)],
            |ts| tt(with_direction(IntervalDirection::After, td(ts[1])?.clone())),
        ),
        // ruleToday
        rule("today", vec![regex("(di )?oggi|in giornata")], |_| tt(today())),
        // ruleDalIntAlInt
        rule(
            "dal <integer> al <integer> (interval)",
            vec![
                regex("dal(?:l')?"),
                pred(is_dom_integer),
                regex("([fs]ino )?al(l')?"),
                pred(is_dom_integer),
            ],
            |ts| {
                let v1 = get_int_value(ts[1])?;
                let v2 = get_int_value(ts[3])?;
                ttd(interval(Closed, &day_of_month(v1), &day_of_month(v2)))
            },
        ),
        // ruleTraIlIntEIlInt
        rule(
            "tra il <integer> e il <integer> (interval)",
            vec![
                regex("tra( (il|l'))?"),
                pred(is_dom_integer),
                regex("e( (il|l'))?"),
                pred(is_dom_integer),
            ],
            |ts| {
                let v1 = get_int_value(ts[1])?;
                let v2 = get_int_value(ts[3])?;
                ttd(interval(Closed, &day_of_month(v1), &day_of_month(v2)))
            },
        ),
        // ruleIlWeekendDelTime
        rule(
            "il week-end del <time>",
            vec![regex("il (week[ -]?end|fine ?settimana|we) del"), pred(is_grain_of_time(G::Day))],
            |ts| {
                let t = td(ts[1])?;
                let from1 = intersect(&cycle_nth_after(false, G::Week, 0, t), &day_of_week(5))?;
                let from = intersect(&from1, &hour(false, 18))?;
                let to1 = intersect(&cycle_nth_after(false, G::Week, 1, t), &day_of_week(1))?;
                let to = intersect(&to1, &hour(false, 0))?;
                ttd(interval(Open, &from, &to))
            },
        ),
    ];
    v.extend(rule_days_of_week());
    v.extend(rule_months());
    v.extend(holidays::rules());
    v
}

static IT_RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &IT_RULES
}
