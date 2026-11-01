//! `Duckling.Time.RU.Rules`.

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

fn not_latent_tod(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t)
}

fn not_latent_hour(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

/// `Token Time . mkLatent . partOfDay <$> interval Open (hour False a) (hour False b)`.
fn latent_pod(a: i64, b: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, a), &hour(false, b)).map(|x| mk_latent(part_of_day(x))))
}

const TIMEZONE_NAME: &str = "YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT";

fn rules_instants() -> Vec<Rule> {
    mk_rule_instants(&[
        ("сейчас", G::Second, 0, "сейчас"),
        ("сегодня", G::Day, 0, "сегодня"),
        ("завтра", G::Day, 1, "завтра"),
        ("вчера", G::Day, -1, "вчера"),
        ("послезавтра", G::Day, 2, "послезавтра"),
        ("послепослезавтра", G::Day, 3, "послепослезавтра"),
        ("позавчера", G::Day, -2, "позавчера"),
        ("позапозавчера", G::Day, -3, "позапозавчера"),
        ("Конец месяца", G::Month, 1, "(конец|конца) месяца"),
        ("Конец года", G::Year, 1, "(конец|конца) года"),
    ])
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Понедельник", "понедельник(а)?|пн"),
        ("Вторник", "вторник(а)?|вт"),
        ("Среда", "сред(а|у)|ср"),
        ("Четверг", "четверг(а)?|чт"),
        ("Пятница", "пятниц(а|у)|пт"),
        ("Суббота", "суббот(а|у)|сб"),
        ("Воскресенье", "воскресенье|вс"),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("Январь", "январ(ь|я)|янв\\.?" ),
        ("Февраль", "феврал(ь|я)|фев\\.?" ),
        ("Март", "март(а)?|мар\\.?" ),
        ("Апрель", "апрел(ь|я)|апр\\.?" ),
        ("Май", "ма(й|я)"),
        ("Июнь", "июн(ь|я)|июн\\.?" ),
        ("Июль", "июл(ь|я)|июл\\.?" ),
        ("Август", "август(а)?|авг\\.?" ),
        ("Сентябрь", "сентябр(ь|я)|сен\\.?" ),
        ("Октябрь", "октябр(ь|я)|окт\\.?" ),
        ("Ноябрь", "ноябр(ь|я)?|ноя\\.?" ),
        ("Декабрь", "декабр(ь|я)|дек\\.?" ),
    ])
}

fn rules_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("лето", "лет(о|а)", month_day(6, 1), month_day(8, 31)),
        ("осень", "осень", month_day(9, 1), month_day(11, 30)),
        ("зима", "зима", month_day(12, 1), month_day(2, 28)),
        ("весна", "весна", month_day(3, 1), month_day(5, 31)),
    ])
}

fn rules_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("Новый год", "Новый год", month_day(1, 1)),
        ("Рождество Христово", "рождество( христово)?", month_day(1, 7)),
        ("День защитника отечества", "день защитника", month_day(2, 23)),
        ("Международный женский день", "международный женский день", month_day(3, 8)),
        ("День смеха", "день смеха", month_day(4, 1)),
        ("Праздник Весны и Труда", "день труда", month_day(5, 1)),
        ("День Победы", "День Победы", month_day(5, 9)),
        ("День России", "день россии", month_day(6, 12)),
        ("День народного единства", "день единства", month_day(11, 4)),
    ])
}

fn rules_main() -> Vec<Rule> {
    vec![
        rule("absorption of , after named day", vec![pred(is_a_day_of_week), regex(",")], |ts| {
            Some(ts[0].clone())
        }),
        rule("after <time-of-day>", vec![regex("после"), dimension(Dim::Time)], |ts| {
            tt(with_direction(IntervalDirection::After, td(ts[1])?.clone()))
        }),
        rule("afternoon", vec![regex("(дня|дн[её]м)|пополудни|после полудня")], |_| {
            latent_pod(12, 19)
        }),
        rule("at <time-of-day>", vec![regex("в"), pred(is_a_time_of_day)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        rule(
            "between <time-of-day> and <time-of-day> (interval)",
            vec![regex("с"), pred(is_a_time_of_day), regex("по"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "<datetime> - <datetime> (interval)",
            vec![pred(is_not_latent), regex("\\-"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "dd.(mm.)? - dd.mm.(yy[yy]?)? (interval)",
            vec![
                regex("(?:с\\s+)?(10|20|30|31|[012]?[1-9])\\.?((?<=\\.)(?:10|11|12|0?[1-9])(?:\\.?))?" ),
                regex("\\-|/|по"),
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
        rule(
            "<day-of-month> (non ordinal) <named-month>",
            vec![pred(is_dom_integer), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
        ),
        rule("<day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
            tt(day_of_month(ordinal_value(ts[0])?))
        }),
        rule(
            "<day-of-month>(ordinal) <named-month>",
            vec![pred(is_dom_ordinal), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
        ),
        rule(
            "<day-of-month>(ordinal) <named-month> year",
            vec![pred(is_dom_ordinal), pred(is_a_month), regex("(\\d{2,4})")],
            |ts| {
                let n = parse_int(&rx(ts[2], 0)?)?;
                let dom = intersect_dom(td(ts[1])?, ts[0])?;
                ttd(intersect(&dom, &year(n)))
            },
        ),
        rule(
            "<duration> after <time>",
            vec![dimension(Dim::Duration), regex("после"), dimension(Dim::Time)],
            |ts| tt(duration_after(dur(ts[0])?, td(ts[2])?)),
        ),
        rule("<duration> ago", vec![dimension(Dim::Duration), regex("(тому )?назад")], |ts| {
            tt(duration_ago(dur(ts[0])?))
        }),
        rule(
            "<duration> before <time>",
            vec![dimension(Dim::Duration), regex("перед"), dimension(Dim::Time)],
            |ts| tt(duration_before(dur(ts[0])?, td(ts[2])?)),
        ),
        rule("evening", vec![regex("вечера|вечером|вечер")], |_| latent_pod(18, 0)),
        rule(
            "hh:mm",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:ч]([0-5]\\d)(?:час(ов|а|у)?|ч)?")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(hour_minute(false, h, m))
            },
        ),
        rule("hhmm (military)", vec![regex("((?:[01]?\\d)|(?:2[0-3]))([0-5]\\d)")], |ts| {
            let hh = parse_int(&rx(ts[0], 0)?)?;
            let mm = parse_int(&rx(ts[0], 1)?)?;
            tt(mk_latent(hour_minute(false, hh, mm)))
        }),
        rule(
            "<hour-of-day> <integer> (as relative minutes)",
            vec![pred(not_latent_hour), pred(is_integer_between_tok(1, 59))],
            |ts| {
                let (hours, is12h) = tod(ts[0])?;
                let n = get_int_value(ts[1])?;
                tt(hour_minute(is12h, hours, n))
            },
        ),
        rule("in <duration>", vec![regex("через"), dimension(Dim::Duration)], |ts| {
            tt(in_duration(dur(ts[1])?))
        }),
        rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule(
            "intersect by ','",
            vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "intersect by 'of', 'from', 's",
            vec![pred(is_not_latent), regex("на"), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "last <cycle>",
            vec![
                regex("(на |в )?прошл(ый|ого|ому|ом|ое|ые|ых|ыми|ым|ая|ой|ую)"),
                dimension(Dim::TimeGrain),
            ],
            |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
        ),
        rule(
            "one <cycle> before last",
            vec![
                regex("(на |в )?позапрошл(ый|ого|ому|ыми|ом|ое|ые|ых|ым|ая|ой|ую)"),
                dimension(Dim::TimeGrain),
            ],
            |ts| tt(cycle_nth(grain_of(ts[1])?, -2)),
        ),
        rule(
            "last <cycle>",
            vec![regex("(в )?прошедш(ий|его|ему|ими|ем|их|ие|им)"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
        ),
        rule(
            "last <cycle> of <time>",
            vec![
                regex("последн(ий|юю|яя|его|ему)"),
                dimension(Dim::TimeGrain),
                regex("в"),
                dimension(Dim::Time),
            ],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "last <cycle> of <time>",
            vec![regex("последн(ий|юю|яя|его|ему)"), dimension(Dim::TimeGrain), dimension(Dim::Time)],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[2])?)),
        ),
        rule(
            "last <day-of-week> of <time>",
            vec![regex("последний"), pred(is_a_day_of_week), regex("в"), dimension(Dim::Time)],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "last n <cycle>",
            vec![regex("последние"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -n))
            },
        ),
        rule(
            "last <time>",
            vec![
                regex("(на |в )?прошл(ый|ого|ому|ом|ое|ые|ых|ыми|ым|ая|ой|ую)"),
                pred(is_ok_with_this_next),
            ],
            |ts| tt(pred_nth(-1, false, td(ts[1])?)),
        ),
        rule("lunch", vec![regex("(в )?обед")], |_| latent_pod(12, 14)),
        rule("midnight|EOD|end of day", vec![regex("кон(ец|ц(а|е)) дня")], |_| {
            tt(hour(false, 0))
        }),
        rule(
            "mm/dd",
            vec![regex("([012]?[1-9]|10|20|30|31)\\.(10|11|12|0?[1-9])\\.?")],
            |ts| {
                let d = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                tt(month_day(m, d))
            },
        ),
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
        rule(
            "<month> dd-dd (interval)",
            vec![
                regex("(?:с\\s+)?([012]?\\d|30|31)(го|\\.)?"),
                regex("\\-|по|до"),
                regex("([012]?\\d|30|31)(ое|\\.)?"),
                pred(is_a_month),
            ],
            |ts| {
                let v1 = parse_int(&rx(ts[0], 0)?)?;
                let v2 = parse_int(&rx(ts[2], 0)?)?;
                let from = intersect(&day_of_month(v1), td(ts[3])?)?;
                let to = intersect(&day_of_month(v2), td(ts[3])?)?;
                ttd(interval(Closed, &from, &to))
            },
        ),
        rule("morning", vec![regex("утром|утро|утра")], |_| latent_pod(3, 12)),
        rule(
            "<named-month> <day-of-month> (non ordinal)",
            vec![pred(is_a_month), pred(is_dom_integer)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        rule(
            "<named-month> <day-of-month> (ordinal)",
            vec![pred(is_a_month), pred(is_dom_ordinal)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        rule(
            "next <cycle>",
            vec![
                regex("(на |в |к )?след(ующ(ий|его|ему|ими|ем|ие|их|им|ей|ая|ую))?"),
                dimension(Dim::TimeGrain),
            ],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 1)),
        ),
        rule(
            "next n <cycle>",
            vec![regex("следующие"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
            |ts| {
                let v = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, v))
            },
        ),
        rule("night", vec![regex("ночью|ночи|ночь")], |_| latent_pod(0, 4)),
        rule("noon", vec![regex("полдень")], |_| tt(hour(false, 12))),
        rule("midnight", vec![regex("полночь")], |_| tt(hour(false, 0))),
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("после"), dimension(Dim::Time)],
            |ts| {
                let v = ordinal_value(ts[0])?;
                tt(pred_nth_after(v - 1, td(ts[1])?, td(ts[3])?))
            },
        ),
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("в"), dimension(Dim::Time)],
            |ts| {
                let v = ordinal_value(ts[0])?;
                ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(v - 1, false, &x)))
            },
        ),
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), dimension(Dim::Time)],
            |ts| {
                let v = ordinal_value(ts[0])?;
                ttd(intersect(td(ts[2])?, td(ts[1])?).map(|x| pred_nth(v - 1, false, &x)))
            },
        ),
        rule("on <date>", vec![regex("во|в|к"), dimension(Dim::Time)], |ts| {
            Some(ts[1].clone())
        }),
        rule(
            "<ordinal> <cycle> of <time>",
            vec![
                dimension(Dim::Ordinal),
                dimension(Dim::TimeGrain),
                regex("в"),
                dimension(Dim::Time),
            ],
            |ts| {
                tt(cycle_nth_after(true, grain_of(ts[1])?, ordinal_value(ts[0])? - 1, td(ts[3])?))
            },
        ),
        rule(
            "<ordinal> quarter",
            vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))],
            |ts| {
                let v = ordinal_value(ts[0])?;
                tt(cycle_nth_after(false, G::Quarter, v - 1, &cycle_nth(G::Year, 0)))
            },
        ),
        rule(
            "<ordinal> quarter <year>",
            vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter)), dimension(Dim::Time)],
            |ts| {
                tt(cycle_nth_after(false, G::Quarter, ordinal_value(ts[0])? - 1, td(ts[2])?))
            },
        ),
        rule("<part-of-day> of <time>", vec![pred(is_a_part_of_day), dimension(Dim::Time)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule(
            "this <cycle>",
            vec![regex("(в )?(эту|этот|этого|этому|эти)"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
        ),
        rule(
            "this <part-of-day>",
            vec![regex("(в )?(эту|этот|этого|этому|эти|это)"), pred(is_a_part_of_day)],
            |ts| ttd(intersect(&today(), td(ts[1])?).map(part_of_day)),
        ),
        rule(
            "this <time>",
            vec![regex("(в )?(эту|этот|этого|этому|эти|это)"), pred(is_ok_with_this_next)],
            |ts| tt(pred_nth(0, false, td(ts[1])?)),
        ),
        rule(
            "this|next <day-of-week>",
            vec![regex("след(ующ(ий|ая|ее|ую))?"), pred(is_a_day_of_week)],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        rule("<time> after next", vec![dimension(Dim::Time), regex("после следующ(ей|его)")], |ts| {
            tt(pred_nth(1, true, td(ts[0])?))
        }),
        rule("<time> <part-of-day>", vec![dimension(Dim::Time), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
            let n = get_int_value(ts[0])?;
            tt(mk_latent(hour(n < 12, n)))
        }),
        rule(
            "<time-of-day>  o'clock",
            vec![pred(is_a_time_of_day), regex("час(у|а|ов)?|ч(?:[\\s'\"-_{}\\[\\]()]|$)")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(not_latent_tod), regex("\\-"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(is_a_time_of_day), regex("\\-|/"), pred(not_latent_tod)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule("until <time-of-day>", vec![regex("до"), dimension(Dim::Time)], |ts| {
            tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone()))
        }),
        rule("week", vec![regex("(эта )?неделя")], |ts| {
            let end = cycle_nth_after(true, G::Day, -2, &cycle_nth(G::Week, 1));
            let m = to_lower(&rx(ts[0], 0)?);
            let match_s = m.trim();
            let period = match match_s {
                "эта" => interval(Closed, &cycle_nth(G::Week, 0), &end)?,
                "неделя" => interval(Open, &today(), &end)?,
                _ => return None,
            };
            let period = if match_s == "неделя" { mk_latent(period) } else { period };
            Some(Token::Time(period))
        }),
        rule("week-end", vec![regex("выходные")], |_| tt(mk_ok_for_this_next(weekend()))),
        rule("within <duration>", vec![regex("в течение"), dimension(Dim::Duration)], |ts| {
            ttd(interval(Open, &now(), &in_duration(dur(ts[1])?)))
        }),
        rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
            tt(year(get_int_value(ts[0])?))
        }),
        rule("year (latent)", vec![pred(is_integer_between_tok(25, 999))], |ts| {
            tt(mk_latent(year(get_int_value(ts[0])?)))
        }),
        rule("year (latent)", vec![pred(is_integer_between_tok(2101, 10000))], |ts| {
            tt(mk_latent(year(get_int_value(ts[0])?)))
        }),
        rule(
            "yyyy-mm-dd",
            vec![regex("(\\d{2,4})-(0?[1-9]|10|11|12)-([012]?[1-9]|10|20|30|31)")],
            |ts| {
                let y = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let d = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        rule(
            "<time> timezone",
            vec![
                pred(not_latent_tod),
                regex(&format!("\\b({TIMEZONE_NAME})\\b")),
            ],
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
    v.extend(rules_holidays());
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
