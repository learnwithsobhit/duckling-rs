//! Port of `Duckling.Api`.

use std::collections::{BTreeSet, HashSet};

use crate::dims;
use crate::engine::parse_and_resolve;
use crate::json::Json;
use crate::locale::{Lang, Locale};
use crate::ranking::{classifiers, rank};
use crate::rules::rules_for;
use crate::types::{Context, Dim, Options, Resolved};

/// A parsed entity in the public API format (`Duckling.Types.Entity`).
#[derive(Clone, Debug)]
pub struct Entity {
    pub dim: &'static str,
    pub body: String,
    pub value: Json,
    pub start: usize,
    pub end: usize,
    pub latent: bool,
}

impl Entity {
    /// JSON exactly as the Haskell server emits it.
    pub fn to_json(&self) -> Json {
        Json::obj([
            ("dim", Json::str(self.dim)),
            ("body", Json::str(self.body.clone())),
            ("value", self.value.clone()),
            ("start", Json::Int(self.start as i64)),
            ("end", Json::Int(self.end as i64)),
            ("latent", Json::Bool(self.latent)),
        ])
    }
}

/// `analyze`: returns the curated list of resolved tokens.
/// When `targets` is non-empty, only tokens of such dimensions are returned.
pub fn analyze(
    input: &str,
    context: &Context,
    options: &Options,
    targets: &HashSet<Dim>,
) -> Vec<Resolved> {
    let rules = rules_for(context.locale, targets);
    let tokens: Vec<Resolved> = parse_and_resolve(&rules, input, context, options)
        .into_iter()
        .filter(|r| targets.is_empty() || targets.contains(&r.dim))
        .collect();
    rank(classifiers(context.locale), targets, tokens)
}

/// `formatToken`.
pub fn format_token(sentence: &str, r: &Resolved) -> Entity {
    let body: String = sentence.chars().skip(r.range.start).take(r.range.end - r.range.start).collect();
    Entity {
        dim: r.dim.name(),
        body,
        value: r.value.clone(),
        start: r.range.start,
        end: r.range.end,
        latent: r.is_latent,
    }
}

/// `parse`: curated list of entities found in `input`.
pub fn parse(input: &str, context: &Context, options: &Options, dims: &[Dim]) -> Vec<Entity> {
    let targets: HashSet<Dim> = dims.iter().copied().collect();
    let mut ents: Vec<Entity> =
        analyze(input, context, options, &targets).iter().map(|r| format_token(input, r)).collect();
    if targets.contains(&Dim::Timezone) {
        sort_location_tz(&mut ents);
    }
    ents
}

fn dim_priority(dim: &str) -> u8 {
    match dim {
        "time" => 0,
        "timezone" => 1,
        _ => 2,
    }
}

fn city_rank(e: &Entity) -> u8 {
    let iana = e.value.get("timezone").and_then(|v| v.as_str()).unwrap_or("");
    let city = e.value.get("city").and_then(|v| v.as_str()).unwrap_or("");
    crate::dims::timezone::rank_of(iana, city)
}

fn sort_location_tz(ents: &mut [Entity]) {
    ents.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then(a.end.cmp(&b.end))
            .then(dim_priority(a.dim).cmp(&dim_priority(b.dim)))
            .then(city_rank(a).cmp(&city_rank(b)))
    });
}

/// Dimensions supported by a language (`Duckling.Dimensions.allDimensions`).
pub fn all_dimensions(lang: Lang) -> Vec<Dim> {
    let mut v: BTreeSet<Dim> = [
        Dim::AmountOfMoney,
        Dim::CreditCardNumber,
        Dim::Email,
        Dim::Numeral,
        Dim::PhoneNumber,
        Dim::Url,
    ]
    .into_iter()
    .collect();
    v.extend(crate::rules::lang_dimensions(lang));
    v.into_iter().collect()
}

/// `supportedDimensions` for one language: `Common.allDimensions ++ <LANG>.allDimensions`, in
/// upstream order and without de-duplication (so e.g. `number` appears twice for EN).
pub fn supported_dimensions(lang: Lang) -> Vec<Dim> {
    let mut v = vec![
        Dim::AmountOfMoney,
        Dim::CreditCardNumber,
        Dim::Email,
        Dim::Numeral,
        Dim::PhoneNumber,
        Dim::Url,
    ];
    v.extend(crate::rules::lang_dimensions(lang));
    v
}

#[allow(dead_code)]
fn _link(_: Locale) {
    let _ = dims::util::to_lower;
}
