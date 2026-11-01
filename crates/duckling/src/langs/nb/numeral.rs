//! `Duckling.Numeral.NB.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, group, pred, regex, rule, Dim, Rule, Token};

const ZERO_TO_NINETEEN: &[(&str, i64)] = &[
    ("null", 0),
    ("ingen", 0),
    ("intet", 0),
    ("en", 1),
    ("ett", 1),
    ("én", 1),
    ("to", 2),
    ("tre", 3),
    ("fire", 4),
    ("fem", 5),
    ("seks", 6),
    ("sju", 7),
    ("syv", 7),
    ("åtte", 8),
    ("ni", 9),
    ("ti", 10),
    ("elleve", 11),
    ("tolv", 12),
    ("tretten", 13),
    ("fjorten", 14),
    ("femten", 15),
    ("seksten", 16),
    ("søtten", 17),
    ("sytten", 17),
    ("atten", 18),
    ("nitten", 19),
];

const TWENTY_TO_HUNDRED: &[(&str, i64)] = &[
    ("tjueen", 21),
    ("tjueén", 21),
    ("tjueto", 22),
    ("tjuetre", 23),
    ("tjuefire", 24),
    ("tjuefem", 25),
    ("tjueseks", 26),
    ("tjuesju", 27),
    ("tjuesyv", 27),
    ("tjueåtte", 28),
    ("tjueni", 29),
    ("trettien", 31),
    ("trettién", 31),
    ("trettito", 32),
    ("trettitre", 33),
    ("trettifire", 34),
    ("trettifem", 35),
    ("trettiseks", 36),
    ("trettisju", 37),
    ("trettisyv", 37),
    ("trettiåtte", 38),
    ("trettini", 39),
    ("førtien", 41),
    ("førtién", 41),
    ("førtito", 42),
    ("førtitre", 43),
    ("førtifire", 44),
    ("førtifem", 45),
    ("førtiseks", 46),
    ("førtisju", 47),
    ("førtisyv", 47),
    ("førtiåtte", 48),
    ("førtini", 49),
    ("femtien", 51),
    ("femtién", 51),
    ("femtito", 52),
    ("femtitre", 53),
    ("femtifire", 54),
    ("femtifem", 55),
    ("femtiseks", 56),
    ("femtisju", 57),
    ("femtisyv", 57),
    ("femtiåtte", 58),
    ("femtini", 59),
    ("sekstien", 61),
    ("sekstién", 61),
    ("sekstito", 62),
    ("sekstitre", 63),
    ("sekstifire", 64),
    ("sekstifem", 65),
    ("sekstiseks", 66),
    ("sekstisju", 67),
    ("sekstisyv", 67),
    ("sekstiåtte", 68),
    ("sekstini", 69),
    ("syttien", 71),
    ("syttién", 71),
    ("syttito", 72),
    ("syttitre", 73),
    ("syttifire", 74),
    ("syttifem", 75),
    ("syttiseks", 76),
    ("syttisju", 77),
    ("syttisyv", 77),
    ("syttiåtte", 78),
    ("syttini", 79),
    ("søttien", 71),
    ("søttién", 71),
    ("søttito", 72),
    ("søttitre", 73),
    ("søttifire", 74),
    ("søttifem", 75),
    ("søttiseks", 76),
    ("søttisju", 77),
    ("søttisyv", 77),
    ("søttiåtte", 78),
    ("søttini", 79),
    ("åttien", 81),
    ("åttién", 81),
    ("åttito", 82),
    ("åttitre", 83),
    ("åttifire", 84),
    ("åttifem", 85),
    ("åttiseks", 86),
    ("åttisju", 87),
    ("åttisyv", 87),
    ("åttiåtte", 88),
    ("åttini", 89),
    ("nittien", 91),
    ("nittién", 91),
    ("nittito", 92),
    ("nittitre", 93),
    ("nittifire", 94),
    ("nittifem", 95),
    ("nittiseks", 96),
    ("nittisju", 97),
    ("nittisyv", 97),
    ("nittiåtte", 98),
    ("nittini", 99),
];

const TENS: &[(&str, i64)] = &[
    ("tyve", 20),
    ("tjue", 20),
    ("tredve", 30),
    ("tretti", 30),
    ("førti", 40),
    ("femti", 50),
    ("seksti", 60),
    ("sytti", 70),
    ("søtti", 70),
    ("åtti", 80),
    ("nitti", 90),
];

fn lookup(table: &[(&str, i64)], k: &str) -> Option<i64> {
    table.iter().find(|(n, _)| *n == k).map(|(_, v)| *v)
}

fn nv(t: &Token) -> Option<f64> {
    match t {
        Token::Numeral(n) => Some(n.value),
        _ => None,
    }
}

fn grain_of(t: &Token) -> Option<i64> {
    match t {
        Token::Numeral(n) => n.grain,
        _ => None,
    }
}

fn not_multipliable_positive(t: &Token) -> bool {
    !is_multipliable(t) && is_positive(t)
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleAPair
        rule("a pair", vec![regex("et par")], |_| integer(2).and_then(|t| with_grain(1, &t))),
        // ruleDecimalNumeral
        rule("decimal number", vec![regex("(\\d*,\\d+)")], |ts| {
            parse_decimal(false, &group(ts, 0)?)
        }),
        // ruleDecimalWithThousandsSeparator
        rule("decimal with thousands separator", vec![regex("(\\d+(\\.\\d\\d\\d)+\\,\\d+)")], |ts| {
            let m = group(ts, 0)?;
            let fmt = m.replace('.', "").replace(',', ".");
            parse_double(&fmt).and_then(double)
        }),
        // ruleDozen
        rule("dozen", vec![regex("dusin")], |_| {
            integer(12).and_then(|t| with_grain(1, &t)).and_then(|t| with_multipliable(&t))
        }),
        // ruleFew
        rule("few", vec![regex("(noen )?få")], |_| integer(3)),
        // ruleInteger
        rule(
            "integer (0..19)",
            vec![regex("(intet|ingen|null|en|ett|én|to|tretten|tre|fire|femten|fem|seksten|seks|syv|sju|åtte|nitten|ni|ti|elleve|tolv|fjorten|sytten|søtten|atten)")],
            |ts| lookup(ZERO_TO_NINETEEN, &to_lower(&group(ts, 0)?)).and_then(integer),
        ),
        // ruleInteger2
        rule(
            "integer (20..90)",
            vec![regex("(tyve|tjue|tredve|tretti|førti|femti|seksti|sytti|søtti|åtti|nitti)")],
            |ts| lookup(TENS, &to_lower(&group(ts, 0)?)).and_then(integer),
        ),
        // ruleInteger3
        rule(
            "integer 21..99",
            vec![one_of(vec![70.0, 20.0, 60.0, 50.0, 40.0, 90.0, 30.0, 80.0]), pred(number_between(1.0, 10.0))],
            |ts| double(nv(ts.first()?)? + nv(ts.get(1)?)?),
        ),
        // ruleInteger4
        rule(
            "integer (21..99)",
            vec![regex("(tjueen|tjueén|tjueto|tjuetre|tjuefire|tjuefem|tjueseks|tjuesju|tjuesyv|tjueåtte|tjueni|trettien|trettién|trettito|trettitre|trettifire|trettifem|trettiseks|trettisju|trettisyv|trettiåtte|trettini|førtien|førtién|førtito|førtitre|førtifire|førtifem|førtiseks|førtisju|førtisyv|førtiåtte|førtini|femtien|femtién|femtito|femtitre|femtifire|femtifem|femtiseks|femtisju|femtisyv|femtiåtte|femtini|sekstien|sekstién|sekstito|sekstitre|sekstifire|sekstifem|sekstiseks|sekstisju|sekstisyv|sekstiåtte|sekstini|syttien|syttién|syttito|syttitre|syttifire|syttifem|syttiseks|syttisju|syttisyv|syttiåtte|syttini|søttien|søttién|søttito|søttitre|søttifire|søttifem|søttiseks|søttisju|søttisyv|søttiåtte|søttini|åttien|åttién|åttito|åttitre|åttifire|åttifem|åttiseks|åttisju|åttisyv|åttiåtte|åttini|nittien|nittién|nittito|nittitre|nittifire|nittifem|nittiseks|nittisju|nittisyv|nittiåtte|nittini)")],
            |ts| lookup(TWENTY_TO_HUNDRED, &to_lower(&group(ts, 0)?)).and_then(integer),
        ),
        // ruleIntegerWithThousandsSeparator
        rule(
            "integer with thousands separator .",
            vec![regex("(\\d{1,3}((?:\\.| )\\d\\d\\d){1,5})")],
            |ts| {
                let m = group(ts, 0)?;
                parse_double(&m.replace(' ', "").replace('.', "")).and_then(double)
            },
        ),
        // ruleIntersect
        rule(
            "intersect",
            vec![pred(has_grain), pred(not_multipliable_positive)],
            |ts| {
                let g = grain_of(ts.first()?)?;
                let val2 = nv(ts.get(1)?)?;
                if 10f64.powf(g as f64) > val2 {
                    double(nv(ts.first()?)? + val2)
                } else {
                    None
                }
            },
        ),
        // ruleIntersectWithAnd
        rule(
            "intersect (with and)",
            vec![pred(has_grain), regex("og"), pred(not_multipliable_positive)],
            |ts| {
                let g = grain_of(ts.first()?)?;
                let val2 = nv(ts.get(2)?)?;
                if 10f64.powf(g as f64) > val2 {
                    double(nv(ts.first()?)? + val2)
                } else {
                    None
                }
            },
        ),
        // ruleMultiply
        rule(
            "compose by multiplication",
            vec![dimension(Dim::Numeral), pred(is_multipliable)],
            |ts| multiply(ts.first()?, ts.get(1)?),
        ),
        // ruleNumeralDotNumeral
        rule(
            "number dot number",
            vec![dimension(Dim::Numeral), regex("komma"), pred(|t| !has_grain(t))],
            |ts| double(nv(ts.first()?)? + decimals_to_double(nv(ts.get(2)?)?)),
        ),
        // ruleNumeralsPrefixWithNegativeOrMinus
        rule(
            "numbers prefix with -, negative or minus",
            vec![regex("-|minus|negativ"), pred(is_positive)],
            |ts| double(nv(ts.get(1)?)? * (-1.0)),
        ),
        // ruleNumeralsSuffixesKMG
        rule(
            "numbers suffixes (K, M, G)",
            vec![dimension(Dim::Numeral), regex("([kmg])(?=[\\W\\$€]|$)")],
            |ts| {
                let v = nv(ts.first()?)?;
                let m = match ts.get(1)? {
                    Token::RegexMatch(g) => g.0.first()?.clone(),
                    _ => return None,
                };
                match to_lower(&m).as_str() {
                    "k" => double(v * 1e3),
                    "m" => double(v * 1e6),
                    "g" => double(v * 1e9),
                    _ => None,
                }
            },
        ),
        // rulePowersOfTen
        rule(
            "powers of tens",
            vec![regex("(hundre(de)?|tusen?|million(er)?|milliard(?:er)?|billion(?:er)?|billiard(?:er)?|trillion(?:er)?|trilliard(?:er)?|kvadrillion(?:er)?|kvadrilliard(?:er)?)")],
            |ts| {
                let (v, g) = match to_lower(&group(ts, 0)?).as_str() {
                    "hundre" => (1e2, 2),
                    "hundrede" => (1e2, 2),
                    "tuse" => (1e3, 3),
                    "tusen" => (1e3, 3),
                    "million" => (1e6, 6),
                    "millioner" => (1e6, 6),
                    "milliard" => (1e9, 9),
                    "milliarder" => (1e9, 9),
                    "billion" => (1e12, 12),
                    "billioner" => (1e12, 12),
                    "billiard" => (1e15, 15),
                    "billiarder" => (1e15, 15),
                    "trillion" => (1e18, 18),
                    "trillioner" => (1e18, 18),
                    "trilliard" => (1e21, 21),
                    "trilliarder" => (1e21, 21),
                    "kvadrillion" => (1e24, 24),
                    "kvadrillioner" => (1e24, 24),
                    "kvadrilliard" => (1e27, 27),
                    "kvadrilliarder" => (1e27, 27),
                    _ => return None,
                };
                double(v).and_then(|t| with_grain(g, &t)).and_then(|t| with_multipliable(&t))
            },
        ),
        // ruleSingle
        rule("single", vec![regex("enkelt")], |_| integer(1).and_then(|t| with_grain(1, &t))),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
