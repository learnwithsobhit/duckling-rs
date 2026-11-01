//! `Duckling.Ordinal.PL.Rules`.

use once_cell::sync::Lazy;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::parse_int;
use crate::types::{dimension, groups, regex, rule, Dim, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

fn ov(t: &Token) -> Option<i64> {
    match t {
        Token::Ordinal(o) => Some(o.value),
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleFifthOrdinal
        rule("fifth ordinal", vec![regex("pi(a|ą)t(y|ego|emu|m|(a|ą)|ej)")], |_| ordinal(5)),
        // ruleFirstOrdinal
        rule("first ordinal", vec![regex("pierw?sz(y|ego|emu|m|(a|ą)|ej)")], |_| ordinal(1)),
        // ruleFourthOrdinal
        rule("fourth ordinal", vec![regex("czwart(y|ego|emu|ym|(a|ą)|ej)")], |_| ordinal(4)),
        // ruleNdOrdinalNoSpace
        rule(
            "22nd ordinal no space",
            vec![regex("dwudziest(ym|y|ego|emu|(a|ą)|ej)drugi?(ego|emu|m|(a|ą)|ej)?")],
            |_| ordinal(22),
        ),
        // ruleOrdinalDigits
        rule(
            "ordinal (digits)",
            vec![regex("0*(\\d+)( |-)?(szy|sza|szym|ego|go|szego|gi(ego|ej)?|st(a|y|ej)|t(ej|y|ego)|ci(ego)?)")],
            |ts| parse_int(groups(ts)?.first()?).and_then(ordinal),
        ),
        // ruleRdOrdinalNoSpace
        rule(
            "23rd ordinal no space",
            vec![regex("dwudziest(ym|y|ego|emu|(a|ą)|ej)trzeci(ego|ch|emu|m|mi|ej|(a|ą))?")],
            |_| ordinal(23),
        ),
        // ruleSecondOrdinal
        rule("second ordinal", vec![regex("drugi?(ego|emu|m|(a|ą)|ej)?")], |_| ordinal(2)),
        // ruleSeventhOrdinal
        rule("seventh ordinal", vec![regex("si(o|ó)dm(y|ego|emu|m|(a|ą)|ej)")], |_| ordinal(7)),
        // ruleSixthOrdinal
        rule("sixth ordinal", vec![regex("sz(o|ó)st(y|ego|emu|m|(a|ą)|ej)")], |_| ordinal(6)),
        // ruleStOrdinalNoSpace
        rule(
            "21st ordinal no space",
            vec![regex("dwudziest(ym|y|ego|emu|(a|ą)|ej)pierw?sz(y|ego|emu|m|(a|ą)|ej)")],
            |_| ordinal(21),
        ),
        // ruleThOrdinal
        rule("8th ordinal", vec![regex("(o|ó|Ó)sm(y|ego|emu|m|(a|ą)|ej)")], |_| ordinal(8)),
        // ruleThOrdinal10
        rule("17th ordinal", vec![regex("siedemnast(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(17)),
        // ruleThOrdinal11
        rule("18th ordinal", vec![regex("osiemnast(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(18)),
        // ruleThOrdinal12
        rule("19th ordinal", vec![regex("dziewi(ę|e)tnast(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(19)),
        // ruleThOrdinal13
        rule("20th ordinal", vec![regex("dwudziest(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(20)),
        // ruleThOrdinal14
        rule(
            "21-29th ordinal",
            vec![regex("dwudziest(ym|y|ego|emu|(a|ą)|ej)( |-)?"), dimension(Dim::Ordinal)],
            |ts| ordinal(20 + ov(ts.get(1)?)?),
        ),
        // ruleThOrdinal15
        rule("30th ordinal", vec![regex("trzydziest(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(30)),
        // ruleThOrdinal16
        rule(
            "31-39th ordinal",
            vec![regex("trzydziest(ym|y|ego|emu|(a|ą)|ej)( |-)?"), dimension(Dim::Ordinal)],
            |ts| ordinal(30 + ov(ts.get(1)?)?),
        ),
        // ruleThOrdinal2
        rule("9th ordinal", vec![regex("dziewi(a|ą)t(ym|y|ego|em|emu|(a|ą)|ej)")], |_| ordinal(9)),
        // ruleThOrdinal3
        rule("10th ordinal", vec![regex("dziesi(a|ą)t(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(10)),
        // ruleThOrdinal4
        rule("11th ordinal", vec![regex("jedenast(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(11)),
        // ruleThOrdinal5
        rule("12th ordinal", vec![regex("dwunast(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(12)),
        // ruleThOrdinal6
        rule("13th ordinal", vec![regex("trzynast(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(13)),
        // ruleThOrdinal7
        rule("14th ordinal", vec![regex("czternast(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(14)),
        // ruleThOrdinal8
        rule("15th ordinal", vec![regex("pi(e|ę)tnast(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(15)),
        // ruleThOrdinal9
        rule("16th ordinal", vec![regex("szesnast(ym|y|ego|emu|(a|ą)|ej)")], |_| ordinal(16)),
        // ruleThOrdinalNoSpace
        rule(
            "24th ordinal no space",
            vec![regex("dwudziest(ym|y|ego|emu|(a|ą)|ej)czwart(y|ego|emu|ym|(a|ą)|ej)")],
            |_| ordinal(24),
        ),
        // ruleThirdOrdinal
        rule("third ordinal", vec![regex("trzeci(ego|ch|emu|m|mi|ej|(a|ą))?")], |_| ordinal(3)),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
