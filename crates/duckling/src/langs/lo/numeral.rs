//! `Duckling.Numeral.LO.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::integer;
use crate::dims::util::to_lower;
use crate::types::{groups, regex, rule, Rule, Token};

static NUMERAL_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ສູນ", 0),
        ("ໜຶ່ງ", 1),
        ("ສອງ", 2),
        ("ສາມ", 3),
        ("ສີ່", 4),
        ("ຫ້າ", 5),
        ("ຫົກ", 6),
        ("ເຈັດ", 7),
        ("ແປດ", 8),
        ("ເກົ້າ", 9),
        ("ສິບ", 10),
    ]
    .into_iter()
    .collect()
});

static ELEVEN_TO_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ສິບເອັດ", 11),
        ("ສິບສອງ", 12),
        ("ສິບສາມ", 13),
        ("ສິບສີ່", 14),
        ("ສິບຫ້າ", 15),
        ("ສິບຫົກ", 16),
        ("ສິບເຈັດ", 17),
        ("ສິບແປດ", 18),
        ("ສິບເກົ້າ", 19),
    ]
    .into_iter()
    .collect()
});

static TWENTYONE_TO_TWENTYNINE: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ຊາວເອັດ", 21),
        ("ຊາວສອງ", 22),
        ("ຊາວສາມ", 23),
        ("ຊາວສີ່", 24),
        ("ຊາວຫ້າ", 25),
        ("ຊາວຫົກ", 26),
        ("ຊາວເຈັດ", 27),
        ("ຊາວແປດ", 28),
        ("ຊາວເກົ້າ", 29),
    ]
    .into_iter()
    .collect()
});

static TENS_MAP: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ຊາວ", 20),
        ("ສາມສິບ", 30),
        ("ສິບສີ່", 40),
        ("ຫ້າສິບ", 50),
        ("ຫົກສິບ", 60),
        ("ເຈັດສິບ", 70),
        ("ແປດສິບ", 80),
        ("ເກົ້າສິບ", 90),
    ]
    .into_iter()
    .collect()
});

fn lookup_int(map: &HashMap<&'static str, i64>, ts: &[&Token]) -> Option<Token> {
    let g = groups(ts)?;
    map.get(to_lower(g.first()?).as_str()).and_then(|v| integer(*v))
}

fn rule_numeral() -> Rule {
    rule(
        "number (0..10)",
        vec![regex("(ສູນ|ໜຶ່ງ|ສອງ|ສາມ|ສີ່|ຫ້າ|ຫົກ|ເຈັດ|ແປດ|ເກົ້າ|ສິບ)")],
        |ts| lookup_int(&NUMERAL_MAP, ts),
    )
}

fn rule_eleven_to_nineteen() -> Rule {
    rule(
        "number (11..19)",
        vec![regex("(ສິບເອັດ|ສິບສອງ|ສິບສາມ|ສິບສີ່|ສິບຫ້າ|ສິບຫົກ|ສິບເຈັດ|ສິບແປດ|ສິບເກົ້າ)")],
        |ts| lookup_int(&ELEVEN_TO_NINETEEN, ts),
    )
}

fn rule_twentyone_to_twentynine() -> Rule {
    rule(
        "number (21..29)",
        vec![regex("(ຊາວເອັດ|ຊາວສອງ|ຊາວສາມ|ຊາວສີ່|ຊາວຫ້າ|ຊາວຫົກ|ຊາວເຈັດ|ຊາວແປດ|ຊາວເກົ້າ)")],
        |ts| lookup_int(&TWENTYONE_TO_TWENTYNINE, ts),
    )
}

fn rule_tens() -> Rule {
    rule(
        "integer (20,30..90)",
        vec![regex("(ຊາວ|ສາມສິບ|ສິບສີ່|ຫ້າສິບ|ຫົກສິບ|ເຈັດສິບ|ແປດສິບ|ເກົ້າສິບ)")],
        |ts| lookup_int(&TENS_MAP, ts),
    )
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer ([3-9][1-9])",
        vec![regex("(ສາມສິບ|ສິບສີ່|ຫ້າສິບ|ຫົກສິບ|ເຈັດສິບ|ແປດສິບ|ເກົ້າສິບ)(ໜຶ່ງ|ສອງ|ສາມ|ສີ່|ຫ້າ|ຫົກ|ເຈັດ|ແປດ|ເກົ້າ)")],
        |ts| {
            let g = groups(ts)?;
            let v1 = TENS_MAP.get(to_lower(g.first()?).as_str())?;
            let v2 = NUMERAL_MAP.get(to_lower(g.get(1)?).as_str())?;
            integer(v1 + v2)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_numeral(),
        rule_eleven_to_nineteen(),
        rule_twentyone_to_twentynine(),
        rule_tens(),
        rule_composite_tens(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
