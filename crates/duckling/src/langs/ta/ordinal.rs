//! `Duckling.Ordinal.TA.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::ordinal::OrdinalData;
use crate::dims::util::{parse_int, to_lower};
use crate::types::{groups, regex, rule, Rule, Token};

fn ordinal(v: i64) -> Option<Token> {
    Some(Token::Ordinal(OrdinalData { value: v }))
}

static ONE_TO_NINE: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("முதல்", 1),
        ("இரண்டாம்", 2),
        ("மூன்றாம்", 3),
        ("நான்காம்", 4),
        ("ஐந்தாம்", 5),
        ("ஆறாம்", 6),
        ("ஏழாம்", 7),
        ("எட்டாம்", 8),
        ("ஒன்பதாம்", 9),
    ]
    .into_iter()
    .collect()
});

static TEN_TO_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("பத்தாம்", 10),
        ("பதினொன்றாம்", 11),
        ("பன்னிரண்டாம்", 12),
        ("பதின்மூன்றாம்", 13),
        ("பதினான்காம்", 14),
        ("பதினைந்தாம்", 15),
        ("பதினாறாம்", 16),
        ("பதினேழாம்", 17),
        ("பதினெட்டாம்", 18),
        ("பத்தொன்பதாம்", 19),
    ]
    .into_iter()
    .collect()
});

static TENS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("இருபதாம்", 20),
        ("முப்பதாம்", 30),
        ("நாற்பதாம்", 40),
        ("ஐம்பதாம்", 50),
        ("அறுபதாம்", 60),
        ("எழுபதாம்", 70),
        ("எண்பதாம்", 80),
        ("தொண்ணூறாம்", 90),
    ]
    .into_iter()
    .collect()
});

static TENS_ORDINAL: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("இருபத்தி", 20),
        ("முப்பத்து", 30),
        ("நாற்பத்து", 40),
        ("ஐம்பத்தி", 50),
        ("அறுபத்", 60),
        ("எழுபத்தி", 70),
        ("எண்பத்தி", 80),
        ("தொண்ணுற்று", 90),
    ]
    .into_iter()
    .collect()
});

static ONE_TO_NINE_ORDINAL: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("ஒன்றாம்", 1),
        ("இரண்டாம்", 2),
        ("மூன்றாம்", 3),
        ("நான்காம்", 4),
        ("ஐந்தாம்", 5),
        ("ஆறாம்", 6),
        ("ஏழாம்", 7),
        ("எட்டாம்", 8),
        ("ஒன்பதாம்", 9),
    ]
    .into_iter()
    .collect()
});

fn rule_one_to_nine() -> Rule {
    rule(
        "integer (1..9)",
        vec![regex("(முதல்|இரண்டாம்|மூன்றாம்|நான்காம்|ஐந்தாம்|ஆறாம்|ஏழாம்|எட்டாம்|ஒன்பதாம்)")],
        |ts| ONE_TO_NINE.get(to_lower(groups(ts)?.first()?).as_str()).copied().and_then(ordinal),
    )
}

fn rule_ten_to_nineteen() -> Rule {
    rule(
        "integer (10..19)",
        vec![regex(
            "(பத்தாம்|பதினொன்றாம்|பன்னிரண்டாம்|பதின்மூன்றாம்|பதினான்காம்|பதினைந்தாம்|பதினாறாம்|பதினேழாம்|பதினெட்டாம்|பத்தொன்பதாம்)",
        )],
        |ts| {
            TEN_TO_NINETEEN
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(ordinal)
        },
    )
}

fn rule_tens() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(இருபதாம்|முப்பதாம்|நாற்பதாம்|ஐம்பதாம்|அறுபதாம்|எழுபதாம்|எண்பதாம்|தொண்ணூறாம்)")],
        |ts| TENS.get(to_lower(groups(ts)?.first()?).as_str()).copied().and_then(ordinal),
    )
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer ([2-9][1-9])",
        vec![regex(
            "(இருபத்தி|முப்பத்து|நாற்பத்து|ஐம்பத்தி|அறுபத்|எழுபத்தி|எண்பத்தி|தொண்ணுற்று)(ஒன்றாம்|இரண்டாம்|மூன்றாம்|நான்காம்|ஐந்தாம்|ஆறாம்|ஏழாம்|எட்டாம்|ஒன்பதாம்)",
        )],
        |ts| {
            let g = groups(ts)?;
            let v1 = *TENS_ORDINAL.get(to_lower(g.first()?).as_str())?;
            let v2 = *ONE_TO_NINE_ORDINAL.get(to_lower(g.get(1)?).as_str())?;
            ordinal(v1 + v2)
        },
    )
}

fn rule_ordinal_digits() -> Rule {
    rule("ordinal (digits)", vec![regex("0*(\\d+)\\.")], |ts| {
        parse_int(groups(ts)?.first()?).and_then(ordinal)
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule_ordinal_digits(),
        rule_one_to_nine(),
        rule_ten_to_nineteen(),
        rule_tens(),
        rule_composite_tens(),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
