//! `Duckling.Time.BG.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::{duration, DurationData};
use crate::dims::time::computed::orthodox_easter;
use crate::dims::time::helpers::*;
use crate::dims::time::holiday_helpers::compute_earth_hour;
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

fn numeral_value(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

fn tod_hours(t: &Token) -> Option<(i64, bool, bool)> {
    match t {
        Token::Time(TimeData {
            form: Some(Form::TimeOfDay { hours: Some(h), is12h }),
            latent,
            ..
        }) => Some((*h, *is12h, *latent)),
        _ => None,
    }
}

fn month_num(t: &Token) -> Option<i64> {
    match t {
        Token::Time(TimeData { form: Some(Form::Month(m)), .. }) => Some(*m),
        _ => None,
    }
}

fn not_latent_finer_than_year(t: &Token) -> bool {
    is_not_latent(t) && is_grain_finer_than(G::Year)(t)
}

fn not_latent_or_year(t: &Token) -> bool {
    is_not_latent(t) || is_grain_of_time(G::Year)(t)
}

fn month_or_year(t: &Token) -> bool {
    is_a_month(t) || is_grain_of_time(G::Year)(t)
}

fn month_or_dow(t: &Token) -> bool {
    is_a_month(t) || is_a_day_of_week(t)
}

fn day_or_month(t: &Token) -> bool {
    is_grain_of_time(G::Day)(t) || is_grain_of_time(G::Month)(t)
}

fn not_latent_tod(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t)
}

fn not_latent_tod_no_tz(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t) && has_no_timezone(t)
}

fn tod_no_tz(t: &Token) -> bool {
    is_a_time_of_day(t) && has_no_timezone(t)
}

const TIMEZONE_NAME: &str = "YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT";

fn rules_instants() -> Vec<Rule> {
    mk_rule_instants(&[
        ("right now", G::Second, 0, "((точно\\s+)?сега)|веднага"),
        ("today", G::Day, 0, "днес|(по това време)"),
        ("tomorrow", G::Day, 1, "утре"),
        ("yesterday", G::Day, -1, "вчера"),
        ("after tomorrow", G::Day, 2, "(в\\s*)?другиден"),
        ("before yesterday", G::Day, -2, "(оня ден)|завчера"),
    ])
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Monday", "понеделник|пон\\.?|пн\\.?"),
        ("Tuesday", "вторник|вт\\.?"),
        ("Wednesday", "сряда|ср\\.?"),
        ("Thursday", "четвъртък|четв\\.?|чт\\.?"),
        ("Friday", "петък|пет\\.?|пт\\.?"),
        ("Saturday", "събота|съб\\.?|сб\\.?"),
        ("Sunday", "неделя|нед\\.?|нд\\.?"),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("January", "януари|ян\\.?"),
        ("February", "февруари|февр\\.?"),
        ("March", "март"),
        ("April", "април|апр\\.?"),
        ("May", "май"),
        ("June", "юни"),
        ("July", "юли"),
        ("August", "август|авг\\.?"),
        ("September", "септември|септ\\.?"),
        ("October", "октомври|окт\\.?"),
        ("November", "ноември|ноем\\.?"),
        ("December", "декември|дек\\.?"),
    ])
}

fn rules_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("summer", "лято", month_day(6, 21), month_day(9, 23)),
        ("fall", "есен", month_day(9, 23), month_day(12, 21)),
        ("winter", "зима", month_day(12, 21), month_day(3, 20)),
        ("spring", "пролет", month_day(3, 20), month_day(6, 21)),
    ])
}

fn rules_periodic_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("April Fools", "първи април", month_day(4, 1)),
        (
            "Bulgarian Liberation Day",
            "(трети март)|(ден(я|ят)? на освобождението на българия( от турско робство)?)",
            month_day(3, 3),
        ),
        (
            "Bulgarian Indenpendence Day",
            "ден(я|ят)? на (обявянето на)? независимостта на българия",
            month_day(9, 22),
        ),
        (
            "Bulgarian Unification Day",
            "ден(я|ят)? на съединението на (княжество )?българия(и източна румелия)?",
            month_day(9, 6),
        ),
        (
            "Slavonic Litaracy Day",
            "(ден(я|ят)? на славянската писменост)|(св(\\.|ети)?(св(\\.|ети)?)? кирил и методи(й)?)",
            month_day(5, 24),
        ),
        ("Saint Geroge's Day", "(св(\\.|ети)? георги)|гергьовден", month_day(5, 6)),
        ("Assumption of Mary", "голяма богородица", month_day(8, 15)),
        ("Christmas", "коледа|(рождество( христово)?)", month_day(12, 25)),
        ("Christmas Eve", "бъдни вечер", month_day(12, 24)),
        ("Earth Day", "ден(я|ят) на земята", month_day(4, 22)),
        ("May Day", "(първи май)|(ден(я|ят)? на труда)", month_day(5, 1)),
        ("New Year's Day", "нова година", month_day(1, 1)),
        ("St. George's Day", "гергьовден|(св(\\.|ети)? георги)", month_day(5, 6)),
        ("Valentine's Day", "св(\\.|ети)? валентин", month_day(2, 14)),
        ("International Women's Day", "ден(я|ят)? на (майката|жената)", month_day(3, 8)),
    ])
}

fn rules_computed_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        (
            "Orthodox Easter Monday",
            "велики понеделник",
            cycle_nth_after(false, G::Day, 1, &orthodox_easter()),
        ),
        ("Orthodox Easter Sunday", "великден", orthodox_easter()),
        (
            "Orthodox Holy Saturday",
            "велика събота",
            cycle_nth_after(false, G::Day, -1, &orthodox_easter()),
        ),
        (
            "Orthodox Great Friday",
            "(велики|разпети) петък",
            cycle_nth_after(false, G::Day, -2, &orthodox_easter()),
        ),
        (
            "Orthodox Palm Sunday",
            "цветница",
            cycle_nth_after(false, G::Day, -7, &orthodox_easter()),
        ),
    ])
}

fn rules_computed_holidays_prime() -> Vec<Rule> {
    let start = cycle_nth_after(false, G::Day, -48, &orthodox_easter());
    let end = cycle_nth_after(false, G::Day, -9, &orthodox_easter());
    mk_rule_holidays_maybe(&[
        ("Great Lent", "велики(те)? пости", interval(Open, &start, &end)),
        ("Earth Hour", "час(а|ът) на земята", compute_earth_hour()),
    ])
}

fn cycle_this_last_next_n(match_: &str) -> Option<i64> {
    match match_ {
        "този" | "тази" | "това" | "настоящия" | "настоящият" | "настоящата" | "настоящото" => Some(0),
        "последния" | "последният" | "последната" | "последното" | "миналия" | "миналият"
        | "миналата" | "миналото" | "предишния" | "предишният" | "предишната" | "предишното" => {
            Some(-1)
        }
        "следващия" | "следващият" | "следващата" | "следващото" | "идния" | "идният" | "идната"
        | "идното" => Some(1),
        "по-последния" | "по-последният" | "по-последната" | "по-последното" | "по-миналия"
        | "по-миналият" | "по-миналата" | "по-миналото" | "по-предишния" | "по-предишният"
        | "по-предишната" | "по-предишното" => Some(-2),
        "по-следващия" | "по-следващият" | "по-следващата" | "по-следващото" => Some(2),
        "по-по-последния" | "по-по-последният" | "по-по-последната" | "по-по-последното"
        | "по-по-миналия" | "по-по-миналият" | "по-по-миналата" | "по-по-миналото"
        | "по-по-предишния" | "по-по-предишният" | "по-по-предишната" | "по-по-предишното" => {
            Some(-3)
        }
        "по-по-следващия" | "по-по-следващият" | "по-по-следващата" | "по-по-следващото" => Some(3),
        _ => None,
    }
}

fn season_n(match_: &str) -> Option<i64> {
    match match_ {
        "това" | "тази" | "настоящото" | "настоящата" => Some(0),
        "миналото" | "миналата" | "последното" | "последната" | "предното" | "предната" => Some(-1),
        "следващото" | "следващата" | "другото" | "другата" => Some(1),
        "по-миналото" | "по-миналата" | "по-последното" | "по-последната" | "по-предното"
        | "по-предната" => Some(-2),
        "по-следващото" | "по-следващата" | "по-другото" | "по-другата" => Some(2),
        "по-по-миналото" | "по-по-миналата" | "по-по-последното" | "по-по-последната"
        | "по-по-предното" | "по-по-предната" => Some(-3),
        "по-по-следващото" | "по-по-следващата" | "по-по-другото" | "по-по-другата" => Some(3),
        _ => None,
    }
}

fn rules_main() -> Vec<Rule> {
    vec![
        rule(
            "intersect",
            vec![pred(not_latent_finer_than_year), pred(not_latent_or_year)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[1])?)),
        ),
        rule(
            "intersect by \",\", \"of\", \"from\", \"'s\"",
            vec![pred(is_not_latent), regex("от|за|на|,"), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?).map(not_latent)),
        ),
        rule(
            "intersect by \",\", \"of\", \"from\" for year",
            vec![pred(is_not_latent), regex("на|,"), pred(is_grain_of_time(G::Year))],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?).map(not_latent)),
        ),
        rule("on <day>", vec![regex("на"), pred(is_grain_of_time(G::Day))], |ts| {
            Some(ts[1].clone())
        }),
        rule(
            "in|during <named-month>|year",
            vec![regex("в|през"), pred(month_or_year)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        rule("absorption of , after named day", vec![pred(is_a_day_of_week), regex(",")], |ts| {
            Some(ts[0].clone())
        }),
        rule("on a <named-day>", vec![regex("в"), pred(is_a_day_of_week)], |ts| {
            Some(ts[1].clone())
        }),
        rule(
            "this|next <day-of-week>",
            vec![regex("(т(о|а)зи)|следващ((ия(т)?)|ата)"), pred(is_a_day_of_week)],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        rule(
            "next <time>",
            vec![regex("следващ((ия(т)?)|ата|ото)"), pred(is_ok_with_this_next)],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        rule(
            "this <time>",
            vec![regex("(т(а|о)зи)|това"), pred(is_ok_with_this_next)],
            |ts| tt(pred_nth(0, false, td(ts[1])?)),
        ),
        rule(
            "last <time>",
            vec![regex("((пред(н|ишн))|минал)((ия(т)?)|ата|ото)"), pred(is_ok_with_this_next)],
            |ts| tt(pred_nth(-1, false, td(ts[1])?)),
        ),
        rule(
            "<time> before last",
            vec![regex("по(\\-|\\s+)((пред(н|ишн))|минал)((ия(т)?)|ата|ото)"), dimension(Dim::Time)],
            |ts| tt(pred_nth(-2, false, td(ts[1])?)),
        ),
        rule(
            "<time> after next",
            vec![regex("по(\\-|\\s+)следващ((ия(т)?)|ата|ото)"), dimension(Dim::Time)],
            |ts| tt(pred_nth(1, true, td(ts[1])?)),
        ),
        rule(
            "last <day-of-week> of <time>",
            vec![regex("последн((ия(т)?)|ата)"), pred(is_a_day_of_week), regex("на"), dimension(Dim::Time)],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "last <cycle> of <time>",
            vec![
                regex("последн((ия(т)?)|ото|ата)"),
                dimension(Dim::TimeGrain),
                regex("на|в|през"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
        ),
        rule("last night", vec![regex("(късно )?снощи")], |ts| {
            let match_ = to_lower(&rx(ts[0], 0)?);
            let hours = if match_ == "късно " { 3 } else { 6 };
            let start = duration_before(&duration(G::Hour, hours), &today());
            ttd(interval(Open, &start, &today()).map(|x| part_of_day(not_latent(x))))
        }),
        rule(
            "last weekend of <named-month>",
            vec![regex("последния(т)? уикенд\\s+(през|на)"), pred(is_a_month)],
            |ts| tt(pred_last_of(&weekend(), td(ts[1])?)),
        ),
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("на|в|през"), dimension(Dim::Time)],
            |ts| {
                let n = ordinal_value(ts[0])? - 1;
                ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(n, false, &x)))
            },
        ),
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("след|от"), dimension(Dim::Time)],
            |ts| tt(pred_nth_after(ordinal_value(ts[0])? - 1, td(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "<integer> <day-of-week> from <time>",
            vec![dimension(Dim::Numeral), pred(is_a_day_of_week), regex("от"), dimension(Dim::Time)],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(pred_nth_after(n - 1, td(ts[1])?, td(ts[3])?))
            },
        ),
        rule("year (latent)", vec![pred(is_integer_between_tok(25, 10000))], |ts| {
            let n = get_int_value(ts[0])?;
            tt(mk_latent(year(n)))
        }),
        rule(
            "<year> (bc|ad)",
            vec![pred(is_integer_between_tok(1, 10000)), regex("(пр|сл)\\.?\\s+(Хр\\.?|н\\.?\\e.?)")],
            |ts| {
                let y = get_int_value(ts[0])?;
                let ab = to_lower(&rx(ts[1], 0)?);
                let n = if ab.chars().next()? == 'п' { -y } else { y };
                tt(year_adbc(n))
            },
        ),
        rule("the <day-of-month> (number)", vec![pred(is_dom_integer)], |ts| {
            let n = get_int_value(ts[0])?;
            tt(mk_latent(day_of_month(n)))
        }),
        rule("the <day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
            tt(day_of_month(ordinal_value(ts[0])?))
        }),
        rule("<day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
            let n = get_int_value(ts[0])?;
            tt(mk_latent(day_of_month(n)))
        }),
        rule(
            "<named-month>|<named-day> <day-of-month> (ordinal)",
            vec![pred(month_or_dow), pred(is_dom_ordinal)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        rule(
            "<named-month> <day-of-month> (non ordinal)",
            vec![pred(is_a_month), pred(is_dom_integer)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        rule(
            "<day-of-month> (ordinal or number) <named-month>",
            vec![pred(is_dom_value), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
        ),
        rule(
            "<day-of-month>(ordinal) <named-month> year",
            vec![pred(is_dom_ordinal), pred(is_a_month), regex("(\\d{2,4})")],
            |ts| {
                let int_val = parse_int(&rx(ts[2], 0)?)?;
                let dom = intersect_dom(td(ts[1])?, ts[0])?;
                ttd(intersect(&dom, &year(int_val)))
            },
        ),
        rule(
            "<day-of-month>(ordinal or number)/<named-month>/year",
            vec![
                pred(is_dom_value),
                regex("[-/\\s]"),
                pred(is_a_month),
                regex("[-/\\s]"),
                regex("(\\d{4})"),
            ],
            |ts| {
                let int_val = parse_int(&rx(ts[4], 0)?)?;
                let dom = intersect_dom(td(ts[2])?, ts[0])?;
                ttd(intersect(&dom, &year(int_val)))
            },
        ),
        rule(
            "the ides of <named-month>",
            vec![regex("средата на"), pred(is_a_month)],
            |ts| {
                let m = month_num(ts[1])?;
                let d = if [3, 5, 7, 10].contains(&m) { 15 } else { 13 };
                ttd(intersect(td(ts[1])?, &day_of_month(d)))
            },
        ),
        rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
            let n = get_int_value(ts[0])?;
            tt(mk_latent(hour(n < 13, n)))
        }),
        rule("at <time-of-day>", vec![regex("в"), pred(is_a_time_of_day)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        rule("<time-of-day> o'clock", vec![pred(is_a_time_of_day), regex("часа")], |ts| {
            tt(not_latent(td(ts[0])?.clone()))
        }),
        rule("<time-of-day> am", vec![pred(is_a_time_of_day), regex("сутрин(та)?")], |ts| {
            tt(time_of_day_ampm(true, td(ts[0])?))
        }),
        rule(
            "<time-of-day> pm",
            vec![pred(is_a_time_of_day), regex("вечер(та)?|след об(е|я)д")],
            |ts| tt(time_of_day_ampm(false, td(ts[0])?)),
        ),
        rule("hh:mm", vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)")], |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(false, h, m))
        }),
        rule(
            "hhhmm",
            vec![regex("(?<!/)((?:[01]?\\d)|(?:2[0-3]))ч(([0-5]\\d)|(?!\\d))")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?).unwrap_or(0);
                tt(hour_minute(false, h, m))
            },
        ),
        rule(
            "hhmm (latent)",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))([0-5]\\d)(?!.\\d)")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(mk_latent(hour_minute(false, h, m)))
            },
        ),
        rule(
            "hh:mm:ss",
            vec![regex("((?:[01]?\\d)|(?:2[0-3])):([0-5]\\d):([0-5]\\d)")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let s = parse_int(&rx(ts[0], 2)?)?;
                tt(hour_minute_second(false, h, m, s))
            },
        ),
        rule(
            "<hour-of-day> <integer>",
            vec![pred(is_an_hour_of_day), pred(is_integer_between_tok(1, 59))],
            |ts| {
                let (hours, is12h, is_latent) = tod_hours(ts[0])?;
                let n = get_int_value(ts[1])?;
                let hm = hour_minute(is12h, hours, n);
                if is_latent {
                    tt(mk_latent(hm))
                } else {
                    tt(hm)
                }
            },
        ),
        rule(
            "<hour-of-day> half",
            vec![pred(is_an_hour_of_day), regex("и половина")],
            |ts| {
                let (hours, is12h, _) = tod_hours(ts[0])?;
                tt(hour_minute(is12h, hours, 30))
            },
        ),
        rule(
            "<hour-of-day> quarter",
            vec![pred(is_an_hour_of_day), regex("и (четвърт|(петнайсе(т)?))")],
            |ts| {
                let (hours, is12h, _) = tod_hours(ts[0])?;
                tt(hour_minute(is12h, hours, 15))
            },
        ),
        rule(
            "<integer> to|till|before <hour-of-day>",
            vec![pred(is_an_hour_of_day), regex("без"), pred(is_integer_between_tok(1, 59))],
            |ts| {
                let n = get_int_value(ts[2])?;
                ttd(minutes_before(n, td(ts[0])?))
            },
        ),
        rule(
            "quarter to|till|before <hour-of-day>",
            vec![pred(is_an_hour_of_day), regex("без (четвърт|(петнайсе(т)?))")],
            |ts| ttd(minutes_before(15, td(ts[0])?)),
        ),
        rule(
            "integer after|past <hour-of-day>",
            vec![pred(is_an_hour_of_day), regex("и"), pred(is_integer_between_tok(1, 59))],
            |ts| {
                let n = get_int_value(ts[2])?;
                ttd(minutes_after(n, td(ts[0])?))
            },
        ),
        rule("yyyyqq", vec![regex("(\\d{2,4})q([1-4])")], |ts| {
            let y = parse_int(&rx(ts[0], 0)?)?;
            let q = parse_int(&rx(ts[0], 1)?)?;
            tt(cycle_nth_after(true, G::Quarter, q - 1, &year(y)))
        }),
        rule("yyyy-mm", vec![regex("(\\d{4})\\s*[/-]\\s*(1[0-2]|0?[1-9])")], |ts| {
            let y = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(year_month(y, m))
        }),
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
        rule(
            "dd.mm.yyyy",
            vec![regex("(3[01]|[12]\\d|0?[1-9])\\.(1[0-2]|0?[1-9])\\.(\\d{2,4})")],
            |ts| {
                let d = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let y = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        rule("mm/yyyy", vec![regex("(0?[1-9]|1[0-2])[/-](\\d{4})")], |ts| {
            let m = parse_int(&rx(ts[0], 0)?)?;
            let y = parse_int(&rx(ts[0], 1)?)?;
            tt(year_month(y, m))
        }),
        rule(
            "noon|midnight|EOD|end of day",
            vec![regex("(обед|обяд|полунощ|((в )?(края на деня)))")],
            |ts| {
                let match_ = to_lower(&rx(ts[0], 0)?);
                let h = if match_ == "обед" || match_ == "обяд" { 12 } else { 0 };
                tt(hour(false, h))
            },
        ),
        rule(
            "part of days",
            vec![regex("(сутрин(та)?|(по|след) ?об(е|я)д|вечер(та)?|(през )?нощ(та)?)")],
            |ts| {
                let match_ = to_lower(&rx(ts[0], 0)?);
                let (start, end) = match match_.as_str() {
                    "сутрин" | "сутринта" => (hour(false, 4), hour(false, 12)),
                    "вечер" | "вечерта" => (hour(false, 18), hour(false, 0)),
                    "нощ" | "нощта" | "през нощта" => (hour(false, 0), hour(false, 4)),
                    "обед" | "по обед" | "обяд" | "по обяд" => (hour(false, 12), hour(false, 14)),
                    _ => (hour(false, 12), hour(false, 19)),
                };
                let pod = interval(Open, &start, &end)?;
                tt(part_of_day(mk_latent(pod)))
            },
        ),
        rule("early morning", vec![regex("рано сутрин(та)?")], |_| {
            ttd(interval(Open, &hour(false, 4), &hour(false, 9)).map(|x| part_of_day(mk_latent(x))))
        }),
        rule("this <part-of-day>", vec![regex("(т(о|а)зи)"), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(&today(), td(ts[1])?).map(|x| part_of_day(not_latent(x))))
        }),
        rule("tonight", vec![regex("(късно )?вечер(та)?")], |ts| {
            let match_ = to_lower(&rx(ts[0], 0)?);
            let h = if match_ == "късно " { 21 } else { 18 };
            let evening = interval(Open, &hour(false, h), &hour(false, 0))?;
            ttd(intersect(&today(), &evening).map(|x| part_of_day(not_latent(x))))
        }),
        rule(
            "after lunch/work/school",
            vec![regex("след[\\s*]?((об(е|я)д)|работа|училище)")],
            |ts| {
                let match_ = to_lower(&rx(ts[0], 0)?);
                let (start, end) = match match_.as_str() {
                    "обед" | "обяд" => (hour(false, 13), hour(false, 17)),
                    "работа" => (hour(false, 17), hour(false, 21)),
                    "училище" => (hour(false, 15), hour(false, 21)),
                    _ => return None,
                };
                let pod = interval(Open, &start, &end)?;
                ttd(intersect(&today(), &pod).map(|x| part_of_day(not_latent(x))))
            },
        ),
        rule("<time> <part-of-day>", vec![dimension(Dim::Time), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(td(ts[1])?, td(ts[0])?))
        }),
        rule(
            "<part-of-day> of <time>",
            vec![pred(is_a_part_of_day), regex("на"), dimension(Dim::Time)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        rule("week-end", vec![regex("(края на седмицата)|(уикенд(а)?)")], |_| {
            tt(mk_ok_for_this_next(weekend()))
        }),
        rule("week", vec![regex("(цяла|(остатъка на( тази)?)) седмица(та)?")], |ts| {
            let match_ = to_lower(&rx(ts[0], 0)?);
            let end = cycle_nth_after(true, G::Day, -2, &cycle_nth(G::Week, 1));
            let period = match match_.as_str() {
                "цяла" => interval(Closed, &cycle_nth(G::Week, 0), &end),
                "остатъка на" | "остатъка на тази" => interval(Open, &today(), &end),
                _ => None,
            };
            if match_ == "тази" {
                ttd(period.map(mk_latent))
            } else {
                ttd(period)
            }
        }),
        rule(
            "<time-of-day> sharp|exactly",
            vec![regex("(в )?точно( в )?"), pred(is_a_time_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        rule(
            "about|exactly <time-of-day>",
            vec![regex("около"), pred(is_grain_finer_than(G::Year))],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        rule(
            "from the <day-of-month> (ordinal or number) to the <day-of-month> (ordinal or number) <named-month> (interval)",
            vec![regex("от"), pred(is_dom_value), regex("\\-|до"), pred(is_dom_value), pred(is_a_month)],
            |ts| {
                let dom1 = intersect_dom(td(ts[4])?, ts[1])?;
                let dom2 = intersect_dom(td(ts[4])?, ts[3])?;
                ttd(interval(Closed, &dom1, &dom2))
            },
        ),
        rule(
            "<month> dd-dd (interval)",
            vec![pred(is_a_month), pred(is_dom_value), regex("\\-|до"), pred(is_dom_value)],
            |ts| {
                let dom1 = intersect_dom(td(ts[0])?, ts[1])?;
                let dom2 = intersect_dom(td(ts[0])?, ts[3])?;
                ttd(interval(Closed, &dom1, &dom2))
            },
        ),
        rule(
            "dd-dd <month> (interval)",
            vec![pred(is_dom_value), regex("\\-|до"), pred(is_dom_value), pred(is_a_month)],
            |ts| {
                let dom1 = intersect_dom(td(ts[3])?, ts[0])?;
                let dom2 = intersect_dom(td(ts[3])?, ts[2])?;
                ttd(interval(Closed, &dom1, &dom2))
            },
        ),
        rule(
            "<datetime> - <datetime> (interval)",
            vec![pred(is_not_latent), regex("\\-|до"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "<datetime>/<datetime> (interval)",
            vec![pred(is_not_latent), regex("/"), pred(is_not_latent)],
            |ts| {
                if same_grain(td(ts[0])?, td(ts[2])?) {
                    ttd(interval(Closed, td(ts[0])?, td(ts[2])?))
                } else {
                    None
                }
            },
        ),
        rule(
            "from <datetime> - <datetime> (interval)",
            vec![regex("от"), dimension(Dim::Time), regex("до"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "between <time> and <time>",
            vec![regex("между"), dimension(Dim::Time), regex("и"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(not_latent_tod), regex("\\-|до"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "between <time-of-day> and <time-of-day> (interval)",
            vec![regex("между"), pred(is_a_time_of_day), regex("и"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        rule("by <time>", vec![regex("до"), dimension(Dim::Time)], |ts| {
            ttd(interval(Open, &now(), td(ts[1])?))
        }),
        rule("by the end of <time>", vec![regex("до края на"), dimension(Dim::Time)], |ts| {
            ttd(interval(Closed, &now(), td(ts[1])?))
        }),
        rule("until <time>", vec![regex("преди|до"), dimension(Dim::Time)], |ts| {
            tt(with_direction(IntervalDirection::Before, not_latent(td(ts[1])?.clone())))
        }),
        rule("from|since|after <time>", vec![regex("след|от"), dimension(Dim::Time)], |ts| {
            tt(with_direction(IntervalDirection::After, not_latent(td(ts[1])?.clone())))
        }),
        rule(
            "this|last|next <cycle>",
            vec![
                regex("(т(о|а)зи|това|((по\\-)*(настоящ|идн|следващ|минал|последн|предишн)(ия(т)?|ата|ото)))"),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let match_ = to_lower(&rx(ts[0], 0)?);
                let n = cycle_this_last_next_n(&match_)?;
                tt(cycle_nth(grain_of(ts[1])?, n))
            },
        ),
        rule(
            "<cycle> after|before <time>",
            vec![dimension(Dim::TimeGrain), regex("след|преди"), dimension(Dim::Time)],
            |ts| {
                let n = if to_lower(&rx(ts[1], 0)?) == "след" { 1 } else { -1 };
                tt(cycle_nth_after(false, grain_of(ts[0])?, n, td(ts[2])?))
            },
        ),
        rule(
            "<day> in <duration>",
            vec![
                pred(day_or_month),
                regex("в|през"),
                pred(is_duration_greater_than(G::Hour)),
            ],
            |ts| ttd(intersect(td(ts[0])?, &in_duration_interval(dur(ts[2])?))),
        ),
        rule(
            "in|within|after <duration>",
            vec![regex("(за|в рамките на|след)"), dimension(Dim::Duration)],
            |ts| {
                let match_ = to_lower(&rx(ts[0], 0)?);
                let dd = dur(ts[1])?;
                match match_.as_str() {
                    "в рамките на" => ttd(interval(Open, &now(), &in_duration(dd))),
                    "след" => tt(with_direction(IntervalDirection::After, in_duration(dd))),
                    "за" => tt(in_duration(dd)),
                    _ => None,
                }
            },
        ),
        rule(
            "<integer> <named-day> ago|back",
            vec![regex("преди"), pred(crate::dims::numeral::is_natural), pred(is_a_day_of_week)],
            |ts| {
                let v = numeral_value(ts[1])?;
                tt(pred_nth(-(v.floor() as i64), false, td(ts[2])?))
            },
        ),
        rule(
            "<duration> hence|ago",
            vec![regex("(от|преди)"), dimension(Dim::Duration)],
            |ts| {
                let match_ = to_lower(&rx(ts[0], 0)?);
                let dd = dur(ts[1])?;
                if match_ == "от" {
                    tt(duration_ago(dd))
                } else {
                    tt(in_duration(dd))
                }
            },
        ),
        rule(
            "for <duration> from <time>",
            vec![regex("за"), dimension(Dim::Duration), regex("от|след"), dimension(Dim::Time)],
            |ts| ttd(interval(Closed, td(ts[3])?, &duration_after(dur(ts[1])?, td(ts[3])?))),
        ),
        rule(
            "from <time> for <duration>",
            vec![regex("от|след"), pred(is_not_latent), regex("за"), dimension(Dim::Duration)],
            |ts| ttd(interval(Closed, td(ts[1])?, &duration_after(dur(ts[3])?, td(ts[1])?))),
        ),
        rule(
            "<time> for <duration>",
            vec![pred(is_not_latent), regex("за"), dimension(Dim::Duration)],
            |ts| ttd(interval(Closed, td(ts[0])?, &duration_after(dur(ts[2])?, td(ts[0])?))),
        ),
        rule(
            "in <number> (implicit minutes)",
            vec![regex("след"), pred(is_integer_between_tok(0, 60))],
            |ts| {
                let v = numeral_value(ts[1])?;
                tt(in_duration(&duration(G::Minute, v.floor() as i64)))
            },
        ),
        rule(
            "<time> timezone",
            vec![
                pred(not_latent_tod_no_tz),
                regex(&format!("\\b({})\\b", TIMEZONE_NAME)),
            ],
            |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
        ),
        rule(
            "<time> (timezone)",
            vec![
                pred(not_latent_tod_no_tz),
                regex(&format!("\\(({})\\)", TIMEZONE_NAME)),
            ],
            |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
        ),
        rule(
            "<datetime> - <datetime> (interval) timezone",
            vec![
                pred(tod_no_tz),
                regex("\\-|до"),
                pred(tod_no_tz),
                regex(&format!("\\b({})\\b", TIMEZONE_NAME)),
            ],
            |ts| {
                let tz = to_upper(&rx(ts[3], 0)?);
                let tdz1 = in_timezone(&tz, td(ts[0])?)?;
                let tdz2 = in_timezone(&tz, td(ts[2])?)?;
                ttd(interval(Closed, &tdz1, &tdz2))
            },
        ),
        rule(
            "part of <named-month>",
            vec![regex("(началото|средата|края) на"), pred(is_a_month)],
            |ts| {
                let match_ = to_lower(&rx(ts[0], 0)?);
                let (sd, ed) = match match_.as_str() {
                    "началото" => (1, 10),
                    "средата" => (11, 20),
                    "края" => (21, -1),
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
        rule(
            "at the beginning|end of <named-month>",
            vec![regex("(в )?(началото|края) на"), pred(is_a_month)],
            |ts| {
                let match_ = to_lower(&rx(ts[0], 1)?);
                let (sd, ed) = match match_.as_str() {
                    "началото" => (1, 10),
                    "края" => (21, -1),
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
        rule(
            "at the beginning|end of <year>",
            vec![regex("(в\\s+)?(началото|края) на"), pred(is_grain_of_time(G::Year))],
            |ts| {
                let match_ = to_lower(&rx(ts[0], 1)?);
                let (sd, ed) = match match_.as_str() {
                    "началото" => (1, 4),
                    "края" => (9, -1),
                    _ => return None,
                };
                let start = intersect(td(ts[1])?, &month(sd))?;
                let end = if ed != -1 {
                    intersect(td(ts[1])?, &cycle_last_of(G::Month, &month(ed)))?
                } else {
                    cycle_nth_after(false, G::Year, 1, &intersect(td(ts[1])?, &month(1))?)
                };
                ttd(interval(Open, &start, &end))
            },
        ),
        rule(
            "at the beginning|end of <week>",
            vec![regex("(в\\s+)?(началото|края) на"), pred(is_grain_of_time(G::Week))],
            |ts| {
                let match1 = to_lower(&rx(ts[0], 1)?);
                let (sd, ed) = match match1.as_str() {
                    "началото" => (1, 3),
                    "края" => (5, 7),
                    _ => return None,
                };
                let start = intersect(td(ts[1])?, &day_of_week(sd))?;
                let end = intersect(td(ts[1])?, &day_of_week(ed))?;
                ttd(interval(Open, &start, &end))
            },
        ),
        rule("now", vec![regex("сега")], |_| tt(now())),
        rule(
            "last|this|next <season>",
            vec![regex(
                "(това|тази|(по\\-)?(по\\-)?((настоящ|следващ|друг|последн|минал|предн)(ото|ата))) seasons?",
            )],
            |ts| {
                let n = season_n(&to_lower(&rx(ts[0], 0)?))?;
                tt(pred_nth(n, false, &season()))
            },
        ),
        rule("end of month", vec![regex("((до|в)\\s+)?края на месеца")], |ts| {
            let match_ = to_lower(&rx(ts[0], 0)?);
            let cycle_month = |n: i64| cycle_nth(G::Month, n);
            let (start, end) = if match_.starts_with("до") {
                (Some(now()), intersect(&day_of_month(1), &cycle_month(1)))
            } else {
                (
                    intersect(&day_of_month(21), &cycle_month(0)),
                    Some(cycle_last_of(G::Day, &cycle_month(0))),
                )
            };
            ttd(interval(Open, &start?, &end?))
        }),
        rule("beginning of month", vec![regex("(в\\s+ )?началото на месеца")], |_| {
            let start = intersect(&day_of_month(1), &cycle_nth(G::Month, 0))?;
            let end = intersect(&day_of_month(10), &cycle_nth(G::Month, 0))?;
            ttd(interval(Open, &start, &end))
        }),
        rule("end of year", vec![regex("((до|в)\\s+)?края на годината")], |ts| {
            let match_ = to_lower(&rx(ts[0], 0)?);
            let cycle_year = |n: i64| cycle_nth(G::Year, n);
            let start = if match_.starts_with("до") {
                now()
            } else {
                intersect(&month(9), &cycle_year(0))?
            };
            let end = intersect(&month(1), &cycle_year(1))?;
            ttd(interval(Open, &start, &end))
        }),
        rule("beginning of year", vec![regex("(в\\s+)?началото на годината")], |_| {
            let start = intersect(&month(1), &cycle_nth(G::Year, 0))?;
            let end = intersect(&month(4), &cycle_nth(G::Year, 0))?;
            ttd(interval(Open, &start, &end))
        }),
        rule("<named-month> year", vec![pred(is_a_month), regex("(\\d{2,4})")], |ts| {
            let int_val = parse_int(&rx(ts[1], 0)?)?;
            ttd(intersect(td(ts[0])?, &year(int_val)))
        }),
    ]
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = rules_main();
    v.extend(rules_instants());
    v.extend(rules_days_of_week());
    v.extend(rules_months());
    v.extend(rules_seasons());
    v.extend(rules_computed_holidays());
    v.extend(rules_computed_holidays_prime());
    v.extend(rules_periodic_holidays());
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
