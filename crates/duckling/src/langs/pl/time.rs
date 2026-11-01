//! `Duckling.Time.PL.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::DurationData;
use crate::dims::time::computed::easter_sunday;
use crate::dims::time::helpers::*;
use crate::dims::time::types::{
    Form, IntervalDirection, TimeData,
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

fn not_latent_tod(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t)
}

fn not_latent_hour(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

fn not_latent_ok_this_next(t: &Token) -> bool {
    is_not_latent(t) && is_ok_with_this_next(t)
}

/// `Token Time . mkLatent . partOfDay <$> interval Open (hour False a) (hour False b)`.
fn latent_pod(a: i64, b: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, a), &hour(false, b)).map(|x| mk_latent(part_of_day(x))))
}

const TIMEZONE_NAME: &str = "YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT";

fn rules_main() -> Vec<Rule> {
    vec![
        // ruleAboutTimeofday
        rule(
            "about <time-of-day>",
            vec![regex("o?ko(l|ł)o|mniej wi(e|ę)cej|tak o"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleAbsorptionOfAfterNamedDay
        rule(
            "absorption of , after named day",
            vec![pred(is_a_day_of_week), regex(",")],
            |ts| Some(ts[0].clone()),
        ),
        // ruleAfterDuration
        rule("after <duration>", vec![regex("po"), dimension(Dim::Duration)], |ts| {
            tt(not_latent(with_direction(IntervalDirection::After, in_duration(dur(ts[1])?))))
        }),
        // ruleAfterTimeofday
        rule("after <time-of-day>", vec![regex("po"), dimension(Dim::Time)], |ts| {
            tt(with_direction(IntervalDirection::After, td(ts[1])?.clone()))
        }),
        // ruleAfternoon
        rule("afternoon", vec![regex("po(l|ł)udni(em|e|a|u)")], |_| latent_pod(12, 19)),
        // ruleAtTimeofday
        rule("at <time-of-day>", vec![regex("o|na|@"), pred(is_a_time_of_day)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        // ruleBetweenDatetimeAndDatetimeInterval
        rule(
            "between <datetime> and <datetime> (interval)",
            vec![regex("(po|po )?mi(e|ę)dzy"), dimension(Dim::Time), regex("a|i"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleBetweenTimeofdayAndTimeofdayInterval
        rule(
            "between <time-of-day> and <time-of-day> (interval)",
            vec![regex("(po|po )?miedzy|między"), pred(is_a_time_of_day), regex("a|i"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleByTheEndOfTime
        rule(
            "by the end of <time>",
            vec![regex("do (ko[ńn]ca )?(tego)?"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, &now(), td(ts[1])?)),
        ),
        // ruleByTime
        rule("by <time>", vec![regex("(a(z|ż) )?do"), dimension(Dim::Time)], |ts| {
            ttd(interval(Open, &now(), td(ts[1])?))
        }),
        // ruleCycleAfterTime
        rule(
            "<cycle> after <time>",
            vec![dimension(Dim::TimeGrain), regex("po"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, 1, td(ts[2])?)),
        ),
        // ruleCycleBeforeTime
        rule(
            "<cycle> before <time>",
            vec![dimension(Dim::TimeGrain), regex("przed"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, -1, td(ts[2])?)),
        ),
        // ruleDatetimeDatetimeInterval
        rule(
            "<datetime> - <datetime> (interval)",
            vec![pred(is_not_latent), regex("\\-|(p|d)o|a(ż|z) (p|d)o"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleDayaftertomorrowSingleword
        rule("day-after-tomorrow (single-word)", vec![regex("(po ?jutr(o|ze))")], |_| {
            tt(cycle_nth(G::Day, 2))
        }),
        // ruleDayBeforeYesterdaySingleWord
        rule("day-before-yesterday (single-word)", vec![regex("przedwczoraj")], |_| {
            tt(cycle_nth(G::Day, -2))
        }),
        // ruleDayofmonthNonOrdinalNamedmonth
        rule(
            "<day-of-month> (non ordinal) <named-month>",
            vec![pred(is_dom_integer), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
        ),
        // ruleDayofmonthNonOrdinalOfNamedmonth
        rule(
            "<day-of-month> (non ordinal) of <named-month>",
            vec![pred(is_dom_integer), regex("of|in"), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
        ),
        // ruleDayofmonthOrdinal
        rule("<day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
            tt(mk_latent(day_of_month(get_int_value(ts[0])?)))
        }),
        // ruleDayofmonthOrdinalOfNamedmonth
        rule(
            "<day-of-month> (ordinal) of <named-month>",
            vec![pred(is_dom_ordinal), regex("of|in"), pred(is_a_month)],
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
        // ruleDurationAfterTime
        rule(
            "<duration> after <time>",
            vec![dimension(Dim::Duration), regex("po"), dimension(Dim::Time)],
            |ts| tt(duration_after(dur(ts[0])?, td(ts[2])?)),
        ),
        // ruleDurationAgo
        rule("<duration> ago", vec![dimension(Dim::Duration), regex("temu")], |ts| {
            tt(duration_ago(dur(ts[0])?))
        }),
        // ruleDurationBeforeTime
        rule(
            "<duration> before <time>",
            vec![dimension(Dim::Duration), regex("do|przed"), dimension(Dim::Time)],
            |ts| tt(duration_before(dur(ts[0])?, td(ts[2])?)),
        ),
        // ruleDurationFromNow
        rule(
            "<duration> from now",
            vec![dimension(Dim::Duration), regex("od (dzi(s|ś)|teraz)")],
            |ts| tt(in_duration(dur(ts[0])?)),
        ),
        // ruleDurationHence
        rule(
            "<duration> hence",
            vec![dimension(Dim::Duration), regex("p(o|ó)(z|ź)niej|potem")],
            |ts| tt(in_duration(dur(ts[0])?)),
        ),
        // ruleEomendOfMonth
        rule(
            "EOM|End of month",
            vec![regex("(na |w )?(koniec|ko(n|ń)ca|ko(n|ń)cu|ko(n|ń)cowi|ko(n|ń)cem|ko(n|ń)c(o|ó)wke) (miesi(a|ą)ca|msc)")],
            |_| tt(cycle_nth(G::Month, 1)),
        ),
        // ruleEoyendOfYear
        rule(
            "EOY|End of year",
            vec![regex("(na |w )?(koniec|ko(n|ń)ca|ko(n|ń)cu|ko(n|ń)cowi|ko(n|ń)cem|ko(n|ń)c(o|ó)wke) (rok(u|owi|iem))")],
            |_| tt(cycle_nth(G::Year, 1)),
        ),
        // ruleEveningnight
        rule("evening|night", vec![regex("wiecz[oó]r(em|owi|ze|a|u)?|nocą?")], |_| {
            latent_pod(18, 0)
        }),
        // ruleExactlyTimeofday
        rule(
            "exactly <time-of-day>",
            vec![regex("(r(o|ó)wno|dok(l|ł)adnie)( o)?"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleFromDatetimeDatetimeInterval
        rule(
            "from <datetime> - <datetime> (interval)",
            vec![
                regex("od|p(o|ó)(z|ź)niej ni(z|ż)"),
                dimension(Dim::Time),
                regex("\\-|do|po|aż do|az do|aż po|az po|ale przed"),
                dimension(Dim::Time),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleFromHourofdayHourofdayInterval
        rule(
            "from <hour-of-day> - <hour-of-day> (interval)",
            vec![
                regex("od"),
                pred(is_a_time_of_day),
                regex("-|to|th?ru|through|until"),
                pred(not_latent_tod),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleFromTimeofdayTimeofdayInterval
        rule(
            "from <time-of-day> - <time-of-day> (interval)",
            vec![
                regex("(niż|niz|od)"),
                pred(is_a_time_of_day),
                regex("((but )?before)|\\-|do|po|aż do|az do|aż po|az po"),
                pred(is_a_time_of_day),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleHhmm
        rule("hh:mm", vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)")], |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(true, h, m))
        }),
        // ruleHhmmMilitary
        rule("hhmm (military)", vec![regex("((?:[01]?\\d)|(?:2[0-3]))([0-5]\\d)")], |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(mk_latent(hour_minute(false, h, m)))
        }),
        // ruleHhmmMilitaryAmpm
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
        // ruleHhmmss
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
        // ruleHourofdayHourofdayInterval
        rule(
            "<hour-of-day> - <hour-of-day> (interval)",
            vec![pred(is_a_time_of_day), regex("-|do|aż po|po"), pred(not_latent_tod)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleHourofdayIntegerAsRelativeMinutes
        rule(
            "<hour-of-day> <integer> (as relative minutes)",
            vec![pred(not_latent_hour), pred(is_integer_between_tok(1, 59))],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                let n = get_int_value(ts[1])?;
                tt(hour_minute(true, hours, n))
            },
        ),
        // ruleHourofdayQuarter
        rule(
            "<hour-of-day> quarter (as relative minutes)",
            vec![pred(is_an_hour_of_day), regex("kwadrans(ie|owi|em|a)?")],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                tt(hour_minute(true, hours, 15))
            },
        ),
        // ruleHourofdayHalf
        rule(
            "<hour-of-day> half (as relative minutes)",
            vec![pred(is_an_hour_of_day), regex("p(o|ó)(l|ł)")],
            |ts| {
                let (hours, _) = tod(ts[0])?;
                tt(hour_minute(true, hours, 30))
            },
        ),
        // ruleInDuration
        rule(
            "in <duration>",
            vec![regex("(w( ?(prze)?ciągu)?|za) ?(jeszcze)?|przez"), dimension(Dim::Duration)],
            |ts| tt(in_duration(dur(ts[1])?)),
        ),
        // ruleInduringThePartofday
        rule(
            "in|during the <part-of-day>",
            vec![regex("(w|na) ?(czas(ie)?)"), pred(is_a_part_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleIntegerLatentTimeofday
        rule(
            "<integer> (latent time-of-day)",
            vec![pred(is_integer_between_tok(0, 23))],
            |ts| tt(mk_latent(hour(true, get_int_value(ts[0])?))),
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
            vec![pred(is_not_latent), regex("z"), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleLastCycle
        rule(
            "last <cycle>",
            vec![
                regex("ostatni(ego|ch|emu|mi|m|(a|ą)|ej|e)?|(po ?)?przedni(ego|ch|emu|mi|m|e|(a|ą)|ej)?"),
                dimension(Dim::TimeGrain),
            ],
            |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
        ),
        // ruleLastCycleOfTime
        rule(
            "last <cycle> of <time>",
            vec![
                regex("ostatni(ego|ch|emu|mi|m|(a|ą)|ej)?"),
                dimension(Dim::TimeGrain),
                regex("w(e)?|z(e)?"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastCycleTime
        rule(
            "last <cycle> <time>",
            vec![regex("ostatni(ego|ch|emu|mi|m|(a|ą)|ej)?"), dimension(Dim::TimeGrain), dimension(Dim::Time)],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[2])?)),
        ),
        // ruleLastDayofweekOfTime
        rule(
            "last <day-of-week> of <time>",
            vec![
                regex("ostatni(ego|ch|emu|mi|m|(a|ą)|ej)?"),
                pred(is_a_day_of_week),
                regex("w(e)?|z(e)?"),
                dimension(Dim::Time),
            ],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastDayofweekTime
        rule(
            "last <day-of-week> <time>",
            vec![regex("ostatni(ego|ch|emu|mi|m|(a|ą)|ej)?"), pred(is_a_day_of_week), dimension(Dim::Time)],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[2])?)),
        ),
        // ruleLastNCycle
        rule(
            "last n <cycle>",
            vec![
                regex("ostatni(ego|ch|emu|mi|m|(a|ą)|ej|e)?|(po ?)?przedni(ego|ch|emu|mi|m|e|(a|ą)|ej)?"),
                pred(is_integer_between_tok(1, 9999)),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let v = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -v))
            },
        ),
        // ruleLastTime
        rule(
            "last <time>",
            vec![
                regex("ostatni(ego|ch|emu|mi|m|(a|ą)|ej)?|(po ?)?przedni(ego|ch|emu|e|mi|m|a)?"),
                pred(is_ok_with_this_next),
            ],
            |ts| tt(pred_nth(-1, false, td(ts[1])?)),
        ),
        // ruleLunch
        rule("lunch", vec![regex("(na )?la?u?nc(z|h)|obiad")], |_| latent_pod(12, 14)),
        // ruleMmdd
        rule("mm/dd", vec![regex("(0?[1-9]|1[0-2])/(3[01]|[12]\\d|0?[1-9])")], |ts| {
            let m = parse_int(&rx(ts[0], 0)?)?;
            let d = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        }),
        // ruleMmddyyyy
        rule(
            "mm/dd/yyyy",
            vec![regex("(0?[1-9]|1[0-2])[/-](3[01]|[12]\\d|0?[1-9])[-/](\\d{2,4})")],
            |ts| {
                let m = parse_int(&rx(ts[0], 0)?)?;
                let d = parse_int(&rx(ts[0], 1)?)?;
                let y = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        // ruleMonthDdddInterval
        rule(
            "<month> dd-dd (interval)",
            vec![
                pred(is_a_month),
                regex("(3[01]|[12]\\d|0?[1-9])"),
                regex("\\-|do|po|aż do|az do|aż po|az po"),
                regex("(3[01]|[12]\\d|0?[1-9])"),
            ],
            |ts| {
                let d1 = parse_int(&rx(ts[1], 0)?)?;
                let d2 = parse_int(&rx(ts[3], 0)?)?;
                let t = td(ts[0])?;
                let from = intersect(&day_of_month(d1), t)?;
                let to = intersect(&day_of_month(d2), t)?;
                ttd(interval(Closed, &from, &to))
            },
        ),
        // ruleMorning
        rule("morning", vec![regex("rano|poran(ek|ku|ka)|z rana|(s|ś)witem")], |_| {
            latent_pod(4, 12)
        }),
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
        // ruleNextCycle
        rule(
            "next <cycle>",
            vec![
                regex("kolejn(ym|y|ego|emu|(a|ą)|ej|e)|nast(e|ę)pn(ym|y|ego|emu|(a|ą)|ej|e)|przysz(l|ł)(ego|emu|ym|(a|ą)|ej|ych|i|ymi|y|e)|za"),
                dimension(Dim::TimeGrain),
            ],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 1)),
        ),
        // ruleNextNCycle
        rule(
            "next n <cycle>",
            vec![
                regex("kolejn(ym|y|ego|emu|(a|ą)|ej|e)|nast(e|ę)pn(ym|y|ego|emu|(a|ą)|ej|e)"),
                pred(is_integer_between_tok(1, 9999)),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let v = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, v))
            },
        ),
        // ruleNextTime
        rule(
            "next <time>",
            vec![
                regex("kolejn(ym|y|ego|emu|(a|ą)|ej|e)|nast(e|ę)pn(ym|y|ego|emu|e|(a|ą)|ej|e)|przysz(l|ł)(ego|emu|ym|(a|ą)|ej|ych|i|ymi|y|e)"),
                pred(not_latent_ok_this_next),
            ],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        // ruleNoon
        rule("noon", vec![regex("po(l|ł)udni(em|e|a|u)")], |_| tt(hour(false, 12))),
        // ruleNow
        rule("now", vec![regex("(w)? ?(tym|tej)? ?(teraz|momencie|chwili|momeńcie)")], |_| {
            tt(now())
        }),
        // ruleNthTimeAfterTime
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("po"), dimension(Dim::Time)],
            |ts| tt(pred_nth_after(get_int_value(ts[0])? - 1, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleNthTimeOfTime
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("w(e)?|z(e)?"), dimension(Dim::Time)],
            |ts| {
                let n = get_int_value(ts[0])?;
                ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(n - 1, false, &x)))
            },
        ),
        // ruleNthTimeTime
        rule(
            "nth <time> <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), dimension(Dim::Time)],
            |ts| {
                let n = get_int_value(ts[0])?;
                ttd(intersect(td(ts[2])?, td(ts[1])?).map(|x| pred_nth(n - 1, false, &x)))
            },
        ),
        // ruleOnANamedday
        rule("on a named-day", vec![regex("we?"), pred(is_a_day_of_week)], |ts| {
            Some(ts[1].clone())
        }),
        // ruleOnDate
        rule("on <date>", vec![regex("we?"), dimension(Dim::Time)], |ts| Some(ts[1].clone())),
        // ruleOrdinalAsHour
        rule("<ordinal> (as hour)", vec![pred(is_ordinal_between(1, 24))], |ts| {
            tt(mk_latent(hour(true, get_int_value(ts[0])?)))
        }),
        // ruleOrdinalCycleAfterTime
        rule(
            "<ordinal> <cycle> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("after"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, get_int_value(ts[0])? - 1, td(ts[3])?)),
        ),
        // ruleOrdinalCycleOfTime
        rule(
            "<ordinal> <cycle> of <time>",
            vec![
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("w(e)?|z(e)?"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, get_int_value(ts[0])? - 1, td(ts[3])?)),
        ),
        // ruleOrdinalCycleTime
        rule(
            "<ordinal> <cycle> <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, get_int_value(ts[0])? - 1, td(ts[2])?)),
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
            |ts| tt(cycle_nth_after(false, G::Quarter, get_int_value(ts[0])? - 1, td(ts[2])?)),
        ),
        // ruleRelativeMinutesAfterpastIntegerHourofday
        rule(
            "relative minutes after|past <integer> (hour-of-day)",
            vec![pred(is_integer_between_tok(1, 59)), regex("po"), pred(is_an_hour_of_day)],
            |ts| {
                let (hours, _) = tod(ts[2])?;
                let n = get_int_value(ts[0])?;
                tt(hour_minute(true, hours, n))
            },
        ),
        // ruleQuarterAfterpastIntegerHourofday
        rule(
            "quarter after|past <integer> (hour-of-day)",
            vec![regex("kwadrans(ie|owi|em|a)? *po"), pred(is_an_hour_of_day)],
            |ts| {
                let (hours, _) = tod(ts[1])?;
                tt(hour_minute(true, hours, 15))
            },
        ),
        // ruleHalfAfterpastIntegerHourofday
        rule(
            "half after|past <integer> (hour-of-day)",
            vec![regex("p(o|ó)(l|ł) *po"), pred(is_an_hour_of_day)],
            |ts| {
                let (hours, _) = tod(ts[1])?;
                tt(hour_minute(true, hours, 30))
            },
        ),
        // ruleRelativeMinutesTotillbeforeIntegerHourofday
        rule(
            "relative minutes to|till|before <integer> (hour-of-day)",
            vec![pred(is_integer_between_tok(1, 59)), regex("do|przed"), pred(is_an_hour_of_day)],
            |ts| {
                let n = get_int_value(ts[0])?;
                ttd(minutes_before(n, td(ts[2])?))
            },
        ),
        // ruleQuarterTotillbeforeIntegerHourofday
        rule(
            "quarter to|till|before <integer> (hour-of-day)",
            vec![regex("kwadrans(ie|owi|em|a)? *(do|przed)"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(15, td(ts[1])?)),
        ),
        // ruleHalfTotillbeforeIntegerHourofday
        rule(
            "half to|till|before <integer> (hour-of-day)",
            vec![regex("p(o|ó)(l|ł) *(do|przed)"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(30, td(ts[1])?)),
        ),
        // ruleTheCycleOfTime
        rule(
            "the <cycle> of <time>",
            vec![regex("the"), dimension(Dim::TimeGrain), regex("of"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, 0, td(ts[3])?)),
        ),
        // ruleTheIdesOfNamedmonth
        rule(
            "the ides of <named-month>",
            vec![regex("the ides? of"), pred(is_a_month)],
            |ts| match td(ts[1])?.form {
                Some(Form::Month(m)) => {
                    let d = if [3, 5, 7, 10].contains(&m) { 15 } else { 13 };
                    ttd(intersect(&day_of_month(d), td(ts[1])?))
                }
                _ => None,
            },
        ),
        // ruleTheOrdinalCycleAfterTime
        rule(
            "the <ordinal> <cycle> after <time>",
            vec![
                regex("the"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("after"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, get_int_value(ts[1])? - 1, td(ts[4])?)),
        ),
        // ruleThisCycle
        rule(
            "this <cycle>",
            vec![
                regex("te(mu|n|go|j)|tym|t(a|ą)|nadchodz(a|ą)c(ym|y|ego|emu|(a|ą)|ej)|obecn(ym|y|emu|ego|nym|(a|ą)|ej)"),
                dimension(Dim::TimeGrain),
            ],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
        ),
        // ruleThisPartofday
        rule(
            "this <part-of-day>",
            vec![regex("ten|tego|ta|to"), pred(is_a_part_of_day)],
            |ts| ttd(intersect(&today(), td(ts[1])?).map(part_of_day)),
        ),
        // ruleThisTime
        rule(
            "this <time>",
            vec![
                regex("te(mu|n|go|j)|ta|to|tym|nadchodz(a|ą)c(ym|y|ego|emu|(a|ą)|ej)"),
                pred(is_ok_with_this_next),
            ],
            |ts| tt(pred_nth(0, false, td(ts[1])?)),
        ),
        // ruleThisnextDayofweek
        rule(
            "this|next <day-of-week>",
            vec![
                regex("kolejn(ym|y|ego|emu|e)|nast(e|ę)pn(ym|y|ego|emu|e|(a|ą)|ej|e)"),
                pred(is_a_day_of_week),
            ],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        // ruleTimeAfterNext
        rule(
            "<time> after next",
            vec![
                dimension(Dim::Time),
                regex("po kolejnym|po nast(e|ę)pn(ym|y|ego|emu|(a|ą)|ej|e)|po przysz(l|ł)(ym|y|ego|emu|(a|ą)|ej)"),
            ],
            |ts| tt(pred_nth(1, true, td(ts[0])?)),
        ),
        // ruleTimeBeforeLast
        rule(
            "<time> before last",
            vec![dimension(Dim::Time), regex("przed ?ostatni(ego|ch|emu|mi|m|(a|ą)|ej)?")],
            |ts| tt(pred_nth(-2, false, td(ts[0])?)),
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
            vec![pred(is_a_time_of_day), regex("o?ko(l|ł)o|mniej wi(e|ę)cej")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        // ruleTimeofdayOclock
        rule("<time-of-day> o'clock", vec![pred(is_a_time_of_day), regex("godzina")], |ts| {
            tt(not_latent(td(ts[0])?.clone()))
        }),
        // ruleTimeofdayPopoudniuwieczoremwNocy
        rule(
            "<time-of-day> popołudniu/wieczorem/w nocy",
            vec![
                pred(is_a_time_of_day),
                regex("(w )?noc(y|(a|ą))|(po ?)?po(l|ł)udni(em|e|a|u)|wiecz(o|ó)r(i|u|a|owi|em|rze)"),
            ],
            |ts| tt(time_of_day_ampm(false, td(ts[0])?)),
        ),
        // ruleTimeofdayRano
        rule(
            "<time-of-day> rano",
            vec![pred(is_a_time_of_day), regex("(z )?ran(o|a|u|em)")],
            |ts| tt(time_of_day_ampm(true, td(ts[0])?)),
        ),
        // ruleTimeofdaySharp
        rule(
            "<time-of-day> sharp",
            vec![pred(is_a_time_of_day), regex("r(o|ó)wno|dok(l|ł)adnie")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        // ruleTimeofdayTimeofdayInterval
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(not_latent_tod), regex("\\-|:|do|po|aż do|az do|aż po|az po"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleToday
        rule(
            "today",
            vec![regex("dzisiejsz[yaą]|dzisiaj|dzi[śs]|w ten dzień|w ten dzien|w obecny dzień|w obecny dzien|obecnego dnia")],
            |_| tt(today()),
        ),
        // ruleTomorrow
        rule("tomorrow", vec![regex("jutr(o|a|u|em|ze(jszy|jsza)?)|jtr|jutr")], |_| {
            tt(cycle_nth(G::Day, 1))
        }),
        // ruleUntilTimeofday
        rule(
            "until <time-of-day>",
            vec![regex("(a(ż|z) )?do|przed"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(with_direction(IntervalDirection::Before, td(ts[1])?.clone()))),
        ),
        // ruleWeekend
        rule("week-end", vec![regex("((wek|week|wik)(\\s|-)?end|wkend)")], |_| {
            tt(mk_ok_for_this_next(weekend()))
        }),
        // ruleWithinDuration
        rule(
            "within <duration>",
            vec![regex("(w )?ci(a|ą)gu|zakresie|obrębie|obrebie"), dimension(Dim::Duration)],
            |ts| ttd(interval(Open, &now(), &in_duration(dur(ts[1])?))),
        ),
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
        rule("yesterday", vec![regex("wczoraj(szym|szy)?|wczrj|wczor")], |_| {
            tt(cycle_nth(G::Day, -1))
        }),
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
        // ruleTimezone
        rule(
            "<time> timezone",
            vec![pred(not_latent_tod), regex(&format!("\\b({})\\b", TIMEZONE_NAME))],
            |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
        ),
    ]
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Monday", "poniedzia(l|ł)(ek|ku|kowi|kiem|kowy)|pon\\.?" ),
        ("Tuesday", "wtorek|wtorku|wtorkowi|wtorkiem|wtr?\\.?" ),
        ("Wednesday", "(Ś|ś|s)rod(a|ą|y|e|ę|zie|owy|o)|(s|ś|Ś)ro?\\.?" ),
        ("Thursday", "czwartek|czwartku|czwartkowi|czwartkiem|czwr?\\.?" ),
        ("Friday", "piątek|piatek|piątku|piatku|piątkowi|piatkowi|piątkiem|piatkiem|pi(ą|a)tkowy|pia\\.?" ),
        ("Saturday", "sobota|soboty|sobocie|sobotę|sobote|sobotą|sobota|sobocie|soboto|sob\\.?" ),
        ("Sunday", "niedziel(a|i|ę|e|ą|o)|n(ie)?dz?\\.?" ),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("January", "stycz(eń|en|nia|niowi|niem|niu)|sty(cz)?\\.?" ),
        ("February", "lu(ty|tego|temu|t?\\.?)" ),
        ("March", "mar(zec|ca|cowi|cem|cu|z?\\.?)" ),
        ("April", "kwie(cień|cien|tnia|tniowi|tniem|tniu|t?\\.?)" ),
        ("May", "maj(a|owi|em|u)?" ),
        ("June", "czerw(iec|ca|cowi|cem|cu)|czer?\\.?" ),
        ("July", "lip(iec|ca|cowi|cem|cu)|lip\\.?" ),
        ("August", "sierp(ie(n|ń)|ni(a|owi|em|u))|sierp\\.?|sier\\.?|sie\\.?" ),
        ("September", "wrze(s|ś)(ie(ń|n)|ni(a|owi|em|u))|wrz\\.?|wrze\\.?" ),
        ("October", "pa(z|ź)dziernik(a|owi|iem|u)?|paź\\.?|paz\\.?" ),
        ("November", "listopad(a|owi|em|zie)?|lis\\.?|list\\.?" ),
        ("December", "grudz(ień|ien)|grudn(ia|iowi|iem|niu)|gru\\.?|grud\\.?" ),
    ])
}

fn rules_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("New Year's Day", "nowy rok", month_day(1, 1)),
        ("Three Kings' Day", "((ś|s)wi(e|ę)to)? ?trzech kr(o|ó)li", month_day(1, 6)),
        ("Valentine's Day", "walentynki", month_day(2, 14)),
        ("Women's Day", "dzie(n|ń) kobiet", month_day(3, 8)),
        ("Labour Day", "(s|ś)wi(e|ę)to pracy", month_day(5, 1)),
        (
            "Constitution Day",
            "(ś|s)wi(e|ę)to konstytucj(i|a) ?((trzeciego|3) maja)?",
            month_day(5, 3),
        ),
        ("Mother's Day", "dzie(n|ń) ma(my|tki|m)", month_day(5, 26)),
        ("Father's Day", "dzie(n|ń) (taty|ojca)", month_day(6, 23)),
        (
            "Assumption Day",
            "(ś|s)wi(e|ę)to wojska polskiego|wniebowzi(e|ę)cie naj(s|ś)wi(e|ę)tszej maryi panny",
            month_day(8, 15),
        ),
        ("Halloween Day", "hall?owe?en( day)?", month_day(10, 31)),
        (
            "All Saints' Day",
            "((ś|s)wi(e|ę)to)? ?wszystkich (s|ś)wi(e|ę)tych",
            month_day(11, 1),
        ),
        (
            "Polish Independence Day",
            "(s|ś)wi(e|ę)t(a|o) niepodleg(l|ł)o(s|ś)ci|(ś|s)w\\.? niepodleg(l|ł)o(s|ś)ci",
            month_day(11, 11),
        ),
        ("Saint Nicholas Day", "mikołajki", month_day(12, 6)),
        (
            "Christmas Eve",
            "(wigilia|wigilii|wigili(e|ę)|wigili(a|ą)|wigilio) ?(bo(z|ż)ego narodzenia)?",
            month_day(12, 24),
        ),
        (
            "Christmas",
            "((Ś|ś|s)wi(e|ę)ta)? ?bo(z|ż)(ym|ego|e) narodzeni(e|a|u)",
            month_day(12, 25),
        ),
        (
            "St Stephen's Day",
            "drugi dzie(n|ń) (s|ś)wi(a|ą)t bo(z|ż)ego narodzenia",
            month_day(12, 26),
        ),
        ("New Year's Eve", "sylwester", month_day(12, 31)),
        (
            "Thanksgiving Day",
            "((s|ś)wiet(a|o)|(dzie(n|ń)))? ?dzi(e|ę)kczynieni(e|a)",
            nth_dow_of_month(4, 4, 11),
        ),
    ])
}

fn rules_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("lato", "lato|lata|latu|latem|lecie", month_day(6, 21), month_day(9, 23)),
        (
            "jesień",
            "jesień|jesien|jesieni|jesienią|jesienia",
            month_day(9, 23),
            month_day(12, 21),
        ),
        (
            "zima",
            "zima|zimy|zimie|zimę|zime|zimą|zima|zimie|zimo",
            month_day(12, 21),
            month_day(3, 20),
        ),
        (
            "wiosna",
            "wiosna|wiosny|wiośnie|wiosnie|wiosnę|wiosne|wiosną|wiosna|wiośnie|wiosnie|wiosno",
            month_day(3, 20),
            month_day(6, 21),
        ),
    ])
}

fn rules_computed_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        (
            "Ash Wednesday",
            "popielec",
            cycle_nth_after(false, G::Day, -46, &easter_sunday()),
        ),
        (
            "Palm Sunday",
            "niedziela palmowa",
            cycle_nth_after(false, G::Day, -7, &easter_sunday()),
        ),
        (
            "Good Friday",
            "wielki pi(a|ą)tek",
            cycle_nth_after(false, G::Day, -2, &easter_sunday()),
        ),
        (
            "Holy Saturday",
            "wielka sobota",
            cycle_nth_after(false, G::Day, -1, &easter_sunday()),
        ),
        ("Easter Sunday", "wielkanoc|niedziela wielkanocna", easter_sunday()),
        (
            "Easter Monday",
            "poniedzia(l|ł)ek wielkanocny",
            cycle_nth_after(false, G::Day, 1, &easter_sunday()),
        ),
        (
            "Pentecost",
            "zielone (s|ś)wi(a|ą)tki|zes(l|ł)anie ducha (ś|s)wi(e|ę)tego",
            cycle_nth_after(false, G::Day, 49, &easter_sunday()),
        ),
        (
            "Corpus Christi",
            "bo(z|ż)e cia(l|ł)o",
            cycle_nth_after(false, G::Day, 60, &easter_sunday()),
        ),
    ])
}

fn build_rules() -> Vec<Rule> {
    let mut v = rules_main();
    v.extend(rules_days_of_week());
    v.extend(rules_months());
    v.extend(rules_holidays());
    v.extend(rules_seasons());
    v.extend(rules_computed_holidays());
    v
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &RULES
}
