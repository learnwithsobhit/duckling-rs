//! `Duckling.Time.DE.Rules`.

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

/// The `TimeData` of a `Token Time`.
fn td(t: &Token) -> Option<&TimeData> {
    match t {
        Token::Time(d) => Some(d),
        _ => None,
    }
}

/// Group `i` of a regex-match token.
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

fn day_part(from: i64, to: i64) -> Option<TimeData> {
    interval(Open, &hour(false, from), &hour(false, to))
}

/// `Atto.parseOnly (Atto.signed Atto.decimal)`: leftover input is kept, so `"10."` is `10`.
fn parse_int_prefix(s: &str) -> Option<i64> {
    let (sign, rest) = match s.as_bytes().first() {
        Some(b'+') => (1, &s[1..]),
        Some(b'-') => (-1, &s[1..]),
        _ => (1, s),
    };
    let n = rest.bytes().take_while(|c| c.is_ascii_digit()).count();
    if n == 0 {
        return None;
    }
    parse_int(&rest[..n]).map(|v| v * sign)
}

const TIMEZONE_NAME: &str = "YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT";

fn rules_instants() -> Vec<Rule> {
    mk_rule_instants(&[
        ("now", G::Second, 0, "(genau)? ?jetzt|diesen moment|in diesem moment|gerade eben"),
        (
            "today",
            G::Day,
            0,
            "heute|(um diese zeit|zu dieser zeit|um diesen zeitpunkt|zu diesem zeitpunkt)",
        ),
        ("tomorrow", G::Day, 1, "morgen"),
        ("yesterday", G::Day, -1, "gestern"),
        ("after tomorrow", G::Day, 2, "(ü)bermorgen"),
        ("before yesterday", G::Day, -2, "vorgestern"),
        ("3 days ago", G::Day, -3, "vorvorgestern"),
        ("EOM|End of month", G::Month, 1, "(das )?ende des monats?"),
        ("EOY|End of year", G::Year, 1, "(das )?(EOY|jahr(es)? ?ende|ende (des )?jahr(es)?)"),
    ])
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Montag", "montags?|mo\\.?" ),
        ("Dienstag", "die?nstags?|di\\.?" ),
        ("Mittwoch", "mittwochs?|mi\\.?" ),
        ("Donnerstag", "donn?erstags?|do\\.?" ),
        ("Freitag", "freitags?|fr\\.?" ),
        ("Samstag", "samstags?|sonnabends?|sa\\.?" ),
        ("Sonntag", "sonntags?|so\\." ),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("Januar", "januar|jan\\.?" ),
        ("Februar", "februar|feb\\.?" ),
        ("Marz", "m(ä)rz|m(ä)r\\.?" ),
        ("April", "april|apr\\.?" ),
        ("Mai", "mai\\.?" ),
        ("Juni", "juni|jun\\.?" ),
        ("Juli", "juli|jul\\.?" ),
        ("August", "august|aug\\.?" ),
        ("September", "september|sept?\\.?" ),
        ("Oktober", "oktober|okt\\.?" ),
        ("November", "november|nov\\.?" ),
        ("Dezember", "dezember|dez\\.?" ),
    ])
}

fn rules_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("sommer", "sommer", month_day(6, 21), month_day(9, 23)),
        ("herbst", "herbst", month_day(9, 23), month_day(12, 21)),
        ("winter", "winter", month_day(12, 21), month_day(3, 20)),
        ("fruhling", "fr(ü)h(ling|jahr)", month_day(3, 20), month_day(6, 21)),
    ])
}

fn rules_main() -> Vec<Rule> {
    vec![
        // ruleADuration
        rule("a <duration>", vec![regex("(in )?eine?(r|n)?"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        // ruleAboutTimeofday
        rule(
            "about <time-of-day>",
            vec![
                regex("so( um)?|(so |um |so um )?circa|zirka|ca\\.?|ungef(ä)hr|(etwa|gegen)( so| um| so um)?"),
                pred(is_a_time_of_day),
            ],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleAbsorptionOfAfterNamedDay
        rule(
            "absorption of , after named day",
            vec![pred(is_a_day_of_week), regex(",")],
            |ts| Some(ts[0].clone()),
        ),
        // ruleAfterDuration
        rule("after <duration>", vec![regex("nach"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        // ruleAfterLunch
        rule("after lunch", vec![regex("nach dem mittagessen|nachmittags?")], |_| {
            let td2 = interval(Open, &hour(false, 13), &hour(false, 17))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleAfterNextTime
        rule(
            "after next <time>",
            vec![regex("(ü)ber ?n(ä)chste[ns]?"), dimension(Dim::Time)],
            |ts| tt(pred_nth(1, true, td(ts[1])?)),
        ),
        // ruleAfterTimeofday
        rule(
            "after <time-of-day>",
            vec![regex("nach|ab|fr(ü)he?stens"), dimension(Dim::Time)],
            |ts| tt(with_direction(IntervalDirection::After, td(ts[1])?.clone())),
        ),
        // ruleAfterTimeofdayPostfix
        rule(
            "<time-of-day> after",
            vec![dimension(Dim::Time), regex("fr(ü)he?stens")],
            |ts| tt(with_direction(IntervalDirection::After, td(ts[0])?.clone())),
        ),
        // ruleAfterWork
        rule("after work", vec![regex("nach (der)? arbeit|(am)? feier ?abend")], |_| {
            let td2 = interval(Open, &hour(false, 17), &hour(false, 21))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleAfternoon
        rule("afternoon", vec![regex("nach ?mittags?")], |_| {
            ttd(day_part(12, 19).map(|t| mk_latent(part_of_day(t))))
        }),
        // ruleAtTimeofday
        rule("at <time-of-day>", vec![regex("um|@"), pred(is_a_time_of_day)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        // ruleBetweenDatetimeAndDatetimeInterval
        rule(
            "between <datetime> and <datetime> (interval)",
            vec![regex("zwischen"), dimension(Dim::Time), regex("und"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleBetweenTimeofdayAndTimeofdayInterval
        rule(
            "between <time-of-day> and <time-of-day> (interval)",
            vec![regex("zwischen"), pred(is_a_time_of_day), regex("und"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleByTheEndOfTime
        rule(
            "by the end of <time>",
            vec![regex("bis (zum)? ende (von)?|(noch)? vor"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, &now())),
        ),
        // ruleDatetimeDatetimeInterval
        rule(
            "<datetime> - <datetime> (interval)",
            vec![pred(is_not_latent), regex("\\-|bis( zum)?|auf( den)?"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleDateDateInterval
        rule(
            "dd.(mm.)? - dd.mm.(yy[yy]?)? (interval)",
            vec![
                regex("(?:vo[nm]\\s+)?(10|20|30|31|[012]?[1-9])\\.?((?<=\\.)(?:10|11|12|0?[1-9])(?:\\.?))?"),
                regex("\\-|/|bis( zum)?|auf( den)?"),
                regex("(10|20|30|31|[012]?[1-9])\\.(10|11|12|0?[1-9])\\.?((?<=\\.)\\d{2,4})?"),
            ],
            |ts| {
                let d1s = rx(ts[0], 0)?;
                let m1s = rx(ts[0], 1).unwrap_or_default();
                let d2s = rx(ts[2], 0)?;
                let m2s = rx(ts[2], 1)?;
                let ys = rx(ts[2], 2).unwrap_or_default();
                if m1s.is_empty() && ys.is_empty() {
                    let d1 = parse_int(&d1s)?;
                    let d2 = parse_int(&d2s)?;
                    let m2 = parse_int(&m2s)?;
                    ttd(interval(Closed, &month_day(m2, d1), &month_day(m2, d2)))
                } else if m1s.is_empty() {
                    let d1 = parse_int(&d1s)?;
                    let d2 = parse_int(&d2s)?;
                    let m2 = parse_int(&m2s)?;
                    let y = parse_int(&ys)?;
                    ttd(interval(Closed, &year_month_day(y, m2, d1), &year_month_day(y, m2, d2)))
                } else if ys.is_empty() {
                    let d1 = parse_int(&d1s)?;
                    let d2 = parse_int(&d2s)?;
                    let m1 = parse_int_prefix(&m1s)?;
                    let m2 = parse_int(&m2s)?;
                    ttd(interval(Closed, &month_day(m1, d1), &month_day(m2, d2)))
                } else {
                    let d1 = parse_int(&d1s)?;
                    let d2 = parse_int(&d2s)?;
                    let m1 = parse_int_prefix(&m1s)?;
                    let m2 = parse_int(&m2s)?;
                    let y = parse_int(&ys)?;
                    ttd(interval(Closed, &year_month_day(y, m1, d1), &year_month_day(y, m2, d2)))
                }
            },
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
            vec![pred(is_dom_integer), regex("vom|von"), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
        ),
        // ruleDayofmonthOrdinal
        rule("<day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
            tt(day_of_month(ordinal_value(ts[0])?))
        }),
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
                let dom = intersect_dom(td(ts[1])?, ts[0])?;
                ttd(intersect(&dom, &year(n)))
            },
        ),
        // ruleDurationAfterTime
        rule(
            "<duration> after <time>",
            vec![dimension(Dim::Duration), regex("nach"), dimension(Dim::Time)],
            |ts| tt(duration_after(dur(ts[0])?, td(ts[2])?)),
        ),
        // ruleDurationAgo
        rule("<duration> ago", vec![regex("vor"), dimension(Dim::Duration)], |ts| {
            tt(duration_ago(dur(ts[1])?))
        }),
        // ruleDurationBeforeTime
        rule(
            "<duration> before <time>",
            vec![dimension(Dim::Duration), regex("vor"), dimension(Dim::Time)],
            |ts| tt(duration_before(dur(ts[0])?, td(ts[2])?)),
        ),
        // ruleDurationFromNow
        rule(
            "<duration> from now",
            vec![dimension(Dim::Duration), regex("ab (heute|jetzt)")],
            |ts| tt(in_duration(dur(ts[0])?)),
        ),
        // ruleDurationHence
        rule("<duration> hence", vec![dimension(Dim::Duration), regex("hence")], |ts| {
            tt(in_duration(dur(ts[0])?))
        }),
        // ruleEvening
        rule("evening", vec![regex("abends?")], |_| {
            ttd(day_part(18, 0).map(|t| mk_latent(part_of_day(t))))
        }),
        // ruleExactlyTimeofday
        rule(
            "exactly <time-of-day>",
            vec![regex("genau|exakt|p(ü)nktlich|punkt( um)?"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleFromDatetimeDatetimeInterval
        rule(
            "from <datetime> - <datetime> (interval)",
            vec![
                regex("vo[nm]"),
                dimension(Dim::Time),
                regex("\\-|bis( zum)?|auf( den)?"),
                dimension(Dim::Time),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleFromTimeofdayTimeofdayInterval
        rule(
            "from <time-of-day> - <time-of-day> (interval)",
            vec![
                regex("(von|nach|ab|fr(ü)hestens (um)?)"),
                pred(is_a_time_of_day),
                regex("((noch|aber|jedoch)? vor)|\\-|bis"),
                pred(is_a_time_of_day),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleHalfIntegerGermanStyleHourofday
        rule(
            "half <integer> (german style hour-of-day)",
            vec![regex("halb"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(30, td(ts[1])?)),
        ),
        // ruleHhmm
        rule(
            "hh:mm",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.h]([0-5]\\d)(?:uhr|h)?")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(hour_minute(false, h, m))
            },
        ),
        // ruleHhmmMilitary
        rule("hhmm (military)", vec![regex("((?:[01]?\\d)|(?:2[0-3]))([0-5]\\d)")], |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(mk_latent(hour_minute(false, h, m)))
        }),
        // ruleHhmmMilitaryAmpm
        rule(
            "hhmm (military) am|pm",
            vec![
                regex("((?:1[012]|0?\\d))([0-5]\\d)"),
                regex("([ap])\\.?m\\.?(?:[\\s'\"-_{}\\[\\]()]|$)"),
            ],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let ap = to_lower(&rx(ts[1], 0)?);
                tt(time_of_day_ampm(ap == "a", &hour_minute(true, h, m)))
            },
        ),
        // ruleHourofdayIntegerAsRelativeMinutes
        rule(
            "<hour-of-day> <integer> (as relative minutes)",
            vec![pred(not_latent_hour), pred(is_integer_between_tok(1, 59))],
            |ts| match &td(ts[0])?.form {
                Some(Form::TimeOfDay { hours: Some(h), is12h }) => {
                    let n = get_int_value(ts[1])?;
                    tt(hour_minute(*is12h, *h, n))
                }
                _ => None,
            },
        ),
        // ruleInDuration
        rule("in <duration>", vec![regex("in"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        // ruleInduringThePartofday
        rule(
            "in|during the <part-of-day>",
            vec![regex("(in|an|am|w(ä)h?rend)( der| dem| des)?"), pred(is_a_part_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleIntersect
        rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        // ruleIntersectBy
        rule(
            "intersect by ','",
            vec![pred(is_not_latent), regex(",( den|r)?"), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleIntersectByOfFromS
        rule(
            "intersect by 'of', 'from', 's",
            vec![pred(is_not_latent), regex("von|der|im"), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleDayOfWeekIntersectDuration
        rule(
            "<day-of-week> in <duration>",
            vec![pred(is_a_day_of_week), regex("(in|vor)"), dimension(Dim::Duration)],
            |ts| {
                let match_s = to_lower(&rx(ts[1], 0)?);
                if match_s == "vor" {
                    ttd(intersect(td(ts[0])?, &duration_interval_ago(dur(ts[2])?)))
                } else {
                    ttd(intersect(td(ts[0])?, &in_duration_interval(dur(ts[2])?)))
                }
            },
        ),
        // ruleLastCycle
        rule(
            "last <cycle>",
            vec![regex("letzte(r|n|s)?|vergangene(r|n|s)?"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
        ),
        // ruleLastCycleOfTime
        rule(
            "last <cycle> of <time>",
            vec![regex("letzte(r|n|s)?"), dimension(Dim::TimeGrain), regex("um|im"), dimension(Dim::Time)],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastDayofweekOfTime
        rule(
            "last <day-of-week> of <time>",
            vec![regex("letzte(r|n|s)?"), pred(is_a_day_of_week), regex("[ui]m"), dimension(Dim::Time)],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastNCycle
        rule(
            "last n <cycle>",
            vec![
                regex("letzten?|vergangenen?"),
                pred(is_integer_between_tok(1, 9999)),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -n))
            },
        ),
        // ruleLastTime
        rule("last <time>", vec![regex("letzten?|letztes"), pred(is_ok_with_this_next)], |ts| {
            tt(pred_nth(-1, false, td(ts[1])?))
        }),
        // ruleLunch
        rule("lunch", vec![regex("(am |zu )?mittags?")], |_| {
            ttd(day_part(12, 14).map(|t| mk_latent(part_of_day(t))))
        }),
        // ruleMidnighteodendOfDay
        rule(
            "midnight|EOD|end of day",
            vec![regex("mitternacht|EOD|tagesende|ende (des)? tag(es)?")],
            |_| tt(hour(false, 0)),
        ),
        // ruleMmdd
        rule(
            "mm/dd",
            vec![regex("(?:am\\s+)?([012]?[1-9]|10|20|30|31)\\.(10|11|12|0?[1-9])\\.?")],
            |ts| {
                let d = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(month_day(m, d))
            },
        ),
        // ruleMmddyyyy
        rule(
            "mm/dd/yyyy",
            vec![regex("([012]?[1-9]|10|20|30|31)\\.(0?[1-9]|10|11|12)\\.(\\d{2,4})")],
            |ts| {
                let y = parse_int(&rx(ts[0], 2)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let d = parse_int(&rx(ts[0], 0)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        // ruleMonthDdddInterval
        rule(
            "<month> dd-dd (interval)",
            vec![
                regex("([012]?\\d|30|31)(ter|\\.)?"),
                regex("\\-|bis( zum)?|auf( den)?"),
                regex("([012]?\\d|30|31)(ter|\\.)?"),
                pred(is_a_month),
            ],
            |ts| {
                let v1 = parse_int(&rx(ts[0], 0)?)?;
                let v2 = parse_int(&rx(ts[2], 0)?)?;
                let m = td(ts[3])?;
                let from = intersect(&day_of_month(v1), m)?;
                let to = intersect(&day_of_month(v2), m)?;
                ttd(interval(Closed, &from, &to))
            },
        ),
        // ruleMorning
        rule("morning", vec![regex("morgens|(in der )?fr(ü)h|vor ?mittags?|am morgen")], |_| {
            ttd(day_part(3, 12).map(|t| mk_latent(part_of_day(t))))
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
            vec![regex("n(ä)chste(r|n|s)?|kommende(r|n|s)?"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 1)),
        ),
        // ruleAfterNextCycle
        rule(
            "after next <cycle>",
            vec![regex("(ü)ber ?n(ä)chste[ns]?"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 2)),
        ),
        // ruleNextNCycle
        rule(
            "next n <cycle>",
            vec![
                regex("n(ä)chsten?|kommenden?"),
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
            vec![regex("(n(ä)chste|kommende)[ns]?"), pred(not_latent_ok_this_next)],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        // ruleNight
        rule("night", vec![regex("nachts?")], |_| {
            ttd(day_part(0, 4).map(|t| mk_latent(part_of_day(t))))
        }),
        // ruleNoon
        rule("noon", vec![regex("mittags?|zw(ö)lf (uhr)?")], |_| tt(hour(false, 12))),
        // ruleNthTimeAfterTime
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("nach"), dimension(Dim::Time)],
            |ts| tt(pred_nth_after(ordinal_value(ts[0])? - 1, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleNthTimeAfterTime2
        rule(
            "nth <time> after <time>",
            vec![
                regex("der|das"),
                dimension(Dim::Ordinal),
                dimension(Dim::Time),
                regex("nach"),
                dimension(Dim::Time),
            ],
            |ts| tt(pred_nth_after(ordinal_value(ts[1])? - 1, td(ts[2])?, td(ts[4])?)),
        ),
        // ruleNthTimeOfTime
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("im"), dimension(Dim::Time)],
            |ts| {
                let v = ordinal_value(ts[0])?;
                ttd(intersect(td(ts[3])?, td(ts[1])?).map(|t| pred_nth(v - 1, false, &t)))
            },
        ),
        // ruleNthTimeOfTime2
        rule(
            "nth <time> of <time>",
            vec![
                regex("der|die|das"),
                dimension(Dim::Ordinal),
                dimension(Dim::Time),
                regex("im"),
                dimension(Dim::Time),
            ],
            |ts| {
                let v = ordinal_value(ts[1])?;
                ttd(intersect(td(ts[4])?, td(ts[2])?).map(|t| pred_nth(v - 1, false, &t)))
            },
        ),
        // ruleOnANamedday
        rule("on a named-day", vec![regex("an einem"), pred(is_a_day_of_week)], |ts| {
            Some(ts[1].clone())
        }),
        // ruleOnDate
        rule("on <date>", vec![regex("am"), dimension(Dim::Time)], |ts| Some(ts[1].clone())),
        // ruleOrdinalCycleAfterTime
        rule(
            "<ordinal> <cycle> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("nach"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?)),
        ),
        // ruleOrdinalCycleOfTime
        rule(
            "<ordinal> <cycle> of <time>",
            vec![
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("im|in|von"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?)),
        ),
        // ruleOrdinalQuarter
        rule(
            "<ordinal> quarter",
            vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))],
            |ts| {
                tt(cycle_nth_after(
                    false,
                    G::Quarter,
                    ordinal_value(ts[0])? - 1,
                    &cycle_nth(G::Year, 0),
                ))
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
            vec![pred(is_a_part_of_day), regex("des|von|vom|am"), dimension(Dim::Time)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleRelativeMinutesAfterpastIntegerHourofday
        rule(
            "relative minutes after|past <integer> (hour-of-day)",
            vec![pred(is_integer_between_tok(1, 59)), regex("nach"), pred(is_an_hour_of_day)],
            |ts| match &td(ts[2])?.form {
                Some(Form::TimeOfDay { hours: Some(h), is12h }) => {
                    let n = get_int_value(ts[0])?;
                    tt(hour_minute(*is12h, *h, n))
                }
                _ => None,
            },
        ),
        // ruleRelativeMinutesTotillbeforeIntegerHourofday
        rule(
            "relative minutes to|till|before <integer> (hour-of-day)",
            vec![pred(is_integer_between_tok(1, 59)), regex("vor"), pred(is_an_hour_of_day)],
            |ts| {
                let n = get_int_value(ts[0])?;
                ttd(minutes_before(n, td(ts[2])?))
            },
        ),
        // ruleTheCycleAfterTime
        rule(
            "the <cycle> after <time>",
            vec![regex("der"), dimension(Dim::TimeGrain), regex("nach"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, 1, td(ts[3])?)),
        ),
        // ruleTheCycleBeforeTime
        rule(
            "the <cycle> before <time>",
            vec![regex("der"), dimension(Dim::TimeGrain), regex("vor"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, -1, td(ts[3])?)),
        ),
        // ruleTheDayofmonthNonOrdinal
        rule(
            "the <day-of-month> (non ordinal)",
            vec![regex("der"), pred(is_integer_between_tok(1, 31))],
            |ts| tt(day_of_month(get_int_value(ts[1])?)),
        ),
        // ruleTheDayofmonthOrdinal
        rule(
            "the <day-of-month> (ordinal)",
            vec![regex("der"), pred(is_dom_ordinal)],
            |ts| tt(day_of_month(ordinal_value(ts[1])?)),
        ),
        // ruleTheIdesOfNamedmonth
        rule(
            "the ides of <named-month>",
            vec![regex("die iden (des?)"), pred(is_a_month)],
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
                regex("nach"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[4])?)),
        ),
        // ruleTheOrdinalCycleOfTime
        rule(
            "the <ordinal> <cycle> of <time>",
            vec![
                regex("der|die|das"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("im|in|von"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[4])?)),
        ),
        // ruleThisCycle
        rule(
            "this <cycle>",
            vec![regex("diese(r|n|s)?|kommende(r|n|s)?"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
        ),
        // ruleThisPartofday
        rule(
            "this <part-of-day>",
            vec![regex("diesen?|dieses|heute"), pred(is_a_part_of_day)],
            |ts| ttd(intersect(&today(), td(ts[1])?).map(part_of_day)),
        ),
        // ruleThisTime
        rule(
            "this <time>",
            vec![regex("diese(n|r|s)?|(im )?laufenden"), pred(is_ok_with_this_next)],
            |ts| tt(pred_nth(0, false, td(ts[1])?)),
        ),
        // ruleThisnextDayofweek
        rule(
            "this|next <day-of-week>",
            vec![regex("diese(n|r)|kommenden|n(ä)chsten"), pred(is_a_day_of_week)],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        // ruleTimeAfterNext
        rule(
            "<time> after next",
            vec![dimension(Dim::Time), regex("nach dem n(ä)chsten")],
            |ts| tt(pred_nth(1, true, td(ts[0])?)),
        ),
        // ruleTimeBeforeLast
        rule(
            "<time> before last",
            vec![regex("vorletzten?|vor ?letztes?"), dimension(Dim::Time)],
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
            vec![pred(is_a_time_of_day), regex("ca\\.?|circa|zirka|ungef(ä)hr|(in )?etwa")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        // ruleTimeofdayLatent
        rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
            let n = get_int_value(ts[0])?;
            tt(mk_latent(hour(n < 12, n)))
        }),
        // ruleTimeofdayOclock
        rule(
            "<time-of-day>  o'clock",
            vec![pred(is_a_time_of_day), regex("uhr|h(?:[\\s'\"-_{}\\[\\]()]|$)")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        // ruleTimeofdaySharp
        rule(
            "<time-of-day> sharp",
            vec![pred(is_a_time_of_day), regex("genau|exakt|p(ü)nktlich|punkt( um)?")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        // ruleTimeofdayTimeofdayInterval
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(not_latent_tod), regex("\\-|bis"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleTimeofdayTimeofdayInterval2
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(is_a_time_of_day), regex("\\-|/|bis"), pred(not_latent_tod)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleTonight
        rule("tonight", vec![regex("heute? (am)? abends?")], |_| {
            let td2 = interval(Open, &hour(false, 18), &hour(false, 0))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleUntilTimeofday
        rule(
            "until <time-of-day>",
            vec![regex("vor|bis( zu[rm]?)?|sp(ä)testens?"), dimension(Dim::Time)],
            |ts| tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone())),
        ),
        // ruleUntilTimeofdayPostfix
        rule(
            "<time-of-day> until",
            vec![dimension(Dim::Time), regex("sp(ä)testens")],
            |ts| tt(with_direction(IntervalDirection::Before, td(ts[0])?.clone())),
        ),
        // ruleWeekend
        rule("week-end", vec![regex("wochen ?ende?")], |_| tt(mk_ok_for_this_next(weekend()))),
        // ruleWithinDuration
        rule(
            "within <duration>",
            vec![regex("binnen|innerhalb( von)?"), dimension(Dim::Duration)],
            |ts| ttd(interval(Open, &now(), &in_duration(dur(ts[1])?))),
        ),
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
        // ruleYyyymmdd
        rule(
            "yyyy-mm-dd",
            vec![regex("(\\d{2,4})-(1[0-2]|0?[1-9])-(3[01]|[12]\\d|0?[1-9])")],
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
            vec![regex("vie?rtel vor"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(15, td(ts[1])?)),
        ),
        // ruleHalfTotillbeforeIntegerHourofday
        rule(
            "half to|till|before <integer> (hour-of-day)",
            vec![regex("halbe? vor"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(30, td(ts[1])?)),
        ),
        // ruleQuarterAfterpastIntegerHourofday
        rule(
            "quarter after|past <integer> (hour-of-day)",
            vec![regex("vie?rtel nach"), pred(is_an_hour_of_day)],
            |ts| match &td(ts[1])?.form {
                Some(Form::TimeOfDay { hours: Some(h), is12h }) => tt(hour_minute(*is12h, *h, 15)),
                _ => None,
            },
        ),
        // ruleHalfAfterpastIntegerHourofday
        rule(
            "half after|past <integer> (hour-of-day)",
            vec![regex("halbe? nach"), pred(is_an_hour_of_day)],
            |ts| match &td(ts[1])?.form {
                Some(Form::TimeOfDay { hours: Some(h), is12h }) => tt(hour_minute(*is12h, *h, 30)),
                _ => None,
            },
        ),
        // ruleHourofdayQuarter
        rule(
            "<hour-of-day> <quarter> (as relative minutes)",
            vec![pred(is_an_hour_of_day), regex("vie?rtel")],
            |ts| match &td(ts[0])?.form {
                Some(Form::TimeOfDay { hours: Some(h), is12h }) => tt(hour_minute(*is12h, *h, 15)),
                _ => None,
            },
        ),
        // ruleHourofdayHalf
        rule(
            "<hour-of-day> <half> (as relative minutes)",
            vec![pred(is_an_hour_of_day), regex("halbe?")],
            |ts| match &td(ts[0])?.form {
                Some(Form::TimeOfDay { hours: Some(h), is12h }) => tt(hour_minute(*is12h, *h, 30)),
                _ => None,
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
    v.extend(rules_instants());
    v.extend(rules_days_of_week());
    v.extend(rules_months());
    v.extend(rules_seasons());
    v.extend(holidays::holidays());
    v.extend(holidays::computed_holidays());
    v.extend(holidays::computed_holidays_prime());
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
