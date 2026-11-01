//! `Duckling.Ordinal.NL.Rules` and `Duckling.TimeGrain.NL.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::dims::time_grain::Grain;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{group, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

// Note: upstream's map has "achste" (not "achtste") and lacks "zeste"/"zesde" as spelled in
// the regex; ported as is.
const ZERO_NINETEEN: &[(&str, i64)] = &[
    ("eerste", 1),
    ("tweede", 2),
    ("derde", 3),
    ("vierde", 4),
    ("vijfde", 5),
    ("zesde", 6),
    ("zevende", 7),
    ("achste", 8),
    ("negende", 9),
    ("tiende", 10),
    ("elfde", 11),
    ("twaalfde", 12),
    ("dertiende", 13),
    ("veertiende", 14),
    ("vijftiende", 15),
    ("zestiende", 16),
    ("zeventiende", 17),
    ("achttiende", 18),
    ("negentiende", 19),
];

static ORDINAL_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule("ordinal (digits)", vec![regex("0*(\\d+)(\\.| ?(ste|de))")], |ts| {
            parse_int(&group(ts, 0)?).and_then(ordinal)
        }),
        rule(
            "ordinals (first..19th)",
            vec![regex("(eerste|tweede|derde|vierde|vijfde|zeste|zevende|achtste|negende|tiende|elfde|twaalfde|veertiende|vijftiende|zestiende|zeventiende|achttiende|negentiende)")],
            |ts| {
                let m = to_lower(&group(ts, 0)?);
                ZERO_NINETEEN.iter().find(|(n, _)| *n == m).and_then(|(_, v)| ordinal(*v))
            },
        ),
    ]
});

pub fn ordinal_rules() -> &'static [Rule] {
    &ORDINAL_RULES
}

static GRAIN_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let grains: [(&str, &str, Grain); 8] = [
        ("second (grain) ", "(seconde(n|s)?|sec|s)", Grain::Second),
        ("minute (grain)", "(minuut|minuten|min|m)", Grain::Minute),
        ("hour (grain)", "(uur|uren|u|h)", Grain::Hour),
        ("day (grain)", "dagen|dag|d", Grain::Day),
        ("week (grain)", "(weken|week|w)", Grain::Week),
        ("month (grain)", "(maanden|maand|mnd)", Grain::Month),
        ("quarter (grain)", "(kwartaal|kwartalen)", Grain::Quarter),
        ("year (grain)", "(jaren|jaar|j)", Grain::Year),
    ];
    grains
        .into_iter()
        .map(|(name, re, g)| rule(name, vec![regex(re)], move |_| Some(Token::TimeGrain(g))))
        .collect()
});

pub fn time_grain_rules() -> &'static [Rule] {
    &GRAIN_RULES
}
