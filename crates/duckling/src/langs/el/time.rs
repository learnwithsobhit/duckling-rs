//! `Duckling.Time.EL.Rules`.

use once_cell::sync::Lazy;

use super::numeral::NUMERAL_MAP_EL;
use crate::dims::duration::{duration, DurationData};
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

/// `Duckling.Duration.Helpers.isGrain` — matches a `TimeGrain` token.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn not_latent_tod(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t)
}

fn dow_or_month(t: &Token) -> bool {
    is_a_day_of_week(t) || is_a_month(t)
}

fn latent_pod(a: i64, b: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, a), &hour(false, b)).map(|x| mk_latent(part_of_day(x))))
}

fn cycle_from_word(s: &str) -> Option<i64> {
    match to_lower(s).as_str() {
        "τρεχ" | "τρέχ" => Some(0),
        "ερχόμεν" | "ερχομέν" => Some(0),
        "επόμεν" | "επομέν" => Some(1),
        "περασμέν" => Some(-1),
        "προηγούμεν" | "προηγουμέν" => Some(-1),
        _ => None,
    }
}

fn lookup_el_num(s: &str) -> Option<i64> {
    NUMERAL_MAP_EL.get(to_lower(s).as_str()).copied()
}

const TIMEZONE_RE: &str = "\\b(YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT)\\b";

fn rules_instants() -> Vec<Rule> {
    mk_rule_instants(&[
        ("now", G::Second, 0, "(αμέσως\\s+)?τώρα(\\s+αμέσως)?|αυτή τη στιγμή"),
        ("today", G::Day, 0, "σήμερα"),
        ("tomorrow", G::Day, 1, "(επ)?αύριο"),
        ("yesterday", G::Day, -1, "ε?χ[θτ][εέ]ς"),
        ("after tomorrow", G::Day, 2, "μεθαύριο"),
        ("before yesterday", G::Day, -2, "προχ[θτ]ές"),
        ("EOD|End of day", G::Day, 1, "τέλου?ς\\s+της\\s+η?μέρας"),
        ("EOM|End of month", G::Month, 1, "τέλου?ς\\s+του\\s+μήνα"),
        ("EOY|End of year", G::Year, 1, "τέλου?ς\\s+του\\s+χρόνου"),
    ])
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("monday", "δευτ(έρας?|\\.?)"),
        ("tuesday", "τρ[ιί](της?|\\.?)"),
        ("wednesday", "τετ(άρτης?|\\.?)"),
        ("thursday", "π[εέ]μ(πτης?|\\.?)"),
        ("friday", "παρ(ασκευής?|\\.?)"),
        ("saturday", "σ[αά]β(β[αά]το[νυ]?|\\.?)"),
        ("sunday", "κυρ(ιακής?|\\.?)"),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("January", "ιαν(ου[αά]ρ[ιί]ο[υς]?)?|γενάρης?"),
        ("February", "φεβ(ρου[αά]ρ[ιί]ο[υς]?)?|φλεβάρης?"),
        ("March", "μ[αά]ρ(τ([ιί]ο([νυ]?)|η)ς?)?"),
        ("April", "απρ([ιί]λ([ιί]ο([νυ]?)|η)ς?)?"),
        ("May", "μ[αά]([ιίϊΐ]ο[νυ]?|η)ς?"),
        ("June", "ιο[υύ]ν([ιί]ο[υν]?|η)?ς?"),
        ("July", "ιο[υύ]λ([ιί]ο[υν]?|η)?ς?"),
        ("August", "α[υύ]γ(ο[υύ]στο(ν|υ|ς)?)?"),
        ("September", "σεπτ([εέ]μβρ([ιί]ο([νυ]?)|η)ς?)?"),
        ("October", "οκτ([ωώ]βρ([ιί]ο([νυ]?)|η)ς?)?"),
        ("November", "νο[εέ](μ(βρ([ιί]ο([νυ]?)|η)ς?)?)?"),
        ("December", "δεκ([εέ]μβρ([ιί]ο([νυ]?)|η)ς?)?"),
    ])
}

fn rules_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("summer", "καλοκα[ιί]ρι(ού)?", month_day(6, 21), month_day(9, 23)),
        ("fall", "φθιν[οό]π[ωώ]ρου?", month_day(9, 23), month_day(12, 21)),
        ("winter", "χειμώνας?", month_day(12, 21), month_day(3, 20)),
        ("spring", "άνοιξης?", month_day(3, 20), month_day(6, 21)),
    ])
}

fn rules_holidays() -> Vec<Rule> {
    // Haskell `ruleHolidays` uses `tt $ mkOkForThisNext td` (no `withHoliday`).
    [
        ("new year's day", month_day(1, 1), "πρωτοχρονιάς?"),
        ("valentine's day", month_day(2, 14), "αγίου\\s+βαλεντίνου"),
        ("halloween day", month_day(10, 31), "halloween"),
        ("Epiphany", month_day(1, 6), "θεοφ(αά)νε[ιί](α|ων)|φ[ωώ]τ(α|ων)"),
        ("annunciation day", month_day(3, 25), "ευαγγελισμ(ός|ού)\\s+της\\s+θεοτόκου"),
        ("revolution day", month_day(3, 25), "η?μέρα\\s+(της\\s+)?(ελληνικής\\s+)?επανάστασης"),
        ("assumption day", month_day(8, 15), "κο[ιί]μ[ηή]σ(η|ις|εως)\\s+της\\s+θεοτόκου"),
        ("christmas eve", month_day(12, 24), "παραμν(ή|έ)ς?\\s+χριστουγέννων"),
        ("christmas", month_day(12, 25), "χριστο[υύ]γ[εέ]νν(α|ων)"),
        ("new year's eve", month_day(12, 31), "παραμον(ή|έ)ς?\\s+πρωτοχρονιάς"),
        ("Mother's Day", nth_dow_of_month(2, 7, 5), "η?μέρας?\\s+της\\s+μητέρας"),
        ("Father's Day", nth_dow_of_month(3, 7, 6), "(γιορτής?|η?μέρας?)\\s+του\\s+πατέρα"),
    ]
    .into_iter()
    .map(|(name, td, pat)| mk_single_regex_rule(name, pat, Some(mk_ok_for_this_next(td))))
    .collect()
}

fn rules_main() -> Vec<Rule> {
    vec![
        // ruleADuration
        rule("a <duration>", vec![regex("[εέ]ν(ας?|ός|ν)|μίας?"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        // ruleAbsorptionOfAfterNamedDay
        rule(
            "absorption of , after named day",
            vec![pred(is_a_day_of_week), regex(",")],
            |ts| Some(ts[0].clone()),
        ),
        // ruleAbsorptionOfArticleBeforeTime
        rule(
            "absorption of article before time",
            vec![regex("σ?τ?(α|η|ι|ο[ιυ]?|ω)ν?ς?"), dimension(Dim::Time)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleAfterDuration
        rule("after <duration>", vec![regex("μετά\\s+από"), dimension(Dim::Duration)], |ts| {
            tt(with_direction(IntervalDirection::After, in_duration(dur(ts[1])?)))
        }),
        // ruleAfterNextTime
        rule(
            "after next <time>",
            vec![regex("μετά τ(ο|η)ν? επόμεν(ο|η)ν?"), dimension(Dim::Time)],
            |ts| tt(pred_nth(1, true, td(ts[1])?)),
        ),
        // ruleAfterTimeofday
        rule("after <time-of-day>", vec![regex("μετά"), dimension(Dim::Time)], |ts| {
            tt(with_direction(IntervalDirection::After, td(ts[1])?.clone()))
        }),
        // ruleAfterWork
        rule("after work", vec![regex("μετά τη δουλειά")], |_| {
            let td2 = interval(Open, &hour(false, 17), &hour(false, 21))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleAfternoon
        rule("afternoon", vec![regex("απ[οό]γε[υύ]μα(τος)?")], |_| latent_pod(12, 18)),
        // ruleAtTimeofday
        rule("at <time-of-day>", vec![regex("στις|@"), pred(is_a_time_of_day)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        // ruleBetweenDatetimeAndDatetimeInterval
        rule(
            "between <datetime> and <datetime> (interval)",
            vec![regex("μεταξύ|ανάμεσα"), pred(is_not_latent), regex("(και|\\-)"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleBetweenTimeofdayAndTimeofdayInterval
        rule(
            "between <time-of-day> and <time-of-day> (interval)",
            vec![
                regex("μεταξύ|ανάμεσα"),
                pred(is_a_time_of_day),
                regex("(και(\\s+τ(ου|ης))?|\\-)"),
                pred(is_a_time_of_day),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleByTheEndOfTime
        rule(
            "by the end of <time>",
            vec![regex("μέχρι το (τέλος|πέρας)"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, &now())),
        ),
        // ruleCycleCurrentLastNext
        rule(
            "this|last|next <cycle>",
            vec![
                regex("(τρ[εέ]χ|επ[οό]μ[εέ]ν|ερχ[οό]μ[εέ]ν|περασμέν|προηγο[υύ]μ[εέ]ν)(ουσ)?(α|ά|ο[υύ]?|η|ή|ε|έ|ω|ώ)ν?ς?"),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let n = cycle_from_word(&rx(ts[0], 0)?)?;
                tt(cycle_nth(grain_of(ts[1])?, n))
            },
        ),
        // ruleCycleThis
        rule(
            "this <cycle>",
            vec![
                regex("αυτ([οό][υύ]?|[ηή]|[εέ]|ώ|ά)ν?ς?|τρ[εέ]χ(ουσ)?(α|ε|ο|ω|ώ)ν?ς?"),
                dimension(Dim::Time),
            ],
            |ts| tt(pred_nth(0, false, td(ts[1])?)),
        ),
        // ruleDatetimeDatetimeInterval
        rule(
            "<datetime> - <datetime> (interval)",
            vec![pred(is_not_latent), regex("\\-|μ[εέ](χρι)?(\\s+τις)?"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?).map(not_latent)),
        ),
        // ruleDayofmonthNonOrdinalNamedmonth
        // Haskell prod is `(Token Time td:token:_)` on `[isDOMInteger, isAMonth]` — never fires.
        rule(
            "<day-of-month> (non-ordinal) <named-month>",
            vec![pred(is_dom_integer), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        // ruleDayofmonthNonOrdinalOfNamedmonth
        rule(
            "<day-of-month> (ordinal or number) of <named-month>",
            vec![pred(is_dom_value), regex("του"), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
        ),
        // ruleDayofmonthOrdinal
        rule("<day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
            tt(mk_latent(day_of_month(ordinal_value(ts[0])?)))
        }),
        // ruleDayofmonthOrdinalNamedmonth
        rule(
            "<day-of-month> (ordinal or number) <named-month>",
            vec![pred(is_dom_value), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
        ),
        // ruleDayofmonthordinalNamedmonthYear
        rule(
            "<day-of-month> ( ordinal) <named-month> year",
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
            vec![dimension(Dim::Duration), regex("μετά(\\s+από)?|από"), dimension(Dim::Time)],
            |ts| tt(duration_after(dur(ts[0])?, td(ts[2])?)),
        ),
        // ruleDurationBeforeTime
        rule(
            "<duration> before <time>",
            vec![dimension(Dim::Duration), regex("πριν"), dimension(Dim::Time)],
            |ts| tt(duration_before(dur(ts[0])?, td(ts[2])?)),
        ),
        // ruleDurationFromNow
        rule("<duration> from now", vec![dimension(Dim::Duration), regex("από τώρα")], |ts| {
            tt(in_duration(dur(ts[0])?))
        }),
        // ruleDurationHenceAgo
        rule(
            "<duration> hence|ago",
            vec![dimension(Dim::Duration), regex("(πριν|μετά)")],
            |ts| {
                let dd = dur(ts[0])?;
                if to_lower(&rx(ts[1], 0)?) == "πριν" {
                    tt(duration_ago(dd))
                } else {
                    tt(in_duration(dd))
                }
            },
        ),
        // ruleEarlyMorning
        rule("early morning", vec![regex("νωρίς\\s+(το\\s+)πρωί")], |_| {
            ttd(interval(Open, &hour(false, 4), &hour(false, 9)).map(|x| part_of_day(mk_latent(x))))
        }),
        // ruleEvening
        rule("evening", vec![regex("βράδυ")], |_| latent_pod(18, 0)),
        // ruleExactlyTimeofday
        // Haskell prod is `(_:Token Time td:_)` on `[isATimeOfDay, regex]` — never fires.
        rule(
            "exactly <time-of-day>",
            vec![pred(is_a_time_of_day), regex("ακριβώς")],
            |ts| tt(not_latent(td(ts.get(1)?)?.clone())),
        ),
        // ruleFromDatetimeDatetimeInterval
        rule(
            "from <datetime> - <datetime> (interval)",
            vec![
                regex("από(\\s+τ(ις|η))?"),
                pred(not_latent_tod),
                regex("\\-|μέχρι(\\s+τ(ις|η))?"),
                pred(is_not_latent),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        // ruleHHMMSS
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
        // ruleHalfAfterIntegerHourofday
        rule(
            "half after|past <integer> (hour-of-day)",
            vec![pred(is_an_hour_of_day), regex("και μισή")],
            |ts| {
                let (hours, is12h) = tod(ts[0])?;
                tt(hour_minute(is12h, hours, 30))
            },
        ),
        // ruleHalfAfterIntegerHourofday2
        rule(
            "<integer>-and-half (hour-of-day)",
            vec![regex(
                "(μιά|ενά|δυό|τρεισή|τεσσερι?σή|πεντέ|εξί|ε[πφ]τά|ο[κχ]τώ|εννιά|δεκά|εντεκά|δωδεκά)μισ[ιη]ς?",
            )],
            |ts| {
                let hours = lookup_el_num(&rx(ts[0], 0)?)?;
                tt(hour_minute(true, hours, 30))
            },
        ),
        // ruleHenceAgoDuration
        rule(
            "before <duration>",
            vec![regex("(πριν(\\s+από)?|εδώ και)"), dimension(Dim::Duration)],
            |ts| {
                let dd = dur(ts[1])?;
                if to_lower(&rx(ts[0], 0)?) == "εδώ και" {
                    tt(in_duration(dd))
                } else {
                    tt(duration_ago(dd))
                }
            },
        ),
        // ruleHhmm
        rule(
            "hh:mm",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:ω]([0-5]\\d)ω?")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(hour_minute(true, h, m))
            },
        ),
        // ruleHhmmMilitary
        rule(
            "hhmm (military)",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))([0-5]\\d)")],
            |ts| {
                let hh = parse_int(&rx(ts[0], 0)?)?;
                let mm = parse_int(&rx(ts[0], 1)?)?;
                tt(mk_latent(hour_minute(false, hh, mm)))
            },
        ),
        // ruleHhmmMilitaryAmpm
        rule(
            "hhmm (military) am|pm",
            vec![regex("((?:1[012]|0?\\d))([0-5]\\d)"), regex("([πμ])\\.?(μ\\.?)?")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let am = to_lower(&rx(ts[1], 0)?) == "π";
                tt(time_of_day_ampm(am, &hour_minute(true, h, m)))
            },
        ),
        // ruleHourofdayIntegerAsRelativeMinutes
        rule(
            "<hour-of-day> <integer> (as relative minutes)",
            vec![pred(not_latent_tod_hour), regex("και"), pred(is_integer_between_tok(1, 59))],
            |ts| {
                let (hours, is12h) = tod(ts[0])?;
                let n = get_int_value(ts[2])?;
                tt(hour_minute(is12h, hours, n))
            },
        ),
        // ruleInDuration
        rule("in <duration>", vec![regex("σε"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        // ruleInDurationFromTime
        rule(
            "in <duration> from <time>",
            vec![
                regex("σε"),
                dimension(Dim::Duration),
                regex("(ξεκινώντας\\s+)?από"),
                dimension(Dim::Time),
            ],
            |ts| tt(duration_after(dur(ts[1])?, td(ts[3])?)),
        ),
        // ruleInduringThePartofday
        rule(
            "in|during the <part-of-day>",
            vec![regex("κατά τη διάρκεια του|(μέσα )?σ?το"), pred(is_a_part_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleIntersect
        rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        // ruleIntersectBy
        rule(
            "intersect by ','",
            vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleIntersectByOfFromS
        rule(
            "intersect by 'of', 'from', 's",
            vec![pred(is_not_latent), regex("τ(ου|ης)"), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // ruleIntervalBy
        rule("until <time>", vec![regex("μέχρι"), dimension(Dim::Time)], |ts| {
            ttd(interval(Open, &now(), td(ts[1])?))
        }),
        // ruleIntervalByTheEndOf
        rule(
            "by the end of <time>",
            vec![regex("μέχρι το τέλος"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, &now(), td(ts[1])?)),
        ),
        // ruleIntervalForDurationFrom
        rule(
            "for <duration> from <time>",
            vec![
                regex("για"),
                dimension(Dim::Duration),
                regex("(ξεκινώντας\\s+)?(μετά(\\s+από)?|από)"),
                dimension(Dim::Time),
            ],
            |ts| {
                let td1 = td(ts[3])?;
                ttd(interval(Open, td1, &duration_after(dur(ts[1])?, td1)))
            },
        ),
        // ruleIntervalTODAMPM
        rule(
            "hh(:mm) - <time-of-day> am|pm",
            vec![
                regex("(?:από )?((?:[01]?\\d)|(?:2[0-3]))([:.]([0-5]\\d))?"),
                regex("\\-|:|μέχρι"),
                pred(is_a_time_of_day),
                regex("([πμ])(\\s|\\.)?(μ\\.?)?"),
            ],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let ampm = to_lower(&rx(ts[3], 0)?) == "π";
                let td1 = match parse_int(&rx(ts[0], 2).unwrap_or_default()) {
                    Some(m) => hour_minute(true, h, m),
                    None => hour(true, h),
                };
                ttd(interval(
                    Closed,
                    &time_of_day_ampm(ampm, &td1),
                    &time_of_day_ampm(ampm, td(ts[2])?),
                ))
            },
        ),
        // ruleIntervalTODAMPMverbose
        rule(
            "hh(:mm) - <time-of-day> am|pm (verbose)",
            vec![
                regex("(?:από\\s+)?((?:[01]?\\d)|(?:2[0-3]))([:.]([0-5]\\d))?"),
                regex("\\-|:|μέχρι"),
                pred(is_a_time_of_day),
                regex("(?:το\\s+)?(πρωί|απόγευμα)"),
            ],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let ampm = to_lower(&rx(ts[3], 0)?) == "πρωί";
                let td1 = match parse_int(&rx(ts[0], 2).unwrap_or_default()) {
                    Some(m) => hour_minute(true, h, m),
                    None => hour(true, h),
                };
                ttd(interval(
                    Closed,
                    &time_of_day_ampm(ampm, &td1),
                    &time_of_day_ampm(ampm, td(ts[2])?),
                ))
            },
        ),
        // ruleIntervalTODDash
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(not_latent_tod), regex("\\-|μέχρι"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleIntervalUntilTOD
        rule("until <time-of-day>", vec![regex("πριν"), dimension(Dim::Time)], |ts| {
            tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone()))
        }),
        // ruleLastCycle
        rule(
            "last <cycle>",
            vec![regex("τελευταί(ου?|ας?)"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
        ),
        // ruleLastCycleOfTime
        rule(
            "last <cycle> of <time>",
            vec![
                regex("τελευταί(ο[ιυ]?ς?|α|ες|ων)"),
                dimension(Dim::TimeGrain),
                regex("σ?τ(ο[υν]?|η[νς]?|ων)"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastCycleOfTimeS
        rule(
            "last <cycle> <time>'s'",
            vec![regex("τελευταί(ο[ιυ]?ς?|α|ες|ων)"), dimension(Dim::TimeGrain), dimension(Dim::Time)],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[2])?)),
        ),
        // ruleLastDayofweekOfTime
        rule(
            "last <day-of-week> of <time>",
            vec![
                regex("τελευταί(ου?|ας?)"),
                pred(is_a_day_of_week),
                regex("τ(ου|ης)"),
                dimension(Dim::Time),
            ],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
        ),
        // ruleLastNCycle
        rule(
            "last n <cycle>",
            vec![
                regex("(τελευταί|περασμέν)(ο[ιυ]?ς?|[εα]ς?|ων)"),
                pred(is_integer_between_tok(1, 9999)),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -n))
            },
        ),
        // ruleLastNight
        rule("lastnight", vec![regex("ε?χ[θτ][εέ]ς\\s+(το\\s+)?βράδυ")], |_| {
            let td1 = cycle_nth(G::Day, -1);
            let td2 = interval(Open, &hour(false, 18), &hour(false, 0))?;
            ttd(intersect(&td1, &td2).map(part_of_day))
        }),
        // ruleLastTime
        rule(
            "last <time>",
            vec![
                regex("(τελευταί|περασμέν|προηγο[υύ]μ[εέ]ν)(α|ά|ο[υύ]?|η|ή|ε|έ|ω|ώ)ν?ς?"),
                pred(is_ok_with_this_next),
            ],
            |ts| tt(pred_nth(-1, false, td(ts[1])?)),
        ),
        // ruleLastWeekendOfMonth
        rule(
            "last weekend of <named-month>",
            vec![regex("τελευταίου?\\s+(σαββατοκύριακου?|σκ)\\s+"), pred(is_a_month)],
            |ts| tt(pred_last_of(&weekend(), td(ts[1])?)),
        ),
        // ruleLastYear
        rule(
            "Last year",
            vec![regex("π[εέ]ρ(υ?σιν|ασμέν)[οόηή]ς?\\s+(έτου?ς|χρ[οό]ν(ου|ιάς?))")],
            |_| tt(cycle_nth(G::Year, -1)),
        ),
        // ruleLastYearOneWord
        rule("Last year", vec![regex("π[εέ]ρυ?σι")], |_| tt(cycle_nth(G::Year, -1))),
        // ruleLunch
        rule("lunch", vec![regex("μεσημεριαν(ό|ού)( γεύμα(τος)?)?")], |_| latent_pod(12, 14)),
        // ruleMidnight
        rule("midnight", vec![regex("μεσάνυχτα")], |_| tt(hour(false, 0))),
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
                let d = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let y = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        // ruleMonthDdddInterval
        rule(
            "dd-dd <month> (interval)",
            vec![
                regex("(από\\s+)?(τ[ιη][νς]?\\s+)?([012]?\\d|30|31)η?ς?"),
                regex("\\-|μ[εέ](χρι)?(?:\\s+τ[ιη][νς]?)?"),
                regex("([012]?\\d|30|31)η?ς?"),
                pred(is_a_month),
            ],
            |ts| {
                let v1 = parse_int(&rx(ts[0], 2)?)?;
                let v2 = parse_int(&rx(ts[2], 0)?)?;
                let from = intersect(&day_of_month(v1), td(ts[3])?)?;
                let to = intersect(&day_of_month(v2), td(ts[3])?)?;
                ttd(interval(Closed, &from, &to))
            },
        ),
        // ruleMorning
        rule("morning", vec![regex("(το\\s+)?πρωί")], |_| latent_pod(4, 12)),
        // ruleNamedmonthDayofmonth
        rule(
            "<named-month> <day-of-month> (ordinal or number)",
            vec![pred(is_a_month), pred(is_dom_value)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        // ruleNextCycle
        rule(
            "next <cycle>",
            vec![regex("ε(π|ρχ)[οό]μ[εέ]ν(ο[ιυ]?ς?|ης?|ες|ων)"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 1)),
        ),
        // ruleNextDayofweek
        rule(
            "next <day-of-week>",
            vec![regex("επ[οό]μ[εέ]ν(ο[ιυ]?ς?|ης?|ες|ων)"), pred(is_a_day_of_week)],
            |ts| tt(pred_nth(1, false, td(ts[1])?)),
        ),
        // ruleNextNCycle
        rule(
            "next n <cycle>",
            vec![
                regex("επ[οό]μ[εέ]ν(ο[ιυ]?ς?|ης?|ες|ων|α)"),
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
            vec![regex("επ[οό]μ[εέ]ν(ο[ιυ]?ς?|ης?|ες|ων)"), pred(is_ok_with_this_next)],
            |ts| tt(pred_nth(1, false, td(ts[1])?)),
        ),
        // ruleNextYear — Haskell name is "Last year"
        rule("Last year", vec![regex("του\\s*χρόνου")], |_| tt(cycle_nth(G::Year, 1))),
        // ruleNight
        rule("night", vec![regex("νύχτα")], |_| latent_pod(20, 8)),
        // ruleNoon
        rule("noon", vec![regex("μεσημ[εέ]ρι(ού)?")], |_| tt(hour(false, 12))),
        // ruleNthTimeAfterTime
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Ordinal), pred(dow_or_month), regex("μετά"), dimension(Dim::Time)],
            |ts| {
                let v = ordinal_value(ts[0])?;
                tt(pred_nth_after(v - 1, td(ts[1])?, td(ts[3])?))
            },
        ),
        // ruleNthTimeOfTime
        rule(
            "nth <time> of <time>",
            vec![
                dimension(Dim::Ordinal),
                pred(is_not_latent),
                regex("τ(ου|ης|ων)"),
                dimension(Dim::Time),
            ],
            |ts| {
                let v = ordinal_value(ts[0])?;
                ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(v - 1, false, &x)))
            },
        ),
        // ruleOrdinalCycleAfterTime
        // Haskell prod skips an extra token (`_:Token Ordinal ...`) — never fires.
        rule(
            "<ordinal> <cycle> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("μετά"), dimension(Dim::Time)],
            |ts| {
                let od = ordinal_value(ts.get(1)?)?;
                tt(cycle_nth_after(true, grain_of(ts.get(2)?)?, od - 1, td(ts.get(4)?)?))
            },
        ),
        // ruleOrdinalCycleOfTime
        rule(
            "<ordinal> <cycle> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), dimension(Dim::Time)],
            |ts| {
                let od = ordinal_value(ts[0])?;
                tt(cycle_nth_after(true, grain_of(ts[1])?, od - 1, td(ts[2])?))
            },
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
            |ts| {
                let od = ordinal_value(ts[0])?;
                tt(cycle_nth_after(false, G::Quarter, od - 1, td(ts[2])?))
            },
        ),
        // rulePartOfMonth
        rule(
            "part of <named-month>",
            vec![regex("(αρχ(?:ές|η)|μέσ[οα]υ?|τέλ(?:ου?ς|η))(?:\\s+του)?"), pred(is_a_month)],
            |ts| {
                let prefix: String = to_lower(&rx(ts[0], 0)?).chars().take(3).collect();
                let (sd, ed) = match prefix.as_str() {
                    "αρχ" => (1, 10),
                    "μέσ" => (11, 20),
                    "τέλ" => (21, -1),
                    _ => return None,
                };
                let start = intersect(td(ts[1])?, &day_of_month(sd))?;
                let end = if ed != -1 {
                    intersect(td(ts[1])?, &day_of_month(ed))?
                } else {
                    cycle_last_of(G::Day, td(ts[1])?)
                };
                ttd(interval(Open, &start, &end))
            },
        ),
        // rulePartofdayOfTime
        rule(
            "<part-of-day> of <time>",
            vec![pred(is_a_part_of_day), regex("τ(ου|ης)"), dimension(Dim::Time)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        // rulePrecisionTimeofday
        rule(
            "about <time-of-day>",
            vec![regex("περίπου|ακριβώς"), pred(is_grain_finer_than(G::Year))],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // ruleQuarterBeforeOrAfterIntegerHourofday
        rule(
            "quarter to|past <integer> (hour-of-day)",
            vec![pred(is_an_hour_of_day), regex("(παρά|και)\\s+τέταρτο")],
            |ts| {
                if to_lower(&rx(ts[1], 0)?) == "παρά" {
                    ttd(minutes_before(15, td(ts[0])?))
                } else {
                    ttd(minutes_after(15, td(ts[0])?))
                }
            },
        ),
        // ruleRelativeIntegerToOrAfterIntegerPartOfDay
        rule(
            "relative integer (minutes) to|till|before|after <integer> (time-of-day)",
            vec![
                pred(is_integer_between_tok(1, 30)),
                regex("(πριν|μετά)"),
                pred(is_a_time_of_day),
            ],
            |ts| {
                let v = get_int_value(ts[0])?;
                let dd = duration(G::Minute, v);
                if to_lower(&rx(ts[1], 0)?) == "πριν" {
                    tt(duration_before(&dd, td(ts[2])?))
                } else {
                    tt(duration_after(&dd, td(ts[2])?))
                }
            },
        ),
        // ruleTheCycleAfterTime
        // Haskell prod is `(_:Token TimeGrain ...)` on `[TimeGrain, regex, Time]` — never fires.
        rule(
            "the <cycle> after <time>",
            vec![dimension(Dim::TimeGrain), regex("μετά τ(ου?|η|ι|α|ου)ν?ς?"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts.get(1)?)?, 1, td(ts.get(3)?)?)),
        ),
        // ruleTheCycleBeforeTime
        rule(
            "the <cycle> before <time>",
            vec![dimension(Dim::TimeGrain), regex("πριν τ(ον?|ην?|α|ους)"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, -1, td(ts[2])?)),
        ),
        // ruleTheCycleOfTime
        rule(
            "the <cycle> of <time>",
            vec![dimension(Dim::TimeGrain), regex("τ(ου|ης)"), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[0])?, 0, td(ts[2])?)),
        ),
        // ruleTheDayOf
        rule(
            "on the day of <day>",
            vec![regex("ανήμερα"), pred(is_grain_of_time(G::Day))],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        // ruleTheDayofmonthNonOrdinal
        rule(
            "the <day-of-month> (non ordinal)",
            vec![regex("τ?η[νς]?"), pred(is_integer_between_tok(1, 31))],
            |ts| tt(day_of_month(get_int_value(ts[1])?)),
        ),
        // ruleThisCycle
        rule(
            "this <cycle>",
            vec![regex("αυτ([οό][νύ]?|ή[νς]?)\\s+τ(ον?|η[νς]?)"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
        ),
        // ruleThisDayofWeek
        rule(
            "this <day-of-week>",
            vec![regex("αυτή[νς]? τη[νς]?"), pred(is_a_day_of_week)],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        // ruleThisDayofWeek2
        rule(
            "(this) coming <day-of-week>",
            vec![regex("ερχ[οό]μ[εέ]νη[ςν]?"), pred(is_a_day_of_week)],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        // ruleThisPartofday
        rule(
            "this <part-of-day>",
            vec![regex("αυτ([οό]ύ?|ή[νς]?) τ(ον?|η[νς]?)"), pred(is_a_part_of_day)],
            |ts| ttd(intersect(&today(), td(ts[1])?).map(part_of_day)),
        ),
        // ruleThisYear
        rule("this year", vec![regex("ε?φ[εέ]τος?")], |_| tt(cycle_nth(G::Year, 0))),
        // ruleTimeAfterNext
        rule(
            "<time> after next",
            vec![regex("μεθεπ[οό]μ[εέ]ν(ο[ιυ]?ς?|ης?|ων)"), dimension(Dim::Time)],
            |ts| tt(pred_nth(2, true, td(ts[1])?)),
        ),
        // ruleTimePartofday
        rule(
            "<time> <part-of-day>",
            vec![dimension(Dim::Time), pred(is_a_part_of_day)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[1])?)),
        ),
        // ruleTimeofdayAmpm
        rule(
            "<time-of-day> am|pm",
            vec![pred(is_a_time_of_day), regex("([πμ])\\.?(μ\\.?)?")],
            |ts| tt(time_of_day_ampm(to_lower(&rx(ts[1], 0)?) == "π", td(ts[0])?)),
        ),
        // ruleTimeofdayAmpmVerbose
        rule(
            "<time-of-day> am|pm (verbose)",
            vec![pred(is_a_time_of_day), regex("το\\s+(πρωί|απόγευμα)")],
            |ts| tt(time_of_day_ampm(to_lower(&rx(ts[1], 0)?) == "πρωί", td(ts[0])?)),
        ),
        // ruleTimeofdayApproximately
        rule(
            "<time-of-day> approximately",
            vec![pred(is_a_time_of_day), regex("περίπου|και κάτι")],
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
            vec![pred(is_a_time_of_day), regex("η ώρα")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        // ruleTimeofdayTimeofdayInterval
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(not_latent_tod), regex("\\-|μέχρι"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        // ruleTimezone
        rule(
            "<time> timezone",
            vec![pred(not_latent_tod), regex(TIMEZONE_RE)],
            |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
        ),
        // ruleTomorrowNight
        rule("tomorrownight", vec![regex("αύριο\\s+(το\\s+)?βράδυ")], |_| {
            let td1 = cycle_nth(G::Day, 1);
            let td2 = interval(Open, &hour(false, 18), &hour(false, 0))?;
            ttd(intersect(&td1, &td2).map(part_of_day))
        }),
        // ruleTonight
        rule("tonight", vec![regex("απόψε")], |_| {
            let td2 = interval(Open, &hour(false, 18), &hour(false, 0))?;
            ttd(intersect(&today(), &td2).map(part_of_day))
        }),
        // ruleUntilTimeofdayPostfix
        rule(
            "<time-of-day> until",
            vec![dimension(Dim::Time), regex("το αργότερο")],
            |ts| tt(with_direction(IntervalDirection::Before, td(ts[0])?.clone())),
        ),
        // ruleWeekend
        rule("week-end", vec![regex("σαββατοκύριακ(ου?|α|ων)|σκ")], |_| {
            tt(mk_ok_for_this_next(weekend()))
        }),
        // ruleWithinDuration
        rule("within <duration>", vec![regex("μέσα(\\s+σε)?"), dimension(Dim::Duration)], |ts| {
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
    ]
}

fn not_latent_tod_hour(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

fn build_rules() -> Vec<Rule> {
    let mut v = rules_main();
    v.extend(rules_days_of_week());
    v.extend(rules_holidays());
    v.extend(rules_instants());
    v.extend(rules_months());
    v.extend(rules_seasons());
    v
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &RULES
}
