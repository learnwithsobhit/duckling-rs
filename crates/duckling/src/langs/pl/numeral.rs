//! `Duckling.Numeral.PL.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn grain2(n: i64) -> Option<Token> {
    integer(n).and_then(|t| with_grain(2, &t))
}

fn intersect_sum(ts: &[&Token]) -> Option<Token> {
    match (ts.first()?, ts.get(1)?) {
        (Token::Numeral(a), Token::Numeral(b)) => {
            let g = a.grain?;
            if 10f64.powf(g as f64) > b.value {
                double(a.value + b.value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn intersect_sum_and(ts: &[&Token]) -> Option<Token> {
    match (ts.first()?, ts.get(2)?) {
        (Token::Numeral(a), Token::Numeral(b)) => {
            let g = a.grain?;
            if 10f64.powf(g as f64) > b.value {
                double(a.value + b.value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn not_multipliable_positive(t: &Token) -> bool {
    !is_multipliable(t) && is_positive(t)
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleAFew
        rule("a few", vec![regex("kilk(a|u)")], |_| integer(3)),
        // ruleAPair
        rule("a pair", vec![regex("para?")], |_| integer(2)),
        // ruleCouple
        rule("couple", vec![regex("pare")], |_| integer(2)),
        // ruleDecimalNumeral
        rule("decimal number", vec![regex("(\\d*\\.\\d+)")], |ts| {
            parse_decimal(true, groups(ts)?.first()?)
        }),
        // ruleDecimalWithThousandsSeparator
        rule("decimal with thousands separator", vec![regex("(\\d+(,\\d\\d\\d)+\\.\\d+)")], |ts| {
            parse_double(&groups(ts)?.first()?.replace(',', "")).and_then(double)
        }),
        // ruleDozen
        rule("dozen", vec![regex("tuzin")], |_| integer(12).and_then(|t| with_multipliable(&t))),
        // ruleEight
        rule("eight", vec![regex("o(s|ś)(iem|miu|mioma)")], |_| integer(8)),
        // ruleEighteen
        rule("eighteen", vec![regex("osiemna(s|ś)(tu|cie|toma)")], |_| integer(18)),
        // ruleEighty
        rule("eighty", vec![regex("osiemdziesi((a|ą)t|(e|ę)ci(u|oma))")], |_| integer(80)),
        // ruleEleven
        rule("eleven", vec![regex("jedena(stu|(s|ś)cie|stoma)")], |_| integer(11)),
        // ruleFifteen
        rule("fifteen", vec![regex("pi(ę)tna(s|ś)(ta|tu|cie|toma)")], |_| integer(15)),
        // ruleFifty
        rule("fifty", vec![regex("pi(ęć)dziesi(ą)t|pi(ęć)dziesi(ę)ci(u|oma)")], |_| integer(50)),
        // ruleFive
        rule("five", vec![regex("pi(e|ę)(c|ć)(iu|oma|u)?")], |_| integer(5)),
        // ruleFour
        rule("four", vec![regex("czter(ej|y|ech|em|ema)")], |_| integer(4)),
        // ruleFourteen
        rule("fourteen", vec![regex("czterna(s|ś)(tu|cie|toma)")], |_| integer(14)),
        // ruleFourty
        rule("fou?rty", vec![regex("czterdzie(ś)ci|czterdziest(u|oma)")], |_| integer(40)),
        // ruleInteger2
        rule(
            "integer 21..99",
            vec![one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]), pred(number_between(1.0, 10.0))],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
                _ => None,
            },
        ),
        // ruleIntegerWithThousandsSeparator
        rule("integer with thousands separator ,", vec![regex("(\\d{1,3}(,\\d\\d\\d){1,5})")], |ts| {
            parse_double(&groups(ts)?.first()?.replace(',', "")).and_then(double)
        }),
        // ruleIntersect
        rule(
            "intersect",
            vec![pred(has_grain), pred(not_multipliable_positive)],
            intersect_sum,
        ),
        // ruleIntersectWithAnd
        rule(
            "intersect (with and)",
            vec![pred(has_grain), regex("i|a"), pred(not_multipliable_positive)],
            intersect_sum_and,
        ),
        // ruleMillion
        rule("million", vec![regex("milion(y|(ó)w)?")], |_| {
            integer(1_000_000).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t))
        }),
        // ruleMultiply
        rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
            multiply(ts.first()?, ts.get(1)?)
        }),
        // ruleNine
        rule("nine", vec![regex("dziewi(e|ę)(ć|c)(iu|ioma)?")], |_| integer(9)),
        // ruleNineteen
        rule("nineteen", vec![regex("dziewietna(s|ś)(tu|cie|toma)")], |_| integer(19)),
        // ruleNinety
        rule("ninety", vec![regex("dziewi(e|ę)(c|ć)dziesi((a|ą)|(e|ę))(t|ci(u|oma))")], |_| {
            integer(90)
        }),
        // ruleNumeral
        rule("number 100", vec![regex("(sto|setki)")], |_| grain2(100)),
        // ruleNumeral2
        rule("number 200", vec![regex("dwie((ś)cie| setki)")], |_| grain2(200)),
        // ruleNumeral3
        rule("number 300", vec![regex("(trzy(sta| setki))")], |_| grain2(300)),
        // ruleNumeral4
        rule("number 400", vec![regex("(cztery(sta| setki))")], |_| grain2(400)),
        // ruleNumeral5
        rule("number 500", vec![regex("pi(ęć)(set| setek)")], |_| grain2(500)),
        // ruleNumeral6
        rule("number 600", vec![regex("(sześć(set| setek))")], |_| grain2(600)),
        // ruleNumeral7
        rule("number 700", vec![regex("(siedem(set| setek))")], |_| grain2(700)),
        // ruleNumeral8
        rule("number 800", vec![regex("(osiem(set| setek))")], |_| grain2(800)),
        // ruleNumeral9
        rule("number 900", vec![regex("dziewi(ęć)(set| setek)")], |_| grain2(900)),
        // ruleNumeralDotNumeral
        rule(
            "number dot number",
            vec![dimension(Dim::Numeral), regex("dot|point"), pred(|t| !has_grain(t))],
            |ts| match (ts.first()?, ts.get(2)?) {
                (Token::Numeral(a), Token::Numeral(b)) => double(a.value + decimals_to_double(b.value)),
                _ => None,
            },
        ),
        // ruleNumeralsPrefixWithNegativeOrMinus
        rule(
            "numbers prefix with -, negative or minus",
            vec![regex("-|minus|negative"), pred(is_positive)],
            |ts| match ts.get(1)? {
                Token::Numeral(nd) => double(nd.value * -1.0),
                _ => None,
            },
        ),
        // ruleNumeralsSuffixesKMG
        rule(
            "numbers suffixes (K, M, G)",
            vec![dimension(Dim::Numeral), regex("([kmg])(?=[\\W\\$€]|$)")],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(nd), Token::RegexMatch(g)) => match to_lower(g.0.first()?).as_str() {
                    "k" => double(nd.value * 1e3),
                    "m" => double(nd.value * 1e6),
                    "g" => double(nd.value * 1e9),
                    _ => None,
                },
                _ => None,
            },
        ),
        // ruleOne
        rule("one", vec![regex("jed(en|nego|nemu|nym|nej|n(a|ą))")], |_| integer(1)),
        // ruleSeven
        rule("seven", vec![regex("sied(miu|em|mioma)")], |_| integer(7)),
        // ruleSeventeen
        rule("seventeen", vec![regex("siedemna(s|ś)(tu|cie|toma)")], |_| integer(17)),
        // ruleSeventy
        rule("seventy", vec![regex("siedemdziesi((a|ą)t|(e|ę)ci(u|oma))")], |_| integer(70)),
        // ruleSingle
        rule("single", vec![regex("pojedynczy")], |_| integer(1)),
        // ruleSix
        rule("six", vec![regex("sze(s|ś)(c|ć)(iu|oma|u)?")], |_| integer(6)),
        // ruleSixteen
        rule("sixteen", vec![regex("szesna(s|ś)(tu|cie|toma)")], |_| integer(16)),
        // ruleSixty
        rule("sixty", vec![regex("sze(ść)dziesi(ą)t|sze(ść)dziesi(ę)ci(u|oma)")], |_| integer(60)),
        // ruleSpecialCompositionForMissingHundredsLikeInOneTwentyTwo
        rule(
            "special composition for missing hundreds like in one twenty two",
            vec![pred(number_between(1.0, 10.0)), pred(number_between(10.0, 100.0))],
            |ts| match (ts.first()?, ts.get(1)?) {
                (Token::Numeral(a), Token::Numeral(b)) => double(a.value * 100.0 + b.value),
                _ => None,
            },
        ),
        // ruleTen
        rule("ten", vec![regex("dzisi(e|ę)(ć|c)(iu|ioma)?")], |_| integer(10)),
        // ruleThirteen
        rule("thirteen", vec![regex("trzyna(ś|s)(tu|cie|toma)")], |_| integer(13)),
        // ruleThirty
        rule("thirty", vec![regex("trzydzie(ś)ci|trzydziest(u|oma)")], |_| integer(30)),
        // ruleThousand
        rule("thousand", vec![regex("ty(s|ś)i(a|ą|ę)c(e|y)?")], |_| {
            integer(1000).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t))
        }),
        // ruleThree
        rule("three", vec![regex("trz(y|ema|ech)")], |_| integer(3)),
        // ruleTwelve
        rule("twelve", vec![regex("dwunast(u|oma)|dwana(ś|s)cie")], |_| integer(12)),
        // ruleTwenty
        rule("twenty", vec![regex("dwadzie(ś|s)cia|dwudziest(u|oma)")], |_| integer(20)),
        // ruleTwo
        rule("two", vec![regex("dw(a|(o|ó)(ch|m)|oma|iema|ie)")], |_| integer(2)),
        // ruleZero
        rule("zero", vec![regex("(zero|nic)")], |_| integer(0)),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
