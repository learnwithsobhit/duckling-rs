//! `Duckling.Numeral.GA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, group, groups, pred, regex, rule, Dim, Rule, Token};

fn lookup(table: &[(&str, i64)], k: &str) -> Option<i64> {
    table.iter().find(|(n, _)| *n == k).map(|(_, v)| *v)
}

const ONE_TO_TEN: &[(&str, i64)] = &[
    ("aon", 1),
    ("dha", 2),
    ("dhá", 2),
    ("trí", 3),
    ("tri", 3),
    ("ceithre", 4),
    ("cuig", 5),
    ("cúig", 5),
    ("sé", 6),
    ("se", 6),
    ("seacht", 7),
    ("ocht", 8),
    ("naoi", 9),
    ("deich", 10),
];

const OLD_VIG: &[(&str, i64)] = &[
    ("dá fhichead", 40),
    ("da fhichead", 40),
    ("dhá fhichead", 40),
    ("dha fhichead", 40),
    ("trí fichid", 60),
    ("tri fichid", 60),
    ("ceithre fichid", 80),
];

const TWENTY_TO_NINETY: &[(&str, i64)] = &[
    ("fiche", 20),
    ("triocha", 30),
    ("tríocha", 30),
    ("daichead", 40),
    ("caoga", 50),
    ("seasca", 60),
    ("seachto", 70),
    ("seachtó", 70),
    ("ochto", 80),
    ("ochtó", 80),
    ("nócha", 90),
    ("nocha", 90),
];

const COUNT_NUMERALS: &[(&str, i64)] = &[
    ("naid", 0),
    ("náid", 0),
    ("haon", 1),
    ("dó", 2),
    ("do", 2),
    ("trí", 3),
    ("tri", 3),
    ("ceathair", 4),
    ("cuig", 5),
    ("cúig", 5),
    ("sé", 6),
    ("se", 6),
    ("seacht", 7),
    ("hocht", 8),
    ("naoi", 9),
    ("deich", 10),
];

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleAmhin
        rule("amháin", vec![regex("amh(á|a)in")], |_| integer(1)),
        // ruleCountNumerals
        rule(
            "count numbers",
            vec![regex(
                "a (n(á|a)id|haon|d(ó|o)|tr(í|i)|ceathair|c(ú|u)ig|s(é|e)|seacht|hocht|naoi|deich)",
            )],
            |ts| lookup(COUNT_NUMERALS, &to_lower(&group(ts, 0)?)).and_then(integer),
        ),
        // ruleDag
        rule("déag", vec![regex("d(é|e)ag")], |_| integer(10)),
        // ruleDecimalNumeral
        rule("decimal number", vec![regex("(\\d*\\.\\d+)")], |ts| {
            parse_decimal(true, &group(ts, 0)?)
        }),
        // ruleDecimalWithThousandsSeparator
        rule(
            "decimal with thousands separator",
            vec![regex("(\\d+(,\\d\\d\\d)+\\.\\d+)")],
            |ts| {
                let m = group(ts, 0)?;
                parse_double(&m.replace(',', "")).and_then(double)
            },
        ),
        // ruleIntegerWithThousandsSeparator
        rule(
            "integer with thousands separator ,",
            vec![regex("(\\d{1,3}(,\\d\\d\\d){1,5})")],
            |ts| {
                let m = group(ts, 0)?;
                parse_double(&m.replace(',', "")).and_then(double)
            },
        ),
        // ruleNumerals
        rule(
            "numbers, 20-90",
            vec![regex(
                "(fiche|tr(í|i)ocha|daichead|caoga|seasca|seacht(ó|o)|ocht(ó|o)|n(ó|o)cha)",
            )],
            |ts| lookup(TWENTY_TO_NINETY, &to_lower(&group(ts, 0)?)).and_then(integer),
        ),
        // ruleNumerals2
        rule(
            "numbers, 1-10",
            vec![regex("(aon|dh(á|a)|tr(í|i)|ceithre|c(ú|u)ig|seacht|s(é|e)|ocht|naoi|deich)")],
            |ts| lookup(ONE_TO_TEN, &to_lower(&group(ts, 0)?)).and_then(integer),
        ),
        // ruleNumeralsPrefixWithNegativeOrMinus
        rule(
            "numbers prefix with -, negative or minus",
            vec![regex("-|m(í|i)neas(\\sa)?"), pred(is_positive)],
            |ts| match ts.get(1)? {
                Token::Numeral(n) => double(n.value * -1.0),
                _ => None,
            },
        ),
        // ruleNumeralsSuffixesKMG
        rule(
            "numbers suffixes (K, M, G)",
            vec![dimension(Dim::Numeral), regex("([kmg])(?=[\\W\\$€]|$)")],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(n), Token::RegexMatch(g)) => match to_lower(g.0.first()?).as_str() {
                    "k" => double(n.value * 1e3),
                    "m" => double(n.value * 1e6),
                    "g" => double(n.value * 1e9),
                    _ => None,
                },
                _ => None,
            },
        ),
        // ruleOldVigesimalNumeralsS
        rule(
            "old vigesimal numbers, 20s",
            vec![regex("(d[ée]ag )?is (dh?(á|a) fhichead|tr(í|i) fichid|ceithre fichid)")],
            |ts| {
                let g = groups(ts)?;
                let ten = g.first()?;
                let match_ = g.get(1)?;
                let x = lookup(OLD_VIG, &to_lower(match_))?;
                integer(if ten.is_empty() { x } else { x + 10 })
            },
        ),
        // ruleOldVigesimalFiche
        rule("old vigesimal 20 + 10", vec![regex("d[ée]ag is fiche")], |_| integer(30)),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
