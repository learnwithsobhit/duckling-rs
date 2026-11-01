//! `Duckling.Distance.RO.Rules`.

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::dims::distance::{with_unit, Unit};
use crate::dims::util::to_lower;
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

static UNIT_MAP: Lazy<HashMap<&'static str, Unit>> = Lazy::new(|| {
    [
        ("cm", Unit::Centimetre),
        ("centimetri", Unit::Centimetre),
        ("centimetru", Unit::Centimetre),
        ("picior", Unit::Foot),
        ("picioare", Unit::Foot),
        ("inch", Unit::Inch),
        ("inchi", Unit::Inch),
        ("inci", Unit::Inch),
        ("km", Unit::Kilometre),
        ("kilometri", Unit::Kilometre),
        ("kilometru", Unit::Kilometre),
        ("m", Unit::Metre),
        ("metri", Unit::Metre),
        ("metru", Unit::Metre),
        ("mila", Unit::Mile),
        ("milă", Unit::Mile),
        ("mile", Unit::Mile),
        ("y", Unit::Yard),
        ("yar", Unit::Yard),
        ("yard", Unit::Yard),
        ("yarzi", Unit::Yard),
        ("yd", Unit::Yard),
        ("yzi", Unit::Yard),
    ]
    .into_iter()
    .collect()
});

fn lookup_unit(ts: &[&Token]) -> Option<Token> {
    match (ts.first()?, ts.get(1)?) {
        (Token::Distance(dd), Token::RegexMatch(g)) => {
            let u = *UNIT_MAP.get(to_lower(g.0.first()?).as_str())?;
            Some(Token::Distance(with_unit(u, dd.clone())))
        }
        _ => None,
    }
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        rule(
            "<latent dist> foot/inch/yard/meter/kilometer/centimeter",
            vec![
                dimension(Dim::Distance),
                regex("(inc(hi?|i)|(centi|kilo)?metr[iu]|mil[eaă]|[ck]?m|picio(are|r)|y(ar)?(zi|d)?)"),
            ],
            lookup_unit,
        ),
        rule(
            "<latent dist> foot/inch/yard/meter/kilometer/centimeter",
            vec![
                dimension(Dim::Distance),
                regex("de (inc(hi?|i)|(centi|kilo)?metr[iu]|mil[eaă]|[ck]?m|picio(are|r)|y(ar)?(zi|d)?)"),
            ],
            lookup_unit,
        ),
    ]
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
