//! Naive Bayes + Laplace smoothing (`Duckling.Ranking.Train`).

use std::collections::HashMap;

/// One class (`ClassData` in `Duckling.Ranking.Types`).
#[derive(Clone, Debug, PartialEq)]
pub struct ClassData {
    pub prior: f64,
    pub unseen: f64,
    pub likelihoods: HashMap<String, f64>,
    pub n: i64,
}

/// `Classifier { okData, koData }`.
#[derive(Clone, Debug, PartialEq)]
pub struct Classifier {
    pub ok: ClassData,
    pub ko: ClassData,
}

pub type Classifiers = HashMap<String, Classifier>;
pub type BagOfFeatures = HashMap<String, i64>;
pub type Datum = (BagOfFeatures, bool);

/// `makeClass`: prior, unseen, and per-feature log-likelihoods with Laplace +1.
///
/// ```text
/// prior       = log(classTotal / total)
/// denum       = vocSize + sum(feature counts)
/// unseen      = log(1 / (denum + 1))
/// likelihood  = log((count + 1) / denum)
/// ```
///
/// `classTotal == 0` yields `prior = -inf` (Haskell `log 0`).
pub fn make_class(feats: &BagOfFeatures, total: i64, class_total: i64, voc_size: i64) -> ClassData {
    let prior = (class_total as f64 / total as f64).ln();
    let feat_sum: i64 = feats.values().copied().sum();
    let denum = voc_size + feat_sum;
    let unseen = (1.0 / (denum as f64 + 1.0)).ln();
    let likelihoods = feats
        .iter()
        .map(|(k, x)| (k.clone(), (((*x as f64) + 1.0) / (denum as f64)).ln()))
        .collect();
    ClassData { prior, unseen, likelihoods, n: class_total }
}

/// `train`: partition datums into ok/ko, merge bags, share vocabulary size.
pub fn train(datums: &[Datum]) -> Classifier {
    let total = datums.len() as i64;
    let mut ok_counts = BagOfFeatures::new();
    let mut ko_counts = BagOfFeatures::new();
    let mut ok_n = 0i64;
    let mut ko_n = 0i64;
    for (bag, ok) in datums {
        let dest = if *ok {
            ok_n += 1;
            &mut ok_counts
        } else {
            ko_n += 1;
            &mut ko_counts
        };
        for (feat, n) in bag {
            *dest.entry(feat.clone()).or_insert(0) += *n;
        }
    }
    let mut voc = ok_counts.keys().cloned().collect::<std::collections::HashSet<_>>();
    voc.extend(ko_counts.keys().cloned());
    let voc_size = voc.len() as i64;
    Classifier {
        ok: make_class(&ok_counts, total, ok_n, voc_size),
        ko: make_class(&ko_counts, total, ko_n, voc_size),
    }
}

/// `makeClassifiers`: train one classifier per rule that has at least one datum.
pub fn make_classifiers(dataset: &HashMap<String, Vec<Datum>>) -> Classifiers {
    dataset.iter().map(|(rule, datums)| (rule.clone(), train(datums))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bag(feats: &[&str]) -> BagOfFeatures {
        feats.iter().map(|f| ((*f).to_string(), 1)).collect()
    }

    #[test]
    fn thursday_all_ok_matches_exported_ar() {
        // Six datums, each with the empty-string feature, all ok.
        let datums: Vec<Datum> = (0..6).map(|_| (bag(&[""]), true)).collect();
        let c = train(&datums);
        assert_eq!(c.ok.n, 6);
        assert_eq!(c.ko.n, 0);
        assert_eq!(c.ok.prior, 0.0);
        assert!(c.ko.prior.is_infinite() && c.ko.prior.is_sign_negative());
        assert!((c.ok.unseen - (1.0f64 / 8.0).ln()).abs() < 1e-15);
        assert!((c.ko.unseen - (1.0f64 / 2.0).ln()).abs() < 1e-15);
        assert_eq!(c.ok.likelihoods.get(""), Some(&0.0));
        assert!(c.ko.likelihoods.is_empty());
    }

    #[test]
    fn two_ok_two_features_matches_exported_timezone() {
        // Two ok datums, each carrying ("at <time-of-day>", 1) and ("hour", 1).
        let datums = vec![
            (bag(&["at <time-of-day>", "hour"]), true),
            (bag(&["at <time-of-day>", "hour"]), true),
        ];
        let c = train(&datums);
        assert_eq!(c.ok.n, 2);
        assert!((c.ok.unseen - (1.0f64 / 7.0).ln()).abs() < 1e-15);
        let want = (0.5f64).ln();
        assert!((c.ok.likelihoods["at <time-of-day>"] - want).abs() < 1e-15);
        assert!((c.ok.likelihoods["hour"] - want).abs() < 1e-15);
    }
}
