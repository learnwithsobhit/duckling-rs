//! `Duckling.Numeral.DE.NumParser`: a (deliberately non-backtracking) parser for German number words.
//!
//! Semantics mirror the Haskell `Parser a = String -> Maybe (a, String)` with `<|>` committing to the
//! first alternative that succeeds.

use once_cell::sync::Lazy;
use std::sync::Arc;

type P = Arc<dyn for<'a> Fn(&'a [char]) -> Option<(i64, &'a [char])> + Send + Sync>;

fn pure(v: i64) -> P {
    Arc::new(move |s| Some((v, s)))
}

fn empty() -> P {
    Arc::new(|_| None)
}

/// `assign value form`: matches `form` literally and yields `value`.
fn assign(value: i64, form: &str) -> P {
    let cs: Vec<char> = form.chars().collect();
    Arc::new(move |s| {
        if s.len() >= cs.len() && s[..cs.len()] == cs[..] {
            Some((value, &s[cs.len()..]))
        } else {
            None
        }
    })
}

/// `.+.`
fn add(a: P, b: P) -> P {
    Arc::new(move |s| {
        let (x, s1) = a(s)?;
        let (y, s2) = b(s1)?;
        Some((x + y, s2))
    })
}

/// `.*.`
fn mul(a: P, b: P) -> P {
    Arc::new(move |s| {
        let (x, s1) = a(s)?;
        let (y, s2) = b(s1)?;
        Some((x * y, s2))
    })
}

/// `<|>`
fn alt(a: P, b: P) -> P {
    Arc::new(move |s| a(s).or_else(|| b(s)))
}

fn opt(p: P) -> P {
    alt(p, pure(0))
}

fn asum(ps: Vec<P>) -> P {
    ps.into_iter().rev().fold(empty(), |acc, p| alt(p, acc))
}

struct NumItem {
    base: P,
    plus10: P,
    times10: Vec<P>,
}

fn ten() -> P {
    assign(10, "zehn")
}

fn ty() -> P {
    assign(10, "zig")
}

fn hundred() -> P {
    assign(100, "hundert")
}

fn thousand() -> P {
    assign(1000, "tausend")
}

fn und() -> P {
    assign(0, "und")
}

fn default_num_item(value: i64, form: &str) -> NumItem {
    NumItem {
        base: assign(value, form),
        plus10: add(assign(value, form), ten()),
        times10: vec![mul(assign(value, form), ty())],
    }
}

fn digit_lexicon() -> Vec<NumItem> {
    let one = NumItem { plus10: assign(11, "elf"), times10: vec![ten()], ..default_num_item(1, "ein") };
    let two = NumItem {
        plus10: assign(12, "zwölf"),
        times10: vec![assign(20, "zwanzig")],
        ..default_num_item(2, "zwei")
    };
    let three = NumItem {
        times10: vec![assign(30, "dreißig"), assign(30, "dreissig")],
        ..default_num_item(3, "drei")
    };
    let four = default_num_item(4, "vier");
    let five = default_num_item(5, "fünf");
    let six = NumItem {
        plus10: assign(16, "sechzehn"),
        times10: vec![assign(60, "sechzig")],
        ..default_num_item(6, "sechs")
    };
    let seven = NumItem {
        plus10: assign(17, "siebzehn"),
        times10: vec![assign(70, "siebzig")],
        ..default_num_item(7, "sieben")
    };
    let eight = default_num_item(8, "acht");
    let nine = default_num_item(9, "neun");
    vec![one, two, three, four, five, six, seven, eight, nine]
}

struct Parsers {
    from1to999999: P,
    from_year: P,
}

static PARSERS: Lazy<Parsers> = Lazy::new(|| {
    let lex = digit_lexicon();
    let from1to9 = asum(lex.iter().map(|i| i.base.clone()).collect());
    let tens_from20 = asum(lex.iter().skip(1).flat_map(|i| i.times10.iter().cloned()).collect());
    let plus10s = asum(lex.iter().map(|i| i.plus10.clone()).collect());
    let from1to99 = alt(
        alt(
            alt(add(opt(add(from1to9.clone(), und())), tens_from20), plus10s),
            ten(),
        ),
        from1to9.clone(),
    );
    let from1to999 = add(
        opt(add(mul(from1to9.clone(), hundred()), opt(und()))),
        opt(from1to99.clone()),
    );
    let from1to999999 = add(
        opt(add(mul(from1to999.clone(), thousand()), opt(und()))),
        opt(from1to999),
    );
    let from_year = add(
        asum(lex.iter().map(|i| mul(i.plus10.clone(), hundred())).collect()),
        opt(add(opt(und()), from1to99)),
    );
    Parsers { from1to999999, from_year }
});

fn starts_with(s: &[char], p: &str) -> bool {
    let cs: Vec<char> = p.chars().collect();
    s.len() >= cs.len() && s[..cs.len()] == cs[..]
}

fn remove_inflection(n: i64, suffix: &[char]) -> Option<i64> {
    let suffix: String = suffix.iter().collect();
    if n.rem_euclid(10) == 1 && ["s", "e", "em", "en", "er", "es"].contains(&suffix.as_str()) {
        return Some(n);
    }
    if suffix.is_empty() {
        return Some(n);
    }
    None
}

/// `parseNumeral`.
pub fn parse_numeral(s: &str) -> Option<i64> {
    let cs: Vec<char> = s.chars().collect();
    let p = &*PARSERS;
    // allNumerals = fromYear1100to1999 <|> from1to999999'
    let (n, rest): (i64, Vec<char>) = match (p.from_year)(&cs) {
        Some((n, rest)) => (n, rest.to_vec()),
        None => {
            if starts_with(&cs, "hundert") || starts_with(&cs, "tausend") {
                let mut v: Vec<char> = "ein".chars().collect();
                v.extend_from_slice(&cs);
                let (n, rest) = (p.from1to999999)(&v)?;
                (n, rest.to_vec())
            } else {
                let (n, rest) = (p.from1to999999)(&cs)?;
                (n, rest.to_vec())
            }
        }
    };
    remove_inflection(n, &rest)
}
