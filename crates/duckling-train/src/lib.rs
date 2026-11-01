//! Classifier training / comparison for the Duckling Rust port.
//!
//! Ports `Duckling.Ranking.Train` (Naive Bayes + Laplace) and the Time-corpus
//! wiring in `Duckling.Ranking.Generate`. Retrained tables are compared to
//! `crates/duckling/data/classifiers/*.json` within the documented float
//! tolerance ([`compare::DEFAULT_ABS_TOL`], [`compare::DEFAULT_REL_TOL`]).

pub mod compare;
pub mod corpus;
pub mod dataset;
pub mod json_io;
pub mod nb;

pub use compare::{compare, Tol, DEFAULT_ABS_TOL, DEFAULT_REL_TOL};
pub use corpus::{exported_locales, load_corpus, TrainCorpus};
pub use dataset::train_locale;
pub use json_io::{classifiers_to_json, load_classifiers_file, parse_classifiers, write_classifiers};
pub use nb::{make_class, make_classifiers, train, ClassData, Classifier, Classifiers};

#[cfg(test)]
mod tests {
    use super::*;
    use duckling::{Lang, Locale};
    use std::path::PathBuf;

    fn exported_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../duckling/data/classifiers")
    }

    fn verify_locale(loc: Locale) {
        let corpus = load_corpus(loc).unwrap();
        let trained = train_locale(&corpus);
        let path = exported_dir().join(format!("{loc}.json"));
        let exported = load_classifiers_file(&path).unwrap();
        let d = compare(&trained, &exported, Tol::default());
        assert!(
            d.ok(),
            "{loc}: missing={:?} extra={:?} field={:?}",
            d.missing_rules,
            d.extra_rules,
            d.field
        );
    }

    #[test]
    fn reproduces_empty_and_ar_and_en() {
        verify_locale(Locale { lang: Lang::AF, region: None });
        verify_locale(Locale { lang: Lang::AR, region: None });
        verify_locale(Locale { lang: Lang::EN, region: None });
    }
}
