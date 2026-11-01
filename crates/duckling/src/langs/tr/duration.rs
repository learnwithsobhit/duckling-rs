//! `Duckling.Duration.TR.Rules`.

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

fn rule_numeral_quotes() -> Rule {
    rule("<integer> + '\"", vec![pred(is_natural), regex("(['\"])")], |ts| {
        match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(n), Token::RegexMatch(g)) => match &**g.0.first()? {
                "'" => tok(duration(Grain::Minute, n.value.floor() as i64)),
                "\"" => tok(duration(Grain::Second, n.value.floor() as i64)),
                _ => None,
            },
            _ => None,
        }
    })
}

fn rule_duration_dot_numeral_hours() -> Rule {
    rule("number.number hours", vec![regex("(\\d+)(\\.|,)(\\d+) *sa(at)?")], |ts| {
        // Haskell: GroupMatch (h:m:_) — first two groups, even though group 2 is the separator.
        let g = groups(ts)?;
        let hh = parse_integer(g.first()?)?;
        let mnum = parse_integer(g.get(1)?)?;
        let mden = 10i128.wrapping_pow(g.get(1)?.chars().count() as u32);
        tok(minutes_from_hour_mixed_fraction(hh, mnum, mden))
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("quarter of an hour", vec![regex("(1/4\\s?sa(at)?|çeyrek saat)")], |_| {
            tok(duration(Grain::Minute, 15))
        }),
        rule("half an hour", vec![regex("(1/2\\s?sa(at)?|yarım saat)")], |_| {
            tok(duration(Grain::Minute, 30))
        }),
        rule(
            "three-quarters of an hour",
            vec![regex("(3/4\\s?sa(at)?|üç çeyrek sa(at)?)")],
            |_| tok(duration(Grain::Minute, 45)),
        ),
        rule("fortnight", vec![regex("iki hafta")], |_| tok(duration(Grain::Day, 14))),
        rule_duration_dot_numeral_hours(),
        rule(
            "<integer> and an half hour",
            vec![pred(is_natural), regex("buçuk sa(at)?")],
            |ts| tok(duration(Grain::Minute, 30 + 60 * floor_v(ts.first()?)?)),
        ),
        rule(
            "<duration> about|exactly",
            vec![regex("(yaklaşık|tam(\\solarak)?)"), dimension(Dim::Duration)],
            |ts| ts.get(1).map(|t| (*t).clone()),
        ),
        rule("<duration> about|exactly", vec![dimension(Dim::Duration), regex("gibi|civarında")], |ts| {
            ts.first().map(|t| (*t).clone())
        }),
        rule_numeral_quotes(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
