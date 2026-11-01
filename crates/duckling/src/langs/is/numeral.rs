//! `Duckling.Numeral.IS.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{group, regex, rule, Rule};

const ZERO_TO_TWENTY: &[(&str, i64)] = &[
    ("núll", 0),
    ("null", 0),
    ("einn", 1),
    ("tveir", 2),
    ("þrír", 3),
    ("fjórir", 4),
    ("fimm", 5),
    ("sex", 6),
    ("sjö", 7),
    ("átta", 8),
    ("níu", 9),
    ("tíu", 10),
    ("ellefu", 11),
    ("tólf", 12),
    ("þrettán", 13),
    ("fjórtán", 14),
    ("fimmtán", 15),
    ("sextán", 16),
    ("sautján", 17),
    ("átján", 18),
    ("nítján", 19),
    ("tuttugu", 20),
];

fn lookup(table: &[(&str, i64)], k: &str) -> Option<i64> {
    table.iter().find(|(n, _)| *n == k).map(|(_, v)| *v)
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // ruleZeroToTwenty
        rule(
            "number (0..20)",
            vec![regex(
                "(n[úu]ll|einn|tveir|þrír|fjórir|fimm(tán)?|sex(tán)?|sjö|átta|níu|tíu|ellefu|tólf|þrettán|fjórtán|sautján|átján|nítján|tuttugu)",
            )],
            |ts| lookup(ZERO_TO_TWENTY, &to_lower(&group(ts, 0)?)).and_then(integer),
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
