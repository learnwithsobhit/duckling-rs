//! `Duckling.Volume.ZH.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::is_positive;
use crate::dims::volume::{
    is_simple_volume, unit_only, volume, with_interval, with_max, with_min, VolumeData, Unit,
};
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

fn tok(d: VolumeData) -> Option<Token> {
    Some(Token::Volume(d))
}

fn rule_unit_teaspoon() -> Rule {
    rule("<numeral> teaspoon", vec![pred(is_positive), regex("茶匙")], |ts| match ts.first()? {
        Token::Numeral(n) => tok(volume(Unit::Millilitre, 5.0 * n.value)),
        _ => None,
    })
}

fn rule_unit_soupspoon() -> Rule {
    rule("<numeral> soupspoon", vec![pred(is_positive), regex("湯匙")], |ts| match ts.first()? {
        Token::Numeral(n) => tok(volume(Unit::Millilitre, 15.0 * n.value)),
        _ => None,
    })
}

fn rule_precision() -> Rule {
    rule("about <volume>", vec![regex("\\~|大約|約"), dimension(Dim::Volume)], |ts| {
        ts.get(1).map(|t| (*t).clone())
    })
}

fn rule_precision2() -> Rule {
    rule("about <volume> (second rule)", vec![dimension(Dim::Volume), regex("左右")], |ts| {
        ts.first().map(|t| (*t).clone())
    })
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> and|to <volume>",
        vec![pred(is_positive), regex("-|~|至|到"), pred(is_simple_volume)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (Token::Numeral(n), Token::Volume(VolumeData { value: Some(to), unit: Some(u), .. }))
                if n.value < *to =>
            {
                tok(with_interval(n.value, *to, unit_only(*u)))
            }
            _ => None,
        },
    )
}

fn rule_interval_between() -> Rule {
    rule(
        "between|from <volume> to|and <volume>",
        vec![pred(is_simple_volume), regex("-|~|至|到"), pred(is_simple_volume)],
        |ts| match (ts.first()?, ts.get(2)?) {
            (
                Token::Volume(VolumeData { value: Some(from), unit: Some(u1), .. }),
                Token::Volume(VolumeData { value: Some(to), unit: Some(u2), .. }),
            ) if *from < *to && u1 == u2 => tok(with_interval(*from, *to, unit_only(*u1))),
            _ => None,
        },
    )
}

fn rule_interval_max() -> Rule {
    rule("at most <volume>", vec![regex("最多"), pred(is_simple_volume)], |ts| match ts.get(1)? {
        Token::Volume(VolumeData { value: Some(to), unit: Some(u), .. }) => {
            tok(with_max(*to, unit_only(*u)))
        }
        _ => None,
    })
}

fn rule_interval_max2() -> Rule {
    rule("<volume> or below", vec![pred(is_simple_volume), regex("(或)?以下")], |ts| {
        match ts.first()? {
            Token::Volume(VolumeData { value: Some(to), unit: Some(u), .. }) => {
                tok(with_max(*to, unit_only(*u)))
            }
            _ => None,
        }
    })
}

fn rule_interval_min() -> Rule {
    rule("more than <volume>", vec![regex("至少|最少|起碼"), pred(is_simple_volume)], |ts| {
        match ts.get(1)? {
            Token::Volume(VolumeData { value: Some(from), unit: Some(u), .. }) => {
                tok(with_min(*from, unit_only(*u)))
            }
            _ => None,
        }
    })
}

fn rule_interval_min2() -> Rule {
    rule("<volume> or above", vec![pred(is_simple_volume), regex("(或)?以上")], |ts| {
        match ts.first()? {
            Token::Volume(VolumeData { value: Some(from), unit: Some(u), .. }) => {
                tok(with_min(*from, unit_only(*u)))
            }
            _ => None,
        }
    })
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = vec![
        rule_unit_teaspoon(),
        rule_unit_soupspoon(),
        rule_precision(),
        rule_precision2(),
        rule_interval_between_numeral(),
        rule_interval_between(),
        rule_interval_max(),
        rule_interval_max2(),
        rule_interval_min(),
        rule_interval_min2(),
    ];
    let volumes: [(&str, &str, Unit); 3] = [
        ("<latent vol> ml", "cc|ml|毫升", Unit::Millilitre),
        ("<vol> liters", "l|L|公升|升", Unit::Litre),
        ("<latent vol> gallon", "加侖", Unit::Gallon),
    ];
    v.extend(volumes.into_iter().map(|(name, re, u)| {
        rule(name, vec![regex(re)], move |_| tok(unit_only(u)))
    }));
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
