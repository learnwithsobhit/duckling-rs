//! Port of `Duckling.Engine` and `Duckling.Types.Stash`: a saturating chart parser.
//!
//! The control flow mirrors the Haskell code function by function so behaviour (including
//! quirks such as non-anchored `matchOnce` followed by an adjacency filter) is preserved.
//!
//! Nodes live in a bump-style [`NodeArena`] (contiguous `Vec`, `NodeId` edges) for the
//! duration of a parse; `Rc<Node>` trees are materialized only for tokens that resolve.
//! Regexes are skipped when a conservative literal prefilter proves they cannot match.

use std::cell::Cell;
use std::collections::hash_map::DefaultHasher;
use std::time::Instant;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use crate::document::Document;
use crate::locale::Lang;
use crate::regex_tr::{Re, Spans};
use crate::types::{
    Context, GroupMatch, Node, Options, PatternItem, Range, Resolved, Rule, Token,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct NodeId(u32);

struct Slot {
    range: Range,
    token: Token,
    children: Vec<NodeId>,
    rule: Option<Arc<str>>,
    hash: u64,
}

struct NodeArena {
    nodes: Vec<Slot>,
    /// Interned `Re` pointer → `match_all` spans for this document.
    rx_all: HashMap<usize, Vec<Spans>>,
    /// `(Re pointer, start position)` → `match_once` spans.
    rx_once: HashMap<(usize, usize), Option<Spans>>,
    passes: u32,
    rx_run: u32,
    rx_cached: u32,
    produce_ok: u32,
    produce_fail: u32,
    matches: u32,
}

impl NodeArena {
    fn new() -> NodeArena {
        NodeArena {
            nodes: Vec::with_capacity(256),
            rx_all: HashMap::new(),
            rx_once: HashMap::new(),
            passes: 0,
            rx_run: 0,
            rx_cached: 0,
            produce_ok: 0,
            produce_fail: 0,
            matches: 0,
        }
    }

    fn re_key(re: &Re) -> usize {
        re as *const Re as usize
    }

    fn alloc(
        &mut self,
        range: Range,
        token: Token,
        children: Vec<NodeId>,
        rule: Option<Arc<str>>,
    ) -> NodeId {
        let mut h = DefaultHasher::new();
        range.hash(&mut h);
        token.hash(&mut h);
        rule.hash(&mut h);
        for c in &children {
            self.nodes[c.0 as usize].hash.hash(&mut h);
        }
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(Slot { range, token, children, rule, hash: h.finish() });
        id
    }

    fn get(&self, id: NodeId) -> &Slot {
        &self.nodes[id.0 as usize]
    }

    fn eq(&self, a: NodeId, b: NodeId) -> bool {
        if a == b {
            return true;
        }
        let na = self.get(a);
        let nb = self.get(b);
        na.hash == nb.hash
            && na.range == nb.range
            && na.rule == nb.rule
            && na.token == nb.token
            && na.children.len() == nb.children.len()
            && na.children.iter().zip(&nb.children).all(|(&x, &y)| self.eq(x, y))
    }

    fn materialize(&self, id: NodeId, memo: &mut Vec<Option<Rc<Node>>>) -> Rc<Node> {
        if let Some(n) = &memo[id.0 as usize] {
            return n.clone();
        }
        let s = self.get(id);
        let children: Vec<Rc<Node>> = s.children.iter().map(|&c| self.materialize(c, memo)).collect();
        let n = Rc::new(Node::new(s.range, s.token.clone(), children, s.rule.clone()));
        memo[id.0 as usize] = Some(n.clone());
        n
    }
}

/// Set of nodes grouped by start position (`Duckling.Types.Stash`).
#[derive(Default, Clone)]
pub struct Stash {
    map: BTreeMap<usize, Vec<NodeId>>,
}

impl Stash {
    pub fn empty() -> Stash {
        Stash::default()
    }

    pub fn is_null(&self) -> bool {
        self.map.is_empty()
    }

    fn insert(&mut self, arena: &NodeArena, n: NodeId) {
        let bucket = self.map.entry(arena.get(n).range.start).or_default();
        if !bucket.iter().any(|&x| x == n || arena.eq(x, n)) {
            bucket.push(n);
        }
    }

    fn from_list(arena: &NodeArena, ns: impl IntoIterator<Item = NodeId>) -> Stash {
        let mut s = Stash::empty();
        for n in ns {
            s.insert(arena, n);
        }
        s
    }

    fn union(&self, arena: &NodeArena, other: &Stash) -> Stash {
        let mut s = self.clone();
        for ns in other.map.values() {
            for &n in ns {
                s.insert(arena, n);
            }
        }
        s
    }

    fn to_pos_ordered_list(&self) -> Vec<NodeId> {
        self.map.values().flatten().copied().collect()
    }

    /// Nodes starting at or after `pos`, in ascending start order.
    fn from_pos(&self, pos: usize) -> impl Iterator<Item = NodeId> + '_ {
        self.map.range(pos..).flat_map(|(_, v)| v.iter().copied())
    }
}

/// A partially matched rule: which pattern item is next, where we are and the nodes so far.
#[derive(Clone)]
struct Match<'r> {
    rule: &'r Rule,
    /// Index of the next pattern item to match; `== pattern.len()` means complete.
    next: usize,
    position: usize,
    route: Vec<NodeId>,
}

impl Match<'_> {
    fn done(&self) -> bool {
        self.next >= self.rule.pattern.len()
    }
}

thread_local! {
    static LAST_NODES: Cell<usize> = const { Cell::new(0) };
    static LAST_PASSES: Cell<u32> = const { Cell::new(0) };
    static LAST_RX_RUN: Cell<u32> = const { Cell::new(0) };
    static LAST_RX_CACHED: Cell<u32> = const { Cell::new(0) };
    static LAST_PRODUCE_OK: Cell<u32> = const { Cell::new(0) };
    static LAST_PRODUCE_FAIL: Cell<u32> = const { Cell::new(0) };
    static LAST_MATCHES: Cell<u32> = const { Cell::new(0) };
    static LAST_PARSE_US: Cell<u64> = const { Cell::new(0) };
    static LAST_RESOLVE_US: Cell<u64> = const { Cell::new(0) };
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
    let mut arena = NodeArena::new();
    let t0 = Instant::now();
    let stash = parse_string(&mut arena, lang, rules, &doc);
    LAST_PARSE_US.with(|c| c.set(t0.elapsed().as_micros() as u64));
    LAST_NODES.with(|c| c.set(arena.nodes.len()));
    LAST_PASSES.with(|c| c.set(arena.passes));
    LAST_RX_RUN.with(|c| c.set(arena.rx_run));
    LAST_RX_CACHED.with(|c| c.set(arena.rx_cached));
    LAST_PRODUCE_OK.with(|c| c.set(arena.produce_ok));
    LAST_PRODUCE_FAIL.with(|c| c.set(arena.produce_fail));
    LAST_MATCHES.with(|c| c.set(arena.matches));
    let mut memo = vec![None; arena.nodes.len()];
    let t1 = Instant::now();
    let out = crate::dims::time::types::with_resolve_context(context, || {
        stash
            .to_pos_ordered_list()
            .into_iter()
            .filter_map(|id| resolve_slot(context, options, &arena, id, &mut memo))
            .collect()
    });
    LAST_RESOLVE_US.with(|c| c.set(t1.elapsed().as_micros() as u64));
    out
}

fn resolve_slot(
    context: &Context,
    options: &Options,
    arena: &NodeArena,
    id: NodeId,
    memo: &mut Vec<Option<Rc<Node>>>,
) -> Option<Resolved> {
    let slot = arena.get(id);
    let (val, latent) = slot.token.resolve(context, options)?;
    let value_text = val.encode();
    let n = arena.materialize(id, memo);
    Some(Resolved {
        range: slot.range,
        dim: n.token.dim(),
        node: n,
        value: val,
        value_text,
        is_latent: latent,
    })
}

fn parse_string<'r>(arena: &mut NodeArena, lang: Lang, rules: &[&'r Rule], doc: &Document) -> Stash {
    // On the first pass we try all the rules.
    let (new, partial) =
        parse_string1(arena, lang, rules, doc, &Stash::empty(), &Stash::empty(), &[]);
    if new.is_null() {
        return Stash::empty();
    }
    // For subsequent passes, we only try rules starting with a predicate.
    let head_predicate_rules: Vec<&Rule> = rules
        .iter()
        .copied()
        .filter(|r| matches!(r.pattern.first(), Some(PatternItem::Predicate(_))))
        .collect();
    saturate_parse_string(arena, lang, &head_predicate_rules, doc, new.clone(), new, partial)
}

fn saturate_parse_string<'r>(
    arena: &mut NodeArena,
    lang: Lang,
    rules: &[&'r Rule],
    doc: &Document,
    mut stash: Stash,
    mut new: Stash,
    mut matches: Vec<Match<'r>>,
) -> Stash {
    loop {
        arena.passes += 1;
        let (new2, matches2) = parse_string1(arena, lang, rules, doc, &stash, &new, &matches);
        let stash2 = stash.union(arena, &new2);
        if new2.is_null() {
            return stash;
        }
        stash = stash2;
        new = new2;
        matches = matches2;
    }
}

fn parse_string1<'r>(
    arena: &mut NodeArena,
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
        new_partial.extend(match_first(arena, doc, lang, new, m));
    }
    // Find new matches resulting from newly added tokens (`new`).
    for r in rules {
        new_partial.extend(match_first_anywhere(arena, doc, lang, new, r));
    }
    let all = match_all(arena, doc, lang, stash, new_partial);
    let (full, partial): (Vec<Match<'r>>, Vec<Match<'r>>) = all.into_iter().partition(|m| m.done());
    arena.matches += full.len() as u32 + partial.len() as u32;
    let mut result_partial = partial;
    result_partial.extend(matches.iter().cloned());
    let produced: Vec<NodeId> = full
        .into_iter()
        .filter_map(|m| {
            let n = produce(arena, m);
            if n.is_some() {
                arena.produce_ok += 1;
            } else {
                arena.produce_fail += 1;
            }
            n
        })
        .collect();
    (Stash::from_list(arena, produced), result_partial)
}

fn match_all<'r>(
    arena: &mut NodeArena,
    doc: &Document,
    lang: Lang,
    stash: &Stash,
    matches: Vec<Match<'r>>,
) -> Vec<Match<'r>> {
    let mut out = Vec::new();
    for m in matches {
        mk_next_matches(arena, doc, lang, stash, m, &mut out);
    }
    out
}

fn mk_next_matches<'r>(
    arena: &mut NodeArena,
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
    let advanced = match_first(arena, doc, lang, stash, &m);
    let mut next_matches = Vec::new();
    for a in advanced {
        mk_next_matches(arena, doc, lang, stash, a, &mut next_matches);
    }
    if !is_regex {
        out.push(m);
    }
    out.extend(next_matches);
}

fn produce(arena: &mut NodeArena, m: Match<'_>) -> Option<NodeId> {
    let route = m.route;
    let last = *route.last()?;
    let first = *route.first()?;
    let range = Range::new(arena.get(first).range.start, arena.get(last).range.end);
    let tokens: Vec<&Token> = route.iter().map(|&id| &arena.get(id).token).collect();
    let token = (m.rule.prod)(&tokens)?;
    drop(tokens);
    Some(arena.alloc(range, token, route, Some(m.rule.name.clone())))
}

fn match_first<'r>(
    arena: &mut NodeArena,
    doc: &Document,
    lang: Lang,
    stash: &Stash,
    m: &Match<'r>,
) -> Vec<Match<'r>> {
    if m.done() {
        return Vec::new();
    }
    lookup_item(arena, doc, lang, &m.rule.pattern[m.next], stash, m.position)
        .into_iter()
        .map(|n| mk_match(arena, m.rule, m.next + 1, &m.route, n))
        .collect()
}

fn match_first_anywhere<'r>(
    arena: &mut NodeArena,
    doc: &Document,
    lang: Lang,
    stash: &Stash,
    rule: &'r Rule,
) -> Vec<Match<'r>> {
    if rule.pattern.is_empty() {
        return Vec::new();
    }
    lookup_item_anywhere(arena, doc, lang, &rule.pattern[0], stash)
        .into_iter()
        .map(|n| mk_match(arena, rule, 1, &[], n))
        .collect()
}

fn mk_match<'r>(
    arena: &NodeArena,
    rule: &'r Rule,
    next: usize,
    route: &[NodeId],
    node: NodeId,
) -> Match<'r> {
    let position = arena.get(node).range.end;
    let mut new_route = Vec::with_capacity(route.len() + 1);
    new_route.extend_from_slice(route);
    new_route.push(node);
    Match { rule, next, position, route: new_route }
}

fn is_position_valid(position: usize, doc: &Document, start: usize) -> bool {
    doc.is_adjacent(position, start)
}

fn lookup_item(
    arena: &mut NodeArena,
    doc: &Document,
    lang: Lang,
    item: &PatternItem,
    stash: &Stash,
    position: usize,
) -> Vec<NodeId> {
    match item {
        PatternItem::Regex(re) => lookup_regex(arena, doc, lang, re, position)
            .into_iter()
            .filter(|&n| is_position_valid(position, doc, arena.get(n).range.start))
            .collect(),
        PatternItem::Predicate(p) => stash
            .from_pos(position)
            .take_while(|&n| is_position_valid(position, doc, arena.get(n).range.start))
            .filter(|&n| p(&arena.get(n).token))
            .collect(),
    }
}

fn lookup_item_anywhere(
    arena: &mut NodeArena,
    doc: &Document,
    lang: Lang,
    item: &PatternItem,
    stash: &Stash,
) -> Vec<NodeId> {
    match item {
        PatternItem::Regex(re) => lookup_regex_anywhere(arena, doc, lang, re),
        PatternItem::Predicate(p) => {
            stash.to_pos_ordered_list().into_iter().filter(|&n| p(&arena.get(n).token)).collect()
        }
    }
}

/// `lookupRegex`: the first match in the text from `position` (not anchored!).
fn lookup_regex(
    arena: &mut NodeArena,
    doc: &Document,
    lang: Lang,
    re: &Re,
    position: usize,
) -> Vec<NodeId> {
    if position >= doc.len() {
        return Vec::new();
    }
    let from_byte = doc.byte_offset(position);
    if !re.may_match(doc.prefilter(), from_byte) {
        return Vec::new();
    }
    let key = (NodeArena::re_key(re), position);
    let spans = if let Some(cached) = arena.rx_once.get(&key) {
        arena.rx_cached += 1;
        cached.clone()
    } else {
        arena.rx_run += 1;
        let found = re.match_once(doc.substring_from(position));
        arena.rx_once.insert(key, found.clone());
        found
    };
    match spans {
        Some(spans) => regex_node(arena, doc, lang, position, &spans).into_iter().collect(),
        None => Vec::new(),
    }
}

/// `lookupRegexAnywhere`: all matches over the whole text.
fn lookup_regex_anywhere(arena: &mut NodeArena, doc: &Document, lang: Lang, re: &Re) -> Vec<NodeId> {
    if !re.may_match(doc.prefilter(), 0) {
        return Vec::new();
    }
    let key = NodeArena::re_key(re);
    let all = if let Some(cached) = arena.rx_all.get(&key) {
        arena.rx_cached += 1;
        cached.clone()
    } else {
        arena.rx_run += 1;
        let found = re.match_all(doc.substring_from(0));
        arena.rx_all.insert(key, found.clone());
        found
    };
    all.iter().filter_map(|spans| regex_node(arena, doc, lang, 0, spans)).collect()
}

fn regex_node(
    arena: &mut NodeArena,
    doc: &Document,
    lang: Lang,
    position: usize,
    spans: &Spans,
) -> Option<NodeId> {
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
    Some(arena.alloc(Range::new(start, end), Token::RegexMatch(GroupMatch(groups)), Vec::new(), None))
}

#[cfg(test)]
mod prefilter_stats {
    use super::*;
    use crate::locale::Locale;
    use crate::regex_prefilter::PrefilterHay;
    use crate::rules::rules_for;
    use crate::types::Dim;
    use std::collections::HashSet;

    fn regex_heads(locale: Locale) -> Vec<&'static Re> {
        let rules = rules_for(locale, &HashSet::from([Dim::Time]));
        rules
            .into_iter()
            .filter_map(|r| match r.pattern.first() {
                Some(PatternItem::Regex(re)) => Some(re.as_ref()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn expensive_sentences_skip_absent_literals() {
        let en = "Monday 3rd at 9.30am or 2.30pm, Saturday 8th at 10.30am, Tuesday 11th at 2pm, Wednesday 12th at 2.30pm, Friday 14th at 12.30pm xx";
        let es = "Horario es de Lunes a Viernes de 2 pm a 10 pm. Sábado de 9 am a 7 pm";
        for (lang, text) in [(Lang::EN, en), (Lang::ES, es)] {
            let hay = PrefilterHay::new(text);
            let res = regex_heads(Locale::new(lang, None));
            let (skip, run, always) = res.iter().fold((0, 0, 0), |(s, r, a), re| {
                let always = matches!(re.prefilter(), crate::regex_prefilter::Prefilter::Always);
                if !re.may_match(&hay, 0) {
                    (s + 1, r, a + always as usize)
                } else {
                    (s, r + 1, a + always as usize)
                }
            });
            let dfa = res.iter().filter(|re| re.uses_dfa()).count();
            let unique: std::collections::HashSet<&str> = res.iter().map(|re| re.source()).collect();
            eprintln!(
                "{lang} time regex-heads: {} total, {} unique, {skip} skipped, {run} run, {always} unfiltered, {dfa} dfa",
                res.len(),
                unique.len()
            );
            let ctx = crate::types::Context {
                reference_time: crate::time_zone::DucklingTime {
                    utc: crate::time_zone::UtcTime {
                        mjd: crate::dims::time::calendar::from_gregorian(2013, 2, 12),
                        psecs: (4 * 3600 + 30 * 60) * 1_000_000_000_000i64,
                    },
                    series: crate::time_zone::TimeZoneSeries::fixed(-120),
                },
                locale: Locale::new(lang, None),
                time_zones: None,
            };
            let rules = rules_for(ctx.locale, &HashSet::from([Dim::Time]));
            let _ = parse_and_resolve(&rules, text, &ctx, &crate::types::Options { with_latent: false });
            eprintln!(
                "  nodes={} passes={} rx_run={} rx_cached={} produce={}/{} matches={} parse={:.2}ms resolve={:.2}ms",
                LAST_NODES.with(|c| c.get()),
                LAST_PASSES.with(|c| c.get()),
                LAST_RX_RUN.with(|c| c.get()),
                LAST_RX_CACHED.with(|c| c.get()),
                LAST_PRODUCE_OK.with(|c| c.get()),
                LAST_PRODUCE_FAIL.with(|c| c.get()),
                LAST_MATCHES.with(|c| c.get()),
                LAST_PARSE_US.with(|c| c.get()) as f64 / 1000.0,
                LAST_RESOLVE_US.with(|c| c.get()) as f64 / 1000.0,
            );
            assert!(skip > 0, "{lang} should skip some regexes");
        }
    }
}
