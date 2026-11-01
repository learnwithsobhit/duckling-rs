//! `Duckling.Numeral.IT.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn rule_negative() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex("-|meno|negativo"), pred(is_positive)],
        |ts| match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * -1.0),
            _ => None,
        },
    )
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule(
        "decimal with thousands separator",
        vec![regex("(\\d+(([\\. ])\\d\\d\\d)+,\\d+)")],
        |ts| {
            let g = groups(ts)?;
            let m = g.first()?;
            let sep = g.get(2)?;
            let fmt = m.replace(sep.as_ref(), "").replace(',', ".");
            parse_double(&fmt).and_then(double)
        },
    )
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex("(\\d*,\\d+)")], |ts| {
        parse_decimal(false, groups(ts)?.first()?)
    })
}

static TENS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("venti", 20),
        ("trenta", 30),
        ("quaranta", 40),
        ("cinquanta", 50),
        ("sessanta", 60),
        ("settanta", 70),
        ("ottanta", 80),
        ("novanta", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral2() -> Rule {
    rule(
        "number (20..90)",
        vec![regex("(venti|trenta|quaranta|cinquanta|sessanta|settanta|ottanta|novanta)")],
        |ts| TENS.get(to_lower(groups(ts)?.first()?).as_str()).and_then(|v| integer(*v)),
    )
}

static ZERO_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("zero", 0),
        ("niente", 0),
        ("nulla", 0),
        ("un", 1),
        ("uno", 1),
        ("due", 2),
        ("tre", 3),
        ("quattro", 4),
        ("cinque", 5),
        ("sei", 6),
        ("sette", 7),
        ("otto", 8),
        ("nove", 9),
        ("dieci", 10),
        ("undici", 11),
        ("dodici", 12),
        ("tredici", 13),
        ("quattordici", 14),
        ("quindici", 15),
        ("sedici", 16),
        ("diciassette", 17),
        ("diciotto", 18),
        ("diciannove", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral() -> Rule {
    rule(
        "number (0..19)",
        vec![regex("(zero|nulla|niente|uno|due|tredici|tre|quattro|cinque|sei|sette|otto|nove|dieci|undici|dodici|quattordici|quindici|sedici|diciassette|diciotto|diciannove|un)")],
        |ts| ZERO_NINETEEN.get(to_lower(groups(ts)?.first()?).as_str()).and_then(|v| integer(*v)),
    )
}

static HUNDREDS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("cento", 100),
        ("duecento", 200),
        ("trecento", 300),
        ("quattrocento", 400),
        ("cinquecento", 500),
        ("seicento", 600),
        ("settecento", 700),
        ("ottocento", 800),
        ("novecento", 900),
        ("mila", 1000),
        ("mille", 1000),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral5() -> Rule {
    rule(
        "number 100..1000 ",
        vec![regex("(due|tre|quattro|cinque|sei|sette|otto|nove)?cento|mil(a|le)")],
        |ts| HUNDREDS.get(to_lower(groups(ts)?.first()?).as_str()).and_then(|v| integer(*v)),
    )
}

fn rule_numeral3() -> Rule {
    rule(
        "number (21..29 31..39 41..49 51..59 61..69 71..79 81..89 91..99)",
        vec![
            one_of(vec![70.0, 20.0, 60.0, 50.0, 40.0, 90.0, 30.0, 80.0]),
            regex("e"),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
            _ => None,
        },
    )
}

fn rule_suffixes() -> Rule {
    rule(
        "numbers suffixes (K, M, G)",
        vec![dimension(Dim::Numeral), regex("([kmg])(?=[\\W\\$€]|$)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(nd), Token::RegexMatch(g)) => {
                let x = match to_lower(g.0.first()?).as_str() {
                    "k" => 1e3,
                    "m" => 1e6,
                    "g" => 1e9,
                    _ => return None,
                };
                double(nd.value * x)
            }
            _ => None,
        },
    )
}

static TWENTYONE_NINETYNINE: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ventuno", 21),
        ("ventidue", 22),
        ("ventitre", 23),
        ("ventitré", 23),
        ("ventiquattro", 24),
        ("venticinque", 25),
        ("ventisei", 26),
        ("ventisette", 27),
        ("ventotto", 28),
        ("ventinove", 29),
        ("trentuno", 31),
        ("trentadue", 32),
        ("trentatre", 33),
        ("trentatré", 33),
        ("trentaquattro", 34),
        ("trentacinque", 35),
        ("trentasei", 36),
        ("trentasette", 37),
        ("trentotto", 38),
        ("trentanove", 39),
        ("quarantuno", 41),
        ("quarantadue", 42),
        ("quarantatre", 43),
        ("quarantatré", 43),
        ("quarantaquattro", 44),
        ("quarantacinque", 45),
        ("quarantasei", 46),
        ("quarantasette", 47),
        ("quarantotto", 48),
        ("quarantanove", 49),
        ("cinquantuno", 51),
        ("cinquantadue", 52),
        ("cinquantatre", 53),
        ("cinquantatré", 53),
        ("cinquantaquattro", 54),
        ("cinquantacinque", 55),
        ("cinquantasei", 56),
        ("cinquantasette", 57),
        ("cinquantotto", 58),
        ("cinquantanove", 59),
        ("sessantuno", 61),
        ("sessantadue", 62),
        ("sessantatré", 63),
        ("sessantatre", 63),
        ("sessantaquattro", 64),
        ("sessantacinque", 65),
        ("sessantasei", 66),
        ("sessantasette", 67),
        ("sessantotto", 68),
        ("sessantanove", 69),
        ("settantuno", 71),
        ("settantadue", 72),
        ("settantatré", 73),
        ("settantatre", 73),
        ("settantaquattro", 74),
        ("settantacinque", 75),
        ("settantasei", 76),
        ("settantasette", 77),
        ("settantotto", 78),
        ("settantanove", 79),
        ("ottantuno", 81),
        ("ottantadue", 82),
        ("ottantatré", 83),
        ("ottantatre", 83),
        ("ottantaquattro", 84),
        ("ottantacinque", 85),
        ("ottantasei", 86),
        ("ottantasette", 87),
        ("ottantotto", 88),
        ("ottantanove", 89),
        ("novantuno", 91),
        ("novantadue", 92),
        ("novantatre", 93),
        ("novantatré", 93),
        ("novantaquattro", 94),
        ("novantacinque", 95),
        ("novantasei", 96),
        ("novantasette", 97),
        ("novantotto", 98),
        ("novantanove", 99),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral4() -> Rule {
    rule(
        "number (21..29 31..39 41..49 51..59 61..69 71..79 81..89 91..99)",
        vec![regex("((venti|trenta|quaranta|cinquanta|sessanta|settanta|ottanta|novanta)(due|tre|tré|quattro|cinque|sei|sette|nove))|((vent|trent|quarant|cinquant|sessant|settant|ottant|novant)(uno|otto))")],
        |ts| {
            TWENTYONE_NINETYNINE.get(to_lower(groups(ts)?.first()?).as_str()).and_then(|v| integer(*v))
        },
    )
}

fn rule_numerals() -> Rule {
    rule(
        "numbers 200..999",
        vec![
            pred(number_between(2.0, 10.0)),
            number_with(|n| n.value, |v| v == 100.0),
            pred(number_between(0.0, 100.0)),
        ],
        |ts| match (ts.first()?, ts.get(1)?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b), Token::Numeral(c)) => {
                double(a.value * b.value + c.value)
            }
            _ => None,
        },
    )
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule(
        "integer with thousands separator .",
        vec![regex("(\\d{1,3}(([\\. ])\\d\\d\\d){1,5})")],
        |ts| {
            let g = groups(ts)?;
            let m = g.first()?;
            let sep = g.get(2)?;
            parse_double(&m.replace(sep.as_ref(), "")).and_then(double)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_integer_with_thousands_separator(),
        rule_numeral(),
        rule_numeral2(),
        rule_numeral3(),
        rule_numeral4(),
        rule_numeral5(),
        rule_numerals(),
        rule_negative(),
        rule_suffixes(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
