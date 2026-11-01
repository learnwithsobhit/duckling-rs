//! `Duckling.Time.DA.Rules`.

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

/// `Duckling.Duration.Helpers.isGrain` — matches a TimeGrain token, not `is_grain_of_time`.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn not_latent_tod(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t)
}

fn not_latent_hour(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

fn latent_pod(from: i64, to: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, from), &hour(false, to)).map(|x| mk_latent(part_of_day(x))))
}

const TIMEZONE_NAME: &str = "YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT";

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Monday", "mandag"),
        ("Tuesday", "tirsdag|tirs?\\.?" ),
        ("Wednesday", "onsdag|ons\\.?" ),
        ("Thursday", "torsdag|tors?\\.?" ),
        ("Friday", "fredag|fre\\.?" ),
        ("Saturday", "lørdag|lør\\.?" ),
        ("Sunday", "søndag|søn\\.?" ),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("January", "januar|jan\\.?" ),
        ("February", "februar|feb\\.?" ),
        ("March", "marts|mar\\.?" ),
        ("April", "april|apr\\.?" ),
        ("May", "maj"),
        ("June", "juni|jun\\.?" ),
        ("July", "juli|jul\\.?" ),
        ("August", "august|aug\\.?" ),
        ("September", "september|sept?\\.?" ),
        ("October", "oktober|okt\\.?" ),
        ("November", "november|nov\\.?" ),
        ("December", "december|dec\\.?" ),
    ])
}

fn rules_main() -> Vec<Rule> {
    vec![
        // ruleADuration
        rule("a <duration>", vec![regex("(om )?en|et|én|ét"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        // ruleAboutDuration
        rule(
            "about <duration>",
            vec![regex("(omkring|cirka|ca.)"), dimension(Dim::Duration)],
            |ts| match ts.get(1)? {
                Token::Duration(dd) => Some(Token::Duration(dd.clone())),
                _ => None,
            },
        ),
        // ruleAboutTimeofday
        rule(
            "about <time-of-day>",
            vec![regex("(omkring|cirka|ca\\.)( klokken| kl\\.?)?"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleAbsorptionOfAfterNamedDay
        rule(
            "absorption of , after named day",
            vec![pred(is_a_day_of_week), regex(",")],
            |ts| Some(ts[0].clone()),
        ),
        // ruleAfterDuration
        rule("after <duration>", vec![regex("efter"), dimension(Dim::Duration)], |ts| {
            tt(with_direction(IntervalDirection::After, in_duration(dur(ts[1])?)))
        }),
        // ruleAfterLunch
        rule("after lunch", vec![regex("efter (frokost|middag)")], |_| {
            let td2 = interval(Open, &hour(false, 13), &hour(false, 17))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleAfterTimeofday
        rule(
            "after <time-of-day>",
            vec![regex("(engang )?efter"), dimension(Dim::Time)],
            |ts| tt(with_direction(IntervalDirection::After, td(ts[1])?.clone())),
        ),
        // ruleAfterWork
        rule("after work", vec![regex("efter arbejde")], |_| {
            let td2 = interval(Open, &hour(false, 17), &hour(false, 21))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleAfternoon
        rule("afternoon", vec![regex("eftermiddag(en)?")], |_| latent_pod(12, 19)),
        // ruleAtTimeofday
        rule(
            "at <time-of-day>",
            vec![regex("klokken|kl\\.?|@"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleBetweenDatetimeAndDatetimeInterval
        rule(
            "between <datetime> and <datetime> (interval)",
            vec![regex("mellem"), dimension(Dim::Time), regex("og"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleBetweenTimeofdayAndTimeofdayInterval
        rule(
            "between <time-of-day> and <time-of-day> (interval)",
            vec![regex("mellem"), pred(is_a_time_of_day), regex("og"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleByTheEndOfTime
        rule(
            "by the end of <time>",
            vec![regex("i slutningen af"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, &now(), td(ts[1])?)),
        ),
        // ruleChristmas
        rule("christmas", vec![regex("((1\\.?)|første)? ?juledag")], |_| tt(month_day(12, 25))),
        // ruleChristmasEve
        rule("christmas eve", vec![regex("juleaften(sdag)?")], |_| tt(month_day(12, 24))),
        // ruleConstitutionDay
        rule("constitution day", vec![regex("grundlov'?s?'? dag")], |_| tt(month_day(6, 5))),
        // ruleCycleAfterTime
        rule(
            "<cycle> after <time>",
            vec![dimension(Dim::TimeGrain), regex("efter"), dimension(Dim::Time)],
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
            vec![pred(is_not_latent), regex("\\-|til|indtil"), pred(is_not_latent)],
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
            vec![pred(is_dom_integer), regex("af|i"), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
        ),
        // ruleDayofmonthOrdinal
        rule("<day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
            tt(mk_latent(day_of_month(ordinal_value(ts[0])?)))
        }),
        // ruleDayofmonthOrdinalOfNamedmonth
        rule(
            "<day-of-month> (ordinal) of <named-month>",
            vec![pred(is_dom_ordinal), regex("af|i"), pred(is_a_month)],
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
                let n = parse_int(&rx(ts[2], 0)?)?;
                let td2 = intersect_dom(td(ts[1])?, ts[0])?;
                ttd(intersect(&td2, &year(n)))
            },
        ),
        // ruleDdmm
        rule("dd/mm", vec![regex("(3[01]|[12]\\d|0?[1-9])[\\/-](0?[1-9]|1[0-2])")], |ts| {
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        }),
        // ruleDdmmyyyy
        rule(
            "dd/mm/yyyy",
            vec![regex("(3[01]|[12]\\d|0?[1-9])[\\/-](0?[1-9]|1[0-2])[\\/-](\\d{2,4})")],
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
            vec![dimension(Dim::Duration), regex("efter"), dimension(Dim::Time)],
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
            vec![dimension(Dim::Duration), regex("fra (i dag|nu)")],
            |ts| tt(in_duration(dur(ts[0])?)),
        ),
        // ruleDurationHence
        rule("<duration> hence", vec![dimension(Dim::Duration), regex("hence")], |ts| {
            tt(in_duration(dur(ts[0])?))
        }),
        // ruleEomendOfMonth
        rule("EOM|End of month", vec![regex("EOM")], |_| tt(cycle_nth(G::Month, 1))),
        // ruleEoyendOfYear
        rule("EOY|End of year", vec![regex("EOY")], |_| tt(cycle_nth(G::Year, 1))),
        // ruleEvening
        rule("evening", vec![regex("afte(s|n(en)?)")], |_| latent_pod(18, 0)),
        // ruleExactlyTimeofday
        rule(
            "exactly <time-of-day>",
            vec![regex("pr(æ)cis( klokken| kl\\.?)?"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleFathersDay
        rule("Father's Day", vec![regex("far'?s?'? dag")], |_| tt(month_day(6, 5))),
        // ruleFromDatetimeDatetimeInterval
        rule(
            "from <datetime> - <datetime> (interval)",
            vec![regex("fra"), dimension(Dim::Time), regex("\\-|til|indtil"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleFromTimeofdayTimeofdayInterval
        rule(
            "from <time-of-day> - <time-of-day> (interval)",
            vec![
                regex("(efter|fra)"),
                pred(is_a_time_of_day),
                regex("((men )?før)|\\-|til|indtil"),
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
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                let n = get_int_value(ts[1])?;
                tt(hour_minute(is12h, h, n))
            },
        ),
        // ruleInDuration
        rule("in <duration>", vec![regex("om"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        // ruleInduringThePartofday
        rule(
            "in|during the <part-of-day>",
            vec![regex("om|i"), pred(is_a_part_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
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
        rule("last <cycle>", vec![regex("sidste|seneste"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, -1))
        }),
        // ruleLastCycleOfTime
        rule(
            "last <cycle> of <time>",
            vec![regex("sidste"), dimension(Dim::TimeGrain), regex("af|i"), dimension(Dim::Time)],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastDayofweekOfTime
        rule(
            "last <day-of-week> of <time>",
            vec![regex("sidste"), pred(is_a_day_of_week), regex("af|i"), dimension(Dim::Time)],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastNCycle
        rule(
            "last n <cycle>",
            vec![regex("sidste|seneste"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -n))
            },
        ),
        // ruleLastTime
        rule("last <time>", vec![regex("(sidste|seneste)"), dimension(Dim::Time)], |ts| {
            tt(pred_nth(-1, false, td(ts[1])?))
        }),
        // ruleLunch
        rule("lunch", vec![regex("(til )?middag")], |_| latent_pod(12, 14)),
        // ruleMidnighteodendOfDay
        rule("midnight|EOD|end of day", vec![regex("midnat|EOD")], |_| tt(hour(false, 0))),
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
                let v1 = parse_int(&rx(ts[0], 0)?)?;
                let v2 = parse_int(&rx(ts[2], 0)?)?;
                let month = td(ts[3])?;
                let from = intersect(month, &day_of_month(v1))?;
                let to = intersect(month, &day_of_month(v2))?;
                ttd(interval(Closed, &from, &to))
            },
        ),
        // ruleMorning
        rule("morning", vec![regex("morgen(en)?")], |_| latent_pod(4, 12)),
        // ruleMothersDay
        rule("Mother's Day", vec![regex("mor'?s? dag")], |_| tt(nth_dow_of_month(1, 7, 5))),
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
        rule("new year's day", vec![regex("nytårsdag")], |_| tt(month_day(1, 1))),
        // ruleNewYearsEve
        rule("new year's eve", vec![regex("nytårsaften(sdag)?")], |_| tt(month_day(12, 31))),
        // ruleNextCycle
        rule("next <cycle>", vec![regex("n(æ)ste|kommende"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, 1))
        }),
        // ruleNextNCycle
        rule(
            "next n <cycle>",
            vec![regex("n(æ)ste"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, n))
            },
        ),
        // ruleNextTime
        rule("next <time>", vec![regex("n(æ)ste|kommende"), pred(is_not_latent)], |ts| {
            tt(pred_nth(0, true, td(ts[1])?))
        }),
        // ruleNight
        rule("night", vec![regex("nat(ten)?")], |_| latent_pod(0, 4)),
        // ruleNoon
        rule("noon", vec![regex("middag|(kl(\\.|okken)?)? tolv")], |_| tt(hour(false, 12))),
        // ruleNow
        rule("now", vec![regex("lige nu|nu|(i )?dette øjeblik")], |_| tt(now())),
        // ruleNthTimeAfterTime
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("efter"), dimension(Dim::Time)],
            |ts| tt(pred_nth_after(ordinal_value(ts[0])? - 1, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleNthTimeAfterTime2
        rule(
            "nth <time> after <time>",
            vec![
                regex("den"),
                dimension(Dim::Ordinal),
                dimension(Dim::Time),
                regex("efter"),
                dimension(Dim::Time),
            ],
            |ts| tt(pred_nth_after(ordinal_value(ts[1])? - 1, td(ts[2])?, td(ts[4])?)),
        ),
        // ruleNthTimeOfTime
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("af|i"), dimension(Dim::Time)],
            |ts| {
                let v = ordinal_value(ts[0])?;
                ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(v - 1, false, &x)))
            },
        ),
        // ruleNthTimeOfTime2
        rule(
            "nth <time> of <time>",
            vec![
                regex("den"),
                dimension(Dim::Ordinal),
                dimension(Dim::Time),
                regex("af|i"),
                dimension(Dim::Time),
            ],
            |ts| {
                let v = ordinal_value(ts[1])?;
                ttd(intersect(td(ts[4])?, td(ts[2])?).map(|x| pred_nth(v - 1, false, &x)))
            },
        ),
        // ruleOnANamedday
        rule("on a named-day", vec![regex("på en"), pred(is_a_day_of_week)], |ts| {
            Some(ts[1].clone())
        }),
        // ruleOnDate
        rule("on <date>", vec![regex("den|på"), dimension(Dim::Time)], |ts| Some(ts[1].clone())),
        // ruleOrdinalCycleAfterTime
        rule(
            "<ordinal> <cycle> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("efter"), dimension(Dim::Time)],
            |ts| {
                tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?))
            },
        ),
        // ruleOrdinalCycleOfTime
        rule(
            "<ordinal> <cycle> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("af|i|fra"), dimension(Dim::Time)],
            |ts| {
                tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?))
            },
        ),
        // ruleOrdinalQuarter
        rule(
            "<ordinal> quarter",
            vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(cycle_nth_after(false, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
            },
        ),
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
            |ts| {
                let (h, is12h) = tod(ts[2])?;
                let n = get_int_value(ts[0])?;
                tt(hour_minute(is12h, h, n))
            },
        ),
        // ruleRelativeMinutesTotillbeforeIntegerHourofday
        rule(
            "relative minutes to|till|before <integer> (hour-of-day)",
            vec![pred(is_integer_between_tok(1, 59)), regex("i"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(get_int_value(ts[0])?, td(ts[2])?)),
        ),
        // ruleSeason (summer)
        rule("season", vec![regex("sommer")], |_| {
            ttd(interval(Open, &month_day(6, 21), &month_day(9, 23)))
        }),
        // ruleSeason2 (autumn)
        rule("season", vec![regex("efterår")], |_| {
            ttd(interval(Open, &month_day(9, 23), &month_day(12, 21)))
        }),
        // ruleSeason3 (winter)
        rule("season", vec![regex("vinter")], |_| {
            ttd(interval(Open, &month_day(12, 21), &month_day(3, 20)))
        }),
        // ruleSeason4 (spring)
        rule("season", vec![regex("forår")], |_| {
            ttd(interval(Open, &month_day(3, 20), &month_day(6, 21)))
        }),
        // ruleTheCycleAfterTime
        rule(
            "the <cycle> after <time>",
            vec![dimension(Dim::TimeGrain), regex("(en|tet|et)? efter"), dimension(Dim::Time)],
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
            vec![dimension(Dim::TimeGrain), regex("(en|tet|et)? af"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[0])?, 0, td(ts[2])?)),
        ),
        // ruleTheDayAfterTomorrow
        rule("the day after tomorrow", vec![regex("i overmorgen")], |_| tt(cycle_nth(G::Day, 2))),
        // ruleTheDayBeforeYesterday
        rule("the day before yesterday", vec![regex("i forgårs")], |_| tt(cycle_nth(G::Day, -2))),
        // ruleTheDayofmonthNonOrdinal
        rule(
            "the <day-of-month> (non ordinal)",
            vec![regex("den"), pred(is_integer_between_tok(1, 31))],
            |ts| tt(day_of_month(get_int_value(ts[1])?)),
        ),
        // ruleTheDayofmonthOrdinal
        rule(
            "the <day-of-month> (ordinal)",
            vec![regex("den"), pred(is_dom_ordinal)],
            |ts| tt(day_of_month(ordinal_value(ts[1])?)),
        ),
        // ruleTheIdesOfNamedmonth
        rule(
            "the ides of <named-month>",
            vec![regex("midten af"), pred(is_a_month)],
            |ts| match &td(ts[1])?.form {
                Some(Form::Month(m)) => {
                    let d = if [3, 5, 7, 10].contains(m) { 15 } else { 13 };
                    ttd(intersect(td(ts[1])?, &day_of_month(d)))
                }
                _ => None,
            },
        ),
        // ruleTheOrdinalCycleAfterTime
        rule(
            "the <ordinal> <cycle> after <time>",
            vec![
                regex("den"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("efter"),
                dimension(Dim::Time),
            ],
            |ts| {
                tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[4])?))
            },
        ),
        // ruleTheOrdinalCycleOfTime
        rule(
            "the <ordinal> <cycle> of <time>",
            vec![
                regex("den"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("af|i|fra"),
                dimension(Dim::Time),
            ],
            |ts| {
                tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[4])?))
            },
        ),
        // ruleThisCycle
        rule(
            "this <cycle>",
            vec![regex("denne|dette|i|nuv(æ)rende|indev(æ)rende"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
        ),
        // ruleThisPartofday
        rule("this <part-of-day>", vec![regex("i|denne"), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(&today(), td(ts[1])?).map(part_of_day))
        }),
        // ruleThisTime
        rule("this <time>", vec![regex("(denne|dette|i|den her)"), dimension(Dim::Time)], |ts| {
            tt(pred_nth(0, false, td(ts[1])?))
        }),
        // ruleThisnextDayofweek
        rule(
            "this|next <day-of-week>",
            vec![regex("(kommende|n(æ)ste)"), pred(is_a_day_of_week)],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        // ruleTimeAfterNext
        rule(
            "<time> after next",
            vec![regex("n(æ)ste"), dimension(Dim::Time), regex("igen")],
            |ts| tt(pred_nth(1, true, td(ts[1])?)),
        ),
        // ruleTimeBeforeLast
        rule(
            "<time> before last",
            vec![regex("sidste"), dimension(Dim::Time), regex("igen")],
            |ts| tt(pred_nth(-2, false, td(ts[1])?)),
        ),
        // ruleTimePartofday
        rule(
            "<time> <part-of-day>",
            vec![dimension(Dim::Time), pred(is_a_part_of_day)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[1])?)),
        ),
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
        rule(
            "<time-of-day> sharp",
            vec![pred(is_a_time_of_day), regex("(sharp|pr(æ)cis)")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        // ruleTimeofdayTimeofdayInterval
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(not_latent_tod), regex("\\-|til|indtil"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleToday
        rule("today", vec![regex("i dag|idag")], |_| tt(today())),
        // ruleTomorrow
        rule("tomorrow", vec![regex("i morgen|imorgen")], |_| tt(cycle_nth(G::Day, 1))),
        // ruleTonight
        rule("tonight", vec![regex("i aften")], |_| {
            let td2 = interval(Open, &hour(false, 18), &hour(false, 0))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleUntilTimeofday
        rule(
            "until <time-of-day>",
            vec![regex("(engang )?inden|indtil|før|op til"), dimension(Dim::Time)],
            |ts| tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone())),
        ),
        // ruleValentinesDay
        rule("valentine's day", vec![regex("valentine'?s?( dag)?")], |_| tt(month_day(2, 14))),
        // ruleWeekend
        rule("week-end", vec![regex("(week(\\s|-)?end)(en)?")], |_| tt(weekend())),
        // ruleWithinDuration
        rule("within <duration>", vec![regex("indenfor"), dimension(Dim::Duration)], |ts| {
            ttd(interval(Open, &now(), &in_duration(dur(ts[1])?)))
        }),
        // ruleYear
        rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
            tt(year(get_int_value(ts[0])?))
        }),
        // ruleYearLatent
        rule("year (latent)", vec![pred(is_integer_between_tok(1, 999))], |ts| {
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
            vec![regex("(\\d{2,4})-(0?[1-9]|1[0-2])-(3[01]|[12]\\d|0?[1-9])")],
            |ts| {
                let y = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let d = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        // ruleQuarterTotillbeforeIntegerHourofday
        rule(
            "quarter to|till|before <integer> (hour-of-day)",
            vec![regex("(et|ét)? ?kvart(er)? i"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(15, td(ts[1])?)),
        ),
        // ruleHalfTotillbeforeIntegerHourofday
        rule(
            "half to|till|before <integer> (hour-of-day)",
            vec![regex("halv time i"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(30, td(ts[1])?)),
        ),
        // ruleQuarterAfterpastIntegerHourofday
        rule(
            "quarter after|past <integer> (hour-of-day)",
            vec![regex("(et|ét)? ?kvart(er)? over"), pred(is_an_hour_of_day)],
            |ts| {
                let (h, is12h) = tod(ts[1])?;
                tt(hour_minute(is12h, h, 15))
            },
        ),
        // ruleHalfAfterpastIntegerHourofday
        rule(
            "half after|past <integer> (hour-of-day)",
            vec![regex("halv time over"), pred(is_an_hour_of_day)],
            |ts| {
                let (h, is12h) = tod(ts[1])?;
                tt(hour_minute(is12h, h, 30))
            },
        ),
        // ruleHourofdayQuart
        rule(
            "<hour-of-day> quarter (as relative minutes)",
            vec![pred(is_an_hour_of_day), regex("(et|ét)? ?kvart(er)?")],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                tt(hour_minute(is12h, h, 15))
            },
        ),
        // ruleHourofdayHalf
        rule(
            "<hour-of-day> half (as relative minutes)",
            vec![pred(is_an_hour_of_day), regex("halv time")],
            |ts| {
                let (h, is12h) = tod(ts[0])?;
                tt(hour_minute(is12h, h, 30))
            },
        ),
        // ruleTimezone
        rule(
            "<time> timezone",
            vec![pred(not_latent_tod), regex(&format!("\\b({})\\b", TIMEZONE_NAME))],
            |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
        ),
    ]
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
