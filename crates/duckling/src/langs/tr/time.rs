//! `Duckling.Time.TR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::DurationData;
use crate::dims::time::computed::{eidal_adha, eidal_fitr};
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

/// `Duckling.Duration.Helpers.isGrain`.
fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

fn not_latent_hour(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

fn not_latent_tod(t: &Token) -> bool {
    is_not_latent(t) && is_a_time_of_day(t)
}

/// `Token Time . mkLatent . partOfDay <$> interval Open (hour False a) (hour False b)`.
fn latent_pod(a: i64, b: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, a), &hour(false, b)).map(|x| mk_latent(part_of_day(x))))
}

fn rules_instants() -> Vec<Rule> {
    mk_rule_instants(&[
        ("now", G::Second, 0, "ş?imdi|şu(\\s)?an"),
        ("today", G::Day, 0, "bugün"),
        ("tomorrow", G::Day, 1, "yarın"),
        ("yesterday", G::Day, -1, "dün"),
        ("after tomorrow", G::Day, 2, "(yarından\\ssonraki)\\s?(gün)?"),
        ("before yesterday", G::Day, -2, "(dün\\sdeğil\\sevvelsi|dünden\\sönceki|öbürsü|öbürki)\\s(gün)?"),
        ("EOM|End of day", G::Day, 1, "gün\\s(sonu(na|ndan?)?|bitimi(ne|nden?)?)?"),
        ("EOM|End of month", G::Month, 1, "ay\\s(sonu(na|ndan?)?|bitimi(ne|nden?)?)?"),
        ("EOY|End of year", G::Year, 1, "yıl\\s(sonu(na|ndan?)?|bitimi(ne|nden?)?)?"),
    ])
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Pazartesi", "pazartesi'?(si|den|ye)?|pzts?"),
        ("Salı", "salı?'?(sı|dan|ya)?"),
        ("Çarşamba", "çar(şamba)?'?(sı|dan|ya)?"),
        ("Perşembe", "per(şembe)?'?(si|den|ye)?"),
        ("Cuma", "cuma?'?(sı|dan|ya)?"),
        ("Cumartesi", "cumartesi'?(si|den|ye)?|cmt"),
        ("Pazar", "paz(ar)?'?(ı|dan|a)?"),
    ])
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("Ocak", "ocak?'?(ğın|tan|ğ?a)?"),
        ("Şubat", "şub(at)?'?(a|ın|tan)?"),
        ("Mart", "mart?'?(ın|a|tan)?"),
        ("Nisan", "nis(an)?'?(ın|a|dan)?"),
        ("Mayıs", "may(ıs)?'?(ın|a|tan)?"),
        ("Haziran", "haz(iran)?'?(ın|a|dan)?"),
        ("Temmuz", "tem(muz)?'?(un|a|dan)?"),
        ("Ağustos", "ağu(stos)?'?(un|a|tan)?"),
        ("Eylül", "eyl(ül)?'?(ün|e|den)?"),
        ("Ekim", "ekim?'?(in|den|e)?"),
        ("Kasım", "kas(ım)?'?(ın|dan|a)?"),
        ("Aralık", "ara(lık?)?'?(ğın|ğa|tan)?"),
    ])
}

fn rules_seasons() -> Vec<Rule> {
    mk_rule_seasons(&[
        ("yaz", "yaz(ın|a|dan)?", month_day(6, 21), month_day(9, 23)),
        ("sonbahar", "sonbahar(ın|a|dan)?", month_day(9, 23), month_day(12, 21)),
        ("kış", "kış(ın|a|tan)?", month_day(12, 21), month_day(3, 20)),
        ("ilkbahar", "ilkbahar(ın|a|dan)?", month_day(3, 20), month_day(6, 21)),
    ])
}

fn rules_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("Yılbaşı", "yılbaşı(ndan|na)?|yılbaşı(\\statili(nden|ne))?", month_day(1, 1)),
        ("Ulusal Egemenlik ve Çocuk Bayramı", "(ulusal\\segemenlik\\sve\\s)?çocuk\\sbayramı", month_day(4, 23)),
        ("Emek ve Dayanışma Günü", "emek\\sve\\sdayanışma\\sgünü", month_day(5, 1)),
        ("Atatürk’ü Anma, Gençlik ve Spor Bayramı", "(gençlik\\sve\\sspor|spor|gençlik)\\sbayramı", month_day(5, 19)),
        ("Zafer Bayramı", "zafer\\sbayramı", month_day(8, 30)),
        ("Cumhuriyet Bayramı", "cumhuriyet\\sbayramı", month_day(10, 29)),
    ])
}

fn rules_computed_holidays() -> Vec<Rule> {
    mk_rule_holidays(&[
        ("Kurban Bayramı", "kurban\\sbayramı", eidal_adha()),
        ("Ramazan Bayramı", "ramazan\\sbayramı", eidal_fitr()),
    ])
}

fn rule_this_time() -> Rule {
    rule("this <time>", vec![regex("bu"), pred(is_ok_with_this_next)], |ts| {
        tt(pred_nth(0, false, td(ts[1])?))
    })
}

fn rules_main() -> Vec<Rule> {
    vec![
        rule("absorption of , after named day", vec![pred(is_a_day_of_week), regex(",")], |ts| {
            Some(ts[0].clone())
        }),
        rule("after <time-of-day>", vec![regex("sonraki|önümüzdeki"), pred(is_a_time_of_day)], |ts| {
            tt(with_direction(IntervalDirection::After, td(ts[1])?.clone()))
        }),
        rule("afternoon", vec![regex("öğleden sonra")], |_| latent_pod(12, 19)),
        rule("at <time-of-day>", vec![regex("saat"), pred(is_a_time_of_day)], |ts| {
            tt(not_latent(td(ts[1])?.clone()))
        }),
        rule(
            "at <time-of-day>",
            vec![regex("saat"), pred(is_a_time_of_day), regex("'?(den|dan)?")],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        // Haskell production skips the first Time token (`_:Token Time td1:...`).
        rule(
            "between <time-of-day> and <time-of-day> (interval)",
            vec![pred(is_a_time_of_day), regex("ile"), pred(is_a_time_of_day), regex("arası")],
            |ts| ttd(interval(Closed, td(ts.get(1)?)?, td(ts.get(3)?)?)),
        ),
        rule(
            "<datetime> - <datetime> (interval)",
            vec![pred(is_not_latent), regex("\\-"), pred(is_not_latent)],
            |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "dd.(mm.)? - dd.mm.(yy[yy]?)? (interval)",
            vec![
                // Cyrillic 'с' is intentional (upstream).
                regex("(?:с\\s+)?(10|20|30|31|[012]?[1-9])\\.?((?<=\\.)(?:10|11|12|0?[1-9])(?:\\.?))?"),
                regex("\\-|/"),
                regex("(10|20|30|31|[012]?[1-9])\\.(10|11|12|0?[1-9])\\.?((?<=\\.)\\d{2,4})?"),
            ],
            |ts| {
                let d1s = rx(ts[0], 0)?;
                let m1s = rx(ts[0], 1).unwrap_or_default();
                let d2s = rx(ts[2], 0)?;
                let m2s = rx(ts[2], 1)?;
                let ys = rx(ts[2], 2).unwrap_or_default();
                let d1 = parse_int(&d1s)?;
                let d2 = parse_int(&d2s)?;
                let m2 = parse_int(&m2s)?;
                if m1s.is_empty() && ys.is_empty() {
                    ttd(interval(Closed, &month_day(m2, d1), &month_day(m2, d2)))
                } else if m1s.is_empty() {
                    let y = parse_int(&ys)?;
                    ttd(interval(Closed, &year_month_day(y, m2, d1), &year_month_day(y, m2, d2)))
                } else if ys.is_empty() {
                    let m1 = parse_int(&m1s)?;
                    ttd(interval(Closed, &month_day(m1, d1), &month_day(m2, d2)))
                } else {
                    let m1 = parse_int(&m1s)?;
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
            "<duration> after <time>",
            vec![dimension(Dim::Time), regex("sonra(ki)?"), dimension(Dim::Duration)],
            |ts| tt(duration_after(dur(ts[2])?, td(ts[0])?)),
        ),
        rule(
            "<duration> after <time>",
            vec![dimension(Dim::Time), dimension(Dim::Duration), regex("sonra")],
            |ts| tt(duration_after(dur(ts[1])?, td(ts[0])?)),
        ),
        rule(
            "for <duration> from <time>",
            vec![
                dimension(Dim::Time),
                regex("itibaren"),
                dimension(Dim::Duration),
                regex("boyunca|süresince"),
            ],
            |ts| {
                let t = td(ts[0])?;
                ttd(interval(Closed, t, &duration_after(dur(ts[2])?, t)))
            },
        ),
        rule("<duration> ago", vec![dimension(Dim::Duration), regex("önce")], |ts| {
            tt(duration_ago(dur(ts[0])?))
        }),
        rule(
            "<duration> before <time>",
            vec![dimension(Dim::Time), regex("önceki"), dimension(Dim::Duration)],
            |ts| tt(duration_before(dur(ts[2])?, td(ts[0])?)),
        ),
        rule("evening", vec![regex("akşam(a|dan)?")], |_| latent_pod(18, 0)),
        rule(
            "hh:mm",
            vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:.ч]([0-5]\\d)(?:час(ов|а|у)?|ч)?")],
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
            |ts| match &td(ts[0])?.form {
                Some(Form::TimeOfDay { hours: Some(h), is12h }) => {
                    tt(hour_minute(*is12h, *h, get_int_value(ts[1])?))
                }
                _ => None,
            },
        ),
        rule("the ides of <named-month>", vec![pred(is_a_month), regex("ortası")], |ts| {
            let t = td(ts[0])?;
            match t.form {
                Some(Form::Month(m)) => {
                    let d = if [3, 5, 7, 10].contains(&m) { 15 } else { 13 };
                    ttd(intersect(t, &day_of_month(d)))
                }
                _ => None,
            }
        }),
        rule("in <duration>", vec![dimension(Dim::Duration), regex("içinde|içerisinde")], |ts| {
            tt(in_duration(dur(ts[0])?))
        }),
        rule("by <time>", vec![dimension(Dim::Time), regex("kadar")], |ts| {
            ttd(interval(Open, &now(), td(ts[0])?))
        }),
        rule("<duration> hence", vec![dimension(Dim::Duration), regex("sonra")], |ts| {
            tt(in_duration(dur(ts[0])?))
        }),
        rule(
            "<duration> from now",
            vec![regex("(bugünden\\s)?sonra(ki)?"), dimension(Dim::Duration)],
            |ts| tt(in_duration(dur(ts[1])?)),
        ),
        rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule("intersect by ','", vec![pred(is_not_latent), regex(","), pred(is_not_latent)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[2])?))
        }),
        rule("last <cycle>", vec![regex("son|geçen|geçtiğimiz|önceki"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, -1))
        }),
        rule(
            "last <cycle> of <time>",
            vec![dimension(Dim::Time), regex("son"), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_last_of(grain_of(ts[2])?, td(ts[0])?)),
        ),
        // Haskell: predLastOf td1 td2 with td1 = container Time, td2 = day-of-week.
        rule(
            "last <day-of-week> of <time>",
            vec![dimension(Dim::Time), regex("son"), pred(is_a_day_of_week)],
            |ts| tt(pred_last_of(td(ts[0])?, td(ts[2])?)),
        ),
        rule(
            "last n <cycle>",
            vec![
                regex("son|geçen|geçtiğimiz|önceki"),
                pred(is_integer_between_tok(1, 9999)),
                dimension(Dim::TimeGrain),
            ],
            |ts| {
                let n = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, -n))
            },
        ),
        rule("last <time>", vec![regex("geçen|önceki|geçtiğimiz|son"), pred(is_ok_with_this_next)], |ts| {
            tt(pred_nth(-1, false, td(ts[1])?))
        }),
        rule_this_time(),
        rule(
            "next <time>",
            vec![regex("önümüzdeki|gelecek|sonraki"), pred(is_ok_with_this_next)],
            |ts| tt(pred_nth(0, true, td(ts[1])?)),
        ),
        rule("lunch", vec![regex("öğlen? (yemeği|arası)")], |_| latent_pod(12, 14)),
        rule("midnight|EOD|end of day", vec![regex("gece\\syarısı|gün\\ssonu|gün\\sbitimi")], |_| {
            tt(hour(false, 0))
        }),
        rule("mm/dd", vec![regex("([012]?[1-9]|10|20|30|31)\\.(10|11|12|0?[1-9])\\.?")], |ts| {
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(month_day(m, d))
        }),
        rule(
            "mm/dd/yyyy",
            vec![regex("(1[0-2]|0?[1-9])[-/\\s](3[01]|[12]\\d|0?[1-9])[-/\\s](\\d{2,4})")],
            |ts| {
                let m = parse_int(&rx(ts[0], 0)?)?;
                let d = parse_int(&rx(ts[0], 1)?)?;
                let y = parse_int(&rx(ts[0], 2)?)?;
                tt(year_month_day(y, m, d))
            },
        ),
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
            "<named-month> <day-of-month> (non ordinal)",
            vec![pred(is_a_month), pred(is_dom_integer)],
            |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
        ),
        rule("next <cycle>", vec![regex("sonraki|önümüzdeki|gelecek"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, 1))
        }),
        rule(
            "next n <cycle>",
            vec![regex("önümüzdeki|sonraki"), pred(is_integer_between_tok(1, 9999)), dimension(Dim::TimeGrain)],
            |ts| {
                let v = get_int_value(ts[1])?;
                tt(cycle_n(true, grain_of(ts[2])?, v))
            },
        ),
        rule("night", vec![regex("gece(ye|den)?")], |_| latent_pod(0, 4)),
        rule("noon", vec![regex("öğle(n|den|ye)?")], |_| tt(hour(false, 12))),
        rule(
            "nth <time> after <time>",
            vec![dimension(Dim::Time), regex("sonra(ki)?"), dimension(Dim::Ordinal), dimension(Dim::Time)],
            |ts| tt(pred_nth_after(ordinal_value(ts[2])? - 1, td(ts[0])?, td(ts[3])?)),
        ),
        rule(
            "nth <time> of <time>",
            vec![dimension(Dim::Time), dimension(Dim::Ordinal), dimension(Dim::Time)],
            |ts| {
                let v = ordinal_value(ts[1])?;
                ttd(intersect(td(ts[2])?, td(ts[0])?).map(|x| pred_nth(v - 1, false, &x)))
            },
        ),
        rule(
            "<ordinal> <cycle> of <time>",
            vec![dimension(Dim::Time), dimension(Dim::Ordinal), dimension(Dim::TimeGrain)],
            |ts| tt(cycle_nth_after(true, grain_of(ts[2])?, ordinal_value(ts[1])? - 1, td(ts[0])?)),
        ),
        rule("<ordinal> quarter", vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter))], |ts| {
            tt(cycle_nth_after(false, G::Quarter, ordinal_value(ts[0])? - 1, &cycle_nth(G::Year, 0)))
        }),
        rule(
            "<ordinal> quarter <year>",
            vec![dimension(Dim::Ordinal), pred(is_grain(G::Quarter)), dimension(Dim::Time)],
            |ts| tt(cycle_nth_after(false, G::Quarter, ordinal_value(ts[0])? - 1, td(ts[2])?)),
        ),
        rule("part of days", vec![regex("(sabah(ı|a|tan)?|öğlen?|akşam|gece|öğle\\syemeği)")], |ts| {
            let m = to_lower(&rx(ts[0], 0)?);
            let (start, end) = match m.as_str() {
                "sabah" | "sabahı" | "sabaha" | "sabahtan" => (hour(false, 0), hour(false, 12)),
                "akşam" | "gece" => (hour(false, 18), hour(false, 0)),
                "öğlen" | "öğle" | "öğle yemeği" => (hour(false, 12), hour(false, 14)),
                _ => (hour(false, 12), hour(false, 19)),
            };
            let x = interval(Open, &start, &end)?;
            tt(part_of_day(mk_latent(x)))
        }),
        rule("<part-of-day> of <time>", vec![pred(is_a_part_of_day), dimension(Dim::Time)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule(
            "about|exactly <time-of-day>",
            vec![pred(is_grain_finer_than(G::Year)), regex("gibi|civarı(nda)?")],
            |ts| tt(not_latent(td(ts[0])?.clone())),
        ),
        rule(
            "about|exactly <time-of-day>",
            vec![regex("yaklaşık|tam(\\solarak)?"), pred(is_grain_finer_than(G::Year))],
            |ts| tt(not_latent(td(ts[1])?.clone())),
        ),
        rule("this <cycle>", vec![regex("bu"), dimension(Dim::TimeGrain)], |ts| {
            tt(cycle_nth(grain_of(ts[1])?, 0))
        }),
        rule("this <part-of-day>", vec![regex("bu"), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(&today(), td(ts[1])?).map(part_of_day))
        }),
        rule_this_time(),
        rule("this|next <day-of-week>", vec![regex("bu|sonraki"), pred(is_a_day_of_week)], |ts| {
            tt(pred_nth(0, true, td(ts[1])?))
        }),
        rule("<time> <part-of-day>", vec![dimension(Dim::Time), pred(is_a_part_of_day)], |ts| {
            ttd(intersect(td(ts[0])?, td(ts[1])?))
        }),
        rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
            let n = get_int_value(ts[0])?;
            tt(mk_latent(hour(n < 12, n)))
        }),
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
        // Haskell production skips the first Time token.
        rule("until <time-of-day>", vec![dimension(Dim::Time), regex("kadar")], |ts| {
            tt(with_direction(IntervalDirection::Before, td(ts.get(1)?)?.clone()))
        }),
        rule(
            "week",
            vec![regex("((bu\\s)?hafta boyunca)|(bu hafta)|(haftanın geri kalanı)")],
            |ts| {
                let m = to_lower(&rx(ts[0], 0)?);
                let end = cycle_nth_after(true, G::Day, -2, &cycle_nth(G::Week, 1));
                let period = match m.as_str() {
                    "hafta boyunca" => interval(Closed, &cycle_nth(G::Week, 0), &end),
                    "haftanın geri kalanı" | "bu hafta" => interval(Open, &today(), &end),
                    _ => None,
                };
                match m.as_str() {
                    "bu hafta" => ttd(period.map(mk_latent)),
                    _ => ttd(period),
                }
            },
        ),
        rule("week-end", vec![regex("hafta sonu")], |_| tt(mk_ok_for_this_next(weekend()))),
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
            "<time> timezone",
            vec![
                pred(not_latent_tod),
                regex(
                    "\\b(YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT)\\b",
                ),
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
    v.extend(rules_computed_holidays());
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
