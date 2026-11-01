//! PCRE -> `fancy-regex` pattern translation.
//!
//! Haskell Duckling compiles patterns with `PCRE.compCaseless + PCRE.compUTF8` and **no**
//! `compUcp`, so `\d`, `\w`, `\s` and `\b` only look at ASCII, while case-insensitive matching
//! still applies Unicode case folding. Rust regex engines default to Unicode semantics, so we
//! rewrite those escapes to explicit ASCII classes. We also reproduce PCRE's `$` (end of subject
//! or before a final newline) and treat a `{` that does not start a quantifier as a literal.

use fancy_regex::Regex;

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

/// Split `pat` on top-level `|` (respecting `\`, `[]`, `()`).
fn split_top_level_alts(pat: &str) -> Vec<String> {
    let p: Vec<char> = pat.chars().collect();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut i = 0;
    let mut depth = 0i32;
    let mut in_class = false;
    while i < p.len() {
        let c = p[i];
        if c == '\\' && i + 1 < p.len() {
            cur.push(c);
            cur.push(p[i + 1]);
            i += 2;
            continue;
        }
        if in_class {
            cur.push(c);
            if c == ']' {
                in_class = false;
            }
            i += 1;
            continue;
        }
        match c {
            '[' => {
                in_class = true;
                cur.push(c);
            }
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth -= 1;
                cur.push(c);
            }
            '|' if depth == 0 => {
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
        i += 1;
    }
    out.push(cur);
    out
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

/// A compiled rule pattern.
#[derive(Debug)]
pub struct Re {
    re: Regex,
    /// Pattern with empty top-level `|` alternatives removed; used for the
    /// `wrapMatchAll` not-empty + anchored retry after a zero-width match.
    re_notempty: Regex,
    source: String,
    n_groups: usize,
}

/// Byte spans of a match: index 0 is the whole match, then capture groups (None = unset).
pub type Spans = Vec<Option<(usize, usize)>>;

impl Re {
    /// Compiles a PCRE pattern, case-insensitively (as `Duckling.Types.regex` does).
    pub fn new(pattern: &str) -> Re {
        let translated = format!("(?i){}", translate(pattern));
        let re = Regex::new(&translated).unwrap_or_else(|e| {
            panic!("failed to compile pattern {:?} (translated {:?}): {}", pattern, translated, e)
        });
        let stripped = strip_empty_top_alts(pattern);
        let ne_src = format!("(?i){}", translate(&stripped));
        let re_notempty = Regex::new(&ne_src).unwrap_or_else(|_| {
            // Fall back to the original if stripping produced something illegal.
            Regex::new(&translated).expect("original pattern compiled")
        });
        let n_groups = re.captures_len() - 1;
        Re { re, re_notempty, source: pattern.to_string(), n_groups }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn n_groups(&self) -> usize {
        self.n_groups
    }

    fn spans(caps: &fancy_regex::Captures<'_>) -> Spans {
        (0..caps.len()).map(|g| caps.get(g).map(|m| (m.start(), m.end()))).collect()
    }

    /// First match anywhere in `text` (like `matchOnce`); `text` is the whole subject.
    pub fn match_once(&self, text: &str) -> Option<Spans> {
        match self.re.captures(text) {
            Ok(Some(c)) => Some(Self::spans(&c)),
            _ => None,
        }
    }

    /// `wrapMatchAll`: after a zero-width match, retry at the same offset with
    /// `PCRE_NOTEMPTY|PCRE_ANCHORED`. If that fails, stop (do not walk the rest
    /// of the subject one character at a time).
    pub fn match_all(&self, text: &str) -> Vec<Spans> {
        let mut res = Vec::new();
        let mut pos = 0usize;
        let mut notempty_anchored = false;
        while pos <= text.len() {
            let caps = if notempty_anchored {
                match self.re_notempty.captures_from_pos(text, pos) {
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
                match self.re.captures_from_pos(text, pos) {
                    Ok(Some(c)) => Some(c),
                    _ => None,
                }
            };
            match caps {
                Some(c) => {
                    let m = c.get(0).unwrap();
                    let (s, e) = (m.start(), m.end());
                    res.push(Self::spans(&c));
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
        assert!(Re::new(r"abc$").match_once("abc\n").is_some());
        assert!(Re::new(r"abc$").match_once("abc\nx").is_none());
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
}
