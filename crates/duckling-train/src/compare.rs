//! Compare trained classifiers to an ExportClassifiers dump.

use crate::nb::{ClassData, Classifiers};

/// Default tolerances: identical integer counts through IEEE-754 `ln` / `/`
/// agree far inside these bounds. Documented for Phase 6 retraining checks.
pub const DEFAULT_ABS_TOL: f64 = 1e-12;
pub const DEFAULT_REL_TOL: f64 = 1e-9;

#[derive(Clone, Copy, Debug)]
pub struct Tol {
    pub abs: f64,
    pub rel: f64,
}

impl Default for Tol {
    fn default() -> Tol {
        Tol { abs: DEFAULT_ABS_TOL, rel: DEFAULT_REL_TOL }
    }
}

/// Two finite floats match when `|a-b| <= max(abs, rel * max(|a|,|b|))`.
/// Infinities must be identical (including sign). NaNs match each other.
pub fn floats_close(a: f64, b: f64, tol: Tol) -> bool {
    if a.is_nan() && b.is_nan() {
        return true;
    }
    if a.is_infinite() || b.is_infinite() {
        return a == b;
    }
    let diff = (a - b).abs();
    diff <= tol.abs.max(tol.rel * a.abs().max(b.abs()))
}

#[derive(Clone, Debug, Default)]
pub struct Diff {
    pub missing_rules: Vec<String>,
    pub extra_rules: Vec<String>,
    pub field: Vec<String>,
    pub max_abs_delta: f64,
}

impl Diff {
    pub fn ok(&self) -> bool {
        self.missing_rules.is_empty() && self.extra_rules.is_empty() && self.field.is_empty()
    }
}

pub fn compare(trained: &Classifiers, exported: &Classifiers, tol: Tol) -> Diff {
    let mut diff = Diff::default();
    for r in exported.keys() {
        if !trained.contains_key(r) {
            diff.missing_rules.push(r.clone());
        }
    }
    for r in trained.keys() {
        if !exported.contains_key(r) {
            diff.extra_rules.push(r.clone());
        }
    }
    diff.missing_rules.sort();
    diff.extra_rules.sort();

    let mut rules: Vec<&String> = trained.keys().filter(|r| exported.contains_key(*r)).collect();
    rules.sort();
    for r in rules {
        let t = &trained[r];
        let e = &exported[r];
        compare_class(&mut diff, r, "ok", &t.ok, &e.ok, tol);
        compare_class(&mut diff, r, "ko", &t.ko, &e.ko, tol);
    }
    diff
}

fn compare_class(diff: &mut Diff, rule: &str, side: &str, t: &ClassData, e: &ClassData, tol: Tol) {
    if t.n != e.n {
        diff.field.push(format!("{rule}.{side}.n trained={} exported={}", t.n, e.n));
    }
    note_float(diff, rule, side, "prior", t.prior, e.prior, tol);
    note_float(diff, rule, side, "unseen", t.unseen, e.unseen, tol);

    let mut keys: Vec<&String> = t.likelihoods.keys().collect();
    for k in e.likelihoods.keys() {
        if !t.likelihoods.contains_key(k) {
            keys.push(k);
        }
    }
    keys.sort();
    keys.dedup();
    for k in keys {
        match (t.likelihoods.get(k), e.likelihoods.get(k)) {
            (None, Some(_)) => {
                diff.field.push(format!("{rule}.{side}.likelihoods[{k:?}] missing in trained"));
            }
            (Some(_), None) => {
                diff.field.push(format!("{rule}.{side}.likelihoods[{k:?}] extra in trained"));
            }
            (Some(&a), Some(&b)) => note_float(diff, rule, side, &format!("likelihoods[{k:?}]"), a, b, tol),
            (None, None) => {}
        }
    }
}

fn note_float(diff: &mut Diff, rule: &str, side: &str, field: &str, a: f64, b: f64, tol: Tol) {
    let delta = if a.is_finite() && b.is_finite() { (a - b).abs() } else { 0.0 };
    if delta > diff.max_abs_delta {
        diff.max_abs_delta = delta;
    }
    if !floats_close(a, b, tol) {
        diff.field.push(format!("{rule}.{side}.{field} trained={a} exported={b} |Δ|={delta}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inf_and_near_equal() {
        let tol = Tol::default();
        assert!(floats_close(f64::NEG_INFINITY, f64::NEG_INFINITY, tol));
        assert!(!floats_close(f64::NEG_INFINITY, -1.0, tol));
        assert!(floats_close(0.0, 1e-16, tol));
        assert!(!floats_close(0.0, 1e-6, tol));
    }
}
