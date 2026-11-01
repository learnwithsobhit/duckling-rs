//! English locale-specific Time rules (`Duckling.Time.EN.<REGION>.Rules`).

use super::helpers::*;
use super::holidays as h;
use crate::dims::util::parse_int;
use once_cell::sync::Lazy;

use crate::locale::Region;
use crate::types::{regex, rule, Rule, Token};

fn rx(t: &Token, i: usize) -> Option<String> {
    match t {
        Token::RegexMatch(g) => g.0.get(i).map(|s| s.to_string()),
        _ => None,
    }
}

fn mmdd() -> Rule {
    rule("mm/dd", vec![regex("(1[0-2]|0?[1-9])\\s?[/-]\\s?(3[01]|[12]\\d|0?[1-9])")], |ts| {
        let m = parse_int(&rx(ts[0], 0)?)?;
        let d = parse_int(&rx(ts[0], 1)?)?;
        tt(month_day(m, d))
    })
}

fn mmddyyyy() -> Rule {
    rule(
        "mm/dd/yyyy",
        vec![regex("(1[0-2]|0?[1-9])[-/\\s](3[01]|[12]\\d|0?[1-9])[-/\\s](\\d{2,4})")],
        |ts| {
            let y = parse_int(&rx(ts[0], 2)?)?;
            let m = parse_int(&rx(ts[0], 0)?)?;
            let d = parse_int(&rx(ts[0], 1)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn mmddyyyy_dot() -> Rule {
    rule(
        "mm.dd.yyyy",
        vec![regex("(1[0-2]|0?[1-9])\\.(3[01]|[12]\\d|0?[1-9])\\.(\\d{4})")],
        |ts| {
            let y = parse_int(&rx(ts[0], 2)?)?;
            let m = parse_int(&rx(ts[0], 0)?)?;
            let d = parse_int(&rx(ts[0], 1)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn ddmm() -> Rule {
    rule("dd/mm", vec![regex("(3[01]|[12]\\d|0?[1-9])\\s?[/-]\\s?(1[0-2]|0?[1-9])")], |ts| {
        let d = parse_int(&rx(ts[0], 0)?)?;
        let m = parse_int(&rx(ts[0], 1)?)?;
        tt(month_day(m, d))
    })
}

fn ddmmyyyy() -> Rule {
    rule(
        "dd/mm/yyyy",
        vec![regex("(3[01]|[12]\\d|0?[1-9])[-/\\s](1[0-2]|0?[1-9])[-/\\s](\\d{2,4})")],
        |ts| {
            let y = parse_int(&rx(ts[0], 2)?)?;
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn ddmmyyyy_dot() -> Rule {
    rule(
        "dd.mm.yyyy",
        vec![regex("(3[01]|[12]\\d|0?[1-9])\\.(1[0-2]|0?[1-9])\\.(\\d{4})")],
        |ts| {
            let y = parse_int(&rx(ts[0], 2)?)?;
            let d = parse_int(&rx(ts[0], 0)?)?;
            let m = parse_int(&rx(ts[0], 1)?)?;
            tt(year_month_day(y, m, d))
        },
    )
}

fn mm_trio() -> Vec<Rule> {
    vec![mmdd(), mmddyyyy(), mmddyyyy_dot()]
}

fn dd_trio() -> Vec<Rule> {
    vec![ddmm(), ddmmyyyy(), ddmmyyyy_dot()]
}

/// `TimeUS.rulesBackwardCompatible`: used when no region is given.
pub fn rules_backward_compatible() -> Vec<Rule> {
    let mut v = mm_trio();
    v.extend(h::us::backward_compatible_holidays());
    v
}

fn cat(parts: Vec<Vec<Rule>>) -> Vec<Rule> {
    parts.into_iter().flatten().collect()
}

fn rules(region: Region) -> Vec<Rule> {
    use Region::*;
    match region {
        AU => cat(vec![
            dd_trio(),
            h::au::computed_holidays(),
            h::au::periodic_holidays(),
            h::au::periodic_holidays_prime(),
        ]),
        BZ => cat(vec![dd_trio(), h::bz::periodic_holidays()]),
        CA => cat(vec![mm_trio(), h::ca::periodic_holidays()]),
        GB => cat(vec![dd_trio(), h::gb::computed_holidays(), h::gb::periodic_holidays()]),
        IE => cat(vec![dd_trio(), h::ie::periodic_holidays()]),
        IN => cat(vec![dd_trio(), h::in_::computed_holidays(), h::in_::periodic_holidays()]),
        JM => cat(vec![dd_trio(), h::jm::periodic_holidays()]),
        NZ => cat(vec![dd_trio(), h::nz::computed_holidays(), h::nz::periodic_holidays()]),
        PH => cat(vec![mm_trio(), h::ph::periodic_holidays()]),
        TT => cat(vec![dd_trio(), h::tt::computed_holidays(), h::tt::periodic_holidays()]),
        US => cat(vec![
            rules_backward_compatible(),
            h::us::computed_holidays(),
            h::us::computed_holidays_prime(),
            h::us::periodic_holidays(),
            h::us::periodic_holidays_prime(),
        ]),
        ZA => cat(vec![
            vec![mmdd(), ddmmyyyy(), ddmmyyyy_dot()],
            h::za::periodic_holidays(),
            h::za::periodic_holidays_prime(),
        ]),
        _ => vec![],
    }
}

static BACKWARD_COMPATIBLE: Lazy<Vec<Rule>> = Lazy::new(rules_backward_compatible);

static BY_REGION: Lazy<Vec<(Region, Vec<Rule>)>> = Lazy::new(|| {
    use Region::*;
    [AU, BZ, CA, GB, IE, IN, JM, NZ, PH, TT, US, ZA].into_iter().map(|r| (r, rules(r))).collect()
});

/// `TimeUS.rulesBackwardCompatible`, cached.
pub fn backward_compatible() -> &'static [Rule] {
    &BACKWARD_COMPATIBLE
}

/// `localeRules <region> (Seal Time)` for English, cached.
pub fn en_rules(region: Region) -> &'static [Rule] {
    BY_REGION.iter().find(|(r, _)| *r == region).map(|(_, v)| v.as_slice()).unwrap_or(&[])
}
