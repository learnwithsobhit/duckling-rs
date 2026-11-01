//! `Duckling.Ordinal.HE.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::parse_int;
use crate::types::{dimension, group, regex, rule, Dim, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

fn ov(t: &Token) -> Option<i64> {
    match t {
        Token::Ordinal(o) => Some(o.value),
        _ => None,
    }
}

fn rule_ordinal4() -> Rule {
    rule("ordinal 4", vec![regex("(ארבעה|רביעי)")], |_| ordinal(4))
}

fn rule_ordinal9() -> Rule {
    rule("ordinal 9", vec![regex("(תשעה|תשיעי)")], |_| ordinal(9))
}

fn rule_ordinal10() -> Rule {
    rule("ordinal 10", vec![regex("(עשרה|עשירי)")], |_| ordinal(10))
}

fn rule_ordinal12() -> Rule {
    rule("ordinal 12", vec![regex("(שניים עשר|תרי עשר)")], |_| ordinal(12))
}

fn rule_ordinal17() -> Rule {
    rule("ordinal 17", vec![regex("(שבע(ה)? עשר(ה)?)")], |_| ordinal(17))
}

fn rule_ordinal18() -> Rule {
    rule("ordinal 18", vec![regex("(שמונה עשר(ה)?)")], |_| ordinal(18))
}

fn rule_ordinal_digits() -> Rule {
    rule("ordinal (digits)", vec![regex("0*(\\d+) ")], |ts| {
        parse_int(group(ts, 0)?.as_ref()).and_then(ordinal)
    })
}

fn rule_ordinal15() -> Rule {
    rule("ordinal 15", vec![regex("(חמישה עשר|חמש עשרה?)")], |_| ordinal(15))
}

fn rule_ordinal5() -> Rule {
    rule("ordinal 5", vec![regex("(חמישי|חמישה)")], |_| ordinal(5))
}

fn rule_ordinal16() -> Rule {
    rule("ordinal 16", vec![regex("(שש(ה)? עשר(ה)?)")], |_| ordinal(16))
}

fn rule_ordinal14() -> Rule {
    rule("ordinal 14", vec![regex("(ארבע(ה)? עשר(ה)?)")], |_| ordinal(14))
}

fn rule_ordinal20() -> Rule {
    rule(
        "ordinal 20..90",
        vec![regex("(עשרים|שלושים|ארבעים|חמישים|שישים|שבעים|שמונים|תשעים)")],
        |ts| match group(ts, 0)?.as_ref() {
            "עשרים" => ordinal(20),
            "שלושים" => ordinal(30),
            "ארבעים" => ordinal(40),
            "חמישים" => ordinal(50),
            "שישים" => ordinal(60),
            "שבעים" => ordinal(70),
            "שמונים" => ordinal(80),
            "תשעים" => ordinal(90),
            _ => None,
        },
    )
}

fn rule_ordinal() -> Rule {
    rule("ordinal 1", vec![regex("(אחד|ראשון)")], |_| ordinal(1))
}

fn rule_ordinal13() -> Rule {
    rule("ordinal 13", vec![regex("(שלוש(ה)? עשר(ה)?)")], |_| ordinal(13))
}

fn rule_ordinal7() -> Rule {
    rule("ordinal 7", vec![regex("(שבעה|שביעי)")], |_| ordinal(7))
}

fn rule_ordinal8() -> Rule {
    rule("ordinal 8", vec![regex("(שמונה|שמיני)")], |_| ordinal(8))
}

fn rule_ordinal2() -> Rule {
    rule("ordinal 2", vec![regex("(שתיים|שניים|שני)")], |_| ordinal(2))
}

fn rule_ordinal11() -> Rule {
    rule("ordinal 11", vec![regex("(אחד עשר(ה)?)")], |_| ordinal(11))
}

fn rule_ordinal3() -> Rule {
    rule("ordinal 3", vec![regex("(שלושה|שלישי)")], |_| ordinal(3))
}

fn rule_ordinal6() -> Rule {
    rule("ordinal 6", vec![regex("(ששה|שישי)")], |_| ordinal(6))
}

fn rule_ordinal19() -> Rule {
    rule("ordinal 19", vec![regex("(תשע(ה)? עשר(ה)?)")], |_| ordinal(19))
}

fn rule_composite_with_and() -> Rule {
    rule(
        "ordinal composition (with and)",
        vec![dimension(Dim::Ordinal), regex("ו"), dimension(Dim::Ordinal)],
        |ts| ordinal(ov(ts.first()?)? + ov(ts.get(2)?)?),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_composite_with_and(),
        rule_ordinal(),
        rule_ordinal10(),
        rule_ordinal11(),
        rule_ordinal12(),
        rule_ordinal13(),
        rule_ordinal14(),
        rule_ordinal15(),
        rule_ordinal16(),
        rule_ordinal17(),
        rule_ordinal18(),
        rule_ordinal19(),
        rule_ordinal2(),
        rule_ordinal20(),
        rule_ordinal3(),
        rule_ordinal4(),
        rule_ordinal5(),
        rule_ordinal6(),
        rule_ordinal7(),
        rule_ordinal8(),
        rule_ordinal9(),
        rule_ordinal_digits(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
