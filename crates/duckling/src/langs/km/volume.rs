//! `Duckling.Volume.KM.Rules`.

use once_cell::sync::Lazy;

use crate::dims::numeral::is_positive;
use crate::dims::volume::{
    is_simple_volume, is_unit_only, unit_only, volume, with_interval, with_max, with_min, VolumeData,
    Unit,
};
use crate::types::{dimension, pred, regex, rule, Dim, Rule, Token};

fn tok(d: VolumeData) -> Option<Token> {
    Some(Token::Volume(d))
}

fn rule_precision() -> Rule {
    rule("about <volume>", vec![regex("\\~|ប្រហែល"), dimension(Dim::Volume)], |ts| {
        ts.get(1).map(|t| (*t).clone())
    })
}

fn rule_interval_between_numeral() -> Rule {
    rule(
        "between|from <numeral> and|to <volume>",
        vec![regex("ចន្លោះ(ពី)?|ចាប់ពី"), pred(is_positive), regex("និង|ដល់"), pred(is_simple_volume)],
        |ts| match (ts.get(1)?, ts.get(3)?) {
            (
                Token::Numeral(n),
                Token::Volume(VolumeData { value: Some(to), unit: Some(u), .. }),
            ) if n.value < *to => tok(with_interval(n.value, *to, unit_only(*u))),
            _ => None,
        },
    )
}

fn rule_interval_between() -> Rule {
    rule(
        "between|from <volume> to|and <volume>",
        vec![
            regex("ចន្លោះ(ពី)?|ចាប់ពី"),
            pred(is_simple_volume),
            regex("និង|ដល់"),
            pred(is_simple_volume),
        ],
        |ts| match (ts.get(1)?, ts.get(3)?) {
            (
                Token::Volume(VolumeData { value: Some(from), unit: Some(u1), .. }),
                Token::Volume(VolumeData { value: Some(to), unit: Some(u2), .. }),
            ) if *from < *to && u1 == u2 => tok(with_interval(*from, *to, unit_only(*u1))),
            _ => None,
        },
    )
}

fn rule_interval_max() -> Rule {
    rule(
        "at most <volume>",
        vec![regex("ក្រោម|តិចជាង|មិនដល់|យ៉ាងច្រើន|មិនលើស"), pred(is_simple_volume)],
        |ts| match ts.get(1)? {
            Token::Volume(VolumeData { value: Some(to), unit: Some(u), .. }) => {
                tok(with_max(*to, unit_only(*u)))
            }
            _ => None,
        },
    )
}

fn rule_interval_min() -> Rule {
    rule(
        "more than <volume>",
        vec![regex("លើស(ពី)?|មិនតិចជាង|លើ|ច្រើនជាង|យ៉ាងតិច|យ៉ាងហោច"), pred(is_simple_volume)],
        |ts| match ts.get(1)? {
            Token::Volume(VolumeData { value: Some(from), unit: Some(u), .. }) => {
                tok(with_min(*from, unit_only(*u)))
            }
            _ => None,
        },
    )
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = vec![
        rule_precision(),
        rule_interval_between_numeral(),
        rule_interval_between(),
        rule_interval_max(),
        rule_interval_min(),
    ];
    let volumes: [(&str, &str, Unit); 2] = [
        ("<latent vol> ml", "ml|មីលីលីត្រ", Unit::Millilitre),
        ("<vol> liters", "l|លីត្រ", Unit::Litre),
    ];
    v.extend(volumes.into_iter().map(|(name, re, u)| {
        rule(name, vec![regex(re)], move |_| tok(unit_only(u)))
    }));
    let fractions: [(&str, &str, f64); 4] = [
        ("half", "កន្លះ|១\\/២", 1.0 / 2.0),
        ("third", "មួយភាគបី|១\\/៣", 1.0 / 3.0),
        ("fourth", "មួយភាគបួន|១\\/៤", 1.0 / 4.0),
        ("fifth", "មួយភាគប្រាំ|១\\/៥", 1.0 / 5.0),
    ];
    v.extend(fractions.into_iter().map(|(name, re, f)| {
        rule(name, vec![regex(re), pred(is_unit_only)], move |ts| match ts.get(1)? {
            Token::Volume(VolumeData { unit: Some(u), .. }) => tok(volume(*u, f)),
            _ => None,
        })
    }));
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
