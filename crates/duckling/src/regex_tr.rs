//! PCRE -> `fancy-regex` pattern translation.
//!
//! Haskell Duckling compiles patterns with `PCRE.compCaseless + PCRE.compUTF8` and **no**
//! `compUcp`, so `\d`, `\w`, `\s` and `\b` only look at ASCII, while case-insensitive matching
//! still applies Unicode case folding. Rust regex engines default to Unicode semantics, so we
//! rewrite those escapes to explicit ASCII classes. We also reproduce PCRE's `$` (end of subject
//! or before a final newline) and treat a `{` that does not start a quantifier as a literal.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::regex_prefilter::{Prefilter, PrefilterHay};

/// Intern compiled patterns so identical rule regexes share one engine and cache key.
pub fn intern(pattern: &str) -> Arc<Re> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Re>>>> = OnceLock::new();
    let mut guard = CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap();
    guard.entry(pattern.to_string()).or_insert_with(|| Arc::new(Re::new(pattern))).clone()
}

const WORD: &str = "a-zA-Z0-9_";
const SPACE: &str = " \\t\\n\\x0B\\f\\r";

fn ascii_boundary() -> String {
    format!(
        "(?:(?<=[{WORD}])(?![{WORD}])|(?<![{WORD}])(?=[{WORD}]))",
        WORD = WORD
    )
}

fn ascii_non_boundary() -> String {
    format!(
        "(?:(?<=[{WORD}])(?=[{WORD}])|(?<![{WORD}])(?![{WORD}]))",
        WORD = WORD
    )
}

/// Returns true if `s` (starting right after a `{`) is a valid quantifier body `n}`, `n,}`, `n,m}`.
fn is_quantifier_body(s: &[char]) -> bool {
    let mut i = 0;
    let digits = |i: &mut usize| {
        let st = *i;
        while *i < s.len() && s[*i].is_ascii_digit() {
            *i += 1;
        }
        *i > st
    };
    if !digits(&mut i) {
        return false;
    }
    if i < s.len() && s[i] == ',' {
        i += 1;
        digits(&mut i);
    }
    i < s.len() && s[i] == '}'
}

/// Expansion of a class escape (`d`, `w`, `s`) when used inside `[...]`.
fn class_escape_body(c: char) -> Option<&'static str> {
    match c {
        'd' => Some("0-9"),
        'w' => Some("a-zA-Z0-9_"),
        's' => Some(SPACE),
        _ => None,
    }
}

/// Translate a PCRE pattern (as used by Duckling rules) into a `fancy-regex` pattern.
pub fn translate(pattern: &str) -> String {
    translate_impl(pattern, false)
}

/// Same rewrite, but `\b`/`$` stay in a form the DFA `regex` crate can compile.
fn translate_rust(pattern: &str) -> String {
    translate_impl(pattern, true)
}

fn translate_impl(pattern: &str, rust_regex: bool) -> String {
    let p: Vec<char> = pattern.chars().collect();
    let mut out = String::with_capacity(pattern.len() + 16);
    let mut i = 0;
    while i < p.len() {
        let c = p[i];
        match c {
            '\\' => {
                if i + 1 >= p.len() {
                    out.push_str("\\\\");
                    i += 1;
                    continue;
                }
                let n = p[i + 1];
                match n {
                    'd' => out.push_str("[0-9]"),
                    'D' => out.push_str("[^0-9]"),
                    'w' => out.push_str(&format!("[{}]", WORD)),
                    'W' => out.push_str(&format!("[^{}]", WORD)),
                    's' => out.push_str(&format!("[{}]", SPACE)),
                    'S' => out.push_str(&format!("[^{}]", SPACE)),
                    'b' if rust_regex => out.push_str("(?-u:\\b)"),
                    'B' if rust_regex => out.push_str("(?-u:\\B)"),
                    'b' => out.push_str(&ascii_boundary()),
                    'B' => out.push_str(&ascii_non_boundary()),
                    _ => {
                        out.push('\\');
                        out.push(n);
                    }
                }
                i += 2;
            }
            '[' => {
                i = translate_class(&p, i, &mut out);
            }
            '$' if rust_regex => {
                out.push('$');
                i += 1;
            }
            '$' => {
                out.push_str("(?=\\n?\\z)");
                i += 1;
            }
            '{' => {
                if is_quantifier_body(&p[i + 1..]) && i > 0 {
                    out.push('{');
                } else {
                    out.push_str("\\{");
                }
                i += 1;
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// Translates a character class starting at `p[start] == '['`; returns index after the class.
fn translate_class(p: &[char], start: usize, out: &mut String) -> usize {
    let mut i = start + 1;
    out.push('[');
    if i < p.len() && p[i] == '^' {
        out.push('^');
        i += 1;
    }
    // A leading `]` is a literal in PCRE.
    if i < p.len() && p[i] == ']' {
        out.push_str("\\]");
        i += 1;
    }
    // Tracks whether the previous emitted item was a class escape (so a following `-` is literal).
    let mut prev_class_escape = false;
    while i < p.len() {
        let c = p[i];
        match c {
            ']' => {
                out.push(']');
                return i + 1;
            }
            '[' => {
                // POSIX class like [:alpha:]
                if i + 1 < p.len() && p[i + 1] == ':' {
                    if let Some(end) = find_posix_end(p, i) {
                        for ch in &p[i..=end] {
                            out.push(*ch);
                        }
                        i = end + 1;
                        prev_class_escape = false;
                        continue;
                    }
                }
                out.push_str("\\[");
                i += 1;
                prev_class_escape = false;
            }
            '\\' if i + 1 < p.len() => {
                let n = p[i + 1];
                if let Some(body) = class_escape_body(n) {
                    out.push_str(body);
                    prev_class_escape = true;
                } else if n == 'D' {
                    out.push_str("[^0-9]");
                    prev_class_escape = true;
                } else if n == 'W' {
                    out.push_str(&format!("[^{}]", WORD));
                    prev_class_escape = true;
                } else if n == 'S' {
                    out.push_str(&format!("[^{}]", SPACE));
                    prev_class_escape = true;
                } else if n == 'b' {
                    out.push_str("\\x08");
                    prev_class_escape = false;
                } else {
                    out.push('\\');
                    out.push(n);
                    prev_class_escape = false;
                }
                i += 2;
                // `-` right before another class escape is literal in PCRE.
                if i + 1 < p.len() && p[i] == '-' && p[i + 1] == '\\' && i + 2 < p.len() {
                    if class_escape_body(p[i + 2]).is_some() || "DWS".contains(p[i + 2]) {
                        out.push_str("\\-");
                        i += 1;
                    }
                }
            }
            '-' => {
                let next_is_end = i + 1 < p.len() && p[i + 1] == ']';
                if prev_class_escape && !next_is_end {
                    out.push_str("\\-");
                } else {
                    out.push('-');
                }
                prev_class_escape = false;
                i += 1;
            }
            '&' => {
                out.push_str("\\&");
                i += 1;
                prev_class_escape = false;
            }
            '~' => {
                out.push_str("\\~");
                i += 1;
                prev_class_escape = false;
            }
            _ => {
                out.push(c);
                i += 1;
                prev_class_escape = false;
            }
        }
    }
    // Unterminated class: let the regex compiler report it.
    i
}

fn find_posix_end(p: &[char], start: usize) -> Option<usize> {
    let mut j = start + 2;
    while j + 1 < p.len() {
        if p[j] == ':' && p[j + 1] == ']' {
            return Some(j + 1);
        }
        if !p[j].is_ascii_alphabetic() && p[j] != '^' {
            return None;
        }
        j += 1;
    }
    None
}

fn split_top_level_alts(pat: &str) -> Vec<String> {
    crate::regex_prefilter::split_top_level_alts(pat)
}

/// Drop empty top-level alternatives (`فوق||foo` → `فوق|foo`) so a
/// `PCRE_NOTEMPTY|PCRE_ANCHORED` retry can see the next alternative.
fn strip_empty_top_alts(pat: &str) -> String {
    let parts = split_top_level_alts(pat);
    let kept: Vec<String> = parts.into_iter().filter(|p| !p.is_empty()).collect();
    if kept.is_empty() {
        pat.to_string()
    } else {
        kept.join("|")
    }
}

#[derive(Debug)]
enum Engine {
    /// DFA engine (`regex` crate) — used when the pattern has no lookaround/backrefs.
    Rust { re: regex::Regex, re_notempty: regex::Regex },
    Fancy { re: fancy_regex::Regex, re_notempty: fancy_regex::Regex },
}

/// A compiled rule pattern.
#[derive(Debug)]
pub struct Re {
    engine: Engine,
    source: String,
    n_groups: usize,
    prefilter: Prefilter,
}

/// Byte spans of a match: index 0 is the whole match, then capture groups (None = unset).
pub type Spans = Vec<Option<(usize, usize)>>;

/// The DFA engine cannot express PCRE `$` (end or before a final newline) or lookaround.
fn needs_fancy_engine(pat: &str) -> bool {
    let p: Vec<char> = pat.chars().collect();
    let mut i = 0;
    let mut in_class = false;
    while i < p.len() {
        if p[i] == '\\' && i + 1 < p.len() {
            let n = p[i + 1];
            if !in_class && n.is_ascii_digit() && n != '0' {
                return true;
            }
            i += 2;
            continue;
        }
        if in_class {
            if p[i] == ']' {
                in_class = false;
            }
            i += 1;
            continue;
        }
        match p[i] {
            '[' => {
                in_class = true;
                i += 1;
            }
            '$' => return true,
            '(' if i + 2 < p.len() && p[i + 1] == '?' && matches!(p[i + 2], '=' | '!' | '<') => {
                return true;
            }
            _ => i += 1,
        }
    }
    false
}

impl Re {
    /// Compiles a PCRE pattern, case-insensitively (as `Duckling.Types.regex` does).
    pub fn new(pattern: &str) -> Re {
        let stripped = strip_empty_top_alts(pattern);
        let prefilter = Prefilter::from_pattern(pattern);
        if !needs_fancy_engine(pattern) && !needs_fancy_engine(&stripped) {
            let rust_src = format!("(?i){}", translate_rust(pattern));
            let rust_ne = format!("(?i){}", translate_rust(&stripped));
            if let (Ok(re), Ok(re_notempty)) = (regex::Regex::new(&rust_src), regex::Regex::new(&rust_ne)) {
                let n_groups = re.captures_len().saturating_sub(1);
                return Re {
                    engine: Engine::Rust { re, re_notempty },
                    source: pattern.to_string(),
                    n_groups,
                    prefilter,
                };
            }
        }
        let translated = format!("(?i){}", translate(pattern));
        let re = fancy_regex::Regex::new(&translated).unwrap_or_else(|e| {
            panic!("failed to compile pattern {:?} (translated {:?}): {}", pattern, translated, e)
        });
        let ne_src = format!("(?i){}", translate(&stripped));
        let re_notempty = fancy_regex::Regex::new(&ne_src).unwrap_or_else(|_| {
            fancy_regex::Regex::new(&translated).expect("original pattern compiled")
        });
        let n_groups = re.captures_len() - 1;
        Re {
            engine: Engine::Fancy { re, re_notempty },
            source: pattern.to_string(),
            n_groups,
            prefilter,
        }
    }

    /// Conservative skip: `false` means the regex cannot match this suffix.
    pub fn may_match(&self, hay: &PrefilterHay, from_byte: usize) -> bool {
        self.prefilter.may_match(hay, from_byte)
    }

    pub fn prefilter(&self) -> &Prefilter {
        &self.prefilter
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn n_groups(&self) -> usize {
        self.n_groups
    }

    #[cfg(test)]
    pub(crate) fn uses_dfa(&self) -> bool {
        matches!(self.engine, Engine::Rust { .. })
    }

    fn fancy_spans(caps: &fancy_regex::Captures<'_>) -> Spans {
        (0..caps.len()).map(|g| caps.get(g).map(|m| (m.start(), m.end()))).collect()
    }

    fn rust_spans(caps: &regex::Captures<'_>, shift: usize) -> Spans {
        (0..caps.len())
            .map(|g| caps.get(g).map(|m| (m.start() + shift, m.end() + shift)))
            .collect()
    }

    /// First match anywhere in `text` (like `matchOnce`); `text` is the whole subject.
    pub fn match_once(&self, text: &str) -> Option<Spans> {
        match &self.engine {
            Engine::Rust { re, .. } => re.captures(text).map(|c| Self::rust_spans(&c, 0)),
            Engine::Fancy { re, .. } => match re.captures(text) {
                Ok(Some(c)) => Some(Self::fancy_spans(&c)),
                _ => None,
            },
        }
    }

    /// `wrapMatchAll`: after a zero-width match, retry at the same offset with
    /// `PCRE_NOTEMPTY|PCRE_ANCHORED`. If that fails, stop (do not walk the rest
    /// of the subject one character at a time).
    pub fn match_all(&self, text: &str) -> Vec<Spans> {
        match &self.engine {
            Engine::Rust { re, re_notempty } => Self::match_all_rust(text, re, re_notempty),
            Engine::Fancy { re, re_notempty } => Self::match_all_fancy(text, re, re_notempty),
        }
    }

    fn match_all_fancy(
        text: &str,
        re: &fancy_regex::Regex,
        re_notempty: &fancy_regex::Regex,
    ) -> Vec<Spans> {
        let mut res = Vec::new();
        let mut pos = 0usize;
        let mut notempty_anchored = false;
        while pos <= text.len() {
            let caps = if notempty_anchored {
                match re_notempty.captures_from_pos(text, pos) {
                    Ok(Some(c)) => {
                        let m = c.get(0).unwrap();
                        if m.start() == pos && m.end() > pos {
                            Some(c)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                match re.captures_from_pos(text, pos) {
                    Ok(Some(c)) => Some(c),
                    _ => None,
                }
            };
            match caps {
                Some(c) => {
                    let m = c.get(0).unwrap();
                    let (s, e) = (m.start(), m.end());
                    res.push(Self::fancy_spans(&c));
                    if s == e {
                        if s == text.len() {
                            break;
                        }
                        notempty_anchored = true;
                        pos = e;
                    } else {
                        notempty_anchored = false;
                        pos = e;
                    }
                }
                None => break,
            }
        }
        res
    }

    fn match_all_rust(text: &str, re: &regex::Regex, re_notempty: &regex::Regex) -> Vec<Spans> {
        let mut res = Vec::new();
        let mut pos = 0usize;
        let mut notempty_anchored = false;
        while pos <= text.len() {
            let suffix = &text[pos..];
            let caps = if notempty_anchored {
                re_notempty.captures(suffix).and_then(|c| {
                    let m = c.get(0).unwrap();
                    if m.start() == 0 && m.end() > 0 {
                        Some(c)
                    } else {
                        None
                    }
                })
            } else {
                re.captures(suffix)
            };
            match caps {
                Some(c) => {
                    let m = c.get(0).unwrap();
                    let (s, e) = (pos + m.start(), pos + m.end());
                    res.push(Self::rust_spans(&c, pos));
                    if s == e {
                        if s == text.len() {
                            break;
                        }
                        notempty_anchored = true;
                        pos = e;
                    } else {
                        notempty_anchored = false;
                        pos = e;
                    }
                }
                None => break,
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_classes_only() {
        let r = Re::new(r"\w+");
        let m = r.match_once("éab").unwrap();
        assert_eq!(m[0], Some((2, 4)));
        assert!(Re::new(r"\bfoo\b").match_once("xfoo").is_none());
        assert!(Re::new(r"\bfoo\b").match_once("a foo b").is_some());
        assert!(Re::new(r"\d+").match_once("٣").is_none());
    }

    #[test]
    fn caseless_unicode() {
        assert!(Re::new("straße").match_once("STRAßE").is_some());
        assert!(Re::new("é").match_once("É").is_some());
    }

    #[test]
    fn class_dash_after_escape() {
        // phone-number pattern fragment: [\s-\.]
        let r = Re::new(r"[\s-\.]+");
        let m = r.match_once("a - . b").unwrap();
        assert_eq!(m[0], Some((1, 6)));
        assert!(Re::new(r"[\s-\.]").match_once("x").is_none());
    }

    #[test]
    fn dollar_matches_before_final_newline() {
        let r = Re::new(r"abc$");
        assert!(!r.uses_dfa());
        assert!(r.match_once("abc\n").is_some());
        assert!(r.match_once("abc\nx").is_none());
    }

    #[test]
    fn literal_brace() {
        assert!(Re::new(r"a{").match_once("a{").is_some());
        assert!(Re::new(r"a{2}").match_once("aa").is_some());
    }

    #[test]
    fn groups_unset() {
        let r = Re::new(r"(a)|(b)");
        let m = r.match_once("b").unwrap();
        assert_eq!(m[1], None);
        assert_eq!(m[2], Some((0, 1)));
        assert_eq!(r.n_groups(), 2);
    }

    #[test]
    fn simple_patterns_use_dfa() {
        assert!(Re::new("monday").uses_dfa());
        assert!(Re::new(r"\bfoo\b").uses_dfa());
        assert!(Re::new(r"\d{1,2}").uses_dfa());
    }
}
