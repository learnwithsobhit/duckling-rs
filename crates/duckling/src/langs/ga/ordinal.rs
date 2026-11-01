//! `Duckling.Ordinal.GA.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{group, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

fn ordinal_word(m: &str) -> Option<i64> {
    match m {
        "t-aonu" => Some(1),
        "aonu" => Some(1),
        "aonú" => Some(1),
        "chéad" => Some(1),
        "chead" => Some(1),
        "t-aonú" => Some(1),
        "dara" => Some(2),
        "triú" => Some(3),
        "tríu" => Some(3),
        "tríú" => Some(3),
        "triu" => Some(3),
        "ceathrú" => Some(4),
        "ceathru" => Some(4),
        "cúigiu" => Some(5),
        "cúigiú" => Some(5),
        "cuigiu" => Some(5),
        "cuigiú" => Some(5),
        "séu" => Some(6),
        "séú" => Some(6),
        "seu" => Some(6),
        "seú" => Some(6),
        "seachtu" => Some(7),
        "seachtú" => Some(7),
        "t-ochtú" => Some(8),
        "ochtu" => Some(8),
        "t-ochtu" => Some(8),
        "ochtú" => Some(8),
        "naou" => Some(9),
        "naoú" => Some(9),
        "deichiu" => Some(10),
        "deichiú" => Some(10),
        "fichiu" => Some(20),
        "fichiú" => Some(20),
        "tríochadu" => Some(30),
        "triochadu" => Some(30),
        "tríochadú" => Some(30),
        "triochadú" => Some(30),
        "daicheadú" => Some(40),
        "daicheadu" => Some(40),
        "caogadu" => Some(50),
        "caogadú" => Some(50),
        "seascadu" => Some(60),
        "seascadú" => Some(60),
        "seachtodu" => Some(70),
        "seachtodú" => Some(70),
        "seachtódú" => Some(70),
        "seachtódu" => Some(70),
        "ochtódu" => Some(80),
        "ochtodu" => Some(80),
        "t-ochtodu" => Some(80),
        "t-ochtódú" => Some(80),
        "t-ochtodú" => Some(80),
        "ochtódú" => Some(80),
        "t-ochtódu" => Some(80),
        "ochtodú" => Some(80),
        "nóchadú" => Some(90),
        "nóchadu" => Some(90),
        "nochadú" => Some(90),
        "nochadu" => Some(90),
        "céadú" => Some(100),
        "ceadú" => Some(100),
        "ceadu" => Some(100),
        "céadu" => Some(100),
        "miliu" => Some(1000),
        "miliú" => Some(1000),
        "milliunú" => Some(1_000_000),
        "milliunu" => Some(1_000_000),
        "milliúnu" => Some(1_000_000),
        "milliúnú" => Some(1_000_000),
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleOrdinalDigits
        rule("ordinal (digits)", vec![regex("0*(\\d+) ?(adh|a|d|ú|u)")], |ts| {
            parse_int(&group(ts, 0)?).and_then(ordinal)
        }),
        // ruleOrdinalsChadDaraEtc
        rule(
            "ordinals (chéad, dara, etc.)",
            vec![regex(
                "(ch(é|e)ad|aon(ú|u)|t-aon(ú|u)|dara|tr(í|i)(ú|u)|ceathr(ú|u)|c(ú|u)igi(ú|u)|s(é|e)(ú|u)|seacht(ú|u)|ocht(ú|u)|t-ocht(ú|u)|nao(ú|u)|deichi(ú|u)|fichi(ú|u)|tr(í|i)ochad(ú|u)|daichead(ú|u)|caogad(ú|u)|seascad(ú|u)|seacht(ó|o)d(ú|u)|ocht(ó|o)d(ú|u)|t-ocht(ó|o)d(ú|u)|n(ó|o)chad(ú|u)|c(é|e)ad(ú|u)|mili(ú|u)|milli(ú|u)n(ú|u))",
            )],
            |ts| ordinal_word(&to_lower(&group(ts, 0)?)).and_then(ordinal),
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
