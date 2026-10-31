//! Port of `Duckling.Ranking.*`: Naive Bayes ranking of candidate parses.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use crate::locale::{Lang, Locale, Region};
use crate::types::{Dim, Node, Range, Resolved, Token};

#[derive(Debug, Default)]
pub struct ClassData {
    pub prior: f64,
    pub unseen: f64,
    pub likelihoods: HashMap<String, f64>,
    pub n: i64,
}

/// Only the positive (`okData`) half is used when ranking.
#[derive(Debug, Default)]
pub struct Classifier {
    pub ok: ClassData,
}

pub type Classifiers = HashMap<String, Classifier>;

fn parse_f64(v: &serde_json::Value) -> f64 {
    match v {
        serde_json::Value::String(s) if s == "-inf" => f64::NEG_INFINITY,
        serde_json::Value::String(s) if s == "inf" => f64::INFINITY,
        serde_json::Value::String(s) if s == "nan" => f64::NAN,
        other => other.as_f64().unwrap_or(0.0),
    }
}

fn parse_class_data(v: &serde_json::Value) -> ClassData {
    let likelihoods = v["likelihoods"]
        .as_object()
        .map(|o| o.iter().map(|(k, x)| (k.clone(), parse_f64(x))).collect())
        .unwrap_or_default();
    ClassData {
        prior: parse_f64(&v["prior"]),
        unseen: parse_f64(&v["unseen"]),
        likelihoods,
        n: v["n"].as_i64().unwrap_or(0),
    }
}

/// Parses the JSON produced by `tools/oracle/export/ExportClassifiers.hs`.
pub fn parse_classifiers(json: &str) -> Classifiers {
    let v: serde_json::Value = serde_json::from_str(json).expect("valid classifier JSON");
    v.as_object()
        .map(|o| {
            o.iter()
                .map(|(rule, c)| (rule.clone(), Classifier { ok: parse_class_data(&c["ok"]) }))
                .collect()
        })
        .unwrap_or_default()
}

/// `Duckling.Ranking.Classifiers.classifiers`: which table a locale uses.
fn table_name(locale: Locale) -> String {
    match (locale.lang, locale.region) {
        (Lang::EN, Some(Region::GB)) => "EN_GB".to_string(),
        (Lang::EN, Some(Region::US)) => "EN_US".to_string(),
        (lang, _) => format!("{}_XX", lang),
    }
}

type Cache = Mutex<HashMap<String, &'static Classifiers>>;

fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Classifiers for a locale (loaded lazily from embedded data; empty if the language is not built in).
pub fn classifiers(locale: Locale) -> &'static Classifiers {
    let name = table_name(locale);
    let mut guard = cache().lock().unwrap();
    if let Some(c) = guard.get(&name) {
        return c;
    }
    let parsed = match crate::classifier_data::lookup(&name) {
        Some(json) => parse_classifiers(json),
        None => Classifiers::new(),
    };
    let leaked: &'static Classifiers = Box::leak(Box::new(parsed));
    guard.insert(name, leaked);
    leaked
}

/// `extractFeatures`: a bag of (at most two) features.
fn extract_features(node: &Node) -> Vec<(String, i64)> {
    let feat_rules: String = node.children.iter().filter_map(|c| c.rule.as_deref()).collect();
    let grains: String = node
        .children
        .iter()
        .filter_map(|c| match &c.token {
            Token::Duration(d) => Some(d.grain.text()),
            Token::Time(t) => Some(t.time_grain.text()),
            Token::TimeGrain(g) => Some(g.text()),
            _ => None,
        })
        .collect();
    let has_grains = node.children.iter().any(|c| {
        matches!(&c.token, Token::Duration(_) | Token::Time(_) | Token::TimeGrain(_))
    });
    let mut feats: Vec<(String, i64)> = vec![(feat_rules, 1)];
    if has_grains {
        // HashMap.fromList: a duplicate key keeps a single entry.
        if !feats.iter().any(|(k, _)| *k == grains) {
            feats.push((grains, 1));
        }
    }
    feats
}

fn ll(feats: &[(String, i64)], cd: &ClassData) -> f64 {
    let mut res = 0.0f64;
    for (feat, x) in feats {
        let l = cd.likelihoods.get(feat).copied().unwrap_or(cd.unseen);
        res += *x as f64 * l;
    }
    cd.prior + res
}

fn score(classifiers: &Classifiers, node: &Node) -> f64 {
    match &node.rule {
        Some(rule) => match classifiers.get(rule.as_ref()) {
            Some(c) => {
                let feats = extract_features(node);
                let own = ll(&feats, &c.ok);
                let mut sum = 0.0f64;
                for ch in &node.children {
                    sum += score(classifiers, ch);
                }
                own + sum
            }
            None => 0.0,
        },
        None => 0.0,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ord3 {
    Lt,
    Eq,
    Gt,
}

fn cmp_usize(a: usize, b: usize) -> Ord3 {
    match a.cmp(&b) {
        std::cmp::Ordering::Less => Ord3::Lt,
        std::cmp::Ordering::Equal => Ord3::Eq,
        std::cmp::Ordering::Greater => Ord3::Gt,
    }
}

fn cmp_f64(a: f64, b: f64) -> Ord3 {
    // GHC's `compare` on Double: NaN compares as GT.
    if a < b {
        Ord3::Lt
    } else if a == b {
        Ord3::Eq
    } else {
        Ord3::Gt
    }
}

struct Candidate<'a> {
    token: &'a Resolved,
    score: f64,
    target: bool,
}

/// Haskell's partial `Ord Candidate`.
fn compare(a: &Candidate, b: &Candidate) -> Ord3 {
    let Range { start: s1, end: e1 } = a.token.range;
    let Range { start: s2, end: e2 } = b.token.range;
    let d1 = a.token.node.token.dim();
    let d2 = b.token.node.token.dim();
    let starts = cmp_usize(s1, s2);
    let ends = cmp_usize(e1, e2);
    let comp_range = match starts {
        Ord3::Eq => ends,
        Ord3::Lt => match ends {
            Ord3::Lt => Ord3::Eq,
            _ => Ord3::Gt,
        },
        Ord3::Gt => match ends {
            Ord3::Gt => Ord3::Eq,
            _ => Ord3::Lt,
        },
    };
    if d1 == d2 {
        match starts {
            Ord3::Eq => match ends {
                Ord3::Eq => cmp_f64(a.score, b.score),
                z => z,
            },
            Ord3::Lt => match ends {
                Ord3::Lt => Ord3::Eq,
                _ => Ord3::Gt,
            },
            Ord3::Gt => match ends {
                Ord3::Gt => Ord3::Eq,
                _ => Ord3::Lt,
            },
        }
    } else if a.target == b.target {
        comp_range
    } else if a.target && comp_range == Ord3::Gt {
        Ord3::Gt
    } else if b.target && comp_range == Ord3::Lt {
        Ord3::Lt
    } else {
        Ord3::Eq
    }
}

/// `rank`: keeps the winning candidates, de-duplicated and sorted.
pub fn rank(
    classifiers: &Classifiers,
    targets: &HashSet<Dim>,
    tokens: Vec<Resolved>,
) -> Vec<Resolved> {
    let cands: Vec<Candidate> = tokens
        .iter()
        .map(|t| Candidate {
            token: t,
            score: score(classifiers, &t.node),
            target: targets.contains(&t.dim),
        })
        .collect();
    // winners xs = filter (\x -> all ((/=) LT . compare x) xs) xs
    let mut winners: Vec<Resolved> = cands
        .iter()
        .filter(|x| cands.iter().all(|y| compare(x, y) != Ord3::Lt))
        .map(|c| c.token.clone())
        .collect();
    winners.sort();
    winners.dedup_by(|a, b| Ord::cmp(&*a, &*b) == std::cmp::Ordering::Equal);
    winners
}
