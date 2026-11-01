//! `Duckling.Time.FR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::DurationData;
use crate::dims::time::helpers::*;
use crate::dims::time::types::{
    Form, IntervalDirection, TimeData,
    TimeIntervalType::{Closed, Open},
};
use crate::dims::time_grain::Grain as G;
use crate::dims::util::parse_int;
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

/// `TimeData {form = Just (TimeOfDay (Just hours) is12H)}`.
fn tod(t: &Token) -> Option<(i64, bool)> {
    match t {
        Token::Time(TimeData { form: Some(Form::TimeOfDay { hours: Some(h), is12h }), .. }) => {
            Some((*h, *is12h))
        }
        _ => None,
    }
}

fn r_aujourdhui() -> Rule {
    rule(
        "aujourd'hui",
        vec![regex("(aujourd'? ?hui)|(ce jour)|(dans la journ(é|e)e?)|(en ce moment)")],
        |_| tt(today()),
    )
}

fn r_dayofmonth_namedmonth() -> Rule {
    rule(
        "<day-of-month> <named-month>",
        vec![pred(is_dom_integer), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
    )
}

fn r_hier() -> Rule {
    rule("hier", vec![regex("hier|la veille")], |_| tt(cycle_nth(G::Day, -1)))
}

/// A rule whose production is `Token Time . partOfDay <$> interval Open (hour False a) (hour False b)`.
fn pod_rule(name: &str, re: &str, a: i64, b: i64) -> Rule {
    rule(name, vec![regex(re)], move |_| {
        ttd(interval(Open, &hour(false, a), &hour(false, b)).map(part_of_day))
    })
}

fn r_debut_de_soiree() -> Rule {
    pod_rule("début de soirée", "(en |au )?d(é|e)but de (la )?soir(é|e)e", 18, 21)
}
fn r_fin_de_soiree() -> Rule {
    pod_rule("fin de soirée", "(en |(à|a) la )?fin de (la )?soir(é|e)e", 21, 0)
}
fn r_debut_de_matinee() -> Rule {
    pod_rule(
        "début de matinée",
        "le matin (tr(e|è)s )?t(ô|o)t|(tr(e|è)s )?t(ô|o)t le matin|(en |au )?d(é|e)but de (la )?matin(é|e)e",
        7,
        9,
    )
}
fn r_milieu_de_matinee() -> Rule {
    pod_rule("milieu de matinée", "(en |au )?milieu de (la )?matin(é|e)e", 9, 11)
}
fn r_fin_de_matinee() -> Rule {
    pod_rule("fin de matinée", "(en |(à|a) la )?fin de (la )?matin(é|e)e", 10, 12)
}
fn r_debut_apres_midi() -> Rule {
    pod_rule(
        "début d'après-midi",
        "(au |en )?d(é|e)but (d'|de l')apr(e|é|è)s( |\\-)midi",
        12,
        14,
    )
}
fn r_milieu_apres_midi() -> Rule {
    pod_rule("milieu d'après-midi", "(au |en )?milieu (d'|de l')apr(e|é|è)s( |\\-)midi", 14, 17)
}
fn r_fin_apres_midi() -> Rule {
    pod_rule("fin d'après-midi", "((à|a) la |en )?fin (d'|de l')apr(e|é|è)s( |\\-)midi", 17, 19)
}
fn r_debut_de_journee() -> Rule {
    pod_rule("début de journée", "(en |au )?d(é|e)but de (la )?journ(é|e)e", 6, 10)
}
fn r_milieu_de_journee() -> Rule {
    pod_rule("milieu de journée", "(en |au )?milieu de (la )?journ(é|e)e", 11, 16)
}
fn r_fin_de_journee() -> Rule {
    pod_rule("fin de journée", "(en |(à|a) la )?fin de (la )?journ(é|e)e", 17, 21)
}

fn dow_interval_rule(name: &str, re: &str, a: i64, b: i64) -> Rule {
    rule(name, vec![regex(re)], move |_| {
        ttd(interval(Open, &day_of_week(a), &day_of_week(b)))
    })
}

fn r_debut_de_semaine() -> Rule {
    dow_interval_rule("début de semaine", "(en |au )?d(é|e)but de (cette |la )?semaine", 1, 2)
}
fn r_milieu_de_semaine() -> Rule {
    dow_interval_rule("milieu de semaine", "(en |au )?milieu de (cette |la )?semaine", 3, 4)
}
fn r_fin_de_semaine() -> Rule {
    dow_interval_rule("fin de semaine", "(en |(à|a) la )?fin de (cette |la )?semaine", 4, 7)
}

fn r_lendemain_du_time() -> Rule {
    rule(
        "le lendemain du <time>",
        vec![regex("(le|au)? ?lendemain du"), pred(is_grain_coarser_than(G::Hour))],
        |ts| tt(cycle_nth_after(true, G::Day, 1, td(ts[1])?)),
    )
}

fn r_ce_partofday() -> Rule {
    rule(
        "ce <part-of-day>",
        vec![regex("cet?t?e?"), pred(is_a_part_of_day)],
        |ts| ttd(intersect(&today(), td(ts[1])?).map(|x| part_of_day(not_latent(x)))),
    )
}

fn r_ce_time() -> Rule {
    rule("ce <time>", vec![regex("ce"), dimension(Dim::Time)], |ts| {
        tt(pred_nth(0, false, td(ts[1])?))
    })
}

fn r_duration_avant_time() -> Rule {
    rule(
        "<duration> avant <time>",
        vec![dimension(Dim::Duration), regex("avant"), dimension(Dim::Time)],
        |ts| tt(duration_before(dur(ts[0])?, td(ts[2])?)),
    )
}

fn r_midi() -> Rule {
    rule("midi", vec![regex("midi")], |_| tt(hour(false, 12)))
}

fn r_datetime_datetime_interval() -> Rule {
    rule(
        "<datetime> - <datetime> (interval)",
        vec![pred(is_not_latent), regex("\\-|au|jusqu'(au?|à)"), pred(is_not_latent)],
        |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
    )
}

fn r_de_timeofday_timeofday_interval() -> Rule {
    rule(
        "de <time-of-day> - <time-of-day> (interval)",
        vec![
            regex("(midi )?de"),
            pred(is_a_time_of_day),
            regex("\\-|(jusqu')?(à|au?)"),
            pred(is_a_time_of_day),
        ],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn r_n_prochains_cycle() -> Rule {
    rule(
        "n prochains <cycle>",
        vec![
            pred(is_integer_between_tok(2, 9999)),
            regex("prochaine?s?|suivante?s?|apr(e|è|é)s"),
            dimension(Dim::TimeGrain),
        ],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_n(true, grain_of(ts[2])?, n))
        },
    )
}

fn r_n_derniers_cycle() -> Rule {
    rule(
        "n derniers <cycle>",
        vec![
            pred(is_integer_between_tok(2, 9999)),
            regex("derni(e|è|é)re?s?"),
            dimension(Dim::TimeGrain),
        ],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_n(true, grain_of(ts[2])?, -n))
        },
    )
}

fn r_nth_time_of_time() -> Rule {
    rule(
        "nth <day-of-week> of <month-or-greater>",
        vec![
            dimension(Dim::Ordinal),
            pred(is_a_day_of_week),
            regex("d[eu]"),
            pred(|t| !is_grain_finer_than(G::Month)(t)),
        ],
        |ts| {
            let od = ordinal_value(ts[0])?;
            let x = intersect(td(ts[3])?, td(ts[1])?)?;
            tt(pred_nth(od - 1, false, &x))
        },
    )
}

fn r_the_nth_time_of_time() -> Rule {
    rule(
        "the nth <day-of-week> of <month-or-greater>",
        vec![
            regex("le"),
            dimension(Dim::Ordinal),
            pred(is_a_day_of_week),
            regex("d[eu]"),
            pred(|t| !is_grain_finer_than(G::Month)(t)),
        ],
        |ts| {
            let od = ordinal_value(ts[1])?;
            let x = intersect(td(ts[4])?, td(ts[2])?)?;
            tt(pred_nth(od - 1, false, &x))
        },
    )
}

fn r_avant_timeofday() -> Rule {
    rule(
        "avant <time-of-day>",
        vec![regex("(n[ ']importe quand )?(avant|jusqu'(a|à))"), dimension(Dim::Time)],
        |ts| tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone())),
    )
}

fn r_entre_datetime_et_datetime_interval() -> Rule {
    rule(
        "entre <datetime> et <datetime> (interval)",
        vec![regex("entre"), dimension(Dim::Time), regex("et"), dimension(Dim::Time)],
        |ts| {
            let (a, b) = (td(ts[1])?, td(ts[3])?);
            if same_grain(a, b) {
                ttd(interval(Closed, a, b))
            } else {
                None
            }
        },
    )
}

fn r_du_datetimedayofweek_dd_monthinterval() -> Rule {
    rule(
        "du <datetime>-<day-of-week> dd <month>(interval)",
        vec![
            regex("du"),
            dimension(Dim::Time),
            regex("\\-|(jusqu')?au"),
            pred(is_a_day_of_week),
            pred(is_dom_value),
            pred(is_a_month),
        ],
        |ts| {
            let td2 = td(ts[5])?;
            let from = intersect(td2, td(ts[1])?)?;
            let to = intersect_dom(td2, ts[4])?;
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn r_du_dd_au_dd_latent_month_interval() -> Rule {
    rule(
        "du dd au dd (interval) (latent month)",
        vec![regex("du"), pred(is_dom_value), regex("(jusqu')?au"), pred(is_dom_value)],
        |ts| {
            let n1 = get_int_value(ts[1])?;
            let n2 = get_int_value(ts[3])?;
            ttd(interval(Closed, &day_of_month(n1), &day_of_month(n2)))
        },
    )
}

fn r_datetimedayofweek_dd_monthinterval() -> Rule {
    rule(
        "<datetime>-<day-of-week> dd <month>(interval)",
        vec![
            pred(is_grain_of_time(G::Day)),
            regex("\\-|(jusqu')?au"),
            pred(is_a_day_of_week),
            pred(is_dom_value),
            pred(is_a_month),
        ],
        |ts| {
            let td2 = td(ts[4])?;
            let from = intersect(td2, td(ts[0])?)?;
            let to = intersect_dom(td2, ts[3])?;
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn r_entre_dd_et_dd_monthinterval() -> Rule {
    rule(
        "entre dd et dd <month>(interval)",
        vec![
            regex("entre( le)?"),
            pred(is_dom_value),
            regex("et( le)?"),
            pred(is_dom_value),
            pred(is_a_month),
        ],
        |ts| {
            let t = td(ts[4])?;
            let dom1 = intersect_dom(t, ts[1])?;
            let dom2 = intersect_dom(t, ts[3])?;
            ttd(interval(Closed, &dom1, &dom2))
        },
    )
}

fn r_datetime_dd_monthinterval() -> Rule {
    rule(
        "<datetime>-dd <month>(interval)",
        vec![
            pred(is_grain_of_time(G::Day)),
            regex("\\-|(jusqu')?au"),
            pred(is_dom_value),
            pred(is_a_month),
        ],
        |ts| {
            let td2 = td(ts[3])?;
            let from = intersect(td2, td(ts[0])?)?;
            let to = intersect_dom(td2, ts[2])?;
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn r_du_dd_au_dd_month_interval() -> Rule {
    rule(
        "du dd/nth au dd/nth month (interval)",
        vec![
            regex("du"),
            pred(is_dom_value),
            regex("\\-|au|jusqu'(au?|à)"),
            pred(is_dom_value),
            pred(is_a_month),
        ],
        |ts| {
            let t = td(ts[4])?;
            let dom1 = intersect_dom(t, ts[1])?;
            let dom2 = intersect_dom(t, ts[3])?;
            ttd(interval(Closed, &dom1, &dom2))
        },
    )
}

fn r_hhhmm_timeofday() -> Rule {
    rule(
        "hh(:|h)mm (time-of-day)",
        vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:h]([0-5]\\d)")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(h < 12, h, m))
        },
    )
}

fn r_dd_mm() -> Rule {
    rule("dd mm", vec![regex("(3[01]|[12]\\d|0?[1-9]) (1[0-2]|0?[1-9])")], |ts| {
        let m = parse_int(&rx(ts[0], 1)?)?;
        let d = parse_int(&rx(ts[0], 0)?)?;
        tt(month_day(m, d))
    })
}

fn r_dd_mm_yyyy() -> Rule {
    rule(
        "dd mm yyyy",
        vec![regex("(3[01]|[12]\\d|0?[1-9]) (1[0-2]|0?[1-9]) (\\d{2,4})")],
        |ts| {
            let y = parse_int(&rx(ts[0], 2)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let d = parse_int(&rx(ts[0], 0)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn r_namedmonth_prochain() -> Rule {
    rule("<named-month> prochain", vec![dimension(Dim::Time), regex("prochain")], |ts| {
        tt(pred_nth(1, false, td(ts[0])?))
    })
}

fn r_dici_duration() -> Rule {
    rule(
        "d'ici <duration>",
        vec![regex("d'ici|dans l('|es?)"), dimension(Dim::Duration)],
        |ts| ttd(interval(Open, &now(), &in_duration(dur(ts[1])?))),
    )
}

fn r_toussaint() -> Rule {
    rule(
        "toussaint",
        vec![regex("((la |la journ(é|e)e de la |jour de la )?toussaint|jour des morts)")],
        |_| tt(month_day(11, 1)),
    )
}

fn r_dernier_cycle_de_time_latent() -> Rule {
    rule(
        "dernier <cycle> de <time> (latent)",
        vec![regex("derni(e|é|è)re?"), dimension(Dim::TimeGrain), regex("d['e]"), dimension(Dim::Time)],
        |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
    )
}

fn r_duration_apres_time() -> Rule {
    rule(
        "<duration> apres <time>",
        vec![dimension(Dim::Duration), regex("apr(e|è)s"), dimension(Dim::Time)],
        |ts| tt(duration_after(dur(ts[0])?, td(ts[2])?)),
    )
}

fn r_n_cycle_apres() -> Rule {
    rule(
        "n <cycle> après",
        vec![
            pred(is_integer_between_tok(2, 9999)),
            dimension(Dim::TimeGrain),
            regex("(d')? ?apr(e|è|é)s|qui sui(t|ves?)|plus tard"),
        ],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth(grain_of(ts[1])?, n))
        },
    )
}

fn r_season4() -> Rule {
    rule("season", vec![regex("(ce )?printemps")], |_| {
        ttd(interval(Open, &month_day(3, 20), &month_day(6, 21)))
    })
}

fn r_year_latent2() -> Rule {
    rule("year (latent)", vec![pred(is_integer_between_tok(2101, 10000))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(mk_latent(year(n)))
    })
}

fn r_n_cycle_avant() -> Rule {
    rule(
        "n <cycle> avant",
        vec![
            pred(is_integer_between_tok(2, 9999)),
            dimension(Dim::TimeGrain),
            regex("(d')? ?avant|plus t(o|ô)t"),
        ],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth(grain_of(ts[1])?, -n))
        },
    )
}

fn r_dim_time_du_matin() -> Rule {
    rule(
        "<dim time> du matin",
        vec![pred(is_a_time_of_day), regex("((du|dans|de) )?((au|le|la) )?mat(in(é|e)?e?)?")],
        |ts| {
            let morning = interval(Open, &hour(false, 0), &hour(false, 12)).map(|x| part_of_day(mk_latent(x)))?;
            ttd(intersect(td(ts[0])?, &morning))
        },
    )
}

fn r_ordinal_weekend_de_time() -> Rule {
    rule(
        "<ordinal> week-end de <time>",
        vec![dimension(Dim::Ordinal), regex("week(\\s|-)?end (d['eu]|en|du mois de)"), pred(is_a_month)],
        |ts| {
            let n = get_int_value(ts[0])?;
            let td2 = intersect(td(ts[2])?, &weekend())?;
            tt(pred_nth(n - 1, false, &td2))
        },
    )
}

fn r_hourofday_et_quart() -> Rule {
    rule("<hour-of-day> et quart", vec![pred(is_an_hour_of_day), regex("(et )?quart")], |ts| {
        let (h, is12h) = tod(ts[0])?;
        tt(hour_minute(is12h, h, 15))
    })
}

fn r_hourofday_et_demi() -> Rule {
    rule("<hour-of-day> et demi", vec![pred(is_an_hour_of_day), regex("et demie?")], |ts| {
        let (h, is12h) = tod(ts[0])?;
        tt(hour_minute(is12h, h, 30))
    })
}

fn r_hourofday_et_trois_quart() -> Rule {
    rule(
        "<hour-of-day> et trois quarts",
        vec![pred(is_an_hour_of_day), regex("(et )?(3|trois) quarts?")],
        |ts| {
            let (h, is12h) = tod(ts[0])?;
            tt(hour_minute(is12h, h, 45))
        },
    )
}

fn r_hourofday_etpass_de_numeral() -> Rule {
    rule(
        "<hour-of-day> et|passé de <number>",
        vec![pred(is_an_hour_of_day), regex("et|(pass(é|e)e? de)"), pred(is_integer_between_tok(1, 59))],
        |ts| {
            let (h, is12h) = tod(ts[0])?;
            let n = get_int_value(ts[2])?;
            tt(hour_minute(is12h, h, n))
        },
    )
}

fn r_hourofday_etpass_de_numeral_minutes() -> Rule {
    rule(
        "<hour-of-day> et|passé de <number> minutes",
        vec![
            pred(is_an_hour_of_day),
            regex("et|(pass(é|e)e? de)"),
            pred(is_integer_between_tok(1, 59)),
            regex("min\\.?(ute)?s?"),
        ],
        |ts| {
            let (h, is12h) = tod(ts[0])?;
            let n = get_int_value(ts[2])?;
            tt(hour_minute(is12h, h, n))
        },
    )
}

fn r_hourofday_integer() -> Rule {
    rule(
        "<hour-of-day> <integer> (as relative minutes)",
        vec![
            pred(|t| is_not_latent(t) && is_an_hour_of_day(t) && has_no_direction(t)),
            pred(is_integer_between_tok(1, 59)),
        ],
        |ts| {
            let (h, is12h) = tod(ts[0])?;
            let n = get_int_value(ts[1])?;
            tt(hour_minute(is12h, h, n))
        },
    )
}

fn r_hourofday_integer_minutes() -> Rule {
    rule(
        "<hour-of-day> <integer> minutes",
        vec![
            pred(|t| is_not_latent(t) && is_an_hour_of_day(t)),
            pred(is_integer_between_tok(1, 59)),
            regex("min\\.?(ute)?s?"),
        ],
        |ts| {
            let (h, is12h) = tod(ts[0])?;
            let n = get_int_value(ts[1])?;
            tt(hour_minute(is12h, h, n))
        },
    )
}

fn r_hourofday_moins_integer() -> Rule {
    rule(
        "<hour-of-day> moins <integer> (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("moins( le)?"), pred(is_integer_between_tok(1, 59))],
        |ts| {
            let n = get_int_value(ts[2])?;
            ttd(minutes_before(n, td(ts[0])?))
        },
    )
}

fn r_hourofday_moins_quart() -> Rule {
    rule(
        "<hour-of-day> moins quart",
        vec![pred(is_an_hour_of_day), regex("moins( le)? quart")],
        |ts| ttd(minutes_before(15, td(ts[0])?)),
    )
}

fn r_apres_le_dayofmonth() -> Rule {
    rule(
        "après le <day-of-month>",
        vec![regex("(apr(e|è)s le|(a|à) partir du)"), pred(is_dom_integer)],
        |ts| {
            let n = get_int_value(ts[1])?;
            tt(with_direction(IntervalDirection::After, day_of_month(n)))
        },
    )
}

fn r_cycle_dernier() -> Rule {
    rule(
        "<cycle> dernier",
        vec![dimension(Dim::TimeGrain), regex("derni(è|e)re?|pass(é|e)e?|pr(e|é)c(e|é)dente?|(d')? ?avant")],
        |ts| tt(cycle_nth(grain_of(ts[0])?, -1)),
    )
}

fn r_le_ordinal_cycle_de_time() -> Rule {
    rule(
        "le <ordinal> <cycle> de <time>",
        vec![
            regex("l[ea]"),
            dimension(Dim::Ordinal),
            dimension(Dim::TimeGrain),
            regex("d['eu]|en"),
            dimension(Dim::Time),
        ],
        |ts| {
            let n = get_int_value(ts[1])?;
            tt(cycle_nth_after(true, grain_of(ts[2])?, n - 1, td(ts[4])?))
        },
    )
}

fn r_en_semaine() -> Rule {
    rule("en semaine", vec![regex("(pendant la |en )?semaine")], |_| {
        ttd(interval(Open, &day_of_week(1), &day_of_week(5)))
    })
}

fn r_ddmm() -> Rule {
    rule("dd/-mm", vec![regex("(3[01]|[12]\\d|0?[1-9])[/-](1[0-2]|0?[1-9])")], |ts| {
        let m = parse_int(&rx(ts[0], 1)?)?;
        let d = parse_int(&rx(ts[0], 0)?)?;
        tt(month_day(m, d))
    })
}

fn r_namedmonthnamedday_dernierpass() -> Rule {
    rule(
        "<named-month|named-day> dernier|passé",
        vec![pred(is_grain_coarser_than(G::Hour)), regex("derni(e|é|è)re?|pass(é|e)e?")],
        |ts| tt(pred_nth(-1, false, td(ts[0])?)),
    )
}

fn r_le_cycle_dernier() -> Rule {
    rule(
        "le <cycle> dernier",
        vec![regex("l[ae']? ?"), dimension(Dim::TimeGrain), regex("derni(è|e)re?|pass(é|e)e?")],
        |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
    )
}

fn r_n_cycle_passes_precedents() -> Rule {
    rule(
        "n <cycle> passes|precedents",
        vec![
            pred(is_integer_between_tok(2, 9999)),
            dimension(Dim::TimeGrain),
            regex("pass(e|è|é)(e|è|é)?s?|pr(e|é)c(e|é)dente?s?|(d')? ?avant|plus t(o|ô)t"),
        ],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_n(true, grain_of(ts[1])?, -n))
        },
    )
}

fn r_soir() -> Rule {
    rule("soir", vec![regex("soir(é|e)?e?")], |_| {
        ttd(interval(Open, &hour(false, 18), &hour(false, 0)).map(|x| mk_latent(part_of_day(x))))
    })
}

fn r_dddd_monthinterval() -> Rule {
    rule(
        "dd-dd <month>(interval)",
        vec![
            regex("(3[01]|[12]\\d|0?[1-9])"),
            regex("\\-|(jusqu')?au"),
            regex("(3[01]|[12]\\d|0?[1-9])"),
            pred(is_a_month),
        ],
        |ts| {
            let d1 = parse_int(&rx(ts[0], 0)?)?;
            let d2 = parse_int(&rx(ts[2], 0)?)?;
            let t = td(ts[3])?;
            let from = intersect(t, &day_of_month(d1))?;
            let to = intersect(t, &day_of_month(d2))?;
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn r_timeofday_latent() -> Rule {
    rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(mk_latent(hour(n < 12, n)))
    })
}

fn r_du_ddd_dayofweek_dd_monthinterval() -> Rule {
    rule(
        "du dd-<day-of-week> dd <month>(interval)",
        vec![
            regex("du"),
            regex("(3[01]|[12]\\d|0?[1-9])"),
            regex("\\-|(jusqu')?au"),
            pred(is_a_day_of_week),
            regex("(3[01]|[12]\\d|0?[1-9])"),
            pred(is_a_month),
        ],
        |ts| {
            let d1 = parse_int(&rx(ts[1], 0)?)?;
            let d2 = parse_int(&rx(ts[4], 0)?)?;
            let t = td(ts[5])?;
            let from = intersect(&day_of_month(d1), t)?;
            let to = intersect(&day_of_month(d2), t)?;
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn r_debut_namedmonthinterval() -> Rule {
    rule(
        "début <named-month>(interval)",
        vec![regex("d(é|e)but( du mois d[e'] ?)?"), pred(is_a_month)],
        |ts| {
            let t = td(ts[1])?;
            let from = intersect(&day_of_month(1), t)?;
            let to = intersect(&day_of_month(5), t)?;
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn r_le_cycle_de_time() -> Rule {
    rule(
        "le <cycle> de <time>",
        vec![regex("l[ea]"), dimension(Dim::TimeGrain), regex("d['eu]|en"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(true, grain_of(ts[1])?, 0, td(ts[3])?)),
    )
}

fn r_season3() -> Rule {
    rule("season", vec![regex("(cet )?hiver")], |_| {
        ttd(interval(Open, &month_day(12, 21), &month_day(3, 20)))
    })
}

fn r_season() -> Rule {
    rule("season", vec![regex("(cet )?(é|e)t(é|e)")], |_| {
        ttd(interval(Open, &month_day(6, 21), &month_day(9, 23)))
    })
}

fn r_apresmidi() -> Rule {
    rule("après-midi", vec![regex("apr(e|é|è)s( |\\-)midi")], |_| {
        ttd(interval(Open, &hour(false, 12), &hour(false, 19)).map(|x| mk_latent(part_of_day(x))))
    })
}

fn r_noel() -> Rule {
    rule("noel", vec![regex("(jour de )?no(e|ë)l")], |_| tt(month_day(12, 25)))
}

fn r_dayofweek_prochain() -> Rule {
    rule("<day-of-week> prochain", vec![pred(is_a_day_of_week), regex("prochain")], |ts| {
        tt(pred_nth(0, true, td(ts[0])?))
    })
}

fn r_demain() -> Rule {
    rule("demain", vec![regex("(le len)?demain")], |_| tt(cycle_nth(G::Day, 1)))
}

fn r_namedmonthnamedday_suivant_dapres() -> Rule {
    rule(
        "<named-month|named-day> suivant|d'après",
        vec![pred(is_grain_coarser_than(G::Hour)), regex("suivante?s?|d'apr(e|é|è)s")],
        |ts| tt(pred_nth(1, false, td(ts[0])?)),
    )
}

fn r_nameddy_en_huit() -> Rule {
    rule("<named-day> en huit", vec![pred(is_a_day_of_week), regex("en (huit|8)")], |ts| {
        tt(pred_nth(1, false, td(ts[0])?))
    })
}

fn r_hhmm_military_timeofday() -> Rule {
    rule(
        "hhmm (military time-of-day)",
        vec![regex("((?:[01]?\\d)|(?:2[0-3]))([0-5]\\d)")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(mk_latent(hour_minute(false, h, m)))
        },
    )
}

fn r_dim_time_du_soir() -> Rule {
    rule(
        "<dim time> du soir",
        vec![pred(is_a_time_of_day), regex("((du|dans|de) )?((au|le|la) )?soir(é|e)?e?")],
        |ts| {
            let td2 = interval(Open, &hour(false, 16), &hour(false, 0)).map(|x| mk_latent(part_of_day(x)))?;
            ttd(intersect(td(ts[0])?, &td2))
        },
    )
}

fn r_apres_timeofday() -> Rule {
    rule(
        "après <time-of-day|day>",
        vec![regex("(apr(e|è)s|(a|à) partir de|(un peu )?plus tard que)"), dimension(Dim::Time)],
        |ts| tt(with_direction(IntervalDirection::After, td(ts[1])?.clone())),
    )
}

/// `td2 <- interval Open (hour False a) (hour False b); Token Time . partOfDay <$> intersect today td2`.
fn today_interval_rule(name: &str, re: &str, a: i64, b: i64) -> Rule {
    rule(name, vec![regex(re)], move |_| {
        let td2 = interval(Open, &hour(false, a), &hour(false, b))?;
        ttd(intersect(&today(), &td2).map(part_of_day))
    })
}

fn r_apres_le_dejeuner() -> Rule {
    today_interval_rule("après le déjeuner", "apr(e|è)s (le )?d(e|é|è)jeuner", 13, 17)
}

fn r_intersect() -> Rule {
    rule("intersect", vec![pred(is_not_latent), pred(is_not_latent)], |ts| {
        ttd(intersect(td(ts[0])?, td(ts[1])?))
    })
}

fn r_1er_mai() -> Rule {
    rule("1er mai", vec![regex("f(e|ê)te du travail")], |_| tt(month_day(5, 1)))
}

fn r_premiere_quinzaine_de_namedmonth() -> Rule {
    rule(
        "première quinzaine de <named-month>(interval)",
        vec![regex("(premi(è|e)re|1 ?(è|e)re) (quinzaine|15 ?aine) d[e']"), pred(is_a_month)],
        |ts| {
            let t = td(ts[1])?;
            let from = intersect(&day_of_month(1), t)?;
            let to = intersect(&day_of_month(14), t)?;
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn r_de_datetime_datetime_interval() -> Rule {
    rule(
        "de <datetime> - <datetime> (interval)",
        vec![regex("de|depuis|du"), dimension(Dim::Time), regex("\\-|au|jusqu'(au?|à)"), dimension(Dim::Time)],
        |ts| {
            let (a, b) = (td(ts[1])?, td(ts[3])?);
            if same_grain(a, b) {
                ttd(interval(Closed, a, b))
            } else {
                None
            }
        },
    )
}

fn r_avant_hier() -> Rule {
    rule("avant-hier", vec![regex("avant[- ]?hier")], |_| tt(cycle_nth(G::Day, -2)))
}

fn r_cycle_prochain_suivant_dapres() -> Rule {
    rule(
        "<cycle> prochain|suivant|d'après",
        vec![dimension(Dim::TimeGrain), regex("prochaine?|suivante?|qui suit|(d')? ?apr(e|è|é)s")],
        |ts| tt(cycle_nth(grain_of(ts[0])?, 1)),
    )
}

fn r_apres_le_travail() -> Rule {
    today_interval_rule("après le travail", "apr(e|è)s (le )?travail", 17, 21)
}

fn r_le_dayofmonth_datetime() -> Rule {
    rule(
        "le <day-of-month> à <datetime>",
        vec![regex("le"), pred(is_dom_integer), regex("(a|à)"), pred(is_grain_finer_than(G::Day))],
        |ts| {
            let n = get_int_value(ts[1])?;
            ttd(intersect(&day_of_month(n), td(ts[3])?))
        },
    )
}

fn r_weekend() -> Rule {
    rule("week-end", vec![regex("week(\\s|-)?end")], |_| tt(weekend()))
}

fn r_ce_dans_le_cycle() -> Rule {
    rule(
        "ce|dans le <cycle>",
        vec![regex("(cet?t?e?s?)|(dans l[ae']? ?)"), dimension(Dim::TimeGrain)],
        |ts| tt(cycle_nth(grain_of(ts[1])?, 0)),
    )
}

fn r_dans_duration() -> Rule {
    rule("dans <duration>", vec![regex("dans"), dimension(Dim::Duration)], |ts| {
        tt(in_duration(dur(ts[1])?))
    })
}

fn r_ordinal_cycle_de_time() -> Rule {
    rule(
        "<ordinal> <cycle> de <time>",
        vec![dimension(Dim::Ordinal), dimension(Dim::TimeGrain), regex("d['eu]|en"), dimension(Dim::Time)],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth_after(true, grain_of(ts[1])?, n - 1, td(ts[3])?))
        },
    )
}

fn r_avant_le_dejeuner() -> Rule {
    today_interval_rule("avant le déjeuner", "avant (le )?d(e|é|è)jeuner", 10, 12)
}

fn r_dernier_weekend_de_time() -> Rule {
    rule(
        "dernier week-end de <time>",
        vec![regex("dernier week(\\s|-)?end (d['eu]|en|du mois de)"), pred(is_a_month)],
        |ts| tt(pred_last_of(&weekend(), td(ts[1])?)),
    )
}

fn r_le_cycle_prochain_suivant_dapres() -> Rule {
    rule(
        "le <cycle> prochain|suivant|d'après",
        vec![
            regex("l[ae']? ?|une? ?"),
            dimension(Dim::TimeGrain),
            regex("prochaine?|suivante?|qui suit|(d'? ?)?apr(e|è|é)s"),
        ],
        |ts| tt(cycle_nth(grain_of(ts[1])?, 1)),
    )
}

fn r_dim_time_partofday() -> Rule {
    rule(
        "<dim time> <part-of-day>",
        vec![pred(is_grain_of_time(G::Day)), pred(is_a_part_of_day)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[1])?)),
    )
}

fn r_en_namedmonth() -> Rule {
    rule("en <named-month>", vec![regex("en|au mois de?'?"), pred(is_a_month)], |ts| {
        Some(ts[1].clone())
    })
}

fn r_entre_timeofday_et_timeofday_interval() -> Rule {
    rule(
        "entre <time-of-day> et <time-of-day> (interval)",
        vec![regex("entre"), pred(is_a_time_of_day), regex("et"), pred(is_a_time_of_day)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    )
}

fn r_intersect_by_de_or() -> Rule {
    rule(
        "intersect by 'de' or ','",
        vec![pred(is_not_latent), regex("de|,"), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn r_deuxieme_quinzaine_de_namedmonth() -> Rule {
    rule(
        "deuxième quinzaine de <named-month>(interval)",
        vec![regex("(deuxi(è|e)me|2 ?(è|e)me) (quinzaine|15 ?aine) d[e']"), pred(is_a_month)],
        |ts| {
            let t = td(ts[1])?;
            let from = intersect(&day_of_month(15), t)?;
            let to = cycle_last_of(G::Day, t);
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn r_partofday_du_dim_time() -> Rule {
    rule(
        "<part-of-day> du <dim time>",
        vec![pred(is_a_part_of_day), regex("du"), dimension(Dim::Time)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn r_yyyymmdd() -> Rule {
    rule(
        "yyyy-mm-dd",
        vec![regex("(\\d{2,4})-(1[0-2]|0?[1-9])-(3[01]|[12]\\d|0?[1-9])")],
        |ts| {
            let y = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let d = parse_int(&rx(ts[0], 2)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn r_timeofday_heures() -> Rule {
    rule("<time-of-day> heures", vec![pred(is_a_time_of_day), regex("h\\.?(eure)?s?")], |ts| {
        tt(not_latent(td(ts[0])?.clone()))
    })
}

fn r_dernier_dayofweek_de_time_latent() -> Rule {
    rule(
        "dernier <day-of-week> de <time> (latent)",
        vec![regex("derni(e|é|è)re?"), pred(is_a_day_of_week), regex("d['e]"), dimension(Dim::Time)],
        |ts| tt(pred_last_of(td(ts[1])?, td(ts[3])?)),
    )
}

fn r_ce_dayofweek() -> Rule {
    rule("ce <day-of-week>", vec![regex("ce"), pred(is_a_day_of_week)], |ts| {
        tt(pred_nth(0, true, td(ts[1])?))
    })
}

fn r_matin() -> Rule {
    rule("matin", vec![regex("mat(in(é|e)?e?)?")], |_| {
        ttd(interval(Open, &hour(false, 4), &hour(false, 12)).map(|x| mk_latent(part_of_day(x))))
    })
}

fn r_year_latent() -> Rule {
    rule("year (latent)", vec![pred(is_integer_between_tok(1, 999))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(mk_latent(year(n)))
    })
}

fn r_season2() -> Rule {
    rule("season", vec![regex("(cet )?automne")], |_| {
        ttd(interval(Open, &month_day(9, 23), &month_day(12, 21)))
    })
}

fn r_vers_timeofday() -> Rule {
    rule(
        "à|vers <time-of-day>",
        vec![
            regex("(vers|autour de|(a|à) environ|aux alentours de|(a|à))"),
            pred(|t| is_a_time_of_day(t) && is_not_latent(t)),
        ],
        |ts| Some(ts[1].clone()),
    )
}

fn r_fin_namedmonthinterval() -> Rule {
    rule(
        "fin <named-month>(interval)",
        vec![regex("fin( du mois d[e']? ?)?"), pred(is_a_month)],
        |ts| {
            let t = td(ts[1])?;
            let from = intersect(&day_of_month(21), t)?;
            let to = cycle_last_of(G::Day, t);
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn r_dayofweek_dayofmonth() -> Rule {
    rule(
        "<day-of-week> <day-of-month>",
        vec![pred(is_a_day_of_week), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    )
}

fn r_nameddy_en_quinze() -> Rule {
    rule("<named-day> en quinze", vec![pred(is_a_day_of_week), regex("en (quinze|15)")], |ts| {
        tt(pred_nth(2, false, td(ts[0])?))
    })
}

fn r_le_dayofmonth_non_ordinal() -> Rule {
    rule("le <day-of-month> (non ordinal)", vec![regex("le"), pred(is_dom_integer)], |ts| {
        let n = get_int_value(ts[1])?;
        tt(day_of_month(n))
    })
}

fn r_intersect_by_mais_par_exemple_plutot() -> Rule {
    rule(
        "intersect by 'mais/par exemple/plutôt'",
        vec![pred(is_not_latent), regex("mais|par exemple|plut(ô|o)t"), pred(is_not_latent)],
        |ts| ttd(intersect(td(ts[0])?, td(ts[2])?)),
    )
}

fn r_le_cycle_apres_suivant_time() -> Rule {
    rule(
        "le <cycle> après|suivant <time>",
        vec![regex("l[ea']?"), dimension(Dim::TimeGrain), regex("suivante?|apr(e|è|é)s"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, 1, td(ts[3])?)),
    )
}

fn r_apres_demain() -> Rule {
    rule("après-demain", vec![regex("apr(e|è)s[- ]?demain")], |_| tt(cycle_nth(G::Day, 2)))
}

fn r_il_y_a_duration() -> Rule {
    rule("il y a <duration>", vec![regex("il y a"), dimension(Dim::Duration)], |ts| {
        tt(duration_ago(dur(ts[1])?))
    })
}

fn r_du_dans_le_partofday() -> Rule {
    rule(
        "du|dans le <part-of-day>",
        vec![regex("du|dans l[ae']? ?|au|en|l[ae' ]|d(è|e)s l?[ae']?"), pred(is_a_part_of_day)],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    )
}

fn r_au_dejeuner() -> Rule {
    rule(
        "au déjeuner",
        vec![regex("((à|a) l'heure du|au moment de|pendant( le)?|au)? d(e|é|è)jeuner")],
        |_| ttd(interval(Open, &hour(false, 12), &hour(false, 14)).map(part_of_day)),
    )
}

fn r_year() -> Rule {
    rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(year(n))
    })
}

fn r_maintenant() -> Rule {
    rule("maintenant", vec![regex("maintenant|tout de suite")], |_| tt(now()))
}

fn r_ddmmyyyy() -> Rule {
    rule(
        "dd/-mm/-yyyy",
        vec![regex("(3[01]|[12]\\d|0?[1-9])[/-](1[0-2]|0?[1-9])[-/](\\d{2,4})")],
        |ts| {
            let y = parse_int(&rx(ts[0], 2)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let d = parse_int(&rx(ts[0], 0)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn r_le_cycle_avant_precedent_time() -> Rule {
    rule(
        "le <cycle> avant|précédent <time>",
        vec![regex("l[ea']?"), dimension(Dim::TimeGrain), regex("avant|pr(é|e)c(é|e)dent"), dimension(Dim::Time)],
        |ts| tt(cycle_nth_after(false, grain_of(ts[1])?, -1, td(ts[3])?)),
    )
}

fn r_day_of_month_premier() -> Rule {
    rule("day of month (premier)", vec![regex("premier|prem\\.?|1 ?er")], |_| tt(day_of_month(1)))
}

fn r_timeofday_timeofday_interval() -> Rule {
    rule(
        "<time-of-day> - <time-of-day> (interval)",
        vec![pred(is_a_time_of_day), regex("\\-|(jusqu')?(au?|à)"), pred(is_a_time_of_day)],
        |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
    )
}

fn r_le_veille_du_time() -> Rule {
    rule(
        "le veille du <time>",
        vec![regex("(la )?veille du"), pred(is_grain_coarser_than(G::Hour))],
        |ts| tt(cycle_nth_after(true, G::Day, -1, td(ts[1])?)),
    )
}

fn r_le_time() -> Rule {
    rule("le <time>", vec![regex("l[ea]"), pred(is_not_latent)], |ts| Some(ts[1].clone()))
}

fn r_soir_de_noel() -> Rule {
    rule("soir de noël", vec![regex("soir(ée)? de no(e|ë)l")], |_| {
        let from = intersect(&month_day(12, 24), &hour(false, 18))?;
        let to = intersect(&month_day(12, 25), &hour(false, 0))?;
        ttd(interval(Open, &from, &to))
    })
}

fn r_dayofweek_dayofmonth_timeofday() -> Rule {
    rule(
        "<day-of-week> <day-of-month> à <time-of-day>)",
        vec![pred(is_a_day_of_week), pred(is_dom_integer), pred(is_a_time_of_day)],
        |ts| {
            let dom = intersect_dom(td(ts[0])?, ts[1])?;
            ttd(intersect(&dom, td(ts[2])?))
        },
    )
}

fn r_jour_de_lan() -> Rule {
    rule("jour de l'an", vec![regex("(jour de l'|nouvel )an")], |_| tt(month_day(1, 1)))
}

fn r_minuit() -> Rule {
    rule("minuit", vec![regex("minuit")], |_| tt(hour(false, 0)))
}

fn r_n_cycle_suivants() -> Rule {
    rule(
        "n <cycle> suivants",
        vec![
            pred(is_integer_between_tok(2, 9999)),
            dimension(Dim::TimeGrain),
            regex("prochaine?s?|suivante?s?|apr(e|è|é)s|qui sui(t|ves?)|plus tard"),
        ],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_n(true, grain_of(ts[1])?, n))
        },
    )
}

fn r_mid_month() -> Rule {
    rule("<named-month>", vec![regex("mi[- ]"), pred(is_a_month)], |ts| {
        let t = td(ts[1])?;
        let from = intersect(&day_of_month(10), t)?;
        let to = intersect(&day_of_month(19), t)?;
        ttd(interval(Open, &from, &to))
    })
}

fn r_dayofweek_er_dayofweek_dd_monthinterval() -> Rule {
    rule(
        "<day-of-week> 1er-<day-of-week> dd <month>(interval)",
        vec![
            pred(is_a_day_of_week),
            regex("premier|prem\\.?|1er|1 er"),
            regex("\\-|(jusqu')?au"),
            pred(is_a_day_of_week),
            regex("(3[01]|[12]\\d|0?[1-9])"),
            pred(is_a_month),
        ],
        |ts| {
            let n = parse_int(&rx(ts[4], 0)?)?;
            let t = td(ts[5])?;
            let from = intersect(&day_of_month(1), t)?;
            let to = intersect(&day_of_month(n), t)?;
            ttd(interval(Closed, &from, &to))
        },
    )
}

fn r_fin_du_mois() -> Rule {
    rule("fin du mois", vec![regex("(en |((à|a) la ))?fin (du|de) (ce )?mois")], |_| {
        ttd(interval(Open, &day_of_month(21), &day_of_month(0)))
    })
}

fn r_debut_du_mois() -> Rule {
    rule("début du mois", vec![regex("(en |au )?d(é|e)but (du|de) (ce )?mois")], |_| {
        ttd(interval(Open, &day_of_month(1), &day_of_month(10)))
    })
}

fn r_fin_dannee() -> Rule {
    rule("fin d'année", vec![regex("(en |(à|a) la )?fin (d'|de l'|de cette )ann(é|e)e")], |_| {
        ttd(interval(Open, &month(11), &month(1)))
    })
}

fn r_debut_dannee() -> Rule {
    rule("début d'année", vec![regex("(en |au )?d(é|e)but (d'|de l'|de cette )ann(é|e)e")], |_| {
        ttd(interval(Open, &month(1), &month(3)))
    })
}

fn r_plus_tard() -> Rule {
    rule("plus tard", vec![regex("(un peu )?plus tard")], |_| {
        tt(with_direction(IntervalDirection::After, cycle_nth(G::Minute, 10)))
    })
}

fn r_plus_tard_partofday() -> Rule {
    rule("plus tard <part-of-day>", vec![regex("(un peu )?plus tard"), pred(is_a_part_of_day)], |ts| {
        tt(not_latent(td(ts[1])?.clone()))
    })
}

fn r_dd_month_yy() -> Rule {
    rule(
        "DD month YY",
        vec![pred(is_integer_between_tok(0, 31)), pred(is_a_month), pred(is_integer_between_tok(32, 99))],
        |ts| {
            let d = get_int_value(ts[0])?;
            let y = get_int_value(ts[2])?;
            let day_month = intersect(td(ts[1])?, &day_of_month(d))?;
            ttd(intersect(&day_month, &year(y)))
        },
    )
}

fn r_timezone() -> Rule {
    rule(
        "<time> timezone",
        vec![
            pred(|t| is_not_latent(t) && is_a_time_of_day(t)),
            regex("\\b(YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT)\\b"),
        ],
        |ts| ttd(in_timezone(&rx(ts[1], 0)?.to_uppercase(), td(ts[0])?)),
    )
}

fn rules_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("Janvier", "janvier|janv\\.?" ),
        ("Fevrier", "f(é|e)vrier|f(é|e)v\\.?" ),
        ("Mars", "mars|mar\\.?" ),
        ("Avril", "avril|avr\\.?" ),
        ("Mai", "mai" ),
        ("Juin", "juin|jun\\.?" ),
        ("Juillet", "juillet|juil?\\.?" ),
        ("Aout", "ao(û|u)t|aou\\.?" ),
        ("Septembre", "septembre|sept?\\.?" ),
        ("Octobre", "octobre|oct\\.?" ),
        ("Novembre", "novembre|nov\\.?" ),
        ("Decembre", "d(é|e)cembre|d(é|e)c\\.?" ),
    ])
}

fn rules_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Lundi", "lun\\.?(di)?" ),
        ("Mardi", "mar\\.?(di)?" ),
        ("Mercredi", "mer\\.?(credi)?" ),
        ("Jeudi", "jeu\\.?(di)?" ),
        ("Vendredi", "ven\\.?(dredi)?" ),
        ("Samedi", "sam\\.?(edi)?" ),
        ("Dimanche", "dim\\.?(anche)?" ),
    ])
}

fn build_rules() -> Vec<Rule> {
    let mut v = vec![
        r_apres_le_dayofmonth(),
        r_apres_le_dejeuner(),
        r_apres_le_travail(),
        r_apres_timeofday(),
        r_apres_demain(),
        r_apresmidi(),
        r_au_dejeuner(),
        r_aujourdhui(),
        r_avant_le_dejeuner(),
        r_avant_timeofday(),
        r_avant_hier(),
        r_ce_dayofweek(),
        r_ce_partofday(),
        r_ce_time(),
        r_ce_dans_le_cycle(),
        r_cycle_dernier(),
        r_cycle_prochain_suivant_dapres(),
        r_dans_duration(),
        r_datetime_datetime_interval(),
        r_datetimedayofweek_dd_monthinterval(),
        r_datetime_dd_monthinterval(),
        r_day_of_month_premier(),
        r_dayofmonth_namedmonth(),
        r_dayofweek_dayofmonth(),
        r_dayofweek_dayofmonth_timeofday(),
        r_mid_month(),
        r_dayofweek_er_dayofweek_dd_monthinterval(),
        r_dayofweek_prochain(),
        r_debut_apres_midi(),
        r_debut_de_journee(),
        r_debut_de_matinee(),
        r_debut_de_semaine(),
        r_debut_namedmonthinterval(),
        r_dd_mm(),
        r_dd_mm_yyyy(),
        r_dddd_monthinterval(),
        r_ddmm(),
        r_ddmmyyyy(),
        r_de_datetime_datetime_interval(),
        r_de_timeofday_timeofday_interval(),
        r_demain(),
        r_dernier_cycle_de_time_latent(),
        r_dernier_dayofweek_de_time_latent(),
        r_dernier_weekend_de_time(),
        r_deuxieme_quinzaine_de_namedmonth(),
        r_dici_duration(),
        r_dim_time_du_matin(),
        r_dim_time_du_soir(),
        r_dim_time_partofday(),
        r_du_datetimedayofweek_dd_monthinterval(),
        r_du_dd_au_dd_latent_month_interval(),
        r_du_dd_au_dd_month_interval(),
        r_du_ddd_dayofweek_dd_monthinterval(),
        r_du_dans_le_partofday(),
        r_duration_apres_time(),
        r_duration_avant_time(),
        r_en_namedmonth(),
        r_en_semaine(),
        r_entre_datetime_et_datetime_interval(),
        r_entre_dd_et_dd_monthinterval(),
        r_entre_timeofday_et_timeofday_interval(),
        r_1er_mai(),
        r_milieu_apres_midi(),
        r_fin_apres_midi(),
        r_milieu_de_journee(),
        r_fin_de_journee(),
        r_milieu_de_matinee(),
        r_fin_de_matinee(),
        r_fin_de_semaine(),
        r_fin_namedmonthinterval(),
        r_hhhmm_timeofday(),
        r_hhmm_military_timeofday(),
        r_hier(),
        r_il_y_a_duration(),
        r_intersect(),
        r_intersect_by_de_or(),
        r_intersect_by_mais_par_exemple_plutot(),
        r_jour_de_lan(),
        r_le_cycle_apres_suivant_time(),
        r_le_cycle_avant_precedent_time(),
        r_le_cycle_de_time(),
        r_le_cycle_dernier(),
        r_le_cycle_prochain_suivant_dapres(),
        r_le_dayofmonth_datetime(),
        r_le_dayofmonth_non_ordinal(),
        r_lendemain_du_time(),
        r_le_ordinal_cycle_de_time(),
        r_le_time(),
        r_le_veille_du_time(),
        r_maintenant(),
        r_matin(),
        r_midi(),
        r_milieu_de_semaine(),
        r_minuit(),
        r_n_cycle_apres(),
        r_n_cycle_avant(),
        r_n_cycle_passes_precedents(),
        r_n_cycle_suivants(),
        r_n_derniers_cycle(),
        r_n_prochains_cycle(),
        r_nameddy_en_huit(),
        r_nameddy_en_quinze(),
        r_namedmonth_prochain(),
        r_namedmonthnamedday_dernierpass(),
        r_namedmonthnamedday_suivant_dapres(),
        r_noel(),
        r_ordinal_cycle_de_time(),
        r_ordinal_weekend_de_time(),
        r_partofday_du_dim_time(),
        r_premiere_quinzaine_de_namedmonth(),
        r_season(),
        r_season2(),
        r_season3(),
        r_season4(),
        r_soir(),
        r_debut_de_soiree(),
        r_fin_de_soiree(),
        r_soir_de_noel(),
        r_timeofday_heures(),
        r_timeofday_latent(),
        r_timeofday_timeofday_interval(),
        r_toussaint(),
        r_vers_timeofday(),
        r_weekend(),
        r_year(),
        r_year_latent(),
        r_year_latent2(),
        r_yyyymmdd(),
        r_hourofday_moins_quart(),
        r_hourofday_moins_integer(),
        r_hourofday_integer_minutes(),
        r_hourofday_integer(),
        r_hourofday_etpass_de_numeral(),
        r_hourofday_etpass_de_numeral_minutes(),
        r_hourofday_et_trois_quart(),
        r_hourofday_et_quart(),
        r_hourofday_et_demi(),
        r_fin_du_mois(),
        r_timezone(),
        r_debut_du_mois(),
        r_fin_dannee(),
        r_debut_dannee(),
        r_plus_tard(),
        r_plus_tard_partofday(),
        r_nth_time_of_time(),
        r_the_nth_time_of_time(),
        r_dd_month_yy(),
    ];
    v.extend(rules_months());
    v.extend(rules_days_of_week());
    v
}

static FR_RULES: Lazy<Vec<Rule>> = Lazy::new(build_rules);

pub fn rules() -> &'static [Rule] {
    &FR_RULES
}
