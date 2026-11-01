//! `Duckling.Time.NB.Rules`.

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

/// `isGrain TG.X` — a TimeGrain token, not a Time token of that grain.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn not_latent_hour(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

fn not_latent_tod(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t)
}

fn tod_hours(t: &Token) -> Option<i64> {
    match t {
        Token::Time(TimeData { form: Some(Form::TimeOfDay { hours: Some(h), .. }), .. }) => Some(*h),
        _ => None,
    }
}

fn month_num(t: &Token) -> Option<i64> {
    match t {
        Token::Time(TimeData { form: Some(Form::Month(m)), .. }) => Some(*m),
        _ => None,
    }
}

fn latent_pod(from: i64, to: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, from), &hour(false, to)).map(|x| mk_latent(part_of_day(x))))
}

fn season_interval(from: TimeData, to: TimeData) -> Option<Token> {
    ttd(interval(Open, &from, &to))
}

const TIMEZONE_NAME: &str = "YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT";

fn rules_main() -> Vec<Rule> {
    vec![
        // ruleADuration
        rule("a <duration>", vec![regex("(om )?en|ett|én|et"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        // ruleAboutTimeofday
        rule(
            "about <time-of-day>",
            vec![regex("(omkring|cirka|ca\\.)( kl\\.| klokken | klokka)?"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleAbsorptionOfAfterNamedDay
        rule("absorption of , after named day", vec![pred(is_a_day_of_week), regex(",")], |ts| {
            Some(ts[0].clone())
        }),
        // ruleAfterDuration
        rule("after <duration>", vec![regex("etter"), dimension(Dim::Duration)], |ts| {
            tt(with_direction(IntervalDirection::After, in_duration(dur(ts[1])?)))
        }),
        // ruleAfterLunch
        rule("after lunch", vec![regex("etter (frokost|middag|lunsj|lunch)")], |_| {
            let td2 = interval(Open, &hour(false, 13), &hour(false, 17))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleAfterTimeofday
        rule("after <time-of-day>", vec![regex("(engang )?etter"), dimension(Dim::Time)], |ts| {
            tt(with_direction(IntervalDirection::After, td(ts[1])?.clone()))
        }),
        // ruleAfterWork
        rule("after work", vec![regex("etter jobb")], |_| {
            let td2 = interval(Open, &hour(false, 17), &hour(false, 21))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleAfternoon
        rule("afternoon", vec![regex("ettermiddag(en)?")], |_| latent_pod(12, 19)),
        // ruleAtTimeofday
        rule("at <time-of-day>", vec![regex("klokken|klokka|kl.|@"), pred(is_a_time_of_day)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        // ruleBetweenDatetimeAndDatetimeInterval
        rule(
            "between <datetime> and <datetime> (interval)",
            vec![regex("mellom"), dimension(Dim::Time), regex("og"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleBetweenTimeofdayAndTimeofdayInterval
        rule(
            "between <time-of-day> and <time-of-day> (interval)",
            vec![regex("mellom"), pred(is_a_time_of_day), regex("og"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleByTheEndOfTime
        rule("by the end of <time>", vec![regex("i slutten av"), dimension(Dim::Time)], |ts| {
            ttd(interval(Closed, &now(), td(ts[1])?))
        }),
        // ruleChristmas
        rule("christmas", vec![regex("((1\\.?)|første)? ?juledag")], |_| tt(month_day(12, 25))),
        // ruleChristmasEve
        rule("christmas eve", vec![regex("julaften?")], |_| tt(month_day(12, 24))),
        // ruleChristmasDays
        rule("christmas days", vec![regex("romjul(a|en)")], |_| {
            ttd(interval(Open, &month_day(12, 24), &month_day(12, 30)))
        }),
        // ruleConstitutionDay
        rule("constitution day", vec![regex("grunnlovsdag(en)")], |_| tt(month_day(5, 17))),
        // ruleCycleAfterTime
        rule(
            "<cycle> after <time>",
            vec![dimension(Dim::TimeGrain), regex("etter"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, 1, td(ts[2])?)),
        ),
        // ruleCycleBeforeTime
        rule(
            "<cycle> before <time>",
            vec![dimension(Dim::TimeGrain), regex("før"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, -1, td(ts[2])?)),
        ),
        // ruleDatetimeDatetimeInterval
        rule(
            "<datetime> - <datetime> (interval)",
            vec![pred(is_not_latent), regex("\\-|til|tilogmed"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleDayofmonthNonOrdinalNamedmonth
        rule(
            "<day-of-month> (non ordinal) <named-month>",
            vec![pred(is_dom_integer), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
        ),
        // ruleDayofmonthNonOrdinalOfNamedmonth
        rule(
            "<day-of-month> (non ordinal) of <named-month>",
            vec![pred(is_dom_integer), regex("av|i"), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
        ),
        // ruleDayofmonthOrdinal
        rule("<day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
            tt(mk_latent(day_of_month(ordinal_value(ts[0])?)))
        }),
        // ruleDayofmonthOrdinalOfNamedmonth
        rule(
            "<day-of-month> (ordinal) of <named-month>",
            vec![pred(is_dom_ordinal), regex("av|i"), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
        ),
        // ruleDayofmonthordinalNamedmonth
        rule(
            "<day-of-month>(ordinal) <named-month>",
            vec![pred(is_dom_ordinal), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
        ),
        // ruleDayofmonthordinalNamedmonthYear
        rule(
            "<day-of-month>(ordinal) <named-month> year",
            vec![pred(is_dom_ordinal), pred(is_a_month), regex("(\\d{2,4})")],
            |ts| {
                let y = parse_int(&rx(ts[2], 0)?)?;
                let dom = intersect_dom(td(ts[1])?, ts[0])?;
                ttd(intersect(&dom, &year(y)))
            },
        ),
        // ruleDdmm
        rule("dd/mm", vec![regex("(3[01]|[12]\\d|0?[1-9])[\\/-](0?[1-9]\\b|1[0-2])")], |ts| {
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        }),
        // ruleDdmmyyyy
        rule(
            "dd/mm/yyyy",
            vec![regex("(3[01]|[12]\\d|0?[1-9])[\\/\\-\\.](0?[1-9]|1[0-2])[\\/\\-\\.](\\d{2,4})")],
            |ts| {
                let d = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let y = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        // ruleDurationAfterTime
        rule(
            "<duration> after <time>",
            vec![dimension(Dim::Duration), regex("etter"), dimension(Dim::Time)],
            |ts| tt(duration_after(dur(ts[0])?, td(ts[2])?)),
        ),
        // ruleDurationAgo
        rule("<duration> ago", vec![dimension(Dim::Duration), regex("siden")], |ts| {
            tt(duration_ago(dur(ts[0])?))
        }),
        // ruleDurationBeforeTime
        rule(
            "<duration> before <time>",
            vec![dimension(Dim::Duration), regex("før"), dimension(Dim::Time)],
            |ts| tt(duration_before(dur(ts[0])?, td(ts[2])?)),
        ),
        // ruleDurationFromNow
        rule(
            "<duration> from now",
            vec![dimension(Dim::Duration), regex("fra (i dag|idag|nå)")],
            |ts| tt(in_duration(dur(ts[0])?)),
        ),
        // ruleEomendOfMonth
        rule("EOM|End of month", vec![regex("EOM")], |_| tt(cycle_nth(G::Month, 1))),
        // ruleEoyendOfYear
        rule("EOY|End of year", vec![regex("EOY")], |_| tt(cycle_nth(G::Year, 1))),
        // ruleEvening
        rule("evening", vec![regex("kveld(en)?")], |_| latent_pod(18, 0)),
        // ruleExactlyTimeofday
        rule(
            "exactly <time-of-day>",
            vec![regex("presis( kl.| klokken | klokka)?"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleFathersDay
        rule("Father's Day", vec![regex("farsdag")], |_| tt(nth_dow_of_month(2, 7, 11))),
        // ruleFromDatetimeDatetimeInterval
        rule(
            "from <datetime> - <datetime> (interval)",
            vec![regex("fra"), dimension(Dim::Time), regex("\\-|til|tilogmed"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleFromTimeofdayTimeofdayInterval
        rule(
            "from <time-of-day> - <time-of-day> (interval)",
            vec![
                regex("(etter|fra)"),
                pred(is_a_time_of_day),
                regex("((men )?før)|\\-|tilogmed|til"),
                pred(is_a_time_of_day),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleHalloweenDay
        rule("halloween day", vec![regex("hall?owe?en")], |_| tt(month_day(10, 31))),
        // ruleHhmm
        rule("hh:mm", vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)")], |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(false, h, m))
        }),
        // ruleHhmmss
        rule(
            "hh:mm:ss",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)[:.]([0-5]\\d)")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let s = parse_int(&rx(ts[0], 2)?)?;
                tt(hour_minute_second(false, h, m, s))
            },
        ),
        // ruleHourofdayIntegerAsRelativeMinutes
        rule(
            "<hour-of-day> <integer> (as relative minutes)",
            vec![pred(not_latent_hour), pred(is_integer_between_tok(1, 59))],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, get_int_value(ts[1])?)),
        ),
        // ruleHourofdayQuarter
        rule(
            "<hour-of-day> quarter (as relative minutes)",
            vec![pred(is_an_hour_of_day), regex("(et)? ?(kvart)(er)?")],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, 15)),
        ),
        // ruleHourofdayHalf
        rule(
            "<hour-of-day> half (as relative minutes)",
            vec![pred(is_an_hour_of_day), regex("halv time")],
            |ts| tt(hour_minute(true, tod_hours(ts[0])?, 30)),
        ),
        // ruleInDuration
        rule("in <duration>", vec![regex("om"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        // ruleInduringThePartofday
        rule("in|during the <part-of-day>", vec![regex("om|i"), pred(is_a_part_of_day)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        // ruleInduringThePartofday2
        rule(
            "in|during the <part-of-day>",
            vec![regex("om|i"), pred(is_a_part_of_day), regex("en|ten")],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleIntersect
        rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        // ruleIntersectBy
        rule(
            "intersect by \",\"",
            vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleIntersectByOfFromS
        rule(
            "intersect by \"of\", \"from\", \"'s\"",
            vec![pred(is_not_latent), regex("i"), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleLastCycle
        rule("last <cycle>", vec![regex("siste?|seneste|forrige"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, -1))
        }),
        // ruleLastCycleOfTime
        rule(
            "last <cycle> of <time>",
            vec![regex("siste"), dimension(Dim::TimeGrain), regex("av|i"), dimension(Dim::Time)],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastDayofweekOfTime
        rule(
            "last <day-of-week> of <time>",
            vec![regex("siste"), pred(is_a_day_of_week), regex("av|i"), dimension(Dim::Time)],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastNCycle
        rule(
            "last n <cycle>",
            vec![
                regex("siste?|seneste|forrige"),
                pred(is_integer_between_tok(1, 9999)),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let v = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -v))
            },
        ),
        // ruleLastTime
        rule("last <time>", vec![regex("(siste?|forrige|seneste)"), dimension(Dim::Time)], |ts| {
            tt(pred_nth(-1, false, td(ts[1])?))
        }),
        // ruleLastYear
        rule("Last year", vec![regex("i fjor")], |_| tt(cycle_nth(G::Year, -1))),
        // ruleLunch
        rule("lunch", vec![regex("(til )?middag")], |_| latent_pod(12, 14)),
        // ruleMidnighteodendOfDay
        rule("midnight|EOD|end of day", vec![regex("midnatt|EOD")], |_| tt(hour(false, 0))),
        // ruleMonthDdddInterval
        rule(
            "<month> dd-dd (interval)",
            vec![
                regex("([012]?\\d|30|31)(ter|\\.)?"),
                regex("\\-|til"),
                regex("([012]?\\d|30|31)(ter|\\.)?"),
                pred(is_a_month),
            ],
            |ts| {
                let dd1 = parse_int(&rx(ts[0], 0)?)?;
                let dd2 = parse_int(&rx(ts[2], 0)?)?;
                let month = td(ts[3])?;
                let dom1 = intersect(&day_of_month(dd1), month)?;
                let dom2 = intersect(&day_of_month(dd2), month)?;
                ttd(interval(Closed, &dom1, &dom2))
            },
        ),
        // ruleMorning
        rule("morning", vec![regex("morgen(en)?|morran")], |_| latent_pod(4, 12)),
        // ruleMothersDay
        rule("Mother's Day", vec![regex("morsdag")], |_| tt(nth_dow_of_month(2, 7, 2))),
        // ruleNamedmonthDayofmonthNonOrdinal
        rule(
            "<named-month> <day-of-month> (non ordinal)",
            vec![pred(is_a_month), pred(is_dom_integer)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        // ruleNamedmonthDayofmonthOrdinal
        rule(
            "<named-month> <day-of-month> (ordinal)",
            vec![pred(is_a_month), pred(is_dom_ordinal)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        // ruleNewYearsDay
        rule("new year's day", vec![regex("nyttårsdag")], |_| tt(month_day(1, 1))),
        // ruleNewYearsEve
        rule("new year's eve", vec![regex("nyttårsaften?")], |_| tt(month_day(12, 31))),
        // ruleNextCycle
        rule("next <cycle>", vec![regex("neste|kommende"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, 1))
        }),
        // ruleNextNCycle
        rule(
            "next n <cycle>",
            vec![regex("neste"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_n(true, grain_of(ts[2])?, get_int_value(ts[1])?)),
        ),
        // ruleNextTime
        rule("next <time>", vec![regex("neste|kommende"), pred(is_not_latent)], |ts| {
            tt(pred_nth(0, true, td(ts[1])?))
        }),
        // ruleNight
        rule("night", vec![regex("natt(en)?")], |_| latent_pod(0, 4)),
        // ruleNoon
        rule("noon", vec![regex("middag|(kl(\\.|okken)?)? tolv")], |_| tt(hour(false, 12))),
        // ruleNow
        rule("now", vec![regex("akkurat nå|nå|(i )?dette øyeblikk")], |_| tt(now())),
        // ruleNthTimeAfterTime
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("etter"), dimension(Dim::Time)],
            |ts| tt(pred_nth_after(ordinal_value(ts[0])? - 1, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleNthTimeAfterTime2
        rule(
            "nth <time> after <time>",
            vec![
                regex("den"),
                dimension(Dim::Ordinal),
                dimension(Dim::Time),
                regex("etter"),
                dimension(Dim::Time),
            ],
            |ts| tt(pred_nth_after(ordinal_value(ts[1])? - 1, td(ts[2])?, td(ts[4])?)),
        ),
        // ruleNthTimeOfTime
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("av|i"), dimension(Dim::Time)],
            |ts| {
                let n = ordinal_value(ts[0])? - 1;
                ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(n, false, &x)))
            },
        ),
        // ruleNthTimeOfTime2 — Haskell prod starts at Token Ordinal (never matches; port as written)
        rule(
            "nth <time> of <time>",
            vec![regex("den"), dimension(Dim::Ordinal), dimension(Dim::Time), regex("af|i"), dimension(Dim::Time)],
            |ts| match (ts.first()?, ts.get(1)?, ts.get(3)?) {
                (Token::Ordinal(od), Token::Time(td1), Token::Time(td2)) => {
                    ttd(intersect(td2, td1).map(|x| pred_nth(od.value - 1, false, &x)))
                }
                _ => None,
            },
        ),
        // ruleOnANamedday
        rule("on a named-day", vec![regex("på en"), pred(is_a_day_of_week)], |ts| Some(ts[1].clone())),
        // ruleOnDate
        rule("on <date>", vec![regex("den|på"), dimension(Dim::Time)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        // ruleOrdinalCycleAfterTime
        rule(
            "<ordinal> <cycle> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("etter"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?)),
        ),
        // ruleOrdinalCycleOfTime
        rule(
            "<ordinal> <cycle> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("av|i|fra"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?)),
        ),
        // ruleOrdinalQuarter
        rule("<ordinal> quarter", vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))], |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth_after(true, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
        }),
        // ruleOrdinalQuarterYear
        rule(
            "<ordinal> quarter <year>",
            vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter)), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, G::Quarter, ordinal_value(ts[0])? - 1, td(ts[2])?)),
        ),
        // rulePartofdayOfTime
        rule(
            "<part-of-day> of <time>",
            vec![pred(is_a_part_of_day), regex("(en |ten )?den"), dimension(Dim::Time)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleRelativeMinutesAfterpastIntegerHourofday
        rule(
            "relative minutes after|past <integer> (hour-of-day)",
            vec![pred(is_integer_between_tok(1, 59)), regex("over"), pred(is_an_hour_of_day)],
            |ts| tt(hour_minute(true, tod_hours(ts[2])?, get_int_value(ts[0])?)),
        ),
        // ruleRelativeMinutesTotillbeforeIntegerHourofday
        rule(
            "relative minutes to|till|before <integer> (hour-of-day)",
            vec![pred(is_integer_between_tok(1, 59)), regex("på"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(get_int_value(ts[0])?, td(ts[2])?)),
        ),
        // ruleHalfAfterpastIntegerHourofday
        rule(
            "half after|past <integer> (hour-of-day)",
            vec![regex("halv time *over"), pred(is_an_hour_of_day)],
            |ts| tt(hour_minute(true, tod_hours(ts[1])?, 30)),
        ),
        // ruleHalfTotillbeforeIntegerHourofday
        rule(
            "half to|till|before <integer> (hour-of-day)",
            vec![regex("halv time *på|halvtime *på|halv"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(30, td(ts[1])?)),
        ),
        // ruleQuarterAfterpastIntegerHourofday
        rule(
            "quarter after|past <integer> (hour-of-day)",
            vec![regex("(et)? ?(kvart)(er)? *over"), pred(is_an_hour_of_day)],
            |ts| tt(hour_minute(true, tod_hours(ts[1])?, 15)),
        ),
        // ruleQuarterTotillbeforeIntegerHourofday
        rule(
            "quarter to|till|before <integer> (hour-of-day)",
            vec![regex("(et)? ?(kvart)(er)? *på"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(15, td(ts[1])?)),
        ),
        // ruleSeason
        rule("season", vec![regex("sommer(en)")], |_| {
            season_interval(month_day(6, 21), month_day(9, 23))
        }),
        // ruleSeason2
        rule("season", vec![regex("vinter(en)")], |_| {
            season_interval(month_day(12, 21), month_day(3, 20))
        }),
        // ruleTheCycleAfterTime
        rule(
            "the <cycle> after <time>",
            vec![dimension(Dim::TimeGrain), regex("(en|tet|et)? etter"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, 1, td(ts[2])?)),
        ),
        // ruleTheCycleBeforeTime
        rule(
            "the <cycle> before <time>",
            vec![dimension(Dim::TimeGrain), regex("(en|tet|et)? før"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, -1, td(ts[2])?)),
        ),
        // ruleTheCycleOfTime
        rule(
            "the <cycle> of <time>",
            vec![dimension(Dim::TimeGrain), regex("(en|tet|et)? av"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[0])?, 0, td(ts[2])?)),
        ),
        // ruleTheDayAfterTomorrow
        rule("the day after tomorrow", vec![regex("i overimorgen|i overimorra")], |_| {
            tt(cycle_nth(G::Day, 2))
        }),
        // ruleTheDayBeforeYesterday
        rule("the day before yesterday", vec![regex("i forigårs")], |_| tt(cycle_nth(G::Day, -2))),
        // ruleTheDayofmonthNonOrdinal
        rule(
            "the <day-of-month> (non ordinal)",
            vec![regex("den"), pred(is_dom_integer)],
            |ts| tt(mk_latent(day_of_month(get_int_value(ts[1])?))),
        ),
        // ruleTheDayofmonthOrdinal
        rule("the <day-of-month> (ordinal)", vec![regex("den"), pred(is_dom_ordinal)], |ts| {
            tt(day_of_month(ordinal_value(ts[1])?))
        }),
        // ruleTheIdesOfNamedmonth
        rule("the ides of <named-month>", vec![regex("midten af"), pred(is_a_month)], |ts| {
            let m = month_num(ts[1])?;
            let d = if [3, 5, 7, 10].contains(&m) { 15 } else { 13 };
            ttd(intersect(&day_of_month(d), td(ts[1])?))
        }),
        // ruleTheOrdinalCycleAfterTime
        rule(
            "the <ordinal> <cycle> after <time>",
            vec![
                regex("den"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("etter"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[4])?)),
        ),
        // ruleTheOrdinalCycleOfTime
        rule(
            "the <ordinal> <cycle> of <time>",
            vec![
                regex("den"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("av|i|fra"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[4])?)),
        ),
        // ruleThisCycle
        rule("this <cycle>", vec![regex("denne|dette|i|nåværende"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, 0))
        }),
        // ruleThisPartofday
        rule("this <part-of-day>", vec![regex("i|denne"), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(&today(), td(ts[1])?).map(part_of_day))
        }),
        // ruleThisTime
        rule("this <time>", vec![regex("(denne|dette|i|den her)"), dimension(Dim::Time)], |ts| {
            tt(pred_nth(0, false, td(ts[1])?))
        }),
        // ruleThisnextDayofweek
        rule("this|next <day-of-week>", vec![regex("(kommende|neste)"), pred(is_a_day_of_week)], |ts| {
            tt(pred_nth(0, true, td(ts[1])?))
        }),
        // ruleTimeAfterNext
        rule("<time> after next", vec![regex("neste"), dimension(Dim::Time), regex("igjen")], |ts| {
            tt(pred_nth(1, true, td(ts[1])?))
        }),
        // ruleTimeBeforeLast
        rule("<time> before last", vec![regex("siste"), dimension(Dim::Time), regex("igjen")], |ts| {
            tt(pred_nth(-2, false, td(ts[1])?))
        }),
        // ruleTimePartofday
        rule("<time> <part-of-day>", vec![dimension(Dim::Time), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        // ruleTimeofdayApproximately
        rule(
            "<time-of-day> approximately",
            vec![pred(is_a_time_of_day), regex("(cirka|ca\\.|-?ish)")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        // ruleTimeofdayLatent
        rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
            tt(mk_latent(hour(false, get_int_value(ts[0])?)))
        }),
        // ruleTimeofdayOclock
        rule("<time-of-day> o'clock", vec![pred(is_a_time_of_day), regex("h")], |ts| {
            tt(not_latent(td(ts[0])?.clone()))
        }),
        // ruleTimeofdaySharp
        rule("<time-of-day> sharp", vec![pred(is_a_time_of_day), regex("(sharp|presis)")], |ts| {
            tt(not_latent(td(ts[0])?.clone()))
        }),
        // ruleTimeofdayTimeofdayInterval
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(not_latent_tod), regex("\\-|tilogmed|til"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleToday
        rule("today", vec![regex("i dag|idag")], |_| tt(today())),
        // ruleTomorrow
        rule("tomorrow", vec![regex("i morgen|imorgen|i morra|imorra")], |_| tt(cycle_nth(G::Day, 1))),
        // ruleTonight
        rule("tonight", vec![regex("i kveld")], |_| {
            let td2 = interval(Open, &hour(false, 18), &hour(false, 0))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleUntilTimeofday
        rule(
            "until <time-of-day>",
            vec![regex("(engang )?innen|før|opptil"), dimension(Dim::Time)],
            |ts| tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone())),
        ),
        // ruleValentinesDay
        rule("valentine's day", vec![regex("valentine'?s?( dag)?")], |_| tt(month_day(2, 14))),
        // ruleWeekend
        rule("week-end", vec![regex("((week(\\s|-)?end)|helg)(en|a)?")], |_| tt(weekend())),
        // ruleWithinDuration
        rule("within <duration>", vec![regex("innenfor"), dimension(Dim::Duration)], |ts| {
            ttd(interval(Open, &now(), &in_duration(dur(ts[1])?)))
        }),
        // ruleYear
        rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
            tt(year(get_int_value(ts[0])?))
        }),
        // ruleYearLatent
        rule("year (latent)", vec![pred(is_integer_between_tok(25, 999))], |ts| {
            tt(mk_latent(year(get_int_value(ts[0])?)))
        }),
        // ruleYearLatent2
        rule("year (latent)", vec![pred(is_integer_between_tok(2101, 10000))], |ts| {
            tt(mk_latent(year(get_int_value(ts[0])?)))
        }),
        // ruleYesterday
        rule("yesterday", vec![regex("i går|igår")], |_| tt(cycle_nth(G::Day, -1))),
        // ruleYyyymmdd
        rule(
            "yyyy-mm-dd",
            vec![regex("(\\d{4})[\\-\\.\\/](0?[1-9]|1[0-2])[\\-\\.\\/](3[01]|[12]\\d|0?[1-9])")],
            |ts| {
                let y = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let d = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        // ruleTimezone
        rule(
            "<time> timezone",
            vec![pred(not_latent_tod), regex(&format!("\\b({})\\b", TIMEZONE_NAME))],
            |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
        ),
        // rule2400
        rule("24:00", vec![regex("24[.:]00")], |_| tt(hour_minute(false, 24, 0))),
        // ruleDmm
        rule("d/mm", vec![regex("([1-9])[\\/\\.\\-]([1-9]\\b|1[0-2]\\b)")], |ts| {
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        }),
        // ruleDDm
        rule("dd/m", vec![regex("([0-2]\\d|3[01])[\\/\\.\\-]([1-9])")], |ts| {
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        }),
        // ruleDdDotMm
        rule("dd.mm", vec![regex("(2[5-9]|3[01])\\.(0[1-9]|1[0-2])")], |ts| {
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        }),
    ]
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Mandag", "mandag|man\\.?" ),
        ("Tirsdag", "tirsdag|tirs?\\.?" ),
        ("Onsdag", "onsdag|ons\\.?" ),
        ("Torsdag", "torsdag|tors?\\.?" ),
        ("Fredag", "fredag|fre\\.?" ),
        ("Lørdag", "lørdag|lør\\.?" ),
        ("Søndag", "søndag|søn\\.?" ),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("Januar", "januar|jan\\.?" ),
        ("Februar", "februar|feb\\.?" ),
        ("Mars", "mars|mar\\.?" ),
        ("April", "april|apr\\.?" ),
        ("Mai", "mai"),
        ("Juni", "juni|jun\\.?" ),
        ("Juli", "juli|jul\\.?" ),
        ("August", "august|aug\\.?" ),
        ("September", "september|sept?\\.?" ),
        ("Oktober", "oktober|okt\\.?" ),
        ("November", "november|nov\\.?" ),
        ("Desember", "desember|des\\.?" ),
    ])
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = rules_main();
    v.extend(rules_days_of_week());
    v.extend(rules_months());
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
