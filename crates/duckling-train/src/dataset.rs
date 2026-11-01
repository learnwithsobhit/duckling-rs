//! `makeDataset` / `makeDataset1` (`Duckling.Ranking.Train`).

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use duckling::engine::parse_and_resolve;
use duckling::ranking::extract_features;
use duckling::rules::rules_for;
use duckling::types::{Dim, Node};

use crate::corpus::{predicate_matches, TrainCorpus};
use crate::nb::{make_classifiers, BagOfFeatures, Classifiers, Datum};

/// `makeDataset` then `makeClassifiers` for one locale's Time rules and corpus.
pub fn train_locale(corpus: &TrainCorpus) -> Classifiers {
    let rules = rules_for(corpus.context.locale, &HashSet::from([Dim::Time]));
    let dataset = make_dataset(&rules, corpus);
    make_classifiers(&dataset)
}

/// `makeDataset`: rule name → labeled feature bags.
pub fn make_dataset(rules: &[&duckling::types::Rule], corpus: &TrainCorpus) -> HashMap<String, Vec<Datum>> {
    let mut dataset: HashMap<String, Vec<Datum>> = HashMap::new();
    for ex in &corpus.examples {
        make_dataset1(rules, corpus, ex, &mut dataset);
    }
    dataset
}

/// `makeDataset1`: parse one sentence, label contributing nodes ok/ko.
fn make_dataset1(
    rules: &[&duckling::types::Rule],
    corpus: &TrainCorpus,
    ex: &crate::corpus::Example,
    dataset: &mut HashMap<String, Vec<Datum>>,
) {
    let tokens = parse_and_resolve(rules, &ex.text, &corpus.context, &corpus.options);
    let (ok, ko): (Vec<_>, Vec<_>) =
        tokens.into_iter().partition(|t| predicate_matches(&ex.expected, &corpus.context, &t.value));

    let mut nodes_ok: HashSet<Rc<Node>> = HashSet::new();
    for t in &ok {
        collect_subnodes(&t.node, &mut nodes_ok);
    }
    let mut nodes_ko: HashSet<Rc<Node>> = HashSet::new();
    for t in &ko {
        collect_subnodes(&t.node, &mut nodes_ko);
    }
    nodes_ko.retain(|n| !nodes_ok.contains(n));

    update_dataset(true, &nodes_ok, dataset);
    update_dataset(false, &nodes_ko, dataset);
}

/// Internal nodes only (`subnodes`: leaves with no children are skipped).
fn collect_subnodes(node: &Rc<Node>, out: &mut HashSet<Rc<Node>>) {
    if node.children.is_empty() {
        return;
    }
    out.insert(node.clone());
    for ch in &node.children {
        collect_subnodes(ch, out);
    }
}

fn update_dataset(ok: bool, nodes: &HashSet<Rc<Node>>, dataset: &mut HashMap<String, Vec<Datum>>) {
    for node in nodes {
        let Some(rule) = node.rule.as_ref() else { continue };
        let bag: BagOfFeatures = extract_features(node).into_iter().collect();
        dataset.entry(rule.to_string()).or_default().push((bag, ok));
    }
}
