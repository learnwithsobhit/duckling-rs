//! `Duckling.Numeral.EL.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

/// `Duckling.Numeral.Helpers.numeralMapEL`.
pub static NUMERAL_MAP_EL: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("δι", 2),
        ("δί", 2),
        ("τρι", 3),
        ("τρί", 3),
        ("τετρ", 4),
        ("πεντ", 5),
        ("πενθ", 5),
        ("εξ", 6),
        ("επτ", 7),
        ("εφτ", 7),
        ("οκτ", 8),
        ("οχτ", 8),
        ("εννι", 9),
        ("δεκ", 10),
        ("δεκαπεντ", 15),
        ("δεκαπενθ", 15),
        ("εικοσ", 20),
        ("εικοσιπεντ", 25),
        ("εικοσιπενθ", 25),
        ("τριαντ", 30),
        ("τριανταπεντ", 35),
        ("τριανταπενθ", 35),
        ("σαραντ", 40),
        ("σαρανταπεντ", 45),
        ("σαρανταπενθ", 45),
        ("πενηντ", 50),
        ("πενηνταπετν", 55),
        ("πενηνταπετθ", 55),
        ("εξηντ", 60),
        ("ενενηντ", 90),
        ("μιά", 1),
        ("ενά", 1),
        ("δυό", 2),
        ("τρεισή", 3),
        ("τεσσερισή", 4),
        ("τεσσερσή", 4),
        ("πεντέ", 5),
        ("εξί", 6),
        ("επτά", 7),
        ("εφτά", 7),
        ("οκτώ", 8),
        ("οχτώ", 8),
        ("εννιά", 9),
        ("δεκά", 10),
        ("εντεκά", 11),
        ("δωδεκά", 12),
    ]
    .into_iter()
    .collect()
});

static ONE_OR_TWO_DIGITS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("μηδέν", 0),
        ("ένα", 1),
        ("ένας", 1),
        ("ενός", 1),
        ("μία", 1),
        ("μια", 1),
        ("δύο", 2),
        ("δυο", 2),
        ("τρία", 3),
        ("τρεις", 3),
        ("τέσσερα", 4),
        ("τέσσερις", 4),
        ("πέντε", 5),
        ("έξι", 6),
        ("επτά", 7),
        ("εφτά", 7),
        ("οκτώ", 8),
        ("οχτώ", 8),
        ("εννιά", 9),
        ("εννέα", 9),
        ("δέκα", 10),
        ("δεκαριά", 10),
        ("έντεκα", 11),
        ("ένδεκα", 11),
        ("δώδεκα", 12),
        ("ντουζίνα", 12),
        ("ντουζίνες", 12),
        ("δεκατρία", 13),
        ("δεκατέσσερα", 14),
        ("δεκαπέντε", 15),
        ("δεκαέξι", 16),
        ("δεκαεπτά", 17),
        ("δεκαοκτώ", 18),
        ("δεκαεννέα", 19),
        ("δεκαεννιά", 19),
        ("είκοσι", 20),
        ("τριάντα", 30),
        ("σαράντα", 40),
        ("πενήντα", 50),
        ("εξήντα", 60),
        ("εβδομήντα", 70),
        ("ογδόντα", 80),
        ("ενενήντα", 90),
    ]
    .into_iter()
    .collect()
});

static HUNDREDS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("δι", 200),
        ("τρι", 300),
        ("τετρ", 400),
        ("πεντ", 500),
        ("εξ", 600),
        ("επτ", 700),
        ("εφτ", 700),
        ("οκτ", 800),
        ("οχτ", 800),
        ("εννι", 900),
    ]
    .into_iter()
    .collect()
});

fn lookup_i64(map: &HashMap<&'static str, i64>, s: &str) -> Option<i64> {
    map.get(to_lower(s).as_str()).copied()
}

fn rule_few() -> Rule {
    rule("few", vec![regex("μερικ(ά|ές|οί)")], |_| integer(3))
}

fn rule_numeral() -> Rule {
    rule(
        "number (0..19, 20, 30..90)",
        vec![regex(
            "(μηδέν|[εέ]ν[αοό]ς?|μ[ιί]ας?|δ[υύ]ο|τρ(ία|εις)|τέσσερ(α|ις)|πέντε|έξι|ε[πφ]τά|ο[κχ]τώ|ενν(ιά|έα)|δέκα|δεκαριά|έν[τδ]εκα|δώδεκα|ντουζίν(α|ες)|δεκα(τρία|τέσσερα|πέντε|έξι|ε[πφ]τά|ο[χκ]τώ|ενν(έα|ιά))|είκοσι|(τριά|σαρά|πενή|εξή|εβδομή|ογδό|ενενή)ντα)",
        )],
        |ts| lookup_i64(&ONE_OR_TWO_DIGITS, groups(ts)?.first()?).and_then(integer),
    )
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer 21..99",
        vec![
            one_of(vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]),
            pred(number_between(1.0, 10.0)),
        ],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(tens), Token::Numeral(units)) => double(tens.value + units.value),
            _ => None,
        },
    )
}

fn rule_hundred() -> Rule {
    rule("number (100)", vec![regex("(εκατόν?)")], |_| {
        integer(100).and_then(|t| with_grain(2, &t))
    })
}

fn rule_hundreds() -> Rule {
    rule(
        "number (200..900)",
        vec![regex("(δι|τρι|τετρ|πεντ|εξ|ε(π|φ)τ|ο(χ|κ)τ|εννι)ακόσι(α|ες|οι)")],
        |ts| {
            lookup_i64(&HUNDREDS, groups(ts)?.first()?)
                .and_then(integer)
                .and_then(|t| with_grain(2, &t))
        },
    )
}

fn rule_powers_of_ten() -> Rule {
    rule(
        "powers of tens",
        vec![regex("(χίλι(α|οι|ες)|χιλιάδες|εκατομμύρι(ο|α)|δις|δισεκατομμύριο)")],
        |ts| {
            let (v, grain) = match to_lower(groups(ts)?.first()?).as_str() {
                "χίλια" => (1e3, 3),
                "χιλιάδες" => (1e3, 3),
                "εκατομμύριο" => (1e6, 6),
                "εκατομμύρια" => (1e6, 6),
                "δις" => (1e9, 9),
                "δισεκατομμύριο" => (1e9, 9),
                "δισεκατομμύρια" => (1e9, 9),
                _ => return None,
            };
            double(v).and_then(|t| with_grain(grain, &t)).and_then(|t| with_multipliable(&t))
        },
    )
}

fn rule_negative() -> Rule {
    rule(
        "negative numbers",
        vec![regex("-|μείον"), pred(is_positive)],
        |ts| match ts.get(1)? {
            Token::Numeral(nd) => double(nd.value * -1.0),
            _ => None,
        },
    )
}

fn rule_sum() -> Rule {
    rule(
        "intersect 2 numbers",
        vec![pred(has_grain), pred(|t| !is_multipliable(t) && is_positive(t))],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(a), Token::Numeral(b)) => {
                let g = a.grain?;
                if 10f64.powf(g as f64) > b.value {
                    double(a.value + b.value)
                } else {
                    None
                }
            }
            _ => None,
        },
    )
}

fn rule_multiply() -> Rule {
    rule(
        "compose by multiplication",
        vec![dimension(Dim::Numeral), pred(is_multipliable)],
        |ts| multiply(ts.first()?, ts.get(1)?),
    )
}

fn rule_decimals() -> Rule {
    rule("decimal number", vec![regex("(\\d+,\\d+)")], |ts| {
        parse_decimal(true, &groups(ts)?.first()?.replace(',', "."))
    })
}

fn rule_comma_spelled_out() -> Rule {
    rule(
        "one point two",
        vec![dimension(Dim::Numeral), regex("κόμμα"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + decimals_to_double(b.value)),
            _ => None,
        },
    )
}

fn rule_dots() -> Rule {
    rule(
        "dot-separated numbers",
        vec![regex("(\\d+(\\.\\d\\d\\d)+(,\\d+)?)")],
        |ts| {
            let fmt = groups(ts)?.first()?.replace('.', "").replace(',', ".");
            parse_double(&fmt).and_then(double)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_few(),
        rule_numeral(),
        rule_composite_tens(),
        rule_powers_of_ten(),
        rule_negative(),
        rule_hundred(),
        rule_hundreds(),
        rule_sum(),
        rule_multiply(),
        rule_decimals(),
        rule_comma_spelled_out(),
        rule_dots(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
