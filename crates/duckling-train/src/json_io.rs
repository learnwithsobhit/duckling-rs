//! Load / dump classifier JSON in the `ExportClassifiers.hs` format.

use std::path::Path;

use serde_json::{json, Map, Value};

use crate::nb::{ClassData, Classifier, Classifiers};

fn parse_f64(v: &Value) -> f64 {
    match v {
        Value::String(s) if s == "-inf" => f64::NEG_INFINITY,
        Value::String(s) if s == "inf" => f64::INFINITY,
        Value::String(s) if s == "nan" => f64::NAN,
        other => other.as_f64().unwrap_or(0.0),
    }
}

fn class_data_from_json(v: &Value) -> ClassData {
    let likelihoods = v
        .get("likelihoods")
        .and_then(Value::as_object)
        .map(|o| o.iter().map(|(k, x)| (k.clone(), parse_f64(x))).collect())
        .unwrap_or_default();
    ClassData {
        prior: parse_f64(v.get("prior").unwrap_or(&Value::Null)),
        unseen: parse_f64(v.get("unseen").unwrap_or(&Value::Null)),
        likelihoods,
        n: v.get("n").and_then(Value::as_i64).unwrap_or(0),
    }
}

/// Parses the JSON produced by `ExportClassifiers.hs` (both `ok` and `ko`).
pub fn parse_classifiers(json: &str) -> Result<Classifiers, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("classifier JSON: {e}"))?;
    let obj = v.as_object().ok_or("classifier JSON: expected object")?;
    Ok(obj
        .iter()
        .map(|(rule, c)| {
            (
                rule.clone(),
                Classifier {
                    ok: class_data_from_json(&c["ok"]),
                    ko: class_data_from_json(&c["ko"]),
                },
            )
        })
        .collect())
}

pub fn load_classifiers_file(path: &Path) -> Result<Classifiers, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_classifiers(&s)
}

fn encode_f64(x: f64) -> Value {
    if x.is_infinite() {
        Value::String(if x.is_sign_positive() { "inf".into() } else { "-inf".into() })
    } else if x.is_nan() {
        Value::String("nan".into())
    } else {
        // Preserve the same number the Haskell `toJSON` wrote when possible.
        json!(x)
    }
}

fn class_data_to_json(cd: &ClassData) -> Value {
    let mut likelihoods = Map::new();
    let mut keys: Vec<&String> = cd.likelihoods.keys().collect();
    keys.sort();
    for k in keys {
        likelihoods.insert(k.clone(), encode_f64(cd.likelihoods[k]));
    }
    json!({
        "prior": encode_f64(cd.prior),
        "unseen": encode_f64(cd.unseen),
        "n": cd.n,
        "likelihoods": Value::Object(likelihoods),
    })
}

/// Dump in ExportClassifiers order: rules sorted, likelihood keys sorted.
pub fn classifiers_to_json(cs: &Classifiers) -> String {
    let mut rules: Vec<&String> = cs.keys().collect();
    rules.sort();
    let mut obj = Map::new();
    for r in rules {
        let c = &cs[r];
        obj.insert(r.clone(), json!({ "ok": class_data_to_json(&c.ok), "ko": class_data_to_json(&c.ko) }));
    }
    // Compact, matching aeson's `encode` (no spaces).
    Value::Object(obj).to_string()
}

pub fn write_classifiers(path: &Path, cs: &Classifiers) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, classifiers_to_json(cs)).map_err(|e| format!("{}: {e}", path.display()))
}

