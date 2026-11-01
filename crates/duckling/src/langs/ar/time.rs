//! `Duckling.Time.AR.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::duration::DurationData;
use crate::dims::time::computed::*;
use crate::dims::time::helpers::*;
use crate::dims::time::types::{
    Form, IntervalDirection,
    TimeData,
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

fn tod(t: &Token) -> Option<(i64, bool)> {
    match t {
        Token::Time(TimeData { form: Some(Form::TimeOfDay { hours: Some(h), is12h }), .. }) => {
            Some((*h, *is12h))
        }
        _ => None,
    }
}

fn not_latent_ok(t: &Token) -> bool {
    is_not_latent(t) && is_ok_with_this_next(t)
}

fn not_latent_hod(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

fn not_latent_tod(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t)
}

fn month_or_dow(t: &Token) -> bool {
    is_a_month(t) || is_a_day_of_week(t)
}

static PAR_OF_DAYS: Lazy<HashMap<&'static str, (i64, i64)>> = Lazy::new(|| {
    [
        ("فجر", (3, 6)),
        ("صبح", (0, 12)),
        ("صباح", (0, 12)),
        ("نهار", (6, 17)),
        ("ظهر", (11, 15)),
        ("ظهيرة", (11, 15)),
        ("عصر", (15, 18)),
        ("مغرب", (17, 21)),
        ("غداء", (13, 16)),
        ("عشاء", (18, 3)),
        ("ليلة", (18, 3)),
        ("ليله", (18, 3)),
        ("مساء", (12, 24)),
    ]
    .into_iter()
    .collect()
});

fn build_rules() -> Vec<Rule> {
    let mut v = vec![
        rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule(
            "intersect by \",\", \"of\", \"from\", \"'s\"",
            vec![pred(is_not_latent), regex("من|,|،|لـ?|بـ?|الموافق( ل)?"), pred(is_not_latent)],
            |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
        ),
        rule("on <date>", vec![regex("في"), dimension(Dim::Time)], |ts| Some(ts[1].clone())),
        rule("on a <named-day>", vec![regex("يوم"), pred(is_a_day_of_week)], |ts| {
            Some(ts[1].clone())
        }),
        rule("in <named-month>", vec![regex("شهر"), pred(is_a_month)], |ts| Some(ts[1].clone())),
        rule("absorption of , after named day", vec![pred(is_a_day_of_week), regex("[,،]")], |ts| {
            Some(ts[0].clone())
        }),
        rule("this <day-of-week>", vec![regex("هذا"), pred(is_a_day_of_week)], |ts| {
            tt(pred_nth(0, true, td(ts[1])?))
        }),
        rule("<day-of-week> next", vec![pred(is_a_day_of_week), regex("القادم|التالي|ال[آا]تي?|القادم")], |ts| {
            tt(pred_nth(0, true, td(ts[0])?))
        }),
        rule("<day-of-week> last", vec![pred(is_a_day_of_week), regex("السابق|الماضي|المنصرم|الفا[يئ]ت")], |ts| {
            tt(pred_nth(-1, true, td(ts[0])?))
        }),
        rule("this <time>", vec![regex("هذا"), pred(not_latent_ok)], |ts| {
            tt(pred_nth(0, false, td(ts[1])?))
        }),
        // Upstream production skips the Time token (`_:Token Time`); never fires.
        rule("<time> next", vec![pred(not_latent_ok), regex("التالي")], |ts| {
            tt(pred_nth(0, true, td(ts.get(1)?)?))
        }),
        rule("last <time>", vec![regex("([اآ]خر)"), pred(not_latent_ok)], |ts| {
            tt(pred_nth(-1, false, td(ts[1])?))
        }),
        rule(
            "<time> last",
            vec![
                pred(not_latent_ok),
                regex("((ال)?ماضيت?(ان|ين|ة|ه)?|(ال)?منصرمت?(ان|ين|ة|ه)?|(ال)?سابقت?(ان|ين|ة|ه)?)"),
            ],
            |ts| tt(pred_nth(-1, false, td(ts[0])?)),
        ),
        rule(
            "last <day-of-week> of <time>",
            vec![regex("[اآ]خر"), pred(is_a_day_of_week), regex("من|في"), dimension(Dim::Time)],
            |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "<day-of-week> the last of <time>",
            vec![pred(is_a_day_of_week), regex("(ال[اأآ]خي?رة?) (من|في|ب|ل)"), dimension(Dim::Time)],
            |ts| tt(pred_last_of(td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "last <cycle> of <time>",
            vec![regex("[اآ]خر"), dimension(Dim::TimeGrain), regex("(من|في|ل|ب)"), dimension(Dim::Time)],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "<cycle> last of <time>",
            vec![regex("ال"), dimension(Dim::TimeGrain), regex("ال[اأ]خيرة? (من|في|ل|ب)"), dimension(Dim::Time)],
            |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("في|من|ب"), dimension(Dim::Time)],
            |ts| {
                let n = get_int_value(ts[0])?;
                ttd(intersect(td(ts[3])?, td(ts[1])?).map(|x| pred_nth(n - 1, false, &x)))
            },
        ),
        rule(
            "<time> nth of <time>",
            vec![dimension(Dim::Time), dimension(Dim::Ordinal), regex("من|في"), dimension(Dim::Time)],
            |ts| {
                let n = get_int_value(ts[1])?;
                ttd(intersect(td(ts[3])?, td(ts[0])?).map(|x| pred_nth(n - 1, false, &x)))
            },
        ),
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::Time), regex("بعد"), dimension(Dim::Time)],
            |ts| tt(pred_nth_after(get_int_value(ts[0])? - 1, td(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "<time> nth after <time>",
            vec![dimension(Dim::Time), dimension(Dim::Ordinal), regex("بعد"), dimension(Dim::Time)],
            |ts| tt(pred_nth_after(get_int_value(ts[1])? - 1, td(ts[0])?, td(ts[3])?)),
        ),
        rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
            tt(year(get_int_value(ts[0])?))
        }),
        rule("past year (latent)", vec![pred(is_integer_between_tok(25, 999))], |ts| {
            tt(mk_latent(year(get_int_value(ts[0])?)))
        }),
        rule("future year (latent)", vec![pred(is_integer_between_tok(2101, 10000))], |ts| {
            tt(mk_latent(year(get_int_value(ts[0])?)))
        }),
        rule("year (integer)", vec![regex("سنة|عام"), pred(is_integer_between_tok(1, 3000))], |ts| {
            tt(year(get_int_value(ts[1])?))
        }),
        rule("month (integer)", vec![regex("شهر"), pred(is_integer_between_tok(1, 12))], |ts| {
            tt(month(get_int_value(ts[1])?))
        }),
        rule("month (ordinal)", vec![regex("الشهر"), dimension(Dim::Ordinal)], |ts| {
            tt(month(get_int_value(ts[1])?))
        }),
        rule("<day-of-month> (ordinal)", vec![pred(is_dom_ordinal)], |ts| {
            tt(mk_latent(day_of_month(get_int_value(ts[0])?)))
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
            "<day-of-month> (ordinal or number) of <named-month>",
            vec![pred(is_dom_value), regex("(من|في|بـ?)"), pred(is_a_month)],
            |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
        ),
        rule(
            "<day-of-month>(ordinal) <named-month> year",
            vec![pred(is_dom_ordinal), pred(is_a_month), regex(r"(\d{2,4})")],
            |ts| {
                let y = parse_int(&rx(ts[2], 0)?)?;
                let dom = intersect_dom(td(ts[1])?, ts[0])?;
                ttd(intersect(&dom, &year(y)))
            },
        ),
        rule(
            "<day-of-month>(ordinal) <named-month> year",
            vec![pred(is_dom_ordinal), regex("يوم( من| في| بـ?)?"), pred(is_a_month), regex(r"(\d{2,4})")],
            |ts| {
                let y = parse_int(&rx(ts[3], 0)?)?;
                let dom = intersect_dom(td(ts[2])?, ts[0])?;
                ttd(intersect(&dom, &year(y)))
            },
        ),
        rule("time-of-day integer (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
            tt(mk_latent(hour(true, get_int_value(ts[0])?)))
        }),
        rule("time-of-day ordinal (latent)", vec![pred(is_ordinal_between(1, 24))], |ts| {
            tt(mk_latent(hour(true, get_int_value(ts[0])?)))
        }),
        rule("at <time-of-day>", vec![regex("عند ((ال)?ساع[ةه])?"), pred(is_a_time_of_day)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        rule(
            "<hour-of-day> and integer",
            vec![pred(not_latent_hod), regex("و"), pred(is_integer_between_tok(0, 60))],
            |ts| {
                let (hours, is12h) = tod(ts[0])?;
                match ts[2] {
                    Token::Numeral(n) => tt(hour_minute(is12h, hours, n.value.floor() as i64)),
                    _ => None,
                }
            },
        ),
        rule(
            "<hour-of-day> and integer minutes",
            vec![
                pred(is_an_hour_of_day),
                regex("و"),
                pred(is_integer_between_tok(0, 60)),
                regex("دقيق[ةه]|دقا[ئي]ق"),
            ],
            |ts| {
                let (hours, is12h) = tod(ts[0])?;
                match ts[2] {
                    Token::Numeral(n) => tt(hour_minute(is12h, hours, n.value.floor() as i64)),
                    _ => None,
                }
            },
        ),
        rule("at <time-of-day>", vec![regex("(ال)?ساع[ةه]"), pred(is_a_time_of_day)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        rule("hh:mm", vec![regex(r"((?:[01]?\d)|(?:2[0-3]))[:.]([0-5]\d)")], |ts| {
            tt(hour_minute(true, parse_int(&rx(ts[0], 0)?)?, parse_int(&rx(ts[0], 1)?)?))
        }),
        rule("hhmm (latent)", vec![regex(r"((?:[01]?\d)|(?:2[0-3]))([0-5]\d)")], |ts| {
            tt(mk_latent(hour_minute(true, parse_int(&rx(ts[0], 0)?)?, parse_int(&rx(ts[0], 1)?)?)))
        }),
        rule("hh:mm:ss", vec![regex(r"((?:[01]?\d)|(?:2[0-3]))[:.]([0-5]\d)[:.]([0-5]\d)")], |ts| {
            tt(hour_minute_second(
                true,
                parse_int(&rx(ts[0], 0)?)?,
                parse_int(&rx(ts[0], 1)?)?,
                parse_int(&rx(ts[0], 2)?)?,
            ))
        }),
        rule("<time-of-day> am|pm", vec![pred(is_a_time_of_day), regex(r"([ap])(\s|\.)?m?\.?")], |ts| {
            // Upstream uses the second capture (`_:ap:_`), not `[ap]`.
            tt(time_of_day_ampm(to_lower(&rx(ts[1], 1)?) == "a", td(ts[0])?))
        }),
        rule(
            "<hour-of-day> <integer>",
            vec![pred(not_latent_hod), pred(is_integer_between_tok(1, 59))],
            |ts| {
                let (hours, is12h) = tod(ts[0])?;
                tt(hour_minute(is12h, hours, get_int_value(ts[1])?))
            },
        ),
        rule("<hour-of-day> half", vec![pred(is_an_hour_of_day), regex("(و ?)?نصف?( ساع[ةه])?")], |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 30))
        }),
        rule("<hour-of-day> third", vec![pred(is_an_hour_of_day), regex("(و ?)?ثلث( ساع[ةه])?")], |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 20))
        }),
        rule("<hour-of-day> quarter", vec![pred(is_an_hour_of_day), regex("(و ?)?ربع( ساع[ةه])?")], |ts| {
            let (hours, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, hours, 15))
        }),
        rule("<hour-of-day> till third", vec![pred(is_an_hour_of_day), regex("[إا]لا ثلثا?")], |ts| {
            ttd(minutes_before(20, td(ts[0])?))
        }),
        rule("<hour-of-day> till quarter", vec![pred(is_an_hour_of_day), regex("[إا]لا ربعا?")], |ts| {
            ttd(minutes_before(15, td(ts[0])?))
        }),
        // Upstream production reads `getIntValue` on the "و" regex token; never fires.
        rule(
            "<hour-of-day>",
            vec![
                pred(is_an_hour_of_day),
                regex("و"),
                pred(is_integer_between_tok(1, 59)),
                regex("(دقيق[ةه]|دقا[ئي]ق)"),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                ttd(minutes_after(n, td(ts[0])?))
            },
        ),
        rule(
            "integer after <hour-of-day>",
            vec![pred(is_integer_between_tok(1, 59)), regex("(دقيق[ةه]|دقا[ئي]ق) بعد"), pred(is_an_hour_of_day)],
            |ts| ttd(minutes_after(get_int_value(ts[0])?, td(ts[2])?)),
        ),
        rule("half after <hour-of-day>", vec![regex("نصف? ساع[ةه] بعد"), pred(is_an_hour_of_day)], |ts| {
            ttd(minutes_after(30, td(ts[1])?))
        }),
        rule("third after <hour-of-day>", vec![regex("ثلث ساع[ةه] بعد"), pred(is_an_hour_of_day)], |ts| {
            ttd(minutes_after(20, td(ts[1])?))
        }),
        rule("quarter after <hour-of-day>", vec![regex("ربع ساع[ةه] بعد"), pred(is_an_hour_of_day)], |ts| {
            ttd(minutes_after(15, td(ts[1])?))
        }),
        rule("dd/mm", vec![regex(r"(3[01]|[12]\d|0?[1-9])\s?[/-]\s?(1[0-2]|0?[1-9])")], |ts| {
            tt(month_day(parse_int(&rx(ts[0], 1)?)?, parse_int(&rx(ts[0], 0)?)?))
        }),
        rule("dd/mm/yyyy", vec![regex(r"(3[01]|[12]\d|0?[1-9])[/-](1[0-2]|0?[1-9])[-/](\d{2,4})")], |ts| {
            tt(year_month_day(
                parse_int(&rx(ts[0], 2)?)?,
                parse_int(&rx(ts[0], 1)?)?,
                parse_int(&rx(ts[0], 0)?)?,
            ))
        }),
        rule("yyyy-mm-dd", vec![regex(r"(\d{2,4})-(0?[1-9]|1[0-2])-(3[01]|[12]\d|0?[1-9])")], |ts| {
            tt(year_month_day(
                parse_int(&rx(ts[0], 0)?)?,
                parse_int(&rx(ts[0], 1)?)?,
                parse_int(&rx(ts[0], 2)?)?,
            ))
        }),
        rule("mm/yyyy", vec![regex(r"(0?[1-9]|1[0-2])[/-](\d{4})")], |ts| {
            tt(year_month_day(parse_int(&rx(ts[0], 1)?)?, parse_int(&rx(ts[0], 0)?)?, 1))
        }),
        rule("noon|midnight|EOD|end of day", vec![regex("(ليلا|مساء|نهاية (ال)يوم)|منتصف الليل")], |ts| {
            let match_ = rx(ts[0], 0)?;
            tt(hour(false, if match_ == "منتصف الليل" { 12 } else { 0 }))
        }),
        rule("part of days", vec![regex("(ال)?(صبا?ح|فجر|ظهر|ظهيرة|فطور|إفطار|عصر|مغرب|مساء|عشاء|ليل[ةه]|غداء)ا?")], |ts| {
            let (start, end) = *PAR_OF_DAYS.get(to_lower(&rx(ts[0], 1)?).as_str())?;
            ttd(interval(Open, &hour(false, start), &hour(false, end)).map(|x| part_of_day(mk_latent(x))))
        }),
        rule("after part of days", vec![regex("بعد ال(فجر|ظهر|ظهيرة|فطور|إفطار|عصر|مغرب|عشاء|ليل|غداء)")], |ts| {
            let (start, end) = *PAR_OF_DAYS.get(to_lower(&rx(ts[0], 0)?).as_str())?;
            ttd(interval(Open, &hour(false, start), &hour(false, end + 1)).map(|x| part_of_day(not_latent(x))))
        }),
        rule("before part of days", vec![regex("قبل ال(فجر|ظهر|ظهيرة|فطور|إفطار|عصر|مغرب|عشاء|ليل|غداء)")], |ts| {
            let (start, end) = *PAR_OF_DAYS.get(to_lower(&rx(ts[0], 0)?).as_str())?;
            ttd(interval(Open, &hour(false, start - 1), &hour(false, end)).map(|x| part_of_day(not_latent(x))))
        }),
        rule("early morning", vec![regex("(ال)?صبا?ح (((ال)?باكر)|بدري)")], |_| {
            ttd(interval(Open, &hour(false, 4), &hour(false, 9)).map(|x| part_of_day(mk_latent(x))))
        }),
        rule(
            "in|during the <part-of-day>",
            vec![regex("(في|خلال|في خلال|عند)(وقت )?( ال)?|وقت ال"), pred(is_a_part_of_day)],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        rule("this <part-of-day>", vec![regex("هذ[اه](ال)?"), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(&today(), td(ts[1])?).map(|x| part_of_day(not_latent(x))))
        }),
        rule("tonight", vec![regex("(هذه )?الليل[ةه]")], |_| {
            let evening = interval(Open, &hour(false, 18), &hour(false, 0))?;
            ttd(intersect(&today(), &evening).map(|x| part_of_day(not_latent(x))))
        }),
        rule(
            "after lunch/work/school",
            vec![regex("بعد[\\s-]?((ال)?غداء|(ال)?عمل|(ال)?شغل|(ال)?مدرس[ةه])")],
            |ts| {
                // Upstream matches the capture against the regex source text; never succeeds.
                let match_ = to_lower(&rx(ts[0], 0)?);
                let (start, end) = match match_.as_str() {
                    "(ال)?غداء" => (hour(false, 13), hour(false, 17)),
                    "(ال)?عمل|(ال)?شغل" => (hour(false, 17), hour(false, 21)),
                    "(ال)?مدرس[ةه]" => (hour(false, 15), hour(false, 21)),
                    _ => return None,
                };
                let pod = interval(Open, &start, &end)?;
                ttd(intersect(&today(), &pod).map(|x| part_of_day(not_latent(x))))
            },
        ),
        rule("<time> <part-of-day>", vec![dimension(Dim::Time), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(td(ts[1])?, td(ts[0])?))
        }),
        rule("week-end", vec![regex("(عطل[ةه] )?نهاي[ةه] ال[اأ]سبوع")], |_| tt(weekend())),
        rule("<time-of-day> sharp|exactly", vec![pred(is_a_time_of_day), regex("(تقريبا)")], |ts| {
            tt(not_latent(td(ts[0])?.clone()))
        }),
        rule(
            "about|exactly <time-of-day>",
            vec![regex("(حول|حوالي?|تقريبا|قراب[ةه]|(ب|في )حدود)"), pred(is_grain_finer_than(G::Year))],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        rule(
            "from <month> dd-dd (interval)",
            vec![
                regex("من"),
                pred(is_a_month),
                pred(is_dom_value),
                regex("\\-|[اإ]لى|لـ?|حتى"),
                pred(is_dom_value),
            ],
            |ts| {
                let month = td(ts[1])?;
                let a = intersect_dom(month, ts[2])?;
                let b = intersect_dom(month, ts[4])?;
                ttd(interval(Closed, &a, &b))
            },
        ),
        rule(
            "from <day-of-month> (ordinal or number) to <day-of-month> (ordinal or number) <named-month> (interval)",
            vec![
                regex("من|بين"),
                pred(is_dom_value),
                regex("(و ?)?(\\-|[اإ]لى|لـ?|حتى)"),
                pred(is_dom_value),
                pred(is_a_month),
            ],
            |ts| {
                let month = td(ts[4])?;
                let a = intersect_dom(month, ts[1])?;
                let b = intersect_dom(month, ts[3])?;
                ttd(interval(Closed, &a, &b))
            },
        ),
        rule(
            "from <day-of-month> (ordinal or number) to <day-of-month> (ordinal or number) in <named-month> (interval)",
            vec![
                regex("من|بين"),
                pred(is_dom_value),
                regex("(و ?)?(\\-|[اإ]لى|لـ?|حتى)"),
                pred(is_dom_value),
                regex("من|في"),
                pred(is_a_month),
            ],
            |ts| {
                let month = td(ts[5])?;
                let a = intersect_dom(month, ts[1])?;
                let b = intersect_dom(month, ts[3])?;
                ttd(interval(Closed, &a, &b))
            },
        ),
        rule(
            "<month> dd-dd (interval)",
            vec![pred(is_a_month), pred(is_dom_value), regex("\\-|[اإ]لى|لـ?|حتى"), pred(is_dom_value)],
            |ts| {
                let month = td(ts[0])?;
                let a = intersect_dom(month, ts[1])?;
                let b = intersect_dom(month, ts[3])?;
                ttd(interval(Closed, &a, &b))
            },
        ),
        rule(
            "dd-dd <month> (interval)",
            vec![pred(is_dom_value), regex("\\-|[اإ]لى|لـ?|حتى"), pred(is_dom_value), pred(is_a_month)],
            |ts| {
                let month = td(ts[3])?;
                let a = intersect_dom(month, ts[0])?;
                let b = intersect_dom(month, ts[2])?;
                ttd(interval(Closed, &a, &b))
            },
        ),
        rule(
            "<datetime> - <datetime> (interval)",
            vec![pred(is_not_latent), regex("(و ?)?(\\-|[اإ]لى|حتى)"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "from <datetime> - <datetime> (interval)",
            vec![regex("من"), pred(is_not_latent), regex("(و ?)?(\\-|[اإ]لى|لـ?|حتى)"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "between <time> and <time>",
            vec![
                regex("بين ?((ال)?ساع[ةه])?"),
                dimension(Dim::Time),
                regex("و ?(بين)? ?((ال)?ساع[ةه])?"),
                dimension(Dim::Time),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "<time-of-day> - <time-of-day> (interval)",
            vec![pred(not_latent_tod), regex("(و ?)?(\\-|:|[اإ]لى|لـ?|حتى)"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "from <time-of-day> - <time-of-day> (interval)",
            vec![
                regex("(من|بين|)((ال)?ساع[ةه]?)"),
                pred(is_a_time_of_day),
                regex("(و ?)?(\\-|[اإ]لى|لـ?|حتى)((ال)?ساع[ةه]?)"),
                pred(is_a_time_of_day),
            ],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        rule(
            "between <time-of-day> and <time-of-day> (interval)",
            vec![regex("بين"), pred(is_a_time_of_day), regex("و"), pred(is_a_time_of_day)],
            |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
        ),
        rule("by <time>", vec![regex("حتى"), dimension(Dim::Time)], |ts| {
            ttd(interval(Open, &now(), td(ts[1])?))
        }),
        rule("until <time-of-day>", vec![regex("([اأ]ي وقت)?(قبل|حتى|[إا]لى)"), dimension(Dim::Time)], |ts| {
            tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone()))
        }),
        rule("after <time-of-day>", vec![regex("([اأ]ي وقت)?(بعد)"), dimension(Dim::Time)], |ts| {
            tt(with_direction(IntervalDirection::After, td(ts[1])?.clone()))
        }),
        rule("since <time-of-day>", vec![regex("منذ"), dimension(Dim::Time)], |ts| {
            tt(with_direction(IntervalDirection::After, td(ts[1])?.clone()))
        }),
        rule("this|last <cycle>", vec![regex("(هذا|هذه|اخر|آخر)"), dimension(Dim::TimeGrain)], |ts| {
            let g = grain_of(ts[1])?;
            match to_lower(&rx(ts[0], 0)?).as_str() {
                "هذا" | "هذه" => tt(cycle_nth(g, 0)),
                "اخر" | "آخر" => tt(cycle_nth(g, -1)),
                _ => None,
            }
        }),
        rule(
            "this|last the <cycle>",
            vec![regex("(هذا|هذه|اخر|آخر)"), regex("(ال)"), dimension(Dim::TimeGrain)],
            |ts| {
                let g = grain_of(ts[2])?;
                match to_lower(&rx(ts[0], 0)?).as_str() {
                    "هذا" | "هذه" => tt(cycle_nth(g, 0)),
                    "اخر" | "آخر" => tt(cycle_nth(g, -1)),
                    _ => None,
                }
            },
        ),
        rule(
            "<cycle> this|last|next",
            vec![
                regex("ال"),
                dimension(Dim::TimeGrain),
                regex("(هذ|القادم|التالي|الحالي|المقبل|الجاي|السابق|الماضي?|الفا[ئي]ت|المنصرم)[اةه]?(ين|ان|ات)?"),
            ],
            |ts| {
                let g = grain_of(ts[1])?;
                match to_lower(&rx(ts[2], 0)?).as_str() {
                    "هذ" | "الحالي" => tt(cycle_nth(g, 0)),
                    "القادم" | "التالي" | "المقبل" | "الجاي" => tt(cycle_nth(g, 1)),
                    "السابق" | "الماضي" | "الماض" | "المنصرم" | "الفائت" | "الفايت" => {
                        tt(cycle_nth(g, -1))
                    }
                    _ => None,
                }
            },
        ),
        rule(
            "the <cycle> after|before <time>",
            vec![regex("ال"), dimension(Dim::TimeGrain), regex("(الذ?ي |ال)(قبل|بعد)"), dimension(Dim::Time)],
            |ts| {
                let g = grain_of(ts[1])?;
                let base = td(ts[3])?;
                match to_lower(&rx(ts[2], 1)?).as_str() {
                    "بعد" => tt(cycle_nth_after(false, g, 1, base)),
                    _ => tt(cycle_nth_after(false, g, -1, base)),
                }
            },
        ),
        rule(
            "<cycle> after|before <time>",
            vec![dimension(Dim::TimeGrain), regex("(قبل|بعد)"), dimension(Dim::Time)],
            |ts| {
                let g = grain_of(ts[0])?;
                let base = td(ts[2])?;
                match to_lower(&rx(ts[1], 0)?).as_str() {
                    "بعد" => tt(cycle_nth_after(false, g, 1, base)),
                    _ => tt(cycle_nth_after(false, g, -1, base)),
                }
            },
        ),
        rule(
            "last n <cycle>",
            vec![regex("([اآ]خر)"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -n))
            },
        ),
        rule(
            "<cycle> n next",
            vec![
                regex("ال"),
                dimension(Dim::TimeGrain),
                regex("ال"),
                pred(is_integer_between_tok(1, 9999)),
                regex("(ال)(تالي|قادم)[ةه]?"),
            ],
            |ts| {
                let n = get_int_value(ts[3])?;
                tt(cycle_n(true, grain_of(ts[1])?, n))
            },
        ),
        rule(
            "the <cycle> the <ordinal> of <time>",
            vec![
                regex("ال"),
                dimension(Dim::TimeGrain),
                dimension(Dim::Ordinal),
                regex("في|من|بـ?"),
                dimension(Dim::Time),
            ],
            |ts| {
                let n = get_int_value(ts[2])?;
                tt(cycle_nth_after(true, grain_of(ts[1])?, n - 1, td(ts[4])?))
            },
        ),
        rule(
            "<ordinal> <cycle> of <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("في|من|بـ?"), dimension(Dim::Time)],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(cycle_nth_after(true, grain_of(ts[1])?, n - 1, td(ts[3])?))
            },
        ),
        rule(
            "<cycle> <ordinal> day? of <time>",
            vec![
                dimension(Dim::TimeGrain),
                dimension(Dim::Ordinal),
                regex("((ال)?يوم )?(بـ?|في|من)"),
                dimension(Dim::Time),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_nth_after(true, grain_of(ts[0])?, n - 1, td(ts[3])?))
            },
        ),
        rule(
            "<ordinal> <cycle> after <time>",
            vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("بعد"), dimension(Dim::Time)],
            |ts| {
                let n = get_int_value(ts[0])?;
                tt(cycle_nth_after(true, grain_of(ts[1])?, n - 1, td(ts[3])?))
            },
        ),
        rule("in|within|after <duration>", vec![regex("(في|خلال|بعد)"), dimension(Dim::Duration)], |ts| {
            let dd = dur(ts[1])?;
            match to_lower(&rx(ts[0], 0)?).as_str() {
                "خلال" => ttd(interval(Open, &now(), &in_duration(dd))),
                "بعد" => tt(with_direction(IntervalDirection::After, in_duration(dd))),
                "في" => tt(in_duration(dd)),
                _ => None,
            }
        }),
        rule("<duration> hence|ago", vec![dimension(Dim::Duration), regex("(مض[تى]|من الان)")], |ts| {
            let dd = dur(ts[0])?;
            match to_lower(&rx(ts[1], 0)?).as_str() {
                "من الان" => tt(in_duration(dd)),
                _ => tt(duration_ago(dd)),
            }
        }),
        rule("before <duration>", vec![regex("من|قبل"), dimension(Dim::Duration)], |ts| {
            tt(duration_ago(dur(ts[1])?))
        }),
        rule(
            "<duration> after|before|from <time>",
            vec![dimension(Dim::Duration), regex("(من|قبل|بعد)"), dimension(Dim::Time)],
            |ts| {
                let dd = dur(ts[0])?;
                let base = td(ts[2])?;
                match to_lower(&rx(ts[1], 0)?).as_str() {
                    "قبل" => tt(duration_before(dd, base)),
                    _ => tt(duration_after(dd, base)),
                }
            },
        ),
        rule(
            "for <duration> from <time>",
            vec![
                regex("لمدة|ل"),
                dimension(Dim::Duration),
                regex("(من بداي[ةه]|ابتداء ب|بداية من|من)"),
                dimension(Dim::Time),
            ],
            |ts| {
                let td1 = td(ts[3])?;
                ttd(interval(Open, td1, &duration_after(dur(ts[1])?, td1)))
            },
        ),
        rule("<time> for <duration>", vec![dimension(Dim::Time), regex("لمدة|ل"), dimension(Dim::Duration)], |ts| {
            let td1 = td(ts[0])?;
            ttd(interval(Open, td1, &duration_after(dur(ts[2])?, td1)))
        }),
        rule("in <number> (implicit minutes)", vec![regex("في|خلال"), pred(is_integer_between_tok(0, 60))], |ts| {
            match ts[1] {
                Token::Numeral(n) => {
                    tt(in_duration(&crate::dims::duration::duration(G::Minute, n.value.floor() as i64)))
                }
                _ => None,
            }
        }),
        rule(
            "<time> timezone",
            vec![
                pred(not_latent_tod),
                regex("(بتوقيت |)?\\b(YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT)\\b"),
            ],
            |ts| ttd(in_timezone(&to_upper(&rx(ts[1], 1)?), td(ts[0])?)),
        ),
        rule("part of <named-month>", vec![regex("([اأ]وائل|[اأ]واخر|نهاي[ةه])-?"), pred(is_a_month)], |ts| {
            let month = td(ts[1])?;
            let (sd, end) = match to_lower(&rx(ts[0], 0)?).as_str() {
                "اوائل" | "أوائل" => (1, intersect(&day_of_month(10), month)?),
                "اواخر" | "أواخر" => (21, cycle_last_of(G::Day, month)),
                _ => return None,
            };
            let start = intersect(month, &day_of_month(sd))?;
            ttd(interval(Open, &start, &end))
        }),
        rule("day of <named-month>", vec![regex("(بداي[ةه]|منتصف|نصف?|[اآ]خر|نهاي[ةه])-?"), pred(is_a_month)], |ts| {
            let month = td(ts[1])?;
            match to_lower(&rx(ts[0], 0)?).as_str() {
                "بداية" | "بدايه" => ttd(intersect(&day_of_month(1), month)),
                "منتصف" | "نصف" | "نص" => ttd(intersect(&day_of_month(15), month)),
                _ => tt(cycle_last_of(G::Day, month)),
            }
        }),
        rule("now", vec![regex("ال[اآ]ن|حالا")], |_| tt(now())),
    ];

    v.extend(mk_rule_instants(&[
        ("right now", G::Second, 0, "الان|حالا|(في )?هذه اللحظ[ةه]"),
        ("today", G::Day, 0, "اليوم"),
        ("tomorrow", G::Day, 1, "((يوم )?(غدا?|الغد))|(بكر[اةه])"),
        ("yesterday", G::Day, -1, "[أا]مس|(ال|ام)بارح[ةه]?"),
        ("end of month", G::Month, 1, "(نهاي[ةه] الشهر)"),
        ("end of year", G::Year, 1, "(نهاي[ةه] (السن[ةه]|العام))"),
    ]));
    v.extend(mk_rule_days_of_week(&[
        ("Monday", "(ال)?[اإ]ثنين"),
        ("Tuesday", "(ال)?ثلاثاء?"),
        ("Wednesday", "(ال)?[اأ]ربعاء?"),
        ("Thursday", "(ال)?خميس"),
        ("Friday", "(ال)?جمع[ةه]"),
        ("Saturday", "(ال)?سبت"),
        ("Sunday", "(ال)?[اأ]حد"),
    ]));
    v.extend(mk_rule_months(&[
        ("January", "يناير|كانون (ال)ثاني?"),
        ("February", "فبراير|شباط"),
        ("March", "مارس|[اآ]ذار"),
        ("April", "[اأ]بريل|نيسان"),
        ("May", "مايو|[اأ]ي[اآ]ر"),
        ("June", "يونيو|حزيران"),
        ("July", "يوليو|تموز"),
        ("August", "[اأ]غسطس|[اآ]ب"),
        ("September", "سي?بتمبر|[اأ]يلول"),
        ("October", "[اأ]كتوبر|تشرين (ال)?[اأ]ول"),
        ("November", "نوفمبر|تشرين (ال)?ثاني"),
        ("December", "ديسمبر|كانون (ال)?[اأ]ول"),
    ]));
    v.extend(mk_rule_seasons(&[
        ("summer", "صيف|الصيف", month_day(6, 21), month_day(9, 23)),
        ("fall", "الخريف|خريف", month_day(9, 23), month_day(12, 21)),
        ("winter", "شتاء|الشتاء", month_day(12, 21), month_day(3, 20)),
        ("spring", "ربيع|الربيع", month_day(3, 20), month_day(6, 21)),
    ]));
    v.extend(mk_rule_holidays(&[
        ("Christmas", "(يوم |عطل[ةه] )?((ال)?كري?سماس)", month_day(12, 25)),
        ("Christmas Eve", "(ليل[ةه] )((ال)?كري?سماس)", month_day(12, 24)),
        ("New Year's Eve", "(ليل[ةه] )(ر[اأ]س السن[ةه])", month_day(12, 31)),
        ("New Year's Day", "(وم |عطل[ةه] )?(ر[اأ]س السن[ةه])", month_day(1, 1)),
        ("Valentine's Day", "(عيد|يوم|عطل[ةه])((ال)?حب|(ال)?فالنتا?ين)", month_day(2, 12)),
        ("Halloween", "(عيد |يوم |عطل[ةه] )?((ال)?هالوي?ين)", month_day(10, 31)),
    ]));
    v.extend(mk_rule_holidays(&[("عيد الميلاد", "عيد الميلاد", month_day(12, 25))]));
    v.extend(mk_rule_holidays(&[
        ("عيد الأضحى", "عيد الأضحى", eidal_adha()),
        ("عيد الفطر", "عيد الفطر", eidal_fitr()),
        ("عيد الفصح", "عيد الفصح", easter_sunday()),
        ("رأس السنة الهجرية", "رأس السنة الهجرية", muharram()),
    ]));
    v.extend(mk_rule_holidays_maybe(&[(
        "رمضان",
        "رمضان",
        {
            let start = ramadan();
            let end = cycle_nth_after(false, G::Day, -1, &eidal_fitr());
            interval(Open, &start, &end)
        },
    )]));
    v
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &RULES
}
