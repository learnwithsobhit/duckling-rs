//! `Duckling.Time.KO.Rules`.

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

/// `TimeData {form = Just (TimeOfDay (Just hours) _)}`.
fn tod_hours(t: &Token) -> Option<i64> {
    match t {
        Token::Time(TimeData { form: Some(Form::TimeOfDay { hours: Some(h), .. }), .. }) => Some(*h),
        _ => None,
    }
}

/// `Duckling.Duration.Helpers.isGrain`.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn not_latent_and_hour(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

fn tod_or_pod(t: &Token) -> bool {
    is_a_time_of_day(t) || is_a_part_of_day(t)
}

fn korean_unit(s: &str) -> i64 {
    match s {
        "일" => 1,
        "이" => 2,
        "삼" => 3,
        "사" => 4,
        "오" => 5,
        "육" => 6,
        "칠" => 7,
        "팔" => 8,
        "구" => 9,
        _ => 1,
    }
}

const TZ: &str = "\\b(YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT)\\b";

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = vec![
        rule("about <time-of-day>", vec![regex("대충|약"), pred(is_a_time_of_day)], |ts| {
            tt(not_latent(td(ts.get(1)?)?.clone()))
        }),
        rule("absorption of , after named day", vec![pred(is_a_day_of_week), regex(",")], |ts| {
            Some(ts[0].clone())
        }),
        rule("after <duration>", vec![dimension(Dim::Duration), regex("(이)?후")], |ts| {
            tt(with_direction(IntervalDirection::After, in_duration(dur(ts[0])?)))
        }),
        rule("after <part-of-day>", vec![pred(is_a_part_of_day), regex("지나서|후에")], |ts| {
            ttd(intersect(&today(), td(ts[0])?).map(part_of_day))
        }),
        rule(
            "after <time-of-day>",
            vec![pred(tod_or_pod), regex("지나서|(이)?후(에)?")],
            |ts| tt(not_latent(with_direction(IntervalDirection::After, td(ts[0])?.clone()))),
        ),
        rule("afternoon", vec![regex("오후")], |_| {
            ttd(interval(Open, &hour(false, 12), &hour(false, 19)).map(|x| mk_latent(part_of_day(x))))
        }),
        rule(
            "am|pm <time-of-day>",
            vec![regex("(오전|아침|오후|저녁)"), pred(is_a_time_of_day)],
            |ts| {
                let m = rx(ts[0], 0)?;
                tt(time_of_day_ampm(m == "오전" || m == "아침", td(ts[1])?))
            },
        ),
        rule("by <time> - 까지", vec![dimension(Dim::Time), regex("까지")], |ts| {
            ttd(interval(Open, &now(), td(ts[0])?))
        }),
        rule("<date>에", vec![dimension(Dim::Time), regex("에")], |ts| Some(ts[0].clone())),
        rule(
            "<datetime> - <datetime> (interval)",
            vec![pred(is_not_latent), regex("\\-|\\~|부터"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule("day", vec![pred(is_dom_integer), regex("일")], |ts| {
            tt(day_of_month(get_int_value(ts[0])?))
        }),
        rule(
            "day with korean number - 십일..삼십일일",
            vec![regex("((이|삼)?십(일|이|삼|사|오|육|칠|팔|구)?)일")],
            |ts| {
                let m1 = rx(ts[0], 1).unwrap_or_default();
                let m2 = rx(ts[0], 2).unwrap_or_default();
                let dozens = match m1.as_str() {
                    "이" => 2,
                    "삼" => 3,
                    _ => 1,
                };
                tt(day_of_month(10 * dozens + korean_unit(&m2)))
            },
        ),
        rule("day with korean number - 일일..구일", vec![regex("(일|이|삼|사|오|육|칠|팔|구)일")], |ts| {
            match rx(ts[0], 0)?.as_str() {
                "일" => tt(day_of_month(1)),
                "이" => tt(day_of_month(2)),
                "삼" => tt(day_of_month(3)),
                "사" => tt(day_of_month(4)),
                "오" => tt(day_of_month(5)),
                "육" => tt(day_of_month(6)),
                "칠" => tt(day_of_month(7)),
                "팔" => tt(day_of_month(8)),
                "구" => tt(day_of_month(9)),
                _ => None,
            }
        }),
        rule("<duration> ago", vec![dimension(Dim::Duration), regex("(이)?전")], |ts| {
            tt(duration_ago(dur(ts[0])?))
        }),
        // Upstream pattern is [regex, Duration] but the production matches Duration first: never fires.
        rule("<duration> from now", vec![regex("지금부터"), dimension(Dim::Duration)], |ts| {
            match ts.first()? {
                Token::Duration(dd) => tt(in_duration(dd)),
                _ => None,
            }
        }),
        rule("end of <time>", vec![dimension(Dim::Time), regex("말")], |ts| {
            tt(pred_nth(1, false, td(ts[0])?))
        }),
        rule("evening|night", vec![regex("저녁|밤")], |_| {
            ttd(interval(Open, &hour(false, 18), &hour(false, 0)).map(|x| part_of_day(mk_latent(x))))
        }),
        rule("exactly <time-of-day>", vec![pred(is_a_time_of_day), regex("정각")], |ts| {
            tt(not_latent(td(ts[0])?.clone()))
        }),
        rule("hh:mm", vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.]([0-5]\\d)")], |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(true, h, m))
        }),
        rule(
            "hhmm (military) am|pm",
            vec![regex("((?:1[012]|0?\\d))([0-5]\\d)"), regex("([ap])\\.?m?\\.?")],
            |ts| {
                let h = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let ap = rx(ts[1], 0)?;
                tt(time_of_day_ampm(to_lower(&ap) == "a", &hour_minute(true, h, m)))
            },
        ),
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
        rule(
            "<hour-of-day> <integer> (as relative minutes)",
            vec![pred(not_latent_and_hour), pred(is_integer_between_tok(1, 59)), regex("분")],
            |ts| {
                let hours = tod_hours(ts[0])?;
                let n = get_int_value(ts[1])?;
                tt(hour_minute(true, hours, n))
            },
        ),
        rule(
            "<hour-of-day> half (as relative minutes)",
            vec![pred(is_an_hour_of_day), regex("반")],
            |ts| {
                let hours = tod_hours(ts[0])?;
                tt(hour_minute(true, hours, 30))
            },
        ),
        rule("in <duration>", vec![dimension(Dim::Duration), regex("(안|내)(에)?")], |ts| {
            tt(in_duration(dur(ts[0])?))
        }),
        rule("in|during the <part-of-day>", vec![pred(is_a_part_of_day), regex("에|동안")], |ts| {
            tt(not_latent(td(ts[0])?.clone()))
        }),
        rule(
            "<integer> (hour-of-day) relative minutes 전",
            vec![pred(not_latent_and_hour), pred(is_integer_between_tok(1, 59)), regex("분전")],
            |ts| ttd(minutes_before(get_int_value(ts[1])?, td(ts[0])?)),
        ),
        rule("half (hour-of-day) relative minutes 전", vec![pred(is_an_hour_of_day), regex("반전")], |ts| {
            ttd(minutes_before(30, td(ts[0])?))
        }),
        rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule(
            "intersect by \",\"",
            vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        rule("last <cycle>", vec![regex("지난|작|전|저번"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, -1))
        }),
        rule(
            "last n <cycle>",
            vec![regex("지난"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -n))
            },
        ),
        rule("last <time>", vec![regex("전|저번|지난"), dimension(Dim::Time)], |ts| {
            tt(pred_nth(-1, false, td(ts[1])?))
        }),
        rule("lunch", vec![regex("점심")], |_| {
            ttd(interval(Open, &hour(false, 12), &hour(false, 14)).map(|x| mk_latent(part_of_day(x))))
        }),
        rule("midnight|EOD|end of day", vec![regex("자정")], |_| tt(hour(false, 0))),
        rule("mm/dd", vec![regex("(0?[1-9]|1[0-2])/(3[01]|[12]\\d|0?[1-9])")], |ts| {
            let m = parse_int(&rx(ts[0], 0)?)?;
            let d = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        }),
        rule(
            "mm/dd/yyyy",
            vec![regex("(\\d{2,4})[-/](0?[1-9]|1[0-2])[/-](3[01]|[12]\\d|0?[1-9])")],
            |ts| {
                let y = parse_int(&rx(ts[0], 0)?)?;
                let m = parse_int(&rx(ts[0], 1)?)?;
                let d = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
        rule("month", vec![pred(is_integer_between_tok(1, 12)), regex("월")], |ts| {
            tt(month(get_int_value(ts[0])?))
        }),
        rule("morning", vec![regex("아침")], |_| {
            ttd(interval(Open, &hour(false, 4), &hour(false, 12)).map(|x| mk_latent(part_of_day(x))))
        }),
        rule("<named-day>에", vec![pred(is_a_day_of_week), regex("에")], |ts| Some(ts[0].clone())),
        rule("<named-month>에", vec![pred(is_a_month), regex("에")], |ts| Some(ts[0].clone())),
        rule("next <cycle>", vec![regex("다음|오는|차|내"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, 1))
        }),
        rule(
            "next n <cycle>",
            vec![regex("다음"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, n))
            },
        ),
        rule("next <time>", vec![regex("다음|오는"), pred(is_not_latent)], |ts| {
            tt(pred_nth(0, true, td(ts[1])?))
        }),
        rule("noon", vec![regex("정오")], |_| tt(hour(false, 12))),
        rule("now", vec![regex("방금|지금|방금|막")], |_| tt(now())),
        rule(
            "<1..4> quarter",
            vec![pred(is_integer_between_tok(1, 4)), pred(is_grain(G::Quarter))],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(cycle_nth_after(false, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
            },
        ),
        rule("since <time-of-day>", vec![pred(is_a_time_of_day), regex("이래로")], |ts| {
            tt(with_direction(IntervalDirection::After, pred_nth(-1, false, td(ts[0])?)))
        }),
        rule("the day after tomorrow - 내일모레", vec![regex("(내일)?모레")], |_| {
            tt(cycle_nth_after(false, G::Day, 1, &cycle_nth(G::Day, 1)))
        }),
        rule("the day before yesterday - 엊그제", vec![regex("(엊)?그(제|재)")], |_| {
            tt(cycle_nth_after(false, G::Day, -1, &cycle_nth(G::Day, -1)))
        }),
        rule("this <cycle>", vec![regex("이번|금|올"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, 0))
        }),
        rule("this <day-of-week>", vec![regex("이번(주)?|금주"), pred(is_a_day_of_week)], |ts| {
            tt(pred_nth(0, false, td(ts[1])?))
        }),
        rule("this <time>", vec![regex("이번"), dimension(Dim::Time)], |ts| {
            tt(pred_nth(0, false, td(ts[1])?))
        }),
        rule("<time> after next", vec![dimension(Dim::Time), regex("after next")], |ts| {
            tt(pred_nth(1, true, td(ts[0])?))
        }),
        // Upstream pattern is [Time, "마지막", TimeGrain] but the production matches TimeGrain first: never fires.
        rule("<time> 마지막 <cycle> ", vec![dimension(Dim::Time), regex("마지막"), dimension(Dim::TimeGrain)], |ts| {
            match (ts.first()?, ts.get(2)?) {
                (Token::TimeGrain(grain), Token::Time(td)) => tt(cycle_last_of(*grain, td)),
                _ => None,
            }
        }),
        rule("<time> 마지막 <cycle>", vec![dimension(Dim::Time), regex("마지막"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_last_of(grain_of(ts[2])?, td(ts[0])?))
        }),
        rule(
            "<time> 마지막 <day-of-week>",
            vec![dimension(Dim::Time), regex("마지막"), pred(is_a_day_of_week)],
            |ts| tt(pred_last_of(td(ts[2])?, td(ts[0])?)),
        ),
        rule(
            "<time> nth <time> - 3월 첫째 화요일",
            vec![dimension(Dim::Time), dimension(Dim::Ordinal), dimension(Dim::Time)],
            |ts| {
                let v = ordinal_value(ts[1])?;
                ttd(intersect(td(ts[0])?, td(ts[2])?).map(|x| pred_nth(v - 1, false, &x)))
            },
        ),
        rule(
            "<time> <ordinal> <cycle>",
            vec![dimension(Dim::Time), dimension(Dim::Ordinal), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[0])?)),
        ),
        rule("<time> <part-of-day>", vec![dimension(Dim::Time), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule("time-of-day", vec![pred(is_integer_between_tok(0, 24)), regex("시")], |ts| {
            tt(hour(true, get_int_value(ts[0])?))
        }),
        // Upstream pattern is [TimeOfDay, "에"] but the production skips the first token: never fires.
        rule("<time-of-day>에", vec![pred(is_a_time_of_day), regex("에")], |ts| match ts.get(1) {
            Some(Token::Time(td)) => tt(not_latent(td.clone())),
            _ => None,
        }),
        rule("<time-of-day> 정각", vec![pred(is_a_time_of_day), regex("정각")], |ts| {
            tt(not_latent(td(ts[0])?.clone()))
        }),
        rule("<time-of-day>이전", vec![pred(is_a_time_of_day), regex("(이)?전")], |ts| {
            tt(with_direction(IntervalDirection::Before, td(ts[0])?.clone()))
        }),
        rule(
            "<time-of-day> am|pm",
            vec![pred(is_a_time_of_day), regex("(in the )?([ap])(\\s|\\.)?m?\\.?")],
            |ts| {
                let ap = rx(ts[1], 1)?;
                tt(time_of_day_ampm(to_lower(&ap) == "a", td(ts[0])?))
            },
        ),
        rule("<time-of-day> approximately", vec![pred(is_a_time_of_day), regex("정도|쯤")], |ts| {
            tt(not_latent(td(ts[0])?.clone()))
        }),
        rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
            tt(mk_latent(hour(true, get_int_value(ts[0])?)))
        }),
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![
                pred(|t| is_not_latent(t) && is_a_time_of_day(t)),
                regex("\\-|\\~|부터"),
                pred(is_a_time_of_day),
            ],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule("today", vec![regex("오늘|당일|금일")], |_| tt(today())),
        rule("tomorrow", vec![regex("내일|명일|낼")], |_| tt(cycle_nth(G::Day, 1))),
        rule("week-end", vec![regex("주말")], |_| tt(weekend())),
        rule("within <duration>", vec![dimension(Dim::Duration), regex("이내(에)?")], |ts| {
            ttd(interval(Open, &now(), &in_duration(dur(ts[0])?)))
        }),
        rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
            tt(year(get_int_value(ts[0])?))
        }),
        rule("year", vec![pred(is_integer_between_tok(1, 2100)), regex("년")], |ts| {
            tt(year(get_int_value(ts[0])?))
        }),
        rule("year (latent)", vec![pred(is_integer_between_tok(1, 999))], |ts| {
            tt(mk_latent(year(get_int_value(ts[0])?)))
        }),
        rule("year (latent)", vec![pred(is_integer_between_tok(2101, 10000))], |ts| {
            tt(mk_latent(year(get_int_value(ts[0])?)))
        }),
        rule(
            "<year> <1..4>quarter",
            vec![dimension(Dim::Time), pred(is_integer_between_tok(1, 4)), pred(is_grain(G::Quarter))],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_nth_after(false, G::Quarter, n - 1, td(ts[0])?))
            },
        ),
        rule("yesterday", vec![regex("어제")], |_| tt(cycle_nth(G::Day, -1))),
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
        rule("seconds", vec![pred(is_integer_between_tok(0, 59)), regex("초")], |ts| {
            tt(second(get_int_value(ts[0])?))
        }),
        rule(
            "<time> timezone",
            vec![pred(|t| is_not_latent(t) && is_a_time_of_day(t)), regex(TZ)],
            |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 0)?), td(ts[0])?)),
        ),
    ];
    v.extend(mk_rule_holidays(&[
        ("Liberation Day", "광복절", month_day(8, 15)),
        ("Constitution Day", "제헌절", month_day(7, 17)),
        ("New Year's Day", "(신정)|(설날)", month_day(1, 1)),
        ("Hangul Day", "한글날", month_day(10, 9)),
        ("National Foundation Day", "개천절", month_day(10, 3)),
        ("Independence Movement Day", "삼일절", month_day(3, 1)),
        ("Memorial Day", "현충일", month_day(6, 6)),
        ("Christmas", "크리스마스", month_day(12, 25)),
        ("Christmas Eve", "(크리스마스)?이브", month_day(12, 24)),
        ("Children's Day", "어린이날", month_day(5, 5)),
    ]));
    v.extend(mk_rule_seasons(&[
        ("Summer", "여름", month_day(6, 21), month_day(9, 23)),
        ("Fall", "가을", month_day(9, 23), month_day(12, 21)),
        ("Winter", "겨울", month_day(12, 21), month_day(3, 20)),
        ("Spring", "봄", month_day(3, 20), month_day(6, 21)),
    ]));
    v.extend(mk_rule_days_of_week(&[
        ("Monday", "월(요일|욜)"),
        ("Tuesday", "화(요일|욜)"),
        ("Wednesday", "수(요일|욜)"),
        ("Thursday", "목(요일|욜)"),
        ("Friday", "금(요일|욜)"),
        ("Saturday", "토(요일|욜)"),
        ("Sunday", "일(요일|욜)"),
    ]));
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
