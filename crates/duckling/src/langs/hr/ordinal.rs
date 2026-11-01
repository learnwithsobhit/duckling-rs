//! `Duckling.Ordinal.HR.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

static ORDINALS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("prva", 1),
        ("prvi", 1),
        ("prvoga", 1),
        ("prvo", 1),
        ("prvog", 1),
        ("drugoga", 2),
        ("drugo", 2),
        ("druga", 2),
        ("drugi", 2),
        ("drugog", 2),
        ("treći", 3),
        ("trećega", 3),
        ("trece", 3),
        ("trećeg", 3),
        ("treca", 3),
        ("treća", 3),
        ("treci", 3),
        ("četvrti", 4),
        ("četvrtoga", 4),
        ("cetvrtoga", 4),
        ("cetvrto", 4),
        ("cetvrtog", 4),
        ("cetvrti", 4),
        ("četvrtog", 4),
        ("cetvrta", 4),
        ("četvrto", 4),
        ("četvrta", 4),
        ("peto", 5),
        ("peti", 5),
        ("petoga", 5),
        ("petog", 5),
        ("peta", 5),
        ("sesti", 6),
        ("šestoga", 6),
        ("šestog", 6),
        ("sestoga", 6),
        ("sesta", 6),
        ("šesta", 6),
        ("šesto", 6),
        ("šesti", 6),
        ("sesto", 6),
        ("sestog", 6),
        ("sedmi", 7),
        ("sedmog", 7),
        ("sedma", 7),
        ("sedmo", 7),
        ("sedmoga", 7),
        ("osmo", 8),
        ("osmog", 8),
        ("osma", 8),
        ("osmi", 8),
        ("osmoga", 8),
        ("devetog", 9),
        ("deveta", 9),
        ("deveto", 9),
        ("devetoga", 9),
        ("deveti", 9),
        ("desetoga", 10),
        ("desetog", 10),
        ("deseta", 10),
        ("deseto", 10),
        ("deseti", 10),
        ("jedanaestoga", 11),
        ("jedanaesta", 11),
        ("jedanaesti", 11),
        ("jedanaesto", 11),
        ("jedanaestog", 11),
        ("dvanaesto", 12),
        ("dvanaestog", 12),
        ("dvanaesti", 12),
        ("dvanaesta", 12),
        ("dvanaestoga", 12),
        ("trinaesta", 13),
        ("trinaestoga", 13),
        ("trinaestog", 13),
        ("trinaesto", 13),
        ("trinaesti", 13),
        ("cetrnaestog", 14),
        ("cetrnaesto", 14),
        ("cetrnaesti", 14),
        ("četrnaestoga", 14),
        ("cetrnaesta", 14),
        ("četrnaesto", 14),
        ("četrnaesta", 14),
        ("cetrnaestoga", 14),
        ("četrnaesti", 14),
        ("četrnaestog", 14),
        ("petnaestog", 15),
        ("petnaesto", 15),
        ("petnaesta", 15),
        ("petnaesti", 15),
        ("petnaestoga", 15),
        ("sesnaesto", 16),
        ("sesnaestog", 16),
        ("sesnaesti", 16),
        ("šesnaestoga", 16),
        ("sesnaesta", 16),
        ("šesnaesto", 16),
        ("sesnaestoga", 16),
        ("šesnaesti", 16),
        ("šesnaesta", 16),
        ("šesnaestog", 16),
        ("sedamnaesto", 17),
        ("sedamnaestoga", 17),
        ("sedamnaesti", 17),
        ("sedamnaesta", 17),
        ("sedamnaestog", 17),
        ("osamnaestoga", 18),
        ("osamnaestog", 18),
        ("osamnaesti", 18),
        ("osamnaesta", 18),
        ("osamnaesto", 18),
        ("devetnaestoga", 19),
        ("devetnaestog", 19),
        ("devetnaesto", 19),
        ("devetnaesti", 19),
        ("devetnaesta", 19),
    ]
    .into_iter()
    .collect()
});

fn rule_ordinals_firstth() -> Rule {
    rule(
        "ordinals (first..19th)",
        vec![regex(
            r"(prv(i|a|o(ga?)?)|drug(i|a|o(ga?)?)|tre(c|ć)(i|a|e(ga?)?)|(č|c)etvrt(i|a|o(ga?)?)|pet(i|a|o(ga?)?)|(š|s)est(i|a|o(ga?)?)|sedm(i|a|o(ga?)?)|osm(i|a|o(ga?)?)|devet(i|a|o(ga?)?)|deset(i|a|o(ga?)?)|jedanaest(i|a|o(ga?)?)|dvanaest(i|a|o(ga?)?)|trinaest(i|a|o(ga?)?)|(c|č)etrnaest(i|a|o(ga?)?)|petnaest(i|a|o(ga?)?)|(s|š)esnaest(i|a|o(ga?)?)|sedamnaest(i|a|o(ga?)?)|osamnaest(i|a|o(ga?)?)|devetnaest(i|a|o(ga?)?))",
        )],
        |ts| {
            ORDINALS_MAP
                .get(to_lower(groups(ts)?.first()?).as_str())
                .and_then(|v| ordinal(*v))
        },
    )
}

fn rule_ordinal_digits() -> Rule {
    rule(
        "ordinal (digits)",
        vec![regex(r"0*(\d+)(\.| ?(t(i|a)(n|r|s)?)|(ste(n|r|s)?))")],
        |ts| parse_int(groups(ts)?.first()?).and_then(ordinal),
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| vec![rule_ordinal_digits(), rule_ordinals_firstth()]);

pub fn rules() -> &'static [Rule] {
    &RULES
}
