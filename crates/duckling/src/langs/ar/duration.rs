//! `Duckling.Duration.AR.Rules`.

use once_cell::sync::Lazy;

use crate::dims::duration::{duration, minutes_from_hour_mixed_fraction};
use crate::dims::numeral::is_natural;
use crate::dims::time_grain::Grain;
use crate::dims::util::parse_integer;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn tok(d: crate::dims::duration::DurationData) -> Option<Token> {
    Some(Token::Duration(d))
}

fn floor_v(t: &Token) -> Option<i64> {
    match t {
        Token::Numeral(n) => Some(n.value.floor() as i64),
        _ => None,
    }
}

fn fixed(name: &str, re: &str, grain: Grain, n: i64) -> Rule {
    rule(name, vec![regex(re)], move |_| tok(duration(grain, n)))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        fixed("quarter of an hour", "(ربع ساعة)", Grain::Minute, 15),
        fixed("half an hour", r"(1/2\s?ساع[ةه]?|نصف? ساع[ةه])", Grain::Minute, 30),
        fixed(
            "three-quarters of an hour",
            r"(3/4\s?(ال)?ساع[ةه]?|ثلاث[ةه]?(\s|-)[أا]رباع (ال)?ساع[ةه])",
            Grain::Minute,
            45,
        ),
        rule("number.number hours", vec![regex(r"(\d+)\.(\d+) *ساع(ة|ات)")], |ts| {
            let g = groups(ts)?;
            let hh = parse_integer(g.first()?)?;
            let mnum = parse_integer(g.get(1)?)?;
            let mden = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
            tok(minutes_from_hour_mixed_fraction(hh, mnum, mden))
        }),
        rule("<integer> and an half hour", vec![pred(is_natural), regex("و ?نصف? ساع[ةه]")], |ts| {
            tok(duration(Grain::Minute, 30 + 60 * floor_v(ts.first()?)?))
        }),
        rule("<integer> + '\"", vec![pred(is_natural), regex("(['\"])")], |ts| {
            match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(n), Token::RegexMatch(g)) => match &**g.0.first()? {
                    "'" => tok(duration(Grain::Minute, n.value.floor() as i64)),
                    "\"" => tok(duration(Grain::Second, n.value.floor() as i64)),
                    _ => None,
                },
                _ => None,
            }
        }),
        fixed("two seconds", "ثانيتين|ثانيتان|لحظتين|لحظتان", Grain::Second, 2),
        fixed("two minutes", "دقيقتين|دقيقتان", Grain::Minute, 2),
        fixed("two hours", "ساعتين|ساعتان", Grain::Hour, 2),
        fixed("dual years", "سنتين|سنتان", Grain::Year, 2),
        rule("dual <unit-of-duration>", vec![dimension(Dim::TimeGrain), regex("(ان|ين)")], |ts| {
            match ts.first()? {
                Token::TimeGrain(g) => tok(duration(*g, 2)),
                _ => None,
            }
        }),
        rule("single <unit-of-duration>", vec![dimension(Dim::TimeGrain)], |ts| {
            match ts.first()? {
                Token::TimeGrain(g) => tok(duration(*g, 1)),
                _ => None,
            }
        }),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
