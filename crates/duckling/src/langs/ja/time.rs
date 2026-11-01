//! `Duckling.Time.JA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::DurationData;
use crate::dims::numeral::is_natural;
use crate::dims::time::helpers::*;
use crate::dims::time::types::{IntervalDirection, TimeData, TimeIntervalType::Closed};
use crate::dims::time_grain::Grain as G;
use crate::dims::util::{parse_int, to_lower};
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

fn imperial_year(era: &str) -> Option<i64> {
    match to_lower(era).as_str() {
        "明治" => Some(1868),
        "大正" => Some(1912),
        "昭和" => Some(1926),
        "平成" => Some(1989),
        "令和" => Some(2019),
        _ => None,
    }
}

fn rule_hhmm() -> Rule {
    rule(
        "hh:mm",
        vec![regex(r"((?:[01]?\d)|(?:2[0-3]))[：:. ]([0-5]\d)")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(h != 0 && h < 12, h, m))
        },
    )
}

fn rule_hhmm_kanji() -> Rule {
    rule(
        "hh時mm分",
        vec![regex(r"((?:[01]?\d)|(?:2[0-3]))時([0-5]\d)分")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(h != 0 && h < 12, h, m))
        },
    )
}

fn rule_hhmm_kanji_numeral() -> Rule {
    rule(
        "hh時mm分",
        vec![pred(is_natural), regex("時"), pred(is_natural), regex("分")],
        |ts| {
            let h = get_int_value(ts[0])?;
            let m = get_int_value(ts[2])?;
            if h > 0 && h < 25 && m < 60 {
                tt(hour_minute(h != 0 && h < 12, h, m))
            } else {
                None
            }
        },
    )
}

fn rule_tod_latent() -> Rule {
    rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(mk_latent(hour(n < 13, n)))
    })
}

fn rule_hh_oclock() -> Rule {
    rule(
        "<time-of-day> o'clock",
        vec![pred(is_a_time_of_day), regex("時(ちょうど|きっかり|ぴったり)")],
        |ts| tt(not_latent(td(ts[0])?.clone())),
    )
}

fn rule_tod_ampm() -> Rule {
    rule(
        "<time-of-day> am|pm",
        vec![pred(is_a_time_of_day), regex(r"([ap])(\s|\.)?m?\.?")],
        |ts| {
            let ap = to_lower(&rx(ts[1], 0)?);
            tt(time_of_day_ampm(ap == "a", td(ts[0])?))
        },
    )
}

fn rule_tod_ampm_kanji() -> Rule {
    rule(
        "午後|午前 <time-of-day>",
        vec![regex("午(前|後)"), pred(is_a_time_of_day)],
        |ts| {
            let ap = rx(ts[0], 0)?;
            tt(time_of_day_ampm(ap == "前", td(ts[1])?))
        },
    )
}

fn rule_at_tod() -> Rule {
    rule(
        "at <time-of-day>",
        vec![pred(is_a_time_of_day), regex("に|で|の")],
        |ts| tt(not_latent(td(ts[0])?.clone())),
    )
}

fn rule_mmdd() -> Rule {
    rule(
        "mm/dd",
        vec![regex(r"(1[0-2]|0?[1-9])\s?[/／]\s?(3[01]|[12]\d|0?[1-9])")],
        |ts| {
            let m = parse_int(&rx(ts[0], 0)?)?;
            let d = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        },
    )
}

fn rule_yyyymm() -> Rule {
    rule(
        "yyyy/mm",
        vec![regex(r"(\d{4})\s*[/／.]\s*(1[0-2]|0?[1-9])")],
        |ts| {
            let y = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(year_month(y, m))
        },
    )
}

fn rule_yyyymmdd() -> Rule {
    rule(
        "yyyy/mm/dd",
        vec![regex(r"(\d{2,4})\s*[/／.]\s*(0?[1-9]|1[0-2])\s*[/／.]\s*(3[01]|[12]\d|0?[1-9])")],
        |ts| {
            let y = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let d = parse_int(&rx(ts[0], 2)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn rule_month_dom() -> Rule {
    rule(
        "<named-month> <day-of-month>",
        vec![pred(is_a_month), pred(is_dom_value), regex("日")],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn rule_year_month_dom() -> Rule {
    rule(
        "<year> <named-month> <day-of-month>",
        vec![regex(r"(\d{2,4})年"), pred(is_a_month), pred(is_dom_value), regex("日")],
        |ts| {
            let int_val = parse_int(&rx(ts[0], 0)?)?;
            let dom = intersect_dom(td(ts[1])?, ts[2])?;
            ttd(intersect(&dom, &year(int_val)))
        },
    )
}

fn rule_imperial_year_month_dom() -> Rule {
    rule(
        "imperial <year> <named-month> <day-of-month>",
        vec![pred(is_grain_of_time(G::Year)), pred(is_a_month), pred(is_dom_value), regex("日")],
        |ts| {
            let year_td = td(ts[0])?;
            let dom = intersect_dom(td(ts[1])?, ts[2])?;
            ttd(intersect(&dom, year_td))
        },
    )
}

fn rule_year_month() -> Rule {
    rule(
        "<year> <named-month>",
        vec![pred(is_grain_of_time(G::Year)), pred(is_a_month)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[1])?).map(not_latent)),
    )
}

fn rule_on_adow() -> Rule {
    rule(
        "on <named-day>",
        vec![regex(r"(\(|（)?"), pred(is_a_day_of_week), regex(r"(\)|）)?(に|は|で)")],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    )
}

fn rule_absorb_on_day() -> Rule {
    rule(
        "on <day>",
        vec![pred(is_grain_of_time(G::Day)), regex("に|は|で")],
        |ts| tt(not_latent(td(ts[0])?.clone())),
    )
}

fn rule_on_day() -> Rule {
    rule(
        "on the <day-of-month>",
        vec![pred(is_dom_value), regex("日(に|は|で)")],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(day_of_month(n))
        },
    )
}

fn rule_in_named_month() -> Rule {
    rule(
        "in <named-month>",
        vec![pred(is_a_month), regex("(の間|中)?(に|は|で)")],
        |ts| Some(ts[0].clone()),
    )
}

fn rule_year_latent() -> Rule {
    rule(
        "year (latent)",
        vec![pred(is_integer_between_tok(25, 10000)), regex("年")],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(mk_latent(year(n)))
        },
    )
}

fn rule_imperial_year_latent() -> Rule {
    rule(
        "Imperial year (latent)",
        vec![regex("(明治|大正|昭和|平成|令和)"), pred(is_integer_between_tok(0, 65)), regex("年")],
        |ts| {
            let n = get_int_value(ts[1])?;
            let ry = imperial_year(&rx(ts[0], 0)?)?;
            tt(mk_latent(year(ry + n - 1)))
        },
    )
}

fn rule_first_imperial_year_latent() -> Rule {
    rule(
        "First Imperial year (latent)",
        vec![regex("(明治|大正|昭和|平成|令和)元年")],
        |ts| {
            let ry = imperial_year(&rx(ts[0], 0)?)?;
            tt(mk_latent(year(ry)))
        },
    )
}

fn rule_in_year() -> Rule {
    rule(
        "in <year>",
        vec![pred(is_grain_of_time(G::Year)), regex("に|は|で")],
        |ts| tt(not_latent(td(ts[0])?.clone())),
    )
}

fn rule_in_month() -> Rule {
    rule(
        "in <month>",
        vec![pred(is_grain_of_time(G::Month)), regex("に")],
        |ts| tt(not_latent(td(ts[0])?.clone())),
    )
}

fn rule_in_duration() -> Rule {
    rule(
        "in <duration>",
        vec![
            dimension(Dim::Duration),
            regex("後|(間)?で|経ったら|経ってから|経過後に|経過してから|経過したら|過ぎに|過ぎたら"),
        ],
        |ts| tt(in_duration(dur(ts[0])?)),
    )
}

fn rule_in_the_last_duration() -> Rule {
    rule(
        "in/for the last <duration>",
        vec![regex("この"), pred(is_duration_greater_than(G::Hour))],
        |ts| ttd(interval(Closed, &duration_before(dur(ts[1])?, &now()), &now())),
    )
}

fn rule_interval_from_time() -> Rule {
    rule(
        "from <time>",
        vec![dimension(Dim::Time), regex("(初め)?以(降|来)に?|～|よりも?後|から")],
        |ts| tt(with_direction(IntervalDirection::After, not_latent(td(ts[0])?.clone()))),
    )
}

fn rule_interval_until_time() -> Rule {
    rule(
        "until <time>",
        vec![dimension(Dim::Time), regex("までに?|よりも?前|以前")],
        |ts| tt(with_direction(IntervalDirection::Before, not_latent(td(ts[0])?.clone()))),
    )
}

fn rule_interval_from_to_time() -> Rule {
    rule(
        "from <time> to <time>",
        vec![
            dimension(Dim::Time),
            regex("(初め)?以(降|来)に?|～|よりも?後?|(初め|頭)?から|以降"),
            dimension(Dim::Time),
            regex("末?まで|(より|以)前|の間|にかけて|いっぱい"),
        ],
        |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_interval_times() -> Rule {
    rule(
        "<time> - <time>",
        vec![dimension(Dim::Time), regex("-|~|～|・|、|から"), dimension(Dim::Time)],
        |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
    )
}

fn rule_interval_year() -> Rule {
    rule(
        "<year> - <year>",
        vec![
            pred(is_integer_between_tok(1000, 10000)),
            regex("-|~|～|・|、|から"),
            pred(is_integer_between_tok(1000, 10000)),
            regex("年"),
        ],
        |ts| {
            let y1 = get_int_value(ts[0])?;
            let y2 = get_int_value(ts[2])?;
            if y1 < y2 {
                ttd(interval(Closed, &year(y1), &year(y2)))
            } else {
                None
            }
        },
    )
}

fn rule_weekend() -> Rule {
    rule("week-end", vec![regex("週末|しゅうまつ")], |_| tt(mk_ok_for_this_next(weekend())))
}

fn rule_now() -> Rule {
    rule("now", vec![regex("今すぐ|いま|即|ただ(いま|ちに)")], |_| tt(now()))
}

fn rule_this_time_grain() -> Rule {
    rule(
        "this|current <time-grain>",
        vec![regex("現在の|(今|こ(ん|の)|当|現)(1|一)?"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
    )
}

fn rule_next_time_grain() -> Rule {
    rule("next <time-grain>", vec![regex("来"), dimension(Dim::TimeGrain)], |ts| {
        tt(cycle_nth(grain_of(ts[1])?, 1))
    })
}

fn rule_last_time_grain() -> Rule {
    rule(
        "last <time-grain>",
        vec![regex("(前|直近|最後|昨|先)の?(1|一)?"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
    )
}

fn rule_duration_last_next() -> Rule {
    rule(
        "last|past|next <duration>",
        vec![regex("(過去|直在|次の)"), dimension(Dim::Duration)],
        |ts| {
            let match_s = to_lower(&rx(ts[0], 0)?);
            let dd = dur(ts[1])?;
            match match_s.as_str() {
                "次の" => tt(cycle_n(true, dd.grain, dd.value)),
                "過去" => tt(cycle_n(true, dd.grain, -dd.value)),
                "直在" => tt(cycle_n(true, dd.grain, -dd.value)),
                _ => None,
            }
        },
    )
}

fn rules_instants() -> Vec<Rule> {
    mk_rule_instants(&[
        ("today", G::Day, 0, "今日(1日)?|きょう|本日|ほんじつ"),
        ("tomorrow", G::Day, 1, "明日|あした|あす|みょうにち"),
        ("yesterday", G::Day, -1, "(前|昨)日|きのう|さくじつ"),
        ("next week", G::Week, 1, "らいしゅう"),
        ("last week", G::Week, -1, "せんしゅう"),
        ("a week after next", G::Week, 2, "再来週|さらいしゅう"),
        ("this month", G::Month, 0, "このひと月"),
        ("this quarter", G::Quarter, 0, "本四半期|今の四半期|現行四半期"),
        ("this year", G::Year, 0, "本年"),
        ("last year", G::Year, -1, "去年"),
    ])
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Monday", "月曜日?|げつようび"),
        ("Tuesday", "火曜日?|かようび"),
        ("Wednesday", "水曜日?|すいようび"),
        ("Thursday", "木曜日?|もくようび"),
        ("Friday", "金曜日?|きんようび"),
        ("Saturday", "土曜日?|どようび"),
        ("Sunday", "日曜日?|にちようび"),
    ])
}

fn rules_days_of_week_latent() -> Vec<Rule> {
    mk_rule_days_of_week_latent(&[
        ("Monday", "月|げつよう"),
        ("Tuesday", "火|かよう"),
        ("Wednesday", "水|すいよう"),
        ("Thursday", "木|もくよう"),
        ("Friday", "金|きんよう"),
        ("Saturday", "土|どよう"),
        ("Sunday", "日|にちよう"),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("January", "(1|一|１)(月|がつ)|いちがつ"),
        ("February", "(2|二|２)(月|がつ)|にがつ"),
        ("March", "(3|三|３)(月|がつ)|さんがつ"),
        ("April", "(4|四|４)(月|がつ)|しがつ"),
        ("May", "(5|五|５)(月|がつ)|ごがつ"),
        ("June", "(6|六|６)(月|がつ)|ろくがつ"),
        ("July", "(7|七|７)(月|がつ)|しちがつ"),
        ("August", "(8|八|８)(月|がつ)|はちがつ"),
        ("September", "(9|九|９)(月|がつ)|くがつ"),
        ("October", "(10|十|１０)(月|がつ)|じゅうがつ"),
        ("November", "(11|十一|１１)(月|がつ)|じゅういちがつ"),
        ("December", "(12|十二|１２)(月|がつ)|じゅうにがつ"),
    ])
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = vec![
        rule_hhmm(),
        rule_hhmm_kanji(),
        rule_hhmm_kanji_numeral(),
        rule_tod_latent(),
        rule_hh_oclock(),
        rule_tod_ampm(),
        rule_tod_ampm_kanji(),
        rule_at_tod(),
        rule_mmdd(),
        rule_yyyymm(),
        rule_yyyymmdd(),
        rule_year_latent(),
        rule_imperial_year_latent(),
        rule_first_imperial_year_latent(),
        rule_month_dom(),
        rule_year_month(),
        rule_year_month_dom(),
        rule_imperial_year_month_dom(),
        rule_on_adow(),
        rule_absorb_on_day(),
        rule_on_day(),
        rule_in_named_month(),
        rule_in_year(),
        rule_in_month(),
        rule_in_duration(),
        rule_in_the_last_duration(),
        rule_interval_from_time(),
        rule_interval_until_time(),
        rule_interval_from_to_time(),
        rule_interval_times(),
        rule_interval_year(),
        rule_weekend(),
        rule_now(),
        rule_this_time_grain(),
        rule_next_time_grain(),
        rule_last_time_grain(),
        rule_duration_last_next(),
    ];
    v.extend(rules_days_of_week());
    v.extend(rules_days_of_week_latent());
    v.extend(rules_months());
    v.extend(rules_instants());
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
