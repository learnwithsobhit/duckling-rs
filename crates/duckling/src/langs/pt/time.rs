//! `Duckling.Time.PT.Rules`.

use once_cell::sync::Lazy;

use crate::dims::time::helpers::*;
use crate::dims::time::types::{
    Form, IntervalDirection, TimeData,
    TimeIntervalType::{Closed, Open},
};
use crate::dims::time_grain::Grain as G;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{dimension, pred, regex, rule, Dim, PatternItem, Rule, Token};

const TIMEZONE_NAME: &str = "YEKT|YEKST|YAKT|YAKST|WITA|WIT|WIB|WGT|WGST|WFT|WET|WEST|WAT|WAST|VUT|VLAT|VLAST|VET|UZT|UYT|UYST|UTC|ULAT|TVT|TMT|TLT|TKT|TJT|TFT|TAHT|SST|SRT|SGT|SCT|SBT|SAST|SAMT|RET|PYT|PYST|PWT|PST|PONT|PMST|PMDT|PKT|PHT|PHOT|PGT|PETT|PETST|PET|PDT|OMST|OMSST|NZST|NZDT|NUT|NST|NPT|NOVT|NOVST|NFT|NDT|NCT|MYT|MVT|MUT|MST|MSK|MSD|MMT|MHT|MDT|MAWT|MART|MAGT|MAGST|LINT|LHST|LHDT|KUYT|KST|KRAT|KRAST|KGT|JST|IST|IRST|IRKT|IRKST|IRDT|IOT|IDT|ICT|HOVT|HKT|GYT|GST|GMT|GILT|GFT|GET|GAMT|GALT|FNT|FKT|FKST|FJT|FJST|EST|EGT|EGST|EET|EEST|EDT|ECT|EAT|EAST|EASST|DAVT|ChST|CXT|CVT|CST|COT|CLT|CLST|CKT|CHAST|CHADT|CET|CEST|CDT|CCT|CAT|CAST|BTT|BST|BRT|BRST|BOT|BNT|AZT|AZST|AZOT|AZOST|AWST|AWDT|AST|ART|AQTT|ANAT|ANAST|AMT|AMST|ALMT|AKST|AKDT|AFT|AEST|AEDT|ADT|ACST|ACDT";

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

fn grain_of(t: &Token) -> Option<G> {
    match t {
        Token::TimeGrain(g) => Some(*g),
        _ => None,
    }
}

fn dur(t: &Token) -> Option<&crate::dims::duration::DurationData> {
    match t {
        Token::Duration(d) => Some(d),
        _ => None,
    }
}

fn is_grain(value: G) -> impl Fn(&Token) -> bool + Send + Sync + 'static {
    move |t| matches!(t, Token::TimeGrain(g) if *g == value)
}

/// `(hours, is12H)` of a time token whose form is `TimeOfDay (Just hours) is12H`.
fn hours_form(t: &Token) -> Option<(i64, bool)> {
    match t {
        Token::Time(TimeData { form: Some(Form::TimeOfDay { hours: Some(h), is12h }), .. }) => {
            Some((*h, *is12h))
        }
        _ => None,
    }
}

fn not_latent_and_hour_of_day(t: &Token) -> bool {
    is_not_latent(t) && is_an_hour_of_day(t)
}

/// `Token Time . mkLatent . partOfDay <$> interval Open from to`.
fn latent_part_of_day(from: i64, to: i64) -> Option<Token> {
    ttd(interval(Open, &hour(false, from), &hour(false, to)).map(|t| mk_latent(part_of_day(t))))
}

/// `intersect td1 td2` of two time tokens at positions `i`, `j`.
fn isect(ts: &[&Token], i: usize, j: usize) -> Option<Token> {
    ttd(intersect(td(ts[i])?, td(ts[j])?))
}

fn dim_time() -> PatternItem {
    dimension(Dim::Time)
}

fn dim_grain() -> PatternItem {
    dimension(Dim::TimeGrain)
}

fn dim_duration() -> PatternItem {
    dimension(Dim::Duration)
}

fn dim_ordinal() -> PatternItem {
    dimension(Dim::Ordinal)
}

/// `<hour-of-day> <minutes>` with fixed minutes: form is `TimeOfDay (Just hours) is12H`.
fn hm_fixed(name: &str, pattern: Vec<PatternItem>, minutes: i64) -> Rule {
    rule(name, pattern, move |ts| {
        let (h, is12h) = hours_form(ts[0])?;
        tt(hour_minute(is12h, h, minutes))
    })
}

/// `<minutes> para as <hour-of-day>`-style rules where the hour token is at `idx`.
fn minutes_before_fixed(name: &str, pattern: Vec<PatternItem>, n: i64, idx: usize) -> Rule {
    rule(name, pattern, move |ts| ttd(minutes_before(n, td(ts[idx])?)))
}

fn rule_s_hourmintimeofday() -> Rule {
    rule(
        "às <hour-min>(time-of-day)",
        vec![regex("(à|a)s?"), pred(is_a_time_of_day), regex("horas?")],
        |ts| Some(ts[1].clone()),
    )
}

fn rule_months() -> Vec<Rule> {
    mk_rule_months(&[
        ("Janeiro", "janeiro|jan\\.?"),
        ("Fevereiro", "fevereiro|fev\\.?"),
        ("Março", "mar(ç|c)o|mar\\.?"),
        ("Abril", "abril|abr\\.?"),
        ("Maio", "maio|mai\\.?"),
        ("Junho", "junho|jun\\.?"),
        ("Julho", "julho|jul\\.?"),
        ("Agosto", "agosto|ago\\.?"),
        ("Setembro", "setembro|set\\.?"),
        ("Outubro", "outubro|out\\.?"),
        ("Novembro", "novembro|nov\\.?"),
        ("Dezembro", "dezembro|dez\\.?"),
    ])
}

fn rule_days_of_week() -> Vec<Rule> {
    mk_rule_days_of_week(&[
        ("Segunda-feira", "segunda((\\s|\\-)feira)?|seg\\.?"),
        ("Terça-feira", "ter(ç|c)a((\\s|\\-)feira)?|ter\\."),
        ("Quart-feira", "quarta((\\s|\\-)feira)?|qua\\.?"),
        ("Quinta-feira", "quinta((\\s|\\-)feira)?|qui\\.?"),
        ("Sexta-feira", "sexta((\\s|\\-)feira)?|sex\\.?"),
        ("Sábada", "s(á|a)bado|s(á|a)b\\.?"),
        ("Domingo", "domingo|dom\\.?"),
    ])
}

fn all_rules() -> Vec<Rule> {
    let mut v: Vec<Rule> = Vec::new();

    // ruleAfternoon
    v.push(rule("afternoon", vec![regex("tarde")], |_| latent_part_of_day(12, 19)));
    // ruleAmanhPelaPartofday
    v.push(rule(
        "amanhã pela <part-of-day>",
        vec![dim_time(), regex("(da|na|pela|a)"), pred(is_a_part_of_day)],
        |ts| isect(ts, 0, 2),
    ));
    // ruleAnoNovo
    v.push(rule("ano novo", vec![regex("ano novo|reveillon")], |_| tt(month_day(1, 1))));
    // ruleAntesDasTimeofday
    v.push(rule(
        "antes das <time-of-day>",
        vec![regex("(antes|at(e|é)|n(a|ã)o mais que) (d?(o|a|à)s?)?"), dim_time()],
        |ts| tt(with_direction(IntervalDirection::Before, td(ts[1])?.clone())),
    ));
    // ruleDatetimeDatetimeInterval
    v.push(rule(
        "<datetime> - <datetime> (interval)",
        vec![pred(is_not_latent), regex("\\-|até( ao?)?|ao?"), pred(is_not_latent)],
        |ts| ttd(interval(Closed, td(ts[0])?, td(ts[2])?)),
    ));
    // ruleDayOfMonthSt
    v.push(rule("day of month (1st)", vec![regex("primeiro|um|1o")], |_| tt(mk_latent(day_of_month(1)))));
    // ruleDayofweekSHourmin
    v.push(rule(
        "<day-of-week> às <hour-min>",
        vec![
            pred(is_a_time_of_day),
            regex("(à|a)s"),
            pred(is_not_latent),
            regex("da|pela"),
            pred(is_a_time_of_day),
        ],
        |ts| {
            let t = intersect(td(ts[0])?, td(ts[2])?)?;
            ttd(intersect(&t, td(ts[4])?))
        },
    ));
    // ruleDdddMonthinterval
    v.push(rule(
        "dd-dd <month>(interval)",
        vec![
            regex("(3[01]|[12]\\d|0?[1-9])"),
            regex("\\-|a"),
            regex("(3[01]|[12]\\d|0?[1-9])"),
            regex("de"),
            pred(is_a_month),
        ],
        |ts| {
            let d1 = parse_int(&rx(ts[0], 0)?)?;
            let d2 = parse_int(&rx(ts[2], 0)?)?;
            let t = td(ts[4])?;
            let from = intersect(&day_of_month(d1), t)?;
            let to = intersect(&day_of_month(d2), t)?;
            ttd(interval(Closed, &from, &to))
        },
    ));
    // ruleDdmm
    v.push(rule(
        "dd[/-]mm",
        vec![regex("(3[01]|[12]\\d|0?[1-9])[\\/\\-](0?[1-9]|1[0-2])")],
        |ts| {
            let m = parse_int(&rx(ts[0], 1)?)?;
            let d = parse_int(&rx(ts[0], 0)?)?;
            tt(month_day(m, d))
        },
    ));
    // ruleDdmmyyyy
    v.push(rule(
        "dd[/-.]mm[/-.]yyyy",
        vec![regex("(3[01]|[12]\\d|0?[1-9])[\\.\\/\\-](0?[1-9]|1[0-2])[\\.\\/\\-](\\d{2,4})")],
        |ts| {
            let y = parse_int(&rx(ts[0], 2)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let d = parse_int(&rx(ts[0], 0)?)?;
            tt(year_month_day(y, m, d))
        },
    ));
    // ruleDeDatetimeDatetimeInterval
    v.push(rule(
        "de <datetime> - <datetime> (interval)",
        vec![regex("de?"), pred(is_a_time_of_day), regex("\\-|até( ao?)?|ao?"), pred(is_a_time_of_day)],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    ));
    // ruleDeYear
    v.push(rule(
        "de <year>",
        vec![regex("de|do ano"), pred(is_integer_between_tok(1000, 2100))],
        |ts| {
            let n = get_int_value(ts[1])?;
            tt(year(n))
        },
    ));
    // ruleDentroDeDuration
    v.push(rule(
        "dentro de <duration>",
        vec![regex("(dentro de)|em"), dim_duration()],
        |ts| ttd(interval(Open, &now(), &in_duration(dur(ts[1])?))),
    ));
    // ruleDepoisDasTimeofday
    v.push(rule(
        "depois das <time-of-day>",
        vec![regex("(depois|ap(ó|o)s) d?((a|á|à)[so]?|os?)"), dim_time()],
        |ts| tt(with_direction(IntervalDirection::After, td(ts[1])?.clone())),
    ));
    // ruleDiaDayofmonthDeNamedmonth
    v.push(rule(
        "dia <day-of-month> de <named-month>",
        vec![regex("dia"), pred(is_dom_integer), regex("de|\\/"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[3])?, ts[1])),
    ));
    // ruleDiaDayofmonthNonOrdinal
    v.push(rule(
        "dia <day-of-month> (non ordinal)",
        vec![regex("dia"), pred(is_dom_integer)],
        |ts| {
            let n = get_int_value(ts[1])?;
            tt(day_of_month(n))
        },
    ));
    // ruleDiaDoTrabalhador
    v.push(rule("Dia do trabalhador", vec![regex("dia do trabalh(o|ador)")], |_| tt(month_day(5, 1))));
    // ruleDimTimeDaMadrugada
    v.push(rule(
        "<dim time> da madrugada",
        vec![dim_time(), regex("(da|na|pela) madruga(da)?")],
        |ts| {
            let td2 = mk_latent(part_of_day(interval(Open, &hour(false, 1), &hour(false, 4))?));
            ttd(intersect(td(ts[0])?, &td2))
        },
    ));
    // ruleDimTimeDaManha
    v.push(rule(
        "<dim time> da manha",
        vec![pred(is_grain_finer_than(G::Year)), regex("(da|na|pela) manh(ã|a)")],
        |ts| {
            let td2 = mk_latent(part_of_day(interval(Open, &hour(false, 4), &hour(false, 12))?));
            ttd(intersect(td(ts[0])?, &td2))
        },
    ));
    // ruleDimTimeDaTarde
    v.push(rule(
        "<dim time> da tarde",
        vec![dim_time(), regex("(da|na|pela) tarde")],
        |ts| {
            let td2 = mk_latent(part_of_day(interval(Open, &hour(false, 12), &hour(false, 18))?));
            ttd(intersect(td(ts[0])?, &td2))
        },
    ));
    // ruleDOMOfMonth
    v.push(rule(
        "<day-of-month> (ordinal or number) de <named-month>",
        vec![pred(is_dom_value), regex("de|\\/"), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[2])?, ts[0])),
    ));
    // ruleDOMMonth
    v.push(rule(
        "<day-of-month> (ordinal or number) <named-month>",
        vec![pred(is_dom_value), pred(is_a_month)],
        |ts| ttd(intersect_dom(td(ts[1])?, ts[0])),
    ));
    // ruleEmDuration
    v.push(rule("em <duration>", vec![regex("em"), dim_duration()], |ts| tt(in_duration(dur(ts[1])?))));
    // ruleEntreDatetimeEDatetimeInterval
    v.push(rule(
        "entre <datetime> e <datetime> (interval)",
        vec![
            regex("entre( [ao])?|desde|(a partir )?d[eo]"),
            dim_time(),
            regex("e|\\-|até( ao?)?|ao?"),
            dim_time(),
        ],
        |ts| ttd(interval(Closed, td(ts[1])?, td(ts[3])?)),
    ));
    // ruleEntreDdEtDdMonthinterval
    v.push(rule(
        "entre dd et dd <month>(interval)",
        vec![
            regex("entre"),
            regex("(0?[1-9]|[12]\\d|3[01])"),
            regex("e?"),
            regex("(0?[1-9]|[12]\\d|3[01])"),
            regex("de"),
            pred(is_a_month),
        ],
        |ts| {
            let dd1 = parse_int(&rx(ts[1], 0)?)?;
            let dd2 = parse_int(&rx(ts[3], 0)?)?;
            let t = td(ts[5])?;
            let dom1 = intersect(&day_of_month(dd1), t)?;
            let dom2 = intersect(&day_of_month(dd2), t)?;
            ttd(interval(Closed, &dom1, &dom2))
        },
    ));
    // ruleEsteCycle
    v.push(rule("este <cycle>", vec![regex("(n?es[st](es?|as?))"), dim_grain()], |ts| {
        tt(cycle_nth(grain_of(ts[1])?, 0))
    }));
    // ruleCycleActual
    v.push(rule("<cycle> actual", vec![dim_grain(), regex("ac?tual")], |ts| {
        tt(cycle_nth(grain_of(ts[0])?, 0))
    }));
    // ruleEvening
    v.push(rule("evening", vec![regex("noite")], |_| latent_part_of_day(18, 0)));
    // ruleFazemDuration
    v.push(rule("fazem <duration>", vec![regex("faz(em)?"), dim_duration()], |ts| {
        tt(duration_ago(dur(ts[1])?))
    }));
    // ruleFinados
    v.push(rule("Finados", vec![regex("finados|dia dos mortos")], |_| tt(month_day(11, 2))));
    // ruleHhhmmTimeofday
    v.push(rule(
        "hh(:|.|h)mm (time-of-day)",
        vec![regex("((?:[01]?\\d)|(?:2[0-3]))[:h\\.]([0-5]\\d)")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(hour_minute(true, h, m))
        },
    ));
    // ruleHhmmMilitaryTimeofday
    v.push(rule(
        "hhmm (military time-of-day)",
        vec![regex("((?:[01]?\\d)|(?:2[0-3]))([0-5]\\d)")],
        |ts| {
            let h = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(mk_latent(hour_minute(false, h, m)))
        },
    ));
    // ruleHourofdayAndRelativeMinutes
    v.push(rule(
        "<hour-of-day> and <relative minutes>",
        vec![pred(is_an_hour_of_day), regex("e"), pred(is_integer_between_tok(1, 59))],
        |ts| {
            let (h, is12h) = hours_form(ts[0])?;
            let n = get_int_value(ts[2])?;
            tt(hour_minute(is12h, h, n))
        },
    ));
    // ruleHourofdayAndRelativeMinutes2
    v.push(rule(
        "<hour-of-day> and <relative minutes> minutos",
        vec![
            pred(is_an_hour_of_day),
            regex("e"),
            pred(is_integer_between_tok(1, 59)),
            regex("min\\.?(uto)?s?"),
        ],
        |ts| {
            let (h, is12h) = hours_form(ts[0])?;
            let n = get_int_value(ts[2])?;
            tt(hour_minute(is12h, h, n))
        },
    ));
    // ruleHourofdayAndQuarter
    v.push(hm_fixed(
        "<hour-of-day> and quinze",
        vec![pred(is_an_hour_of_day), regex("e quinze")],
        15,
    ));
    // ruleHourofdayAndThreeQuarter
    v.push(hm_fixed(
        "<hour-of-day> and 3/4",
        vec![pred(is_an_hour_of_day), regex("e quarenta e cinco")],
        45,
    ));
    // ruleHourofdayAndHalf
    v.push(hm_fixed(
        "<hour-of-day> and half",
        vec![pred(is_an_hour_of_day), regex("e (meia|trinta)")],
        30,
    ));
    // ruleHourofdayIntegerAsRelativeMinutes
    v.push(rule(
        "<hour-of-day> <integer> (as relative minutes)",
        vec![pred(not_latent_and_hour_of_day), pred(is_integer_between_tok(1, 59))],
        |ts| {
            let (h, is12h) = hours_form(ts[0])?;
            let n = get_int_value(ts[1])?;
            tt(hour_minute(is12h, h, n))
        },
    ));
    // ruleHourofdayIntegerAsRelativeMinutes2
    v.push(rule(
        "<hour-of-day> <integer> (as relative minutes) minutos",
        vec![
            pred(not_latent_and_hour_of_day),
            pred(is_integer_between_tok(1, 59)),
            regex("min\\.?(uto)?s?"),
        ],
        |ts| {
            let (h, is12h) = hours_form(ts[0])?;
            let n = get_int_value(ts[1])?;
            tt(hour_minute(is12h, h, n))
        },
    ));
    // ruleHourofdayQuarter
    v.push(hm_fixed(
        "<hour-of-day> quarter (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("quinze")],
        15,
    ));
    // ruleHourofdayHalf
    v.push(hm_fixed(
        "<hour-of-day> half (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("meia|trinta")],
        30,
    ));
    // ruleHourofdayThreeQuarter
    v.push(hm_fixed(
        "<hour-of-day> 3/4 (as relative minutes)",
        vec![pred(is_an_hour_of_day), regex("quarenta e cinco")],
        45,
    ));
    // ruleInThePartofday
    v.push(rule(
        "in the <part-of-day>",
        vec![regex("(de|pela|a|à)"), pred(is_a_part_of_day)],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    ));
    // ruleIndependecia
    v.push(rule("Independecia", vec![regex("independ(ê|e)ncia")], |_| tt(month_day(9, 7))));
    // ruleIntegerParaAsHourofdayAsRelativeMinutes
    v.push(rule(
        "<integer> para as <hour-of-day> (as relative minutes)",
        vec![pred(is_integer_between_tok(1, 59)), regex("para ((o|a|à)s?)?"), pred(is_an_hour_of_day)],
        |ts| {
            let n = get_int_value(ts[0])?;
            ttd(minutes_before(n, td(ts[2])?))
        },
    ));
    // ruleIntegerParaAsHourofdayAsRelativeMinutes2
    v.push(rule(
        "<integer> minutos para as <hour-of-day> (as relative minutes)",
        vec![
            pred(is_integer_between_tok(1, 59)),
            regex("min\\.?(uto)?s?"),
            regex("para ((o|a|à)s?)?"),
            pred(is_an_hour_of_day),
        ],
        |ts| {
            let n = get_int_value(ts[0])?;
            ttd(minutes_before(n, td(ts[3])?))
        },
    ));
    // ruleHalfParaAsHourofdayAsRelativeMinutes
    v.push(minutes_before_fixed(
        "half para as <hour-of-day> (as relative minutes)",
        vec![regex("(meia|trinta) para ((o|a|à)s?)?"), pred(is_an_hour_of_day)],
        30,
        1,
    ));
    // ruleQuarterParaAsHourofdayAsRelativeMinutes
    v.push(minutes_before_fixed(
        "quinze para as <hour-of-day> (as relative minutes)",
        vec![regex("quinze para ((o|a|à)s?)?"), pred(is_an_hour_of_day)],
        15,
        1,
    ));
    // ruleThreeQuarterParaAsHourofdayAsRelativeMinutes
    v.push(minutes_before_fixed(
        "3/4 para as <hour-of-day> (as relative minutes)",
        vec![regex("quarenta e cinco para ((o|a|à)s?)?"), pred(is_an_hour_of_day)],
        45,
        1,
    ));
    // ruleIntersect
    v.push(rule(
        "intersect",
        vec![pred(is_not_latent), pred(is_not_latent)],
        |ts| isect(ts, 0, 1),
    ));
    // ruleIntersectByDaOrDe
    v.push(rule(
        "intersect by `da` or `de`",
        vec![dim_time(), regex("d[ae]"), pred(is_not_latent)],
        |ts| isect(ts, 0, 2),
    ));
    // ruleLastTime
    v.push(rule("last <time>", vec![regex("[úu]ltim[ao]s?"), dim_time()], |ts| {
        tt(pred_nth(-1, false, td(ts[1])?))
    }));
    // ruleMidnight
    v.push(rule("midnight", vec![regex("meia[\\s\\-]?noite")], |_| tt(hour(false, 0))));
    // ruleMonthDOM
    v.push(rule(
        "<named-month> <day-of-month>",
        vec![pred(is_a_month), pred(is_dom_integer)],
        |ts| ttd(intersect_dom(td(ts[0])?, ts[1])),
    ));
    // ruleMorning
    v.push(rule("morning", vec![regex("manh(ã|a)")], |_| latent_part_of_day(4, 12)));
    // ruleNCycleAtras
    v.push(rule(
        "n <cycle> atras",
        vec![pred(is_integer_between_tok(2, 9999)), dim_grain(), regex("atr(a|á)s")],
        |ts| {
            let v = get_int_value(ts[0])?;
            tt(cycle_n(true, grain_of(ts[1])?, -v))
        },
    ));
    // ruleNCycleProximoqueVem
    v.push(rule(
        "n <cycle> (proximo|que vem)",
        vec![
            pred(is_integer_between_tok(2, 9999)),
            dim_grain(),
            regex("(pr(ó|o)xim(o|a)s?|que vem?|seguintes?)"),
        ],
        |ts| {
            let v = get_int_value(ts[0])?;
            tt(cycle_n(true, grain_of(ts[1])?, v))
        },
    ));
    // ruleNPassadosCycle
    v.push(rule(
        "n passados <cycle>",
        vec![
            pred(is_integer_between_tok(2, 9999)),
            regex("(passad|[úu]ltim)[ao]s?|anterior(es)?"),
            dim_grain(),
        ],
        |ts| {
            let v = get_int_value(ts[0])?;
            tt(cycle_n(true, grain_of(ts[2])?, -v))
        },
    ));
    // ruleNProximasCycle
    v.push(rule(
        "n proximas <cycle>",
        vec![pred(is_integer_between_tok(2, 9999)), regex("pr(ó|o)xim(o|a)s?"), dim_grain()],
        |ts| {
            let v = get_int_value(ts[0])?;
            tt(cycle_n(true, grain_of(ts[2])?, v))
        },
    ));
    // ruleNamedmonthnameddayNext
    v.push(rule(
        "<named-month|named-day> next",
        vec![dim_time(), regex("(da pr(o|ó)xima semana)|(da semana)? que vem")],
        |ts| tt(pred_nth(1, false, td(ts[0])?)),
    ));
    // ruleNamedmonthnameddayPast
    v.push(rule(
        "<named-month|named-day> past",
        vec![pred(is_not_latent), regex("(da semana)? passad(o|a)")],
        |ts| tt(pred_nth(-1, false, td(ts[0])?)),
    ));
    // ruleNaoDate
    v.push(rule(
        "n[ao] <date>",
        vec![regex("n[ao]"), pred(is_grain_of_time(G::Day))],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    ));
    // ruleNatal
    v.push(rule("natal", vec![regex("natal")], |_| tt(month_day(12, 25))));
    // ruleNextTime
    v.push(rule(
        "next <time>",
        vec![regex("(d[ao]) pr(ó|o)xim[ao]s?|que vem"), pred(is_not_latent)],
        |ts| tt(pred_nth(0, true, td(ts[1])?)),
    ));
    // ruleCycleQueVem
    v.push(rule("<cycle> (que vem)", vec![dim_grain(), regex("que vem|seguintes?")], |ts| {
        tt(cycle_nth(grain_of(ts[0])?, 1))
    }));
    // ruleProximoCycle
    v.push(rule("proximo <cycle> ", vec![regex("pr(ó|o)xim(o|a)s?"), dim_grain()], |ts| {
        tt(cycle_nth(grain_of(ts[1])?, 1))
    }));
    // ruleNoon
    v.push(rule("noon", vec![regex("meio[\\s\\-]?dia")], |_| tt(hour(false, 12))));
    // ruleNossaSenhoraAparecida
    v.push(rule(
        "Nossa Senhora Aparecida",
        vec![regex("nossa senhora( aparecida)?")],
        |_| tt(month_day(10, 12)),
    ));
    // ruleNow
    v.push(rule("now", vec![regex("(hoje)|(neste|nesse) momento")], |_| tt(today())));
    // ruleCycleAntesDeTime
    v.push(rule(
        "<cycle> antes de <time>",
        vec![dim_grain(), regex("antes d[eo]"), dim_time()],
        |ts| tt(cycle_nth_after(false, grain_of(ts[0])?, -1, td(ts[2])?)),
    ));
    // ruleCyclePassado
    v.push(rule(
        "<cycle> passado",
        vec![dim_grain(), regex("passad[ao]s?|anterior(es)?")],
        |ts| tt(cycle_nth(grain_of(ts[0])?, -1)),
    ));
    // rulePassadoCycle
    v.push(rule(
        "passado <cycle>",
        vec![regex("(passad|[úu]ltim)[ao]s?|anterior(es)?"), dim_grain()],
        |ts| tt(cycle_nth(grain_of(ts[1])?, -1)),
    ));
    // rulePartofdayDessaSemana
    v.push(rule(
        "<part-of-day> dessa semana",
        vec![pred(is_not_latent), regex("(d?es[ts]a semana)|agora")],
        |ts| ttd(intersect(&today(), td(ts[0])?).map(part_of_day)),
    ));
    // rulePassadosNCycle
    v.push(rule(
        "passados n <cycle>",
        vec![
            regex("(passad|[úu]ltim)[ao]s?"),
            pred(is_integer_between_tok(2, 9999)),
            dim_grain(),
        ],
        |ts| {
            let v = get_int_value(ts[1])?;
            tt(cycle_n(true, grain_of(ts[2])?, -v))
        },
    ));
    // ruleNCyclePassados
    v.push(rule(
        "n <cycle> passados",
        vec![
            pred(is_integer_between_tok(2, 9999)),
            dim_grain(),
            regex("(passad|[úu]ltim)[ao]s?|anterior(es)?"),
        ],
        |ts| {
            let v = get_int_value(ts[0])?;
            tt(cycle_n(true, grain_of(ts[1])?, -v))
        },
    ));
    // ruleProclamaoDaRepblica
    v.push(rule(
        "Proclamação da República",
        vec![regex("proclama(c|ç)(a|ã)o da rep(ú|u)blica")],
        |_| tt(month_day(11, 15)),
    ));
    // ruleProximasNCycle
    v.push(rule(
        "proximas n <cycle>",
        vec![regex("pr(ó|o)xim(o|a)s?"), pred(is_integer_between_tok(2, 9999)), dim_grain()],
        |ts| {
            let v = get_int_value(ts[1])?;
            tt(cycle_n(true, grain_of(ts[2])?, v))
        },
    ));
    // ruleRightNow
    v.push(rule("right now", vec![regex("agora|j(á|a)|(nesse|neste) instante")], |_| tt(now())));
    // ruleSHourminTimeofday
    v.push(rule(
        "às <hour-min> <time-of-day>",
        vec![regex("[àa]s"), pred(is_not_latent), regex("da"), pred(is_a_time_of_day)],
        |ts| isect(ts, 1, 3),
    ));
    // ruleSHourmintimeofday
    v.push(rule_s_hourmintimeofday());
    // ruleSTimeofday
    v.push(rule(
        "às <time-of-day>",
        vec![regex("(à|a)s?"), pred(is_a_time_of_day)],
        |ts| tt(not_latent(td(ts[1])?.clone())),
    ));
    // ruleSeason
    v.push(rule("season", vec![regex("ver(ã|a)o")], |_| {
        ttd(interval(Open, &month_day(6, 21), &month_day(9, 23)))
    }));
    // ruleSeason2
    v.push(rule("season", vec![regex("outono")], |_| {
        ttd(interval(Open, &month_day(9, 23), &month_day(12, 21)))
    }));
    // ruleSeason3
    v.push(rule("season", vec![regex("inverno")], |_| {
        ttd(interval(Open, &month_day(12, 21), &month_day(3, 20)))
    }));
    // ruleSeason4
    v.push(rule("season", vec![regex("primavera")], |_| {
        ttd(interval(Open, &month_day(3, 20), &month_day(6, 21)))
    }));
    // ruleTheDayAfterTomorrow
    v.push(rule(
        "the day after tomorrow",
        vec![regex("depois de amanh(ã|a)")],
        |_| tt(cycle_nth(G::Day, 2)),
    ));
    // ruleTheDayBeforeYesterday
    v.push(rule(
        "the day before yesterday",
        vec![regex("anteontem|antes de ontem")],
        |_| tt(cycle_nth(G::Day, -2)),
    ));
    // ruleThisPartofday
    v.push(rule(
        "this <part-of-day>",
        vec![regex("es[ts]a"), pred(is_a_part_of_day)],
        |ts| ttd(intersect(&today(), td(ts[1])?).map(part_of_day)),
    ));
    // ruleThisTime
    v.push(rule("this <time>", vec![regex("es[ts][ae]"), dim_time()], |ts| {
        tt(pred_nth(0, false, td(ts[1])?))
    }));
    // ruleThisnextDayofweek
    v.push(rule(
        "this|next <day-of-week>",
        vec![regex("es[ts][ae]|pr(ó|o)xim[ao]"), pred(is_a_day_of_week)],
        |ts| tt(pred_nth(0, true, td(ts[1])?)),
    ));
    // ruleTimeofdayAmpm
    v.push(rule(
        "<time-of-day> am|pm",
        vec![pred(is_a_time_of_day), regex("([ap])\\.?m?\\.?")],
        |ts| {
            let ap = rx(ts[1], 0)?;
            tt(time_of_day_ampm(to_lower(&ap) == "a", td(ts[0])?))
        },
    ));
    // ruleTimeofdayHoras
    v.push(rule(
        "<time-of-day> horas",
        vec![pred(is_a_time_of_day), regex("h\\.?(ora)?s?")],
        |ts| tt(not_latent(td(ts[0])?.clone())),
    ));
    // ruleTimeofdayLatent
    v.push(rule("time-of-day (latent)", vec![pred(is_integer_between_tok(0, 23))], |ts| {
        let v = get_int_value(ts[0])?;
        tt(mk_latent(hour(true, v)))
    }));
    // ruleTimeofdayPartofday
    v.push(rule(
        "<time-of-day> <part-of-day>",
        vec![dim_time(), regex("(da|na|pela)"), pred(is_a_part_of_day)],
        |ts| isect(ts, 0, 2),
    ));
    // ruleTiradentes
    v.push(rule("Tiradentes", vec![regex("tiradentes")], |_| tt(month_day(4, 21))));
    // ruleTomorrow
    v.push(rule("tomorrow", vec![regex("amanh(ã|a)")], |_| tt(cycle_nth(G::Day, 1))));
    // ruleTwoTimeTokensSeparatedBy
    v.push(rule(
        "two time tokens separated by \",\"",
        vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
        |ts| isect(ts, 0, 2),
    ));
    // ruleTwoTimeTokensSeparatedBy2
    v.push(rule(
        "two time tokens separated by \",\"2",
        vec![pred(is_not_latent), regex(","), pred(is_not_latent)],
        |ts| isect(ts, 2, 0),
    ));
    // ruleUltimoTime
    v.push(rule("ultimo <time>", vec![regex("(u|ú)ltimo"), dim_time()], |ts| {
        tt(pred_nth(-1, false, td(ts[1])?))
    }));
    // ruleVesperaDeAnoNovo
    v.push(rule(
        "vespera de ano novo",
        vec![regex("v(é|e)spera d[eo] ano[\\s\\-]novo")],
        |_| tt(month_day(12, 31)),
    ));
    // ruleWeekend
    v.push(rule("week-end", vec![regex("final de semana|fim de semana|fds")], |_| tt(weekend())));
    // ruleYear
    v.push(rule("year", vec![pred(is_integer_between_tok(1000, 2100))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(year(n))
    }));
    // ruleYearLatent
    v.push(rule("year (latent)", vec![pred(is_integer_between_tok(1, 999))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(mk_latent(year(n)))
    }));
    // ruleYearLatent2
    v.push(rule("year (latent)", vec![pred(is_integer_between_tok(2101, 10000))], |ts| {
        let n = get_int_value(ts[0])?;
        tt(mk_latent(year(n)))
    }));
    // ruleYesterday
    v.push(rule("yesterday", vec![regex("ontem")], |_| tt(cycle_nth(G::Day, -1))));
    // ruleYyyymmdd
    v.push(rule(
        "yyyy-mm-dd",
        vec![regex("(\\d{2,4})-(0?[1-9]|1[0-2])-(3[01]|[12]\\d|0?[1-9])")],
        |ts| {
            let y = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            let d = parse_int(&rx(ts[0], 2)?)?;
            tt(year_month_day(y, m, d))
        },
    ));
    // ruleTimezone
    v.push(rule(
        "<time> timezone",
        vec![pred(is_a_time_of_day), regex(&format!("\\b({})\\b", TIMEZONE_NAME))],
        |ts| ttd(in_timezone(&rx(ts[1], 0)?.to_uppercase(), td(ts[0])?)),
    ));
    // ruleCycleOrdinalOfTime
    v.push(rule(
        "<ordinal> <cycle> de <time>",
        vec![dim_ordinal(), dim_grain(), regex("d[eo]|em"), dim_time()],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth_after(true, grain_of(ts[1])?, n - 1, td(ts[3])?))
        },
    ));
    // ruleCycleOrdinalTime
    v.push(rule(
        "<ordinal> <cycle> <time>",
        vec![dim_ordinal(), dim_grain(), dim_time()],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth_after(true, grain_of(ts[1])?, n - 1, td(ts[2])?))
        },
    ));
    // ruleCycleTheOrdinalOfTime
    v.push(rule(
        "o <ordinal> <cycle> de <time>",
        vec![regex("o|a"), dim_ordinal(), dim_grain(), regex("d[eo]|em"), dim_time()],
        |ts| {
            let n = get_int_value(ts[1])?;
            tt(cycle_nth_after(true, grain_of(ts[2])?, n - 1, td(ts[4])?))
        },
    ));
    // ruleOrdinalTrimestre
    v.push(rule(
        "<ordinal> trimestre",
        vec![dim_ordinal(), pred(is_grain(G::Quarter))],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth_after(true, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
        },
    ));
    // ruleTheOrdinalTrimestre
    v.push(rule(
        "o <ordinal> trimestre",
        vec![regex("o|a"), dim_ordinal(), pred(is_grain(G::Quarter))],
        |ts| {
            let n = get_int_value(ts[1])?;
            tt(cycle_nth_after(true, G::Quarter, n - 1, &cycle_nth(G::Year, 0)))
        },
    ));
    // ruleOrdinalTrimestreYear
    v.push(rule(
        "<ordinal> trimestre <year>",
        vec![dim_ordinal(), pred(is_grain(G::Quarter)), dim_time()],
        |ts| {
            let n = get_int_value(ts[0])?;
            tt(cycle_nth_after(false, G::Quarter, n - 1, td(ts[2])?))
        },
    ));
    // ruleLastCycleOfTime
    v.push(rule(
        "último <cycle> de <time>",
        vec![regex("[úu]ltim[ao]s?"), dim_grain(), regex("d[eo]|em"), dim_time()],
        |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[3])?)),
    ));
    // ruleLastCycleTime
    v.push(rule(
        "último <cycle> <time>",
        vec![regex("[úu]ltim[ao]s?"), dim_grain(), dim_time()],
        |ts| tt(cycle_last_of(grain_of(ts[1])?, td(ts[2])?)),
    ));
    // ruleIntervalFromMonthDDDDOf
    v.push(rule(
        "desde <month> dd-dd de (interval)",
        vec![
            regex("desde|a partir d[eo]"),
            pred(is_dom_value),
            regex("até( ao?)?|ao?"),
            pred(is_dom_value),
            regex("d[eo]|em"),
            pred(is_a_month),
        ],
        |ts| {
            let t = td(ts[5])?;
            let dom1 = intersect_dom(t, ts[1])?;
            let dom2 = intersect_dom(t, ts[3])?;
            ttd(interval(Closed, &dom1, &dom2))
        },
    ));
    // ruleIntervalDDDDMonthOf
    v.push(rule(
        "dd-dd <month> de (interval)",
        vec![
            pred(is_dom_value),
            regex("\\-|a(té)?"),
            pred(is_dom_value),
            regex("d[eo]|em"),
            pred(is_a_month),
        ],
        |ts| {
            let t = td(ts[4])?;
            let dom1 = intersect_dom(t, ts[0])?;
            let dom2 = intersect_dom(t, ts[2])?;
            ttd(interval(Closed, &dom1, &dom2))
        },
    ));
    // ruleIntervalFromMonthDDDD
    v.push(rule(
        "desde <month> dd-dd (interval)",
        vec![
            regex("desde|a partir d[eo]"),
            pred(is_dom_value),
            regex("até( ao?)?|ao?"),
            pred(is_dom_value),
            pred(is_a_month),
        ],
        |ts| {
            let t = td(ts[4])?;
            let dom1 = intersect_dom(t, ts[1])?;
            let dom2 = intersect_dom(t, ts[3])?;
            ttd(interval(Closed, &dom1, &dom2))
        },
    ));
    // ruleIntervalDDDDMonth
    v.push(rule(
        "dd-dd <month> (interval)",
        vec![pred(is_dom_value), regex("\\-|a(té)?"), pred(is_dom_value), pred(is_a_month)],
        |ts| {
            let t = td(ts[3])?;
            let dom1 = intersect_dom(t, ts[0])?;
            let dom2 = intersect_dom(t, ts[2])?;
            ttd(interval(Closed, &dom1, &dom2))
        },
    ));
    // ruleOrdinalCycleOfYearOrdinalCycleOfYear
    v.push(rule(
        "<ordinal> <cycle> de <year> - <ordinal> <cycle> de <year>",
        vec![
            dim_ordinal(),
            dim_grain(),
            regex("d[eo]|em"),
            dim_time(),
            regex("\\-|até( ao?)?|ao?"),
            dim_ordinal(),
            dim_grain(),
            regex("d[eo]|em"),
            dim_time(),
        ],
        |ts| {
            let n1 = get_int_value(ts[0])?;
            let n2 = get_int_value(ts[5])?;
            let from = cycle_nth_after(false, grain_of(ts[1])?, n1 - 1, td(ts[3])?);
            let to = cycle_nth_after(false, grain_of(ts[6])?, n2 - 1, td(ts[8])?);
            ttd(interval(Closed, &from, &to))
        },
    ));
    // ruleOrdinalCycleYearOrdinalCycleYear
    v.push(rule(
        "<ordinal> <cycle> <year> - <ordinal> <cycle> <year>",
        vec![
            dim_ordinal(),
            dim_grain(),
            dim_time(),
            regex("\\-|até( ao?)?|ao?"),
            dim_ordinal(),
            dim_grain(),
            dim_time(),
        ],
        |ts| {
            let n1 = get_int_value(ts[0])?;
            let n2 = get_int_value(ts[4])?;
            let from = cycle_nth_after(false, grain_of(ts[1])?, n1 - 1, td(ts[2])?);
            let to = cycle_nth_after(false, grain_of(ts[5])?, n2 - 1, td(ts[6])?);
            ttd(interval(Closed, &from, &to))
        },
    ));

    v.extend(rule_months());
    v.extend(rule_days_of_week());
    v
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(all_rules);

pub fn rules() -> &'static [Rule] {
    &RULES
}
