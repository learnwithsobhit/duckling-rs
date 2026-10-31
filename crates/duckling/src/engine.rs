//! Port of `Duckling.Engine` and `Duckling.Types.Stash`: a saturating chart parser.
//!
//! The control flow mirrors the Haskell code function by function so behaviour (including
//! quirks such as non-anchored `matchOnce` followed by an adjacency filter) is preserved.

use std::collections::BTreeMap;
use std::rc::Rc;

use crate::document::Document;
use crate::locale::Lang;
use crate::regex_tr::{Re, Spans};
use crate::types::{
    Context, GroupMatch, Node, Options, PatternItem, Range, Resolved, Rule, Token,
};

/// Set of nodes grouped by start position (`Duckling.Types.Stash`).
#[derive(Default, Clone)]
pub struct Stash {
    map: BTreeMap<usize, Vec<Rc<Node>>>,
}

impl Stash {
    pub fn empty() -> Stash {
        Stash::default()
    }

    pub fn is_null(&self) -> bool {
        self.map.is_empty()
    }

    fn insert(&mut self, n: Rc<Node>) {
        let bucket = self.map.entry(n.range.start).or_default();
        if !bucket.iter().any(|x| Rc::ptr_eq(x, &n) || **x == *n) {
            bucket.push(n);
        }
    }

    pub fn from_list(ns: impl IntoIterator<Item = Rc<Node>>) -> Stash {
        let mut s = Stash::empty();
        for n in ns {
            s.insert(n);
        }
        s
    }

    pub fn union(&self, other: &Stash) -> Stash {
        let mut s = self.clone();
        for ns in other.map.values() {
            for n in ns {
                s.insert(n.clone());
            }
        }
        s
    }

    pub fn to_pos_ordered_list(&self) -> Vec<Rc<Node>> {
        self.map.values().flat_map(|v| v.iter().cloned()).collect()
    }

    /// Nodes starting at or after `pos`, in ascending start order.
    pub fn from_pos<'a>(&'a self, pos: usize) -> impl Iterator<Item = &'a Rc<Node>> + 'a {
        self.map.range(pos..).flat_map(|(_, v)| v.iter())
    }
}

/// A partially matched rule: which pattern item is next, where we are and the nodes so far.
#[derive(Clone)]
struct Match<'r> {
    rule: &'r Rule,
    /// Index of the next pattern item to match; `== pattern.len()` means complete.
    next: usize,
    position: usize,
    route: Vec<Rc<Node>>,
}

impl Match<'_> {
    fn done(&self) -> bool {
        self.next >= self.rule.pattern.len()
    }
}

/// `parseAndResolve`.
pub fn parse_and_resolve(
    rules: &[&Rule],
    input: &str,
    context: &Context,
    options: &Options,
) -> Vec<Resolved> {
    let lang = context.locale.lang;
    let doc = Document::new(input);
    let stash = parse_string(lang, rules, &doc);
    stash
        .to_pos_ordered_list()
        .into_iter()
        .filter_map(|n| resolve_node(context, options, n))
        .collect()
}

fn resolve_node(context: &Context, options: &Options, n: Rc<Node>) -> Option<Resolved> {
    let (val, latent) = n.token.resolve(context, options)?;
    let value_text = val.encode();
    Some(Resolved {
        range: n.range,
        dim: n.token.dim(),
        node: n,
        value: val,
        value_text,
        is_latent: latent,
    })
}

fn parse_string<'r>(lang: Lang, rules: &[&'r Rule], doc: &Document) -> Stash {
    // On the first pass we try all the rules.
    let (new, partial) = parse_string1(lang, rules, doc, &Stash::empty(), &Stash::empty(), &[]);
    if new.is_null() {
        return Stash::empty();
    }
    // For subsequent passes, we only try rules starting with a predicate.
    let head_predicate_rules: Vec<&Rule> = rules
        .iter()
        .copied()
        .filter(|r| matches!(r.pattern.first(), Some(PatternItem::Predicate(_))))
        .collect();
    saturate_parse_string(lang, &head_predicate_rules, doc, new.clone(), new, partial)
}

fn saturate_parse_string<'r>(
    lang: Lang,
    rules: &[&'r Rule],
    doc: &Document,
    mut stash: Stash,
    mut new: Stash,
    mut matches: Vec<Match<'r>>,
) -> Stash {
    loop {
        let (new2, matches2) = parse_string1(lang, rules, doc, &stash, &new, &matches);
        let stash2 = stash.union(&new2);
        if new2.is_null() {
            return stash;
        }
        stash = stash2;
        new = new2;
        matches = matches2;
    }
}

fn parse_string1<'r>(
    lang: Lang,
    rules: &[&'r Rule],
    doc: &Document,
    stash: &Stash,
    new: &Stash,
    matches: &[Match<'r>],
) -> (Stash, Vec<Match<'r>>) {
    // Find which `matches` can advance because of `new`.
    let mut new_partial: Vec<Match<'r>> = Vec::new();
    for m in matches {
        new_partial.extend(match_first(doc, lang, new, m));
    }
    // Find new matches resulting from newly added tokens (`new`).
    for r in rules {
        new_partial.extend(match_first_anywhere(doc, lang, new, r));
    }
    let all = match_all(doc, lang, stash, new_partial);
    let (full, partial): (Vec<Match<'r>>, Vec<Match<'r>>) = all.into_iter().partition(|m| m.done());
    let mut result_partial = partial;
    result_partial.extend(matches.iter().cloned());
    (Stash::from_list(full.into_iter().filter_map(produce)), result_partial)
}

fn match_all<'r>(
    doc: &Document,
    lang: Lang,
    stash: &Stash,
    matches: Vec<Match<'r>>,
) -> Vec<Match<'r>> {
    let mut out = Vec::new();
    for m in matches {
        mk_next_matches(doc, lang, stash, m, &mut out);
    }
    out
}

fn mk_next_matches<'r>(
    doc: &Document,
    lang: Lang,
    stash: &Stash,
    m: Match<'r>,
    out: &mut Vec<Match<'r>>,
) {
    if m.done() {
        out.push(m);
        return;
    }
    let is_regex = matches!(m.rule.pattern[m.next], PatternItem::Regex(_));
    let advanced = match_first(doc, lang, stash, &m);
    let mut next_matches = Vec::new();
    for a in advanced {
        mk_next_matches(doc, lang, stash, a, &mut next_matches);
    }
    if !is_regex {
        out.push(m);
    }
    out.extend(next_matches);
}

fn produce(m: Match<'_>) -> Option<Rc<Node>> {
    let route = m.route;
    let last = route.last()?;
    let first = route.first()?;
    let range = Range::new(first.range.start, last.range.end);
    let tokens: Vec<&Token> = route.iter().map(|n| &n.token).collect();
    let token = (m.rule.prod)(&tokens)?;
    drop(tokens);
    Some(Rc::new(Node::new(range, token, route, Some(m.rule.name.clone()))))
}

fn match_first<'r>(doc: &Document, lang: Lang, stash: &Stash, m: &Match<'r>) -> Vec<Match<'r>> {
    if m.done() {
        return Vec::new();
    }
    lookup_item(doc, lang, &m.rule.pattern[m.next], stash, m.position)
        .into_iter()
        .map(|n| mk_match(m.rule, m.next + 1, &m.route, n))
        .collect()
}

fn match_first_anywhere<'r>(
    doc: &Document,
    lang: Lang,
    stash: &Stash,
    rule: &'r Rule,
) -> Vec<Match<'r>> {
    if rule.pattern.is_empty() {
        return Vec::new();
    }
    lookup_item_anywhere(doc, lang, &rule.pattern[0], stash)
        .into_iter()
        .map(|n| mk_match(rule, 1, &[], n))
        .collect()
}

fn mk_match<'r>(rule: &'r Rule, next: usize, route: &[Rc<Node>], node: Rc<Node>) -> Match<'r> {
    let position = node.range.end;
    let mut new_route = Vec::with_capacity(route.len() + 1);
    new_route.extend(route.iter().cloned());
    new_route.push(node);
    Match { rule, next, position, route: new_route }
}

fn is_position_valid(position: usize, doc: &Document, n: &Node) -> bool {
    doc.is_adjacent(position, n.range.start)
}

fn lookup_item(
    doc: &Document,
    lang: Lang,
    item: &PatternItem,
    stash: &Stash,
    position: usize,
) -> Vec<Rc<Node>> {
    match item {
        PatternItem::Regex(re) => lookup_regex(doc, lang, re, position)
            .into_iter()
            .filter(|n| is_position_valid(position, doc, n))
            .collect(),
        PatternItem::Predicate(p) => stash
            .from_pos(position)
            .take_while(|n| is_position_valid(position, doc, n))
            .filter(|n| p(&n.token))
            .cloned()
            .collect(),
    }
}

fn lookup_item_anywhere(
    doc: &Document,
    lang: Lang,
    item: &PatternItem,
    stash: &Stash,
) -> Vec<Rc<Node>> {
    match item {
        PatternItem::Regex(re) => lookup_regex_anywhere(doc, lang, re),
        PatternItem::Predicate(p) => {
            stash.to_pos_ordered_list().into_iter().filter(|n| p(&n.token)).collect()
        }
    }
}

/// `lookupRegex`: the first match in the text from `position` (not anchored!).
fn lookup_regex(doc: &Document, lang: Lang, re: &Re, position: usize) -> Vec<Rc<Node>> {
    if position >= doc.len() {
        return Vec::new();
    }
    let sub = doc.substring_from(position);
    match re.match_once(sub) {
        Some(spans) => regex_node(doc, lang, position, &spans).into_iter().collect(),
        None => Vec::new(),
    }
}

/// `lookupRegexAnywhere`: all matches over the whole text.
fn lookup_regex_anywhere(doc: &Document, lang: Lang, re: &Re) -> Vec<Rc<Node>> {
    re.match_all(doc.substring_from(0))
        .iter()
        .filter_map(|spans| regex_node(doc, lang, 0, spans))
        .collect()
}

fn regex_node(doc: &Document, lang: Lang, position: usize, spans: &Spans) -> Option<Rc<Node>> {
    let (bs_start, bs_end) = spans.first().copied().flatten()?;
    if bs_start == 0 && bs_end == 0 {
        return None;
    }
    let base = doc.byte_offset(position);
    let start = doc.char_offset(base + bs_start);
    let end = doc.char_offset(base + bs_end);
    if !doc.is_range_valid(lang, start, end) {
        return None;
    }
    let groups: Vec<Rc<str>> = spans[1..]
        .iter()
        .map(|g| match g {
            Some((s, e)) => Rc::from(&doc.raw()[base + s..base + e]),
            None => Rc::from(""),
        })
        .collect();
    Some(Rc::new(Node::new(
        Range::new(start, end),
        Token::RegexMatch(GroupMatch(groups)),
        Vec::new(),
        None,
    )))
}
