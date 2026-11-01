//! Port of `Duckling.Types.Document`.
//!
//! All ranges are expressed in Unicode scalar value (code point) offsets, like Haskell's
//! `Text`. Regexes run on the UTF-8 encoding, so we keep translation tables between
//! code point offsets and byte offsets.

use crate::locale::Lang;
use unicode_general_category::{get_general_category, GeneralCategory};

#[derive(Debug, Clone)]
pub struct Document {
    raw: String,
    chars: Vec<char>,
    /// For each char index `i` (0..=len): the byte offset of that char in `raw`.
    char_to_byte: Vec<usize>,
    /// For each byte offset (0..=raw.len()): the char index; non-boundary bytes round up.
    byte_to_char: Vec<usize>,
    /// For index `i`: first index `j >= i` where `chars[j]` is not an adjacent separator.
    first_non_adjacent: Vec<usize>,
}

fn is_adjacent_separator(c: char) -> bool {
    c == ' ' || c == '\t'
}

impl Document {
    pub fn new(text: &str) -> Document {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        let mut char_to_byte = Vec::with_capacity(n + 1);
        let mut acc = 0usize;
        for c in &chars {
            char_to_byte.push(acc);
            acc += c.len_utf8();
        }
        char_to_byte.push(acc);
        // Like Haskell's `bsDropToTDrop`: bytes strictly after the previous char boundary up to
        // and including this boundary map to this char index (non-boundary bytes round up).
        let mut byte_to_char = vec![0usize; text.len() + 1];
        for ci in 1..=n {
            for slot in byte_to_char
                .iter_mut()
                .take(char_to_byte[ci] + 1)
                .skip(char_to_byte[ci - 1] + 1)
            {
                *slot = ci;
            }
        }
        let mut first_non_adjacent = vec![0usize; n];
        let mut best = n;
        for i in (0..n).rev() {
            if !is_adjacent_separator(chars[i]) {
                best = i;
            }
            first_non_adjacent[i] = best;
        }
        Document { raw: text.to_string(), chars, char_to_byte, byte_to_char, first_non_adjacent }
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn char_at(&self, ix: usize) -> char {
        self.chars[ix]
    }

    pub fn char_at_opt(&self, ix: isize) -> Option<char> {
        if ix >= 0 && (ix as usize) < self.chars.len() {
            Some(self.chars[ix as usize])
        } else {
            None
        }
    }

    /// Text between two code point offsets.
    pub fn slice(&self, start: usize, end: usize) -> &str {
        &self.raw[self.char_to_byte[start]..self.char_to_byte[end]]
    }

    /// UTF-8 substring starting at the code point offset `pos`.
    pub fn substring_from(&self, pos: usize) -> &str {
        &self.raw[self.char_to_byte[pos]..]
    }

    pub fn byte_offset(&self, pos: usize) -> usize {
        self.char_to_byte[pos]
    }

    /// Translate a byte offset in `raw` to a code point offset.
    pub fn char_offset(&self, byte: usize) -> usize {
        self.byte_to_char[byte]
    }

    /// True iff `a` is followed only by spaces/tabs up to `b`.
    pub fn is_adjacent(&self, a: usize, b: usize) -> bool {
        // Haskell: b >= a && firstNonAdjacent ! a >= b. Index `a` may equal len here only when
        // called from a position at the end, where no node can start anyway.
        if a >= self.chars.len() {
            return b == a;
        }
        b >= a && self.first_non_adjacent[a] >= b
    }

    pub fn is_range_valid(&self, lang: Lang, start: usize, end: usize) -> bool {
        match lang {
            Lang::AR => self.ar_is_range_valid(start, end),
            Lang::ZH => self.zh_is_range_valid(start, end),
            _ => self.default_is_range_valid(start, end),
        }
    }

    fn default_is_range_valid(&self, start: usize, end: usize) -> bool {
        let n = self.len();
        // Empty match at EOF (`start == n`) would index `chars[start]` out of bounds.
        // Haskell PCRE `matchAll` does not emit a match starting at `length`; reject it.
        // Empty document (`start == end == 0 == n`) stays valid.
        if start >= n || end > n {
            return start == 0 && end == 0 && n == 0;
        }
        (start == 0 || is_different(self.chars[start - 1], self.chars[start]))
            && (end == n || is_different(self.chars[end - 1], self.chars[end]))
    }

    fn zh_is_range_valid(&self, start: usize, end: usize) -> bool {
        let at = |i: isize| self.char_at_opt(i);
        (start == 0 || zh_is_different(at(start as isize - 1), at(start as isize)))
            && (end == self.len() || zh_is_different(at(end as isize - 1), at(end as isize)))
    }

    fn ar_is_range_valid(&self, start: usize, end: usize) -> bool {
        let len = self.len();
        // Empty-alternative regexes (e.g. AR money `فوق||…`) can yield a 0-width
        // match at EOF (`start == end == len`). Haskell `doc ! start` would throw;
        // reject instead of indexing `chars[len]`.
        if start >= len || end > len {
            return false;
        }
        let d = |i: usize| self.chars[i];
        let diff = |a: usize, b: usize| ar_is_different(d(a), d(b));
        ((start == 0 || diff(start - 1, start)) && (end == len || diff(end - 1, end)))
            // Is Arabic proclitic?
            || (start + 1 == end
                && is_arabic_proclitic(d(start))
                && (start == 0 || diff(start - 1, start)))
            || (start + 2 == end
                && is_arabic_proclitic2(d(start), d(start + 1))
                && (start == 0 || diff(start - 1, start)))
            // Is preceded by proclitic
            || (start != 0
                && is_arabic_proclitic(d(start - 1))
                && (end == len || diff(end - 1, end)))
            // Is Arabic enclitic?
            || (start + 2 == end
                && is_arabic_enclitic(d(start), d(end - 1))
                && (end == len || (end + 1 < len && diff(end, end + 1))))
            // Is followed by enclitic
            || ((start == 0 || diff(start - 1, start))
                && (end + 2 <= len && is_arabic_enclitic(d(end), d(end + 1))))
    }
}

/// Haskell's `Data.Char.isLower` (GHC 9.2): general category `LowercaseLetter`.
pub fn hs_is_lower(c: char) -> bool {
    get_general_category(c) == GeneralCategory::LowercaseLetter
}

/// Haskell's `Data.Char.isUpper` (GHC 9.2): `UppercaseLetter` or `TitlecaseLetter`.
pub fn hs_is_upper(c: char) -> bool {
    matches!(
        get_general_category(c),
        GeneralCategory::UppercaseLetter | GeneralCategory::TitlecaseLetter
    )
}

/// Haskell's `Data.Char.isDigit`: ASCII digits only.
pub fn hs_is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum CharClass {
    Alpha,
    Digit,
    Self_(char),
}

fn char_class(c: char) -> CharClass {
    if hs_is_lower(c) || hs_is_upper(c) {
        CharClass::Alpha
    } else if hs_is_digit(c) {
        CharClass::Digit
    } else {
        CharClass::Self_(c)
    }
}

fn is_different(a: char, b: char) -> bool {
    char_class(a) != char_class(b)
}

const ARABIC: &[char] = &[
    'ا', 'ب', 'ت', 'ة', 'ث', 'ج', 'ح', 'خ', 'د', 'ذ', 'ر', 'ز', 'س', 'ش', 'ص', 'ض', 'ط', 'ظ', 'ع',
    'غ', 'ف', 'ق', 'ك', 'ل', 'م', 'ن', 'ه', 'ي', 'ء', 'آ', 'أ', 'إ', 'ؤ', 'و', 'ئ', 'ى',
];

fn is_arabic_proclitic(c: char) -> bool {
    ['و', 'ف', 'ل', 'ب', 'ك'].contains(&c)
}

fn is_arabic_proclitic2(c1: char, c2: char) -> bool {
    ['ا', 'ل'].contains(&c1) && c2 == 'ل'
}

fn is_arabic_enclitic(c1: char, c2: char) -> bool {
    ['ا', 'ي'].contains(&c1) && c2 == 'ن'
}

fn ar_char_class(c: char) -> CharClass {
    if hs_is_lower(c) || hs_is_upper(c) || ARABIC.contains(&c) {
        CharClass::Alpha
    } else if hs_is_digit(c) {
        CharClass::Digit
    } else {
        CharClass::Self_(c)
    }
}

fn ar_is_different(a: char, b: char) -> bool {
    ar_char_class(a) != ar_char_class(b)
}

fn zh_char_class(c: char) -> Option<CharClass> {
    if hs_is_lower(c) || hs_is_upper(c) {
        Some(CharClass::Alpha)
    } else if hs_is_digit(c) {
        Some(CharClass::Digit)
    } else {
        None
    }
}

fn zh_is_different(a: Option<char>, b: Option<char>) -> bool {
    match (a, b) {
        (None, None) => false,
        (None, _) | (_, None) => true,
        (Some(c1), Some(c2)) => match (zh_char_class(c1), zh_char_class(c2)) {
            (None, None) => true,
            (cc1, cc2) => cc1 != cc2,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_roundtrip() {
        let d = Document::new("żółty x");
        assert_eq!(d.len(), 7);
        assert_eq!(d.byte_offset(1), 2);
        // Matches Haskell's bsDropToTDrop for "żółty": [0,1,1,2,2,3,3,4,5]
        let d2 = Document::new("żółty");
        let got: Vec<usize> = (0..=8).map(|b| d2.char_offset(b)).collect();
        assert_eq!(got, vec![0, 1, 1, 2, 2, 3, 3, 4, 5]);
    }

    #[test]
    fn adjacency() {
        let d = Document::new("a  b");
        assert!(d.is_adjacent(1, 3));
        assert!(!d.is_adjacent(0, 3));
        assert!(d.is_adjacent(3, 3));
    }

    #[test]
    fn range_validity() {
        let d = Document::new("abc 12");
        assert!(!d.is_range_valid(Lang::EN, 0, 2)); // splits "abc"
        assert!(d.is_range_valid(Lang::EN, 0, 3));
        assert!(d.is_range_valid(Lang::EN, 4, 6));
    }
}
