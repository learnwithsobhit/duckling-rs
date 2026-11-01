//! `Duckling.Numeral.TA.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::numeral::*;
use crate::dims::util::to_lower;
use crate::types::{groups, regex, rule, Rule};

static ZERO_TO_NINE: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("பூஜ்ஜியம்", 0),
        ("ஒன்று", 1),
        ("இரண்டு", 2),
        ("மூன்று", 3),
        ("நான்கு", 4),
        ("ஐந்து", 5),
        ("ஆறு", 6),
        ("ஏழு", 7),
        ("எட்டு", 8),
        ("ஒன்பது", 9),
    ]
    .into_iter()
    .collect()
});

static TEN_TO_NINETEEN: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("பத்து", 10),
        ("பதினொன்று", 11),
        ("பன்னிரண்டு", 12),
        ("பதின்மூன்று", 13),
        ("பதினான்கு", 14),
        ("பதினைந்து", 15),
        ("பதினாறு", 16),
        ("பதினேழு", 17),
        ("பதினெட்டு", 18),
        ("பத்தொன்பது", 19),
    ]
    .into_iter()
    .collect()
});

static TENS: Lazy<HashMap<&'static str, i64>> = Lazy::new(|| {
    [
        ("இருபது", 20),
        ("இருபத்தி", 20),
        ("முப்பது", 30),
        ("முப்பத்து", 30),
        ("நாற்பது", 40),
        ("நாற்பத்து", 40),
        ("ஐம்பது", 50),
        ("ஐம்பத்தி", 50),
        ("அறுபது", 60),
        ("அறுபத்", 60),
        ("எழுபது", 70),
        ("எழுபத்தி", 70),
        ("எண்பது", 80),
        ("எண்பத்", 80),
        ("தொண்ணூறு", 90),
        ("தொண்ணுற்று", 90),
    ]
    .into_iter()
    .collect()
});

fn rule_zero_to_nine() -> Rule {
    rule(
        "integer (0..9)",
        vec![regex("(பூஜ்ஜியம்|ஒன்று|இரண்டு|மூன்று|நான்கு|ஐந்து|ஆறு|ஏழு|எட்டு|ஒன்பது)")],
        |ts| ZERO_TO_NINE.get(to_lower(groups(ts)?.first()?).as_str()).copied().and_then(integer),
    )
}

fn rule_ten_to_nineteen() -> Rule {
    rule(
        "integer (10..19)",
        vec![regex(
            "(பத்து|பதினொன்று|பன்னிரண்டு|பதின்மூன்று|பதினான்கு|பதினைந்து|பதினாறு|பதினேழு|பதினெட்டு|பத்தொன்பது)",
        )],
        |ts| {
            TEN_TO_NINETEEN
                .get(to_lower(groups(ts)?.first()?).as_str())
                .copied()
                .and_then(integer)
        },
    )
}

fn rule_tens() -> Rule {
    rule(
        "integer (20..90)",
        vec![regex("(இருபது|முப்பது|நாற்பது|ஐம்பது|அறுபது|எழுபது|எண்பது|தொண்ணூறு)")],
        |ts| TENS.get(to_lower(groups(ts)?.first()?).as_str()).copied().and_then(integer),
    )
}

fn rule_composite_tens() -> Rule {
    rule(
        "integer ([2-9][1-9])",
        vec![regex(
            "(இருபத்தி|முப்பத்து|நாற்பத்து|ஐம்பத்தி|அறுபத்|எழுபத்தி|எண்பத்|தொண்ணுற்று)(ஒன்று|இரண்டு|மூன்று|நான்கு|ஐந்து|ஆறு|ஏழு|எட்டு|ஒன்பது)",
        )],
        |ts| {
            let g = groups(ts)?;
            let v1 = *TENS.get(to_lower(g.first()?).as_str())?;
            let v2 = *ZERO_TO_NINE.get(to_lower(g.get(1)?).as_str())?;
            integer(v1 + v2)
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![rule_zero_to_nine(), rule_ten_to_nineteen(), rule_tens(), rule_composite_tens()]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
