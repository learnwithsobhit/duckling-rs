//! Conservative literal prefilter for Duckling PCRE rule patterns.
//!
//! A regex is skipped only when every top-level alternative has a required atom
//! that cannot appear in the subject. When extraction is uncertain the filter
//! falls back to "always run" so match semantics stay identical to Haskell PCRE.

/// What must be present in the subject for one alternative to have a chance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    /// ASCII-lowercase substring (search the ASCII-lowercased haystack).
    AsciiLit(String),
    /// Unicode-lowercase substring (search a Unicode-lowercased haystack).
    UniLit(String),
    Digit,
    Word,
    Space,
    /// Any of these lowercase chars must appear.
    OneOf(String),
}

/// Prefilter computed once when a rule regex is compiled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prefilter {
    /// Could not prove a required atom; always run the regex.
    Always,
    /// Run the regex if any constraint is satisfied.
    Any(Vec<Constraint>),
}

/// Case-folded view of a document used by [`Prefilter::may_match`].
#[derive(Debug, Clone)]
pub struct PrefilterHay {
    /// Same byte length as the original: only ASCII letters are lowered.
    ascii_lower: String,
    uni_lower: String,
    has_digit: bool,
    has_word: bool,
    has_space: bool,
}

impl PrefilterHay {
    pub fn new(text: &str) -> PrefilterHay {
        let ascii_lower: String = text
            .chars()
            .map(|c| if c.is_ascii_uppercase() { c.to_ascii_lowercase() } else { c })
            .collect();
        let has_digit = text.bytes().any(|b| b.is_ascii_digit());
        let has_word = text.bytes().any(|b| b.is_ascii_alphanumeric() || b == b'_');
        let has_space = text.bytes().any(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\x0B' | b'\x0C' | b'\r'));
        PrefilterHay {
            ascii_lower,
            uni_lower: text.to_lowercase(),
            has_digit,
            has_word,
            has_space,
        }
    }

    pub fn ascii_lower(&self) -> &str {
        &self.ascii_lower
    }
}

impl Prefilter {
    pub fn from_pattern(pattern: &str) -> Prefilter {
        let mut cs = Vec::new();
        for alt in split_top_level_alts(pattern) {
            match alt_constraints(&alt) {
                Some(more) => cs.extend(more),
                None => return Prefilter::Always,
            }
        }
        if cs.is_empty() {
            Prefilter::Always
        } else {
            Prefilter::Any(cs)
        }
    }

    /// `from_byte` is a UTF-8 offset into the original subject (and `ascii_lower`).
    pub fn may_match(&self, hay: &PrefilterHay, from_byte: usize) -> bool {
        match self {
            Prefilter::Always => true,
            Prefilter::Any(cs) => cs.iter().any(|c| constraint_holds(c, hay, from_byte)),
        }
    }
}

fn constraint_holds(c: &Constraint, hay: &PrefilterHay, from_byte: usize) -> bool {
    match c {
        Constraint::AsciiLit(s) => {
            let h = if from_byte == 0 {
                hay.ascii_lower.as_str()
            } else if from_byte <= hay.ascii_lower.len() {
                &hay.ascii_lower[from_byte..]
            } else {
                ""
            };
            h.contains(s)
        }
        Constraint::UniLit(s) => {
            if from_byte == 0 {
                hay.uni_lower.contains(s)
            } else if from_byte < hay.ascii_lower.len() {
                // Unicode lowercase can change length; fold only the suffix.
                let raw_suffix = {
                    // ascii_lower has the same byte length as raw; from_byte is a raw offset.
                    // Reconstruct from ascii_lower suffix (non-ASCII bytes are unchanged).
                    hay.ascii_lower.get(from_byte..).unwrap_or("").to_lowercase()
                };
                raw_suffix.contains(s)
            } else {
                false
            }
        }
        Constraint::Digit => {
            if from_byte == 0 {
                hay.has_digit
            } else {
                hay.ascii_lower.get(from_byte..).is_some_and(|s| s.bytes().any(|b| b.is_ascii_digit()))
            }
        }
        Constraint::Word => {
            if from_byte == 0 {
                hay.has_word
            } else {
                hay.ascii_lower
                    .get(from_byte..)
                    .is_some_and(|s| s.bytes().any(|b| b.is_ascii_alphanumeric() || b == b'_'))
            }
        }
        Constraint::Space => {
            if from_byte == 0 {
                hay.has_space
            } else {
                hay.ascii_lower.get(from_byte..).is_some_and(|s| {
                    s.bytes().any(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\x0B' | b'\x0C' | b'\r'))
                })
            }
        }
        Constraint::OneOf(chars) => {
            let h = if from_byte == 0 {
                hay.ascii_lower.as_str()
            } else {
                hay.ascii_lower.get(from_byte..).unwrap_or("")
            };
            chars.chars().any(|ch| {
                if ch.is_ascii() {
                    h.contains(ch)
                } else {
                    hay.uni_lower.contains(ch)
                        || (from_byte > 0
                            && hay
                                .ascii_lower
                                .get(from_byte..)
                                .is_some_and(|s| s.to_lowercase().contains(ch)))
                }
            })
        }
    }
}

/// Split `pat` on top-level `|` (respecting `\`, `[]`, `()`).
pub fn split_top_level_alts(pat: &str) -> Vec<String> {
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

fn alt_constraints(alt: &str) -> Option<Vec<Constraint>> {
    let p: Vec<char> = alt.chars().collect();
    let mut i = 0;
    loop {
        if i >= p.len() {
            // Empty or all-optional: can match without a known atom.
            return None;
        }
        let atom = parse_atom(&p, &mut i)?;
        let q = parse_quantifier(&p, &mut i);
        if q.min == 0 {
            continue;
        }
        // Zero-width assertions (`\b`, `^`, `$`, lookaround) never constrain the subject.
        if matches!(&atom, Atom::Group(s) if s.is_empty()) {
            continue;
        }
        // Coalesce a run of required literal characters.
        if let Atom::Lit(ch) = atom {
            let mut lit = String::new();
            lit.push(ch);
            while i < p.len() {
                match parse_atom(&p, &mut i) {
                    Some(Atom::Lit(c2)) => {
                        let q2 = parse_quantifier(&p, &mut i);
                        if q2.min == 0 {
                            break;
                        }
                        lit.push(c2);
                    }
                    _ => break,
                }
            }
            return Some(vec![lit_constraint(lit)]);
        }
        return atom_constraints(atom);
    }
}

fn lit_constraint(lit: String) -> Constraint {
    if lit.is_ascii() {
        Constraint::AsciiLit(lit.to_ascii_lowercase())
    } else {
        Constraint::UniLit(lit.to_lowercase())
    }
}

#[derive(Debug)]
enum Atom {
    Lit(char),
    Digit,
    Word,
    Space,
    OneOf(String),
    Group(String),
    Other,
}

#[derive(Clone, Copy)]
struct Quant {
    min: u32,
}

fn parse_atom(p: &[char], i: &mut usize) -> Option<Atom> {
    if *i >= p.len() {
        return None;
    }
    let c = p[*i];
    match c {
        '\\' => {
            if *i + 1 >= p.len() {
                *i += 1;
                return Some(Atom::Lit('\\'));
            }
            let n = p[*i + 1];
            *i += 2;
            Some(match n {
                'd' => Atom::Digit,
                'w' => Atom::Word,
                's' => Atom::Space,
                'D' | 'W' | 'S' | '.' => Atom::Other,
                'b' | 'B' | 'A' | 'z' | 'Z' | 'G' => {
                    // Zero-width assertion: skip by treating as optional empty.
                    // Represented as a group-less no-op: empty literal that we ignore via Other+min0?
                    // Safer: treat as Other with implicit min 0 by returning a dedicated skip.
                    // Callers see Other + required → unfilterable. That drops `\bfoo`.
                    // `\b` is zero-width; we want to skip it. Fake min=0 via Space? No.
                    // Use empty Group("") which is all-optional... handle below.
                    Atom::Group(String::new())
                }
                'n' => Atom::Lit('\n'),
                't' => Atom::Lit('\t'),
                'r' => Atom::Lit('\r'),
                other => Atom::Lit(other),
            })
        }
        '[' => {
            let start = *i;
            *i = skip_class(p, *i);
            let body: String = p[start..*i].iter().collect();
            Some(class_atom(&body))
        }
        '(' => {
            *i += 1;
            let inner_start = *i;
            let mut depth = 1i32;
            let mut in_class = false;
            while *i < p.len() && depth > 0 {
                let ch = p[*i];
                if ch == '\\' && *i + 1 < p.len() {
                    *i += 2;
                    continue;
                }
                if in_class {
                    if ch == ']' {
                        in_class = false;
                    }
                    *i += 1;
                    continue;
                }
                match ch {
                    '[' => {
                        in_class = true;
                        *i += 1;
                    }
                    '(' => {
                        depth += 1;
                        *i += 1;
                    }
                    ')' => {
                        depth -= 1;
                        *i += 1;
                    }
                    _ => *i += 1,
                }
            }
            // inner is [inner_start, i-1) if we consumed ')'
            let end = if depth == 0 { *i - 1 } else { *i };
            let inner: String = p[inner_start..end].iter().collect();
            Some(normalize_group(inner))
        }
        '.' => {
            *i += 1;
            Some(Atom::Other)
        }
        '^' | '$' => {
            *i += 1;
            Some(Atom::Group(String::new()))
        }
        _ => {
            *i += 1;
            Some(Atom::Lit(c))
        }
    }
}

fn normalize_group(inner: String) -> Atom {
    let s = inner.as_str();
    if s.is_empty() {
        return Atom::Group(String::new());
    }
    if let Some(rest) = s.strip_prefix("?:") {
        return Atom::Group(rest.to_string());
    }
    if s.starts_with('?') {
        // Lookaround, flags, comments: zero-width / unknown. Skip (empty group).
        return Atom::Group(String::new());
    }
    Atom::Group(inner)
}

fn skip_class(p: &[char], start: usize) -> usize {
    let mut i = start + 1;
    if i < p.len() && p[i] == '^' {
        i += 1;
    }
    if i < p.len() && p[i] == ']' {
        i += 1;
    }
    while i < p.len() {
        if p[i] == '\\' && i + 1 < p.len() {
            i += 2;
            continue;
        }
        if p[i] == ']' {
            return i + 1;
        }
        i += 1;
    }
    i
}

fn class_atom(class: &str) -> Atom {
    // `class` includes the brackets, e.g. `[0-9]`.
    let chars: Vec<char> = class.chars().collect();
    if chars.len() < 2 || chars[0] != '[' {
        return Atom::Other;
    }
    let mut i = 1;
    let negated = i < chars.len() - 1 && chars[i] == '^';
    if negated {
        return Atom::Other;
    }
    let mut members = String::new();
    let mut saw_digit = false;
    let mut saw_word = false;
    let mut saw_space = false;
    let mut other = false;
    if i < chars.len() - 1 && chars[i] == ']' {
        members.push(']');
        i += 1;
    }
    while i < chars.len() && chars[i] != ']' {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'd' => saw_digit = true,
                'w' => saw_word = true,
                's' => saw_space = true,
                'D' | 'W' | 'S' => other = true,
                n => members.push(n),
            }
            i += 2;
            continue;
        }
        if chars[i] == '[' && i + 1 < chars.len() && chars[i + 1] == ':' {
            if let Some(end) = chars[i + 2..].iter().position(|&c| c == ':') {
                let name: String = chars[i + 2..i + 2 + end].iter().collect();
                match name.as_str() {
                    "digit" => saw_digit = true,
                    "space" => saw_space = true,
                    "word" | "alnum" => saw_word = true,
                    _ => other = true,
                }
                i = i + 2 + end + 2; // skip :]
                continue;
            }
        }
        if i + 2 < chars.len() && chars[i + 1] == '-' && chars[i + 2] != ']' {
            let a = chars[i];
            let b = chars[i + 2];
            if a.is_ascii() && b.is_ascii() && a <= b {
                let span = b as u32 - a as u32;
                if span <= 10 {
                    for cu in a..=b {
                        members.push(cu);
                    }
                } else if a == '0' && b == '9' {
                    saw_digit = true;
                } else {
                    other = true;
                }
            } else {
                other = true;
            }
            i += 3;
            continue;
        }
        members.push(chars[i]);
        i += 1;
    }
    if other {
        return Atom::Other;
    }
    let flags = (saw_digit as u8, saw_word as u8, saw_space as u8, !members.is_empty() as u8);
    match flags {
        (1, 0, 0, 0) => Atom::Digit,
        (0, 1, 0, 0) => Atom::Word,
        (0, 0, 1, 0) => Atom::Space,
        (0, 0, 0, 1) if members.chars().count() <= 12 => Atom::OneOf(members.to_lowercase()),
        _ => Atom::Other,
    }
}

fn parse_quantifier(p: &[char], i: &mut usize) -> Quant {
    if *i >= p.len() {
        return Quant { min: 1 };
    }
    let q = match p[*i] {
        '?' => {
            *i += 1;
            Quant { min: 0 }
        }
        '*' => {
            *i += 1;
            Quant { min: 0 }
        }
        '+' => {
            *i += 1;
            Quant { min: 1 }
        }
        '{' => {
            if let Some((min, next)) = parse_brace_quant(p, *i) {
                *i = next;
                Quant { min }
            } else {
                Quant { min: 1 }
            }
        }
        _ => return Quant { min: 1 },
    };
    // Lazy marker.
    if *i < p.len() && p[*i] == '?' {
        *i += 1;
    }
    q
}

fn parse_brace_quant(p: &[char], start: usize) -> Option<(u32, usize)> {
    // start at '{'. Returns (min, index after closing } and optional ?)
    let mut i = start + 1;
    let mut n = 0u32;
    let mut seen = false;
    while i < p.len() && p[i].is_ascii_digit() {
        seen = true;
        n = n.saturating_mul(10).saturating_add(p[i].to_digit(10).unwrap());
        i += 1;
    }
    if !seen || i >= p.len() {
        return None;
    }
    match p[i] {
        '}' => Some((n, i + 1)),
        ',' => {
            i += 1;
            while i < p.len() && p[i].is_ascii_digit() {
                i += 1;
            }
            if i < p.len() && p[i] == '}' {
                Some((n, i + 1))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn atom_constraints(atom: Atom) -> Option<Vec<Constraint>> {
    match atom {
        Atom::Lit(c) => Some(vec![lit_constraint(c.to_string())]),
        Atom::Digit => Some(vec![Constraint::Digit]),
        Atom::Word => Some(vec![Constraint::Word]),
        Atom::Space => Some(vec![Constraint::Space]),
        Atom::OneOf(s) => Some(vec![Constraint::OneOf(s)]),
        Atom::Group(inner) => {
            if inner.is_empty() {
                // Zero-width / skipped; treat as optional and continue — but we already
                // consumed it as the first *required* atom. Empty group with min>=1 is
                // still zero-width, so keep scanning: caller already stopped. Signal
                // "skip this atom" by returning empty vec? That would make Prefilter
                // empty → Always. Instead return None only when we truly cannot filter.
                // An empty required group is zero-width: equivalent to no atom. We cannot
                // recover the rest of the pattern here. Conservative: unfilterable.
                None
            } else {
                let mut out = Vec::new();
                for alt in split_top_level_alts(&inner) {
                    match alt_constraints(&alt) {
                        Some(cs) => out.extend(cs),
                        None => return None,
                    }
                }
                if out.is_empty() {
                    None
                } else {
                    Some(out)
                }
            }
        }
        Atom::Other => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hay(s: &str) -> PrefilterHay {
        PrefilterHay::new(s)
    }

    #[test]
    fn named_days_skip_when_absent() {
        let p = Prefilter::from_pattern("monday|tuesday|wednesday");
        assert!(!p.may_match(&hay("hello world"), 0));
        assert!(p.may_match(&hay("see you Tuesday"), 0));
    }

    #[test]
    fn optional_prefix() {
        let p = Prefilter::from_pattern("(this )?christmas");
        assert!(!p.may_match(&hay("Monday 3rd"), 0));
        assert!(p.may_match(&hay("this Christmas"), 0));
    }

    #[test]
    fn am_pm_not_false_skip() {
        // `a.?m.?` has a required 'a' of length 1 — still a constraint.
        let p = Prefilter::from_pattern(r"a\.?m\.?");
        assert!(p.may_match(&hay("9.30am"), 0));
        assert!(!p.may_match(&hay("xyz"), 0));
    }

    #[test]
    fn digits() {
        let p = Prefilter::from_pattern(r"\d{1,2}");
        assert!(matches!(p, Prefilter::Any(_)));
        assert!(!p.may_match(&hay("no numbers here"), 0));
        assert!(p.may_match(&hay("room 12"), 0));
    }

    #[test]
    fn unfilterable_dot() {
        let p = Prefilter::from_pattern(".*");
        assert_eq!(p, Prefilter::Always);
    }

    #[test]
    fn suffix_position() {
        let p = Prefilter::from_pattern("monday");
        let h = hay("monday and stuff");
        assert!(p.may_match(&h, 0));
        assert!(!p.may_match(&h, 6));
    }

    #[test]
    fn word_boundary_prefix() {
        let p = Prefilter::from_pattern(r"\bmonday\b");
        assert!(!p.may_match(&hay("hello"), 0));
        assert!(p.may_match(&hay("Monday"), 0));
    }
}
