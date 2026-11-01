//! `Duckling.Numeral.TR.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{dimension, groups, pred, regex, rule, Dim, Rule, Token};

fn lookup_i64(map: &HashMap<&str, i64>, ts: &[&Token]) -> Option<Token> {
    let g = groups(ts)?;
    map.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
}

fn lookup_f64(map: &HashMap<&str, f64>, ts: &[&Token]) -> Option<Token> {
    let g = groups(ts)?;
    map.get(to_lower(g.first()?).as_str()).copied().and_then(double)
}

static HUNDREDS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("yüz", 100),
        ("ikiyüz", 200),
        ("üçyüz", 300),
        ("dörtyüz", 400),
        ("beşyüz", 500),
        ("altıyüz", 600),
        ("yediyüz", 700),
        ("sekizyüz", 800),
        ("dokuzyüz", 900),
    ]
    .into_iter()
    .collect()
});

fn rule_integer5() -> Rule {
    rule(
        "integer 100..900",
        vec![regex("(yüz|ikiyüz|üçyüz|dörtyüz|beşyüz|altıyüz|yediyüz|sekizyüz|dokuzyüz)")],
        |ts| lookup_i64(&HUNDREDS_MAP, ts).and_then(|t| with_grain(2, &t)),
    )
}

fn rule_numerals_prefix_with_negative_or_minus() -> Rule {
    rule(
        "numbers prefix with -, negative or minus",
        vec![regex("-|eksi|negatif"), pred(is_positive)],
        |ts| match ts.get(1)? {
            Token::Numeral(n) => double(n.value * -1.0),
            _ => None,
        },
    )
}

fn rule_a_couple_of() -> Rule {
    rule("a couple (of)", vec![regex("(bir )?çift")], |_| {
        integer(2).and_then(|t| with_grain(1, &t))
    })
}

fn rule_few() -> Rule {
    rule("few", vec![regex("(bir)?az")], |_| integer(3))
}

fn rule_ten() -> Rule {
    rule("ten", vec![regex("on")], |_| {
        integer(10).and_then(|t| with_grain(1, &t)).and_then(|t| with_multipliable(&t))
    })
}

fn rule_decimal_with_thousands_separator() -> Rule {
    rule("decimal with thousands separator", vec![regex("(\\d+(\\.\\d\\d\\d)+,\\d+)")], |ts| {
        let g = groups(ts)?;
        let fmt = g.first()?.replace('.', "").replace(',', ".");
        parse_double(&fmt).and_then(double)
    })
}

fn rule_multiply() -> Rule {
    rule("compose by multiplication", vec![dimension(Dim::Numeral), pred(is_multipliable)], |ts| {
        match (ts.first()?, ts.get(1)?) {
            (
                Token::Numeral(n1),
                Token::Numeral(NumeralData { value: val2, grain: g2, .. }),
            ) if g2.is_none() || (g2.is_some() && *val2 > n1.value) => match g2 {
                None => double(n1.value * val2),
                Some(g) => double(n1.value * val2).and_then(|t| with_grain(*g, &t)),
            },
            _ => None,
        }
    })
}

fn rule_decimal_numeral() -> Rule {
    rule("decimal number", vec![regex("(\\d*,\\d+)")], |ts| {
        parse_decimal(false, groups(ts)?.first()?)
    })
}

static NUMERAL_SUFFIXES_HALFSUFFIX_TEXT_MAP: Lazy<HashMap<&'static str, f64>> = Lazy::new(|| {
    [
        ("birbuçuk", 1.5),
        ("bibuçuk", 1.5),
        ("ikibuçuk", 2.5),
        ("üçbuçuk", 3.5),
        ("dörtbuçuk", 4.5),
        ("beşbuçuk", 5.5),
        ("altıbuçuk", 6.5),
        ("yedibuçuk", 7.5),
        ("sekizbuçuk", 8.5),
        ("dokuzbuçuk", 9.5),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral_suffixes_halfsuffix_text() -> Rule {
    rule(
        "number suffixes (half-suffix text) (1..9)",
        vec![regex("((bir?|iki|üçü?|dört|beş|altı|yedi|sekiz|dokuz)(buçuk))")],
        |ts| lookup_f64(&NUMERAL_SUFFIXES_HALFSUFFIX_TEXT_MAP, ts),
    )
}

static TEN_TO_NINETYNINE_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("onbi", 11),
        ("onbir", 11),
        ("oniki", 12),
        ("onüç", 13),
        ("ondört", 14),
        ("onbeş", 15),
        ("onaltı", 16),
        ("onyedi", 17),
        ("onsekiz", 18),
        ("ondokuz", 19),
        ("yirmibi", 21),
        ("yirmibir", 21),
        ("yirmiiki", 22),
        ("yirmiüç", 23),
        ("yirmidört", 24),
        ("yirmibeş", 25),
        ("yirmialtı", 26),
        ("yirmiyedi", 27),
        ("yirmisekiz", 28),
        ("yirmidokuz", 29),
        ("otuzbi", 31),
        ("otuzbir", 31),
        ("otuziki", 32),
        ("otuzüç", 33),
        ("otuzdört", 34),
        ("otuzbeş", 35),
        ("otuzaltı", 36),
        ("otuzyedi", 37),
        ("otuzsekiz", 38),
        ("otuzdokuz", 39),
        ("kırkbir", 41),
        ("kırkbi", 41),
        ("kırkiki", 42),
        ("kırküç", 43),
        ("kırkdört", 44),
        ("kırkbeş", 45),
        ("kırkaltı", 46),
        ("kırkyedi", 47),
        ("kırksekiz", 48),
        ("kırkdokuz", 49),
        ("ellibi", 51),
        ("ellibir", 51),
        ("elliiki", 52),
        ("elliüç", 53),
        ("ellidört", 54),
        ("ellibeş", 55),
        ("ellialtı", 56),
        ("elliyedi", 57),
        ("ellisekiz", 58),
        ("ellidokuz", 59),
        ("altmışbir", 61),
        ("atmışbir", 61),
        ("atmışiki", 62),
        ("altmışiki", 62),
        ("atmışüç", 63),
        ("altmışüç", 63),
        ("atmışdört", 64),
        ("altmışdört", 64),
        ("atmışbeş", 65),
        ("altmışbeş", 65),
        ("atmışaltı", 66),
        ("altmışaltı", 66),
        ("altmışyedi", 67),
        ("atmışyedi", 67),
        ("altmışsekiz", 68),
        ("atmışsekiz", 68),
        ("atmışdokuz", 69),
        ("altmışdokuz", 69),
        ("yetmişbir", 71),
        ("yetmişbi", 71),
        ("yetmişiki", 72),
        ("yetmişüç", 73),
        ("yetmişdört", 74),
        ("yetmişbeş", 75),
        ("yetmişaltı", 76),
        ("yetmişyedi", 77),
        ("yetmişsekiz", 78),
        ("yetmişdokuz", 79),
        ("seksenbir", 81),
        ("seksenbi", 81),
        ("sekseniki", 82),
        ("seksenüç", 83),
        ("seksendört", 84),
        ("seksenbeş", 85),
        ("seksenaltı", 86),
        ("seksenyedi", 87),
        ("seksensekiz", 88),
        ("seksendokuz", 89),
        ("doksanbi", 91),
        ("doksanbir", 91),
        ("doksaniki", 92),
        ("doksanüç", 93),
        ("doksandört", 94),
        ("doksanbeş", 95),
        ("doksanaltı", 96),
        ("doksanyedi", 97),
        ("doksansekiz", 98),
        ("doksandokuz", 99),
    ]
    .into_iter()
    .collect()
});

fn rule_integer3() -> Rule {
    rule(
        "integer 11..19 21..29 31..39 41..49 51..59 61..69 71..79 81..89 91..99",
        vec![regex(
            "((on|yirmi|otuz|kırk|elli|atmış|altmış|yetmiş|seksen|doksan)(bir|bi|iki|üç|dört|beş|altı|yedi|sekiz|dokuz))",
        )],
        |ts| lookup_i64(&TEN_TO_NINETYNINE_MAP, ts),
    )
}

static THOUSANDS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("bin", 1000),
        ("ikibin", 2000),
        ("üçbin", 3000),
        ("dörtbin", 4000),
        ("beşbin", 5000),
        ("altıbin", 6000),
        ("yedibin", 7000),
        ("sekizbin", 8000),
        ("dokuzbin", 9000),
    ]
    .into_iter()
    .collect()
});

fn rule_integer6() -> Rule {
    rule(
        "integer 1000..9000",
        vec![regex("(bin|ikibin|üçbin|dörtbin|beşbin|altıbin|yedibin|sekizbin|dokuzbin)")],
        |ts| lookup_i64(&THOUSANDS_MAP, ts).and_then(|t| with_grain(3, &t)),
    )
}

fn rule_intersect() -> Rule {
    rule(
        "intersect",
        vec![pred(has_grain), pred(|t| !is_multipliable(t) && is_positive(t))],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(a), Token::Numeral(b)) => {
                let g = a.grain?;
                if 10f64.powi(g as i32) > b.value {
                    double(a.value + b.value)
                } else {
                    None
                }
            }
            _ => None,
        },
    )
}

fn rule_numerals_suffixes_kmg() -> Rule {
    rule(
        "numbers suffixes (K, M, G)",
        vec![dimension(Dim::Numeral), regex("([kmgb])(?=[\\W\\$€]|$)")],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(nd), Token::RegexMatch(g)) => match to_lower(g.0.first()?).as_str() {
                "k" => double(nd.value * 1e3),
                "b" => double(nd.value * 1e3),
                "m" => double(nd.value * 1e6),
                "g" => double(nd.value * 1e9),
                _ => None,
            },
            _ => None,
        },
    )
}

fn rule_powers_of_ten() -> Rule {
    rule("powers of tens", vec![regex("(y(ü)z|bin|milyon)")], |ts| {
        let g = groups(ts)?;
        match to_lower(g.first()?).as_str() {
            "yüz" => double(1e2).and_then(|t| with_grain(2, &t)).and_then(|t| with_multipliable(&t)),
            "bin" => double(1e3).and_then(|t| with_grain(3, &t)).and_then(|t| with_multipliable(&t)),
            "milyon" => {
                double(1e6).and_then(|t| with_grain(6, &t)).and_then(|t| with_multipliable(&t))
            }
            _ => None,
        }
    })
}

static TEN_THOUSANDS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("onbin", 10000),
        ("yirmibin", 20000),
        ("otuzbin", 30000),
        ("kırkbin", 40000),
        ("ellibin", 50000),
        ("altmışbin", 60000),
        ("atmışbin", 60000),
        ("yetmişbin", 70000),
        ("seksenbin", 80000),
        ("doksanbin", 90000),
    ]
    .into_iter()
    .collect()
});

fn rule_integer7() -> Rule {
    rule(
        "integer 10000..90000",
        vec![regex(
            "(onbin|yirmibin|otuzbin|kırkbin|ellibin|atmışbin|altmışbin|yetmişbin|seksenbin|doksanbin)",
        )],
        |ts| lookup_i64(&TEN_THOUSANDS_MAP, ts).and_then(|t| with_grain(4, &t)),
    )
}

static HUNDRED_THOUSANDS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("yüzbin", 100000),
        ("ikiyüzbin", 200000),
        ("üçyüzbin", 300000),
        ("dörtyüzbin", 400000),
        ("beşyüzbin", 500000),
        ("altıyüzbin", 600000),
        ("yediyüzbin", 700000),
        ("sekizyüzbin", 800000),
        ("dokuzyüzbin", 900000),
    ]
    .into_iter()
    .collect()
});

fn rule_integer8() -> Rule {
    rule(
        "integer 100000..900000",
        vec![regex(
            "(yüzbin|ikiyüzbin|üçyüzbin|dörtyüzbin|beşyüzbin|altıyüzbin|yediyüzbin|sekizyüzbin|dokuzyüzbin)",
        )],
        |ts| lookup_i64(&HUNDRED_THOUSANDS_MAP, ts).and_then(|t| with_grain(5, &t)),
    )
}

fn rule_half() -> Rule {
    rule("half", vec![regex("(yarım)")], |_| double(0.5))
}

static INTEGER9_MAP: Lazy<HashMap<&'static str, f64>> = Lazy::new(|| {
    [
        ("onbirbuçuk", 11.5),
        ("onbibuçuk", 11.5),
        ("onikibuçuk", 12.5),
        ("onüçbuçuk", 13.5),
        ("ondörtbuçuk", 14.5),
        ("onbeşbuçuk", 15.5),
        ("onaltıbuçuk", 16.5),
        ("onyedibuçuk", 17.5),
        ("onsekizbuçuk", 18.5),
        ("ondokuzbuçuk", 19.5),
        ("yirmibibuçuk", 21.5),
        ("yirmibirbuçuk", 21.5),
        ("yirmiikibuçuk", 22.5),
        ("yirmiüçbuçuk", 23.5),
        ("yirmidörtbuçuk", 24.5),
        ("yirmibeşbuçuk", 25.5),
        ("yirmialtıbuçuk", 26.5),
        ("yirmiyedibuçuk", 27.5),
        ("yirmisekizbuçuk", 28.5),
        ("yirmidokuzbuçuk", 29.5),
        ("otuzbibuçuk", 31.5),
        ("otuzbirbuçuk", 31.5),
        ("otuzikibuçuk", 32.5),
        ("otuzüçbuçuk", 33.5),
        ("otuzdörtbuçuk", 34.5),
        ("otuzbeşbuçuk", 35.5),
        ("otuzaltıbuçuk", 36.5),
        ("otuzyedibuçuk", 37.5),
        ("otuzsekizbuçuk", 38.5),
        ("otuzdokuzbuçuk", 39.5),
        ("kırkbirbuçuk", 41.5),
        ("kırkbibuçuk", 41.5),
        ("kırkikibuçuk", 42.5),
        ("kırküçbuçuk", 43.5),
        ("kırkdörtbuçuk", 44.5),
        ("kırkbeşbuçuk", 45.5),
        ("kırkaltıbuçuk", 46.5),
        ("kırkyedibuçuk", 47.5),
        ("kırksekizbuçuk", 48.5),
        ("kırkdokuzbuçuk", 49.5),
        ("ellibibuçuk", 51.5),
        ("ellibirbuçuk", 51.5),
        ("elliikibuçuk", 52.5),
        ("elliüçbuçuk", 53.5),
        ("ellidörtbuçuk", 54.5),
        ("ellibeşbuçuk", 55.5),
        ("ellialtıbuçuk", 56.5),
        ("elliyedibuçuk", 57.5),
        ("ellisekizbuçuk", 58.5),
        ("ellidokuzbuçuk", 59.5),
        ("altmışbirbuçuk", 61.5),
        ("atmışbirbuçuk", 61.5),
        ("altmışikibuçuk", 62.5),
        ("atmışikibuçuk", 62.5),
        ("atmışüçbuçuk", 63.5),
        ("altmışüçbuçuk", 63.5),
        ("altmışdörtbuçuk", 64.5),
        ("atmışdörtbuçuk", 64.5),
        ("altmışbeşbuçuk", 65.5),
        ("atmışbeşbuçuk", 65.5),
        ("altmışaltıbuçuk", 66.5),
        ("atmışaltıbuçuk", 66.5),
        ("atmışyedibuçuk", 67.5),
        ("altmışyedibuçuk", 67.5),
        ("altmışsekizbuçuk", 68.5),
        ("atmışsekizbuçuk", 68.5),
        ("altmışdokuzbuçuk", 69.5),
        ("atmışdokuzbuçuk", 69.5),
        ("yetmişbibuçuk", 71.5),
        ("yetmişbirbuçuk", 71.5),
        ("yetmişikibuçuk", 72.5),
        ("yetmişüçbuçuk", 73.5),
        ("yetmişdörtbuçuk", 74.5),
        ("yetmişbeşbuçuk", 75.5),
        ("yetmişaltıbuçuk", 76.5),
        ("yetmişyedibuçuk", 77.5),
        ("yetmişsekizbuçuk", 78.5),
        ("yetmişdokuzbuçuk", 79.5),
        ("seksenbibuçuk", 81.5),
        ("seksenbirbuçuk", 81.5),
        ("seksenikibuçuk", 82.5),
        ("seksenüçbuçuk", 83.5),
        ("seksendörtbuçuk", 84.5),
        ("seksenbeşbuçuk", 85.5),
        ("seksenaltıbuçuk", 86.5),
        ("seksenyedibuçuk", 87.5),
        ("seksensekizbuçuk", 88.5),
        ("seksendokuzbuçuk", 89.5),
        ("doksanbirbuçuk", 91.5),
        ("doksanbibuçuk", 91.5),
        ("doksanikibuçuk", 92.5),
        ("doksanüçbuçuk", 93.5),
        ("doksandörtbuçuk", 94.5),
        ("doksanbeşbuçuk", 95.5),
        ("doksanaltıbuçuk", 96.5),
        ("doksanyedibuçuk", 97.5),
        ("doksansekizbuçuk", 98.5),
        ("doksandokuzbuçuk", 99.5),
    ]
    .into_iter()
    .collect()
});

fn rule_integer9() -> Rule {
    rule(
        "integer 11..19 21..29 31..39 41..49 51..59 61..69 71..79 81..89 91..99",
        vec![regex(
            "((on|yirmi|otuz|kırk|elli|atmış|altmış|yetmiş|seksen|doksan)(bir|bi|iki|üçü?|dört|beş|altı|yedi|sekiz|dokuz)(buçuk))",
        )],
        |ts| lookup_f64(&INTEGER9_MAP, ts),
    )
}

fn rule_dozen() -> Rule {
    rule("dozen", vec![regex("düzine")], |_| {
        integer(12).and_then(|t| with_grain(1, &t)).and_then(|t| with_multipliable(&t))
    })
}

static ONE_TO_NINE_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("sıfır", 0),
        ("yok", 0),
        ("hiç", 0),
        ("bir", 1),
        ("bi", 1),
        ("yek", 1),
        ("tek", 1),
        ("iki", 2),
        ("üç", 3),
        ("üçü", 3),
        ("dört", 4),
        ("beş", 5),
        ("altı", 6),
        ("yedi", 7),
        ("sekiz", 8),
        ("dokuz", 9),
    ]
    .into_iter()
    .collect()
});

fn rule_integer() -> Rule {
    rule(
        "integer (0..9)",
        vec![regex(
            "(yok|hi(ç)|s(ı)f(ı)r|bir?|[ty]ek|iki|(ü)(ç)(ü)?|d(ö)rt|be(ş)|alt(ı)|yedi|sekiz|dokuz)",
        )],
        |ts| lookup_i64(&ONE_TO_NINE_MAP, ts),
    )
}

static NUMERAL_SUFFIXES_HALFSUFFIX_TEXT2_MAP: Lazy<HashMap<&'static str, f64>> = Lazy::new(|| {
    [
        ("onbuçuk", 10.5),
        ("yirmibuçuk", 20.5),
        ("otuzbuçuk", 30.5),
        ("kırkbuçuk", 40.5),
        ("ellibuçuk", 50.5),
        ("atmışbuçuk", 60.5),
        ("altmışbuçuk", 60.5),
        ("yetmişbuçuk", 70.5),
        ("seksenbuçuk", 80.5),
        ("doksanbuçuk", 90.5),
    ]
    .into_iter()
    .collect()
});

fn rule_numeral_suffixes_halfsuffix_text2() -> Rule {
    rule(
        "number suffixes (half-suffix text) (10..90)",
        vec![regex("((on|yirmi|otuz|kırk|elli|atmış|altmış|yetmiş|seksen|doksan)(buçuk))")],
        |ts| lookup_f64(&NUMERAL_SUFFIXES_HALFSUFFIX_TEXT2_MAP, ts),
    )
}

fn rule_numeral_suffixes_half_suffix() -> Rule {
    rule(
        "number suffixes (half-suffix)",
        vec![dimension(Dim::Numeral), regex("(buçuk)(?=[\\W\\$€]|$)")],
        |ts| match ts.first()? {
            Token::Numeral(n) => double(n.value + 0.5),
            _ => None,
        },
    )
}

fn rule_integer4() -> Rule {
    rule(
        "integer 11..99",
        vec![one_of(vec![70.0, 20.0, 60.0, 50.0, 40.0, 90.0, 30.0, 10.0, 80.0]), pred(number_between(1.0, 10.0))],
        |ts| match (ts.first()?, ts.get(1)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + b.value),
            _ => None,
        },
    )
}

fn rule_group_of_tens() -> Rule {
    rule("group of ten(s)", vec![regex("deste")], |_| {
        integer(10).and_then(|t| with_grain(1, &t)).and_then(|t| with_multipliable(&t))
    })
}

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("on", 10),
        ("yirmi", 20),
        ("otuz", 30),
        ("kırk", 40),
        ("elli", 50),
        ("altmış", 60),
        ("atmış", 60),
        ("yetmiş", 70),
        ("seksen", 80),
        ("doksan", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_integer2() -> Rule {
    rule(
        "integer (10..90)",
        vec![regex("(on|yirmi|otuz|kırk|elli|atmış|altmış|yetmiş|seksen|doksan)")],
        |ts| lookup_i64(&TENS_MAP, ts),
    )
}

fn rule_quarter() -> Rule {
    rule("quarter", vec![regex("(çeyrek)")], |_| double(0.25))
}

fn rule_numeral_dot_numeral() -> Rule {
    rule(
        "number dot number",
        vec![dimension(Dim::Numeral), regex("nokta|virgül"), pred(|t| !has_grain(t))],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(a), Token::Numeral(b)) => double(a.value + decimals_to_double(b.value)),
            _ => None,
        },
    )
}

fn rule_integer_with_thousands_separator() -> Rule {
    rule("integer with thousands separator .", vec![regex("(\\d{1,3}(\\.\\d\\d\\d){1,5})")], |ts| {
        let g = groups(ts)?;
        parse_double(&g.first()?.replace('.', "")).and_then(double)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_a_couple_of(),
        rule_decimal_numeral(),
        rule_decimal_with_thousands_separator(),
        rule_dozen(),
        rule_few(),
        rule_group_of_tens(),
        rule_half(),
        rule_integer(),
        rule_integer2(),
        rule_integer3(),
        rule_integer4(),
        rule_integer5(),
        rule_integer6(),
        rule_integer7(),
        rule_integer8(),
        rule_integer9(),
        rule_integer_with_thousands_separator(),
        rule_intersect(),
        rule_numeral_dot_numeral(),
        rule_numeral_suffixes_half_suffix(),
        rule_numeral_suffixes_halfsuffix_text(),
        rule_numeral_suffixes_halfsuffix_text2(),
        rule_numerals_prefix_with_negative_or_minus(),
        rule_numerals_suffixes_kmg(),
        rule_powers_of_ten(),
        rule_quarter(),
        rule_ten(),
        rule_multiply(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
