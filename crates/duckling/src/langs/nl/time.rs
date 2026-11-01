//! `Duckling.Time.NL.Rules`, `Duckling.Time.NL.BE.Rules` and `Duckling.Time.NL.NL.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::{duration, DurationData};
use crate::dims::time::helpers::*;
use crate::dims::time::holiday_helpers::compute_kings_day;
use crate::dims::time::types::{
    Form, IntervalDirection, TimeData,
    TimeIntervalType::{Closed, Open},
};
use crate::dims::time_grain::Grain as G;
use crate::dims::util::{parse_int, to_lower, to_upper};
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

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

fn numeral_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
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

fn ok_not_tod(t: &Token) -> bool {
    is_ok_with_this_next(t) && !is_a_time_of_day(t)
}

fn day_part(from: i64, to: i64) -> Option<TimeData> {
    interval(Open, &hour(false, from), &hour(false, to))
}

/// `day offset` of "today/tomorrow/yesterday ..." rules.
fn part_of_day_of(day: TimeData, from: i64, to: i64) -> Option<Token> {
    let td2 = day_part(from, to)?;
    ttd(intersect(&day, &td2).map(part_of_day))
}

const TIMEZONE_NAME: &str = "YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT";

const AMPM_SUFFIX: &str = "([ap])\\.?m\\.?(?:[\\s'\"-_{}\\[\\]()]|$)";

fn rules_instants() -> Vec<Rule> {
    mk_rule_instants(&[
        ("now", G::Second, 0, "meteen|nu|direct|zojuist"),
        ("today", G::Day, 0, "vandaag|op deze dag"),
        ("tomorrow", G::Day, 1, "morgen"),
        ("yesterday", G::Day, -1, "gisteren"),
        ("after tomorrow", G::Day, 2, "overmorgen"),
        ("before yesterday", G::Day, -2, "eergisteren"),
        ("EOM|End of month", G::Month, 1, "(het )?einde? van de maand"),
        ("EOY|End of year", G::Year, 1, "(het )? einde? van het jaar"),
    ])
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("monday", "maandags?|ma\\."),
        ("tuesday", "dinsdags?|di\\."),
        ("wednesday", "woensdags?|woe\\."),
        ("thursday", "donderdags?|do\\."),
        ("friday", "vrijdags?|vr(ij)?\\."),
        ("saturday", "zaterdags?|zat?\\."),
        ("sunday", "zondags?|zon?\\."),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("January", "januari|jan\\.?"),
        ("February", "februari|feb\\.?"),
        ("March", "maart|mar\\.?"),
        ("April", "april|apr\\.?"),
        ("May", "mei\\.?"),
        ("June", "juni?\\.?"),
        ("July", "juli?\\.?"),
        ("August", "augustus|aug\\.?"),
        ("September", "september|sept?\\.?"),
        ("October", "oktober|okt\\.?"),
        ("November", "november|nov\\.?"),
        ("December", "december|dec\\.?"),
    ])
}

fn rules_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("summer", "zomer", month_day(6, 21), month_day(9, 23)),
        ("fall", "herfst|najaar", month_day(9, 23), month_day(12, 21)),
        ("winter", "winter", month_day(12, 21), month_day(3, 20)),
        ("spring", "lente|voorjaar", month_day(3, 20), month_day(6, 21)),
    ])
}

fn rules_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("Nieuwjaarsdag", "nieuwjaars?(dag)?", month_day(1, 1)),
        ("Valentijnsdag", "valentijns?(dag)?", month_day(2, 14)),
        ("Halloween", "hall?oween?", month_day(10, 31)),
        ("Allerheiligen", "allerheiligen?|aller heiligen?", month_day(11, 1)),
        ("Kerstavond", "kerstavond", month_day(12, 24)),
        ("Tweede Kerstdag", "tweede kerstdag", month_day(12, 26)),
        ("Kerstmis", "kerstmis|(eerste )?kerstdag|kerst", month_day(12, 25)),
        ("Oudjaar", "oudjaar|oudejaars?avond", month_day(12, 31)),
        ("Moederdag", "moederdag", nth_dow_of_month(2, 7, 5)),
        ("Vaderdag", "vaderdag", nth_dow_of_month(3, 7, 6)),
    ])
}

fn rules_computed_holidays() -> Vec<Rule> {
    mk_rule_holidays_maybe(&[("Koningsdag", "king's day|koningsdag", compute_kings_day())])
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
            vec![regex("ongeveer|circa|ca\\.|tegen|rond"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleAbsorptionOfAfterNamedDay
        rule(
            "absorption of , after named day",
            vec![pred(is_a_day_of_week), regex(",")],
            |ts| Some(ts[0].clone()),
        ),
        // ruleAfterDuration
        rule("after <duration>", vec![regex("over|na"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        // ruleAfterLunch
        rule("after lunch", vec![regex("na de lunch|na het middageten")], |_| {
            part_of_day_of(today(), 13, 17)
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
            vec![regex("na|vanaf|op zijn vroegst"), dimension(Dim::Time)],
            |ts| tt(with_direction(IntervalDirection::After, td(ts[1])?.clone())),
        ),
        // ruleAfterTimeofdayPostfix
        rule(
            "<time-of-day> after",
            vec![dimension(Dim::Time), regex("op zijn vroegst")],
            |ts| tt(with_direction(IntervalDirection::After, td(ts[0])?.clone())),
        ),
        // ruleAfterWork
        rule("after work", vec![regex("na het werk")], |_| part_of_day_of(today(), 17, 21)),
        // ruleAfternoon
        rule(
            "afternoon",
            vec![regex("('s )?middags?|(in de )?middag|namiddag")],
            |_| ttd(day_part(12, 18).map(|t| mk_latent(part_of_day(t)))),
        ),
        // ruleAfternoons
        rule(
            "today/tomorrow/yesterday afternoon",
            vec![regex("(vanmiddag|morgenmiddags?|gistermiddag|gisteren(na)?middag)")],
            |ts| {
                let td1 = match to_lower(&rx(ts[0], 0)?).as_str() {
                    "vanmiddag" => today(),
                    "morgenmiddag" => cycle_nth(G::Day, 1),
                    "morgenmiddags" => cycle_nth(G::Day, 1),
                    _ => cycle_nth(G::Day, -1),
                };
                part_of_day_of(td1, 12, 18)
            },
        ),
        // ruleAtTimeofday
        rule("at <time-of-day>", vec![regex("om|@"), pred(is_a_time_of_day)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        // ruleBetweenDatetimeAndDatetimeInterval
        rule(
            "between <datetime> and <datetime> (interval)",
            vec![regex("tussen"), dimension(Dim::Time), regex("en"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleBetweenTimeofdayAndTimeofdayInterval
        rule(
            "between <time-of-day> and <time-of-day> (interval)",
            vec![regex("tussen"), pred(is_a_time_of_day), regex("en"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleByTheEndOfTime
        rule(
            "by the end of <time>",
            vec![regex("tot( het)? einde( van)?"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, &now(), td(ts[1])?)),
        ),
        // ruleDatetimeDatetimeInterval
        rule(
            "<datetime> - <datetime> (interval)",
            vec![pred(is_not_latent), regex("\\-|t\\/m|tot en met"), pred(is_not_latent)],
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
            vec![pred(is_dom_integer), regex("van"), pred(is_a_month)],
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
            vec![dimension(Dim::Duration), regex("na"), dimension(Dim::Time)],
            |ts| tt(duration_after(dur(ts[0])?, td(ts[2])?)),
        ),
        // ruleDurationHenceAgo
        rule(
            "<duration> hence|ago",
            vec![dimension(Dim::Duration), regex("(later|geleden)")],
            |ts| {
                let dd = dur(ts[0])?;
                match to_lower(&rx(ts[1], 0)?).as_str() {
                    "geleden" => tt(duration_ago(dd)),
                    _ => tt(in_duration(dd)),
                }
            },
        ),
        // ruleDurationBeforeTime
        rule(
            "<duration> before <time>",
            vec![dimension(Dim::Duration), regex("voor"), dimension(Dim::Time)],
            |ts| tt(duration_before(dur(ts[0])?, td(ts[2])?)),
        ),
        // ruleDurationFromNow
        rule(
            "<duration> from now",
            vec![dimension(Dim::Duration), regex("vanaf (vandaag|nu)")],
            |ts| tt(in_duration(dur(ts[0])?)),
        ),
        // ruleEvening
        rule("evening", vec![regex("\\'s avonds|avond")], |_| {
            ttd(day_part_open(18, 0).map(|t| mk_latent(part_of_day(t))))
        }),
        // ruleEvenings
        rule(
            "today/tomorrow/yesterday evening",
            vec![regex("(van|morgen|gister(en)?)avond")],
            |ts| {
                let td1 = match to_lower(&rx(ts[0], 0)?).as_str() {
                    "van" => today(),
                    "morgen" => cycle_nth(G::Day, 1),
                    _ => cycle_nth(G::Day, -1),
                };
                part_of_day_of(td1, 18, 0)
            },
        ),
        // ruleExactlyTimeofday
        rule(
            "exactly <time-of-day>",
            vec![regex("(stipt|exact|precies)( om)?"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleFromDatetimeDatetimeInterval
        rule(
            "from <datetime> - <datetime> (interval)",
            vec![regex("van"), dimension(Dim::Time), regex("\\-|tot"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleFromTimeofdayTimeofdayInterval
        rule(
            "from <time-of-day> - <time-of-day> (interval)",
            vec![
                regex("(van|vanaf|na|vroegst om|om)"),
                pred(is_a_time_of_day),
                regex("((maar)? voor)|\\-|tot"),
                pred(is_a_time_of_day),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleHhmm
        rule(
            "hh:mm",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:hu]([0-5]\\d)(?:uur|u|h)?")],
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
            vec![regex("((?:1[012]|0?\\d))([0-5]\\d)"), regex(AMPM_SUFFIX)],
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
        rule("in <duration>", vec![regex("in|over"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        // ruleInduringThePartofday
        rule(
            "in|during the <part-of-day>",
            vec![regex("(tijdens|gedurende?)( de)?"), pred(is_a_part_of_day)],
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
        // ruleLastCycle
        rule("last <cycle>", vec![regex("vorige?|afgelopen"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, -1))
        }),
        // ruleLastCycleOfTime
        rule(
            "last <cycle> of <time>",
            vec![
                regex("laatste|vorige|afgelopen"),
                dimension(Dim::TimeGrain),
                regex("van|in"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastDayofweekOfTime
        rule(
            "last <day-of-week> of <time>",
            vec![regex("laatste"), pred(is_a_day_of_week), regex("in|van"), dimension(Dim::Time)],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastNCycle
        rule(
            "last n <cycle>",
            vec![
                regex("laatste|afgelopen|vorige?"),
                pred(is_integer_between_tok(1, 9999)),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -n))
            },
        ),
        // ruleLastTime
        rule("last <time>", vec![regex("afgelopen|vorige?"), pred(is_ok_with_this_next)], |ts| {
            tt(pred_nth(-1, false, td(ts[1])?))
        }),
        // ruleLunch
        rule("lunch", vec![regex("(am |zu )?mittags?")], |_| {
            ttd(day_part_open(12, 14).map(|t| mk_latent(part_of_day(t))))
        }),
        // ruleMidnighteodendOfDay
        rule(
            "midnight|EOD|end of day",
            vec![regex("middernacht|EOD|einde? van de dag")],
            |_| tt(hour(false, 0)),
        ),
        // ruleMmdd
        rule(
            "dd/mm",
            vec![regex("(3[01]|[12]\\d|0?[1-9])\\s?[/.-]\\s?(1[0-2]|0?[1-9])")],
            |ts| {
                let d = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(month_day(m, d))
            },
        ),
        // ruleMmddyyyy
        rule(
            "dd/mm/yyyy",
            vec![regex("(3[01]|[12]\\d|0?[1-9])[/.-](1[0-2]|0?[1-9])[/.-](\\d{2,4})")],
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
                regex("([012]?\\d|30|31)(ste|de)?"),
                regex("\\-|tot en met|t\\/m"),
                regex("([012]?\\d|30|31)(ste|de)?"),
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
        rule(
            "morning",
            vec![regex("(in de )?morgen( van)?|(in de )?ochtend|\\'s ochtends|\\'s morgens")],
            |_| ttd(day_part_open(0, 12).map(|t| mk_latent(part_of_day(t)))),
        ),
        // ruleMornings
        rule(
            "today/tomorrow/yesterday morning",
            vec![regex("(vanmorgen|morgenochtend|gisterenochtend)")],
            |ts| {
                let td1 = match to_lower(&rx(ts[0], 0)?).as_str() {
                    "vanmorgen" => today(),
                    "morgenochtend" => cycle_nth(G::Day, 1),
                    _ => cycle_nth(G::Day, -1),
                };
                part_of_day_of(td1, 0, 12)
            },
        ),
        // ruleNamedmonthDayofmonthNonOrdinal
        rule(
            "<named-month> <day-of-month> (non ordinal)",
            vec![pred(is_a_month), pred(is_dom_integer)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        // ruleNamedmonthDayofmonthOrdinal (upstream: first token is an ordinal, so this never fires)
        rule(
            "<day-of-month> (ordinal) <named-month>",
            vec![pred(is_dom_ordinal), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        // ruleNextCycle
        rule(
            "next <cycle>",
            vec![regex("aanstaande|komende?|volgende?"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 1)),
        ),
        // ruleNextNCycle
        rule(
            "next n <cycle>",
            vec![
                regex("(volgende?|komende?)"),
                pred(is_integer_between_tok(1, 9999)),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let v = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, v))
            },
        ),
        // ruleNextTime
        rule("next <time>", vec![regex("(volgende?|komende?)"), pred(ok_not_tod)], |ts| {
            tt(pred_nth(0, true, td(ts[1])?))
        }),
        // ruleNight
        rule("night", vec![regex("(\\'s nachts|(in de )?nacht)")], |_| {
            ttd(day_part_open(20, 8).map(|t| mk_latent(part_of_day(t))))
        }),
        // ruleNights
        rule("yesterday/tomorrow night", vec![regex("(morgen|gisteren)nacht")], |ts| {
            let td1 = match to_lower(&rx(ts[0], 0)?).as_str() {
                "morgen" => cycle_nth(G::Day, 1),
                _ => cycle_nth(G::Day, -1),
            };
            part_of_day_of(td1, 18, 0)
        }),
        // ruleNthTimeAfterTime
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("na"), dimension(Dim::Time)],
            |ts| tt(pred_nth_after(ordinal_value(ts[0])? - 1, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleNthTimeAfterTime2
        rule(
            "nth <time> after <time>",
            vec![
                regex("de|het"),
                dimension(Dim::Ordinal),
                dimension(Dim::Time),
                regex("na"),
                dimension(Dim::Time),
            ],
            |ts| tt(pred_nth_after(ordinal_value(ts[1])? - 1, td(ts[2])?, td(ts[4])?)),
        ),
        // ruleNthTimeOfTime
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("in|van"), dimension(Dim::Time)],
            |ts| {
                let v = ordinal_value(ts[0])?;
                intersect(td(ts[3])?, td(ts[1])?).map(|x| Token::Time(pred_nth(v - 1, false, &x)))
            },
        ),
        // ruleNthTimeOfTime2
        rule(
            "nth <time> of <time>",
            vec![
                regex("de|het"),
                dimension(Dim::Ordinal),
                dimension(Dim::Time),
                regex("in"),
                dimension(Dim::Time),
            ],
            |ts| {
                let v = ordinal_value(ts[1])?;
                intersect(td(ts[4])?, td(ts[2])?).map(|x| Token::Time(pred_nth(v - 1, false, &x)))
            },
        ),
        // ruleOnANamedday
        rule("on a named-day", vec![regex("op( een)?"), pred(is_a_day_of_week)], |ts| {
            Some(ts[1].clone())
        }),
        // ruleOnDate
        rule("on <date>", vec![regex("op( de)?"), dimension(Dim::Time)], |ts| Some(ts[1].clone())),
        // ruleOrdinalCycleAfterTime
        rule(
            "<ordinal> <cycle> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("na"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?)),
        ),
        // ruleOrdinalCycleOfTime
        rule(
            "<ordinal> <cycle> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("in|van"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?)),
        ),
        // ruleOrdinalQuarter
        rule(
            "<ordinal> quarter",
            vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))],
            |ts| {
                let v = ordinal_value(ts[0])?;
                tt(cycle_nth_after(false, G::Quarter, v - 1, &cycle_nth(G::Year, 0)))
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
            vec![pred(is_a_part_of_day), regex("(van|op|in)( de)?"), dimension(Dim::Time)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleRelativeMinutesToOrAfterIntegerPartOfDay
        rule(
            "relative minutes to|till|before|after <integer> (time-of-day)",
            vec![pred(is_integer_between_tok(1, 59)), regex("(voor|over|na)"), pred(is_a_time_of_day)],
            |ts| {
                let v = numeral_value(ts[0])?.floor() as i64;
                let t = td(ts[2])?;
                match to_lower(&rx(ts[1], 0)?).as_str() {
                    "voor" => tt(duration_before(&duration(G::Minute, v), t)),
                    _ => tt(duration_after(&duration(G::Minute, v), t)),
                }
            },
        ),
        // ruleTheCycleAfterTime
        rule(
            "the <cycle> after <time>",
            vec![regex("de|het"), dimension(Dim::TimeGrain), regex("na"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, 1, td(ts[3])?)),
        ),
        // ruleTheCycleBeforeTime
        rule(
            "the <cycle> before <time>",
            vec![regex("de"), dimension(Dim::TimeGrain), regex("voor"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, -1, td(ts[3])?)),
        ),
        // ruleTheDayofmonthNonOrdinal
        rule(
            "the <day-of-month> (non ordinal)",
            vec![regex("de"), pred(is_integer_between_tok(1, 31))],
            |ts| tt(day_of_month(get_int_value(ts[1])?)),
        ),
        // ruleTheDayofmonthOrdinal
        rule(
            "the <day-of-month> (ordinal)",
            vec![regex("der"), pred(is_dom_ordinal)],
            |ts| tt(day_of_month(ordinal_value(ts[1])?)),
        ),
        // ruleTheOrdinalCycleAfterTime
        rule(
            "the <ordinal> <cycle> after <time>",
            vec![
                regex("de|het"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("na"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[4])?)),
        ),
        // ruleTheOrdinalCycleOfTime
        rule(
            "the <ordinal> <cycle> of <time>",
            vec![
                regex("de|het"),
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("van|in"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[4])?)),
        ),
        // ruleThisCycle
        rule("this <cycle>", vec![regex("dit|deze|komende?"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, 0))
        }),
        // ruleThisPartofday
        rule(
            "this <part-of-day>",
            vec![regex("deze|dit"), pred(is_a_part_of_day)],
            |ts| ttd(intersect(&today(), td(ts[1])?).map(part_of_day)),
        ),
        // ruleThisTime
        rule(
            "this <time>",
            vec![regex("dit|deze|huidige?"), pred(is_ok_with_this_next)],
            |ts| tt(pred_nth(0, false, td(ts[1])?)),
        ),
        // ruleThisnextDayofweek
        rule(
            "this|next <day-of-week>",
            vec![regex("aanstaande|komende|volgende"), pred(is_a_day_of_week)],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        // ruleTimeAfterNext
        rule("<time> after next", vec![dimension(Dim::Time), regex("navolgend")], |ts| {
            tt(pred_nth(1, true, td(ts[0])?))
        }),
        // ruleTimePartofday
        rule(
            "<time> <part-of-day>",
            vec![dimension(Dim::Time), pred(is_a_part_of_day)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[1])?)),
        ),
        // ruleTimeofdayAmpm
        rule(
            "<time-of-day> am|pm",
            vec![pred(is_a_time_of_day), regex(AMPM_SUFFIX)],
            |ts| {
                let ap = to_lower(&rx(ts[1], 0)?);
                tt(time_of_day_ampm(ap == "a", td(ts[0])?))
            },
        ),
        // ruleTimeofdayApproximately
        rule(
            "<time-of-day> approximately",
            vec![pred(is_a_time_of_day), regex("ongeveer")],
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
            vec![pred(is_a_time_of_day), regex("uur|h|u(?:[\\s'\"-_{}\\[\\]()]|$)")],
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
            vec![pred(not_latent_tod), regex("\\-|tot"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleTimeofdayTimeofdayInterval2
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(is_a_time_of_day), regex("\\-|tot"), pred(not_latent_tod)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleUntilTimeofday
        rule(
            "until <time-of-day>",
            vec![regex("voor|tot|op zijn laatst om"), dimension(Dim::Time)],
            |ts| tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone())),
        ),
        // ruleUntilTimeofdayPostfix
        rule(
            "<time-of-day> until",
            vec![dimension(Dim::Time), regex("op zijn laatst")],
            |ts| tt(with_direction(IntervalDirection::Before, td(ts[0])?.clone())),
        ),
        // ruleWeekend
        rule("week-end", vec![regex("week(\\s|-)?ei?nde?")], |_| tt(mk_ok_for_this_next(weekend()))),
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
            vec![regex("(\\d{2,4})[.-](1[0-2]|0?[1-9])[.-](3[01]|[12]\\d|0?[1-9])")],
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
            vec![regex("kwart(ier)? voor"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(15, td(ts[1])?)),
        ),
        // ruleHalfTotillbeforeIntegerHourofday
        rule(
            "half to|till|before <integer> (hour-of-day)",
            vec![regex("half"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_before(30, td(ts[1])?)),
        ),
        // ruleQuarterAfterpastIntegerHourofday
        rule(
            "quarter after|past <integer> (hour-of-day)",
            vec![regex("kwart(ier)? over"), pred(is_an_hour_of_day)],
            |ts| match &td(ts[1])?.form {
                Some(Form::TimeOfDay { hours: Some(h), is12h }) => tt(hour_minute(*is12h, *h, 15)),
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

fn day_part_open(from: i64, to: i64) -> Option<TimeData> {
    day_part(from, to)
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = rules_main();
    v.extend(rules_instants());
    v.extend(rules_days_of_week());
    v.extend(rules_months());
    v.extend(rules_seasons());
    v.extend(rules_holidays());
    v.extend(rules_computed_holidays());
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}

/// `Duckling.Time.NL.BE.Rules.rules`.
static BE_RULES: Lazy<Vec<Rule>> =
    Lazy::new(|| mk_rule_holidays(&[("Sinterklaas", "sinterklaas", month_day(12, 6))]));

pub fn be_rules() -> &'static [Rule] {
    &BE_RULES
}

/// `Duckling.Time.NL.NL.Rules.rulesBackwardCompatible`.
static BACKWARD_COMPATIBLE: Lazy<Vec<Rule>> = Lazy::new(|| {
    mk_rule_holidays(&[("Sinterklaas", "sinterklaas(avond)?|pakjesavond", month_day(12, 5))])
});

pub fn rules_backward_compatible() -> &'static [Rule] {
    &BACKWARD_COMPATIBLE
}
