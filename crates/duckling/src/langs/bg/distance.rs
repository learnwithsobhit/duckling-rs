//! `Duckling.Distance.BG.Rules`.

use once_cell::sync::Lazy;

use crate::dims::distance::*;
use crate::types::{dimension, regex, rule, Dim, Rule, Token};

fn tok(d: DistanceData) -> Option<Token> {
    Some(Token::Distance(d))
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let distances: [(&str, &str, Unit); 8] = [
        ("<latent dist> km", "км|километ(ра|ри|ър)", Unit::Kilometre),
        ("<latent dist> feet", "('|фут(а|ове)?)", Unit::Foot),
        ("<latent dist> inch", "(\"|''|инч(а|ове)?)", Unit::Inch),
        ("<latent dist> yard", "ярд(а|ове)?", Unit::Yard),
        ("<dist> meters", "м(етър|етр(а|и))?", Unit::Metre),
        ("<dist> centimeters", "см|сантимет(ри|ра|ър)", Unit::Centimetre),
        ("<dist> millimeters", "мм|милимет(ра|ри|ър)", Unit::Millimetre),
        ("<dist> miles", "мил(я|и)", Unit::Mile),
    ];
    distances
        .into_iter()
        .map(|(name, re, u)| {
            rule(name, vec![dimension(Dim::Distance), regex(re)], move |ts| match ts.first()? {
                Token::Distance(dd) => tok(with_unit(u, dd.clone())),
                _ => None,
            })
        })
        .collect()
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
