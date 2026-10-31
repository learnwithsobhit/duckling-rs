//! Core types (port of `Duckling.Types`, `Duckling.Resolve` and `Duckling.Regex.Types`).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use crate::dims::amount_of_money::AmountOfMoneyData;
use crate::dims::credit_card::CreditCardNumberData;
use crate::dims::distance::DistanceData;
use crate::dims::duration::DurationData;
use crate::dims::email::EmailData;
use crate::dims::numeral::NumeralData;
use crate::dims::ordinal::OrdinalData;
use crate::dims::phone_number::PhoneNumberData;
use crate::dims::quantity::QuantityData;
use crate::dims::temperature::TemperatureData;
use crate::dims::time::types::TimeData;
use crate::dims::time_grain::Grain;
use crate::dims::url::UrlData;
use crate::dims::volume::VolumeData;
use crate::json::Json;
use crate::locale::Locale;
use crate::regex_tr::Re;
use crate::time_zone::DucklingTime;

/// Half-open range of code point offsets. Ordered like Haskell's derived `Ord`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

impl Range {
    pub fn new(start: usize, end: usize) -> Range {
        Range { start, end }
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    pub reference_time: DucklingTime,
    pub locale: Locale,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// Include less certain parses, e.g. "7" as an hour of the day.
    pub with_latent: bool,
}

/// The dimensions Duckling can extract (`Duckling.Types.Dimension`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Dim {
    RegexMatch,
    AmountOfMoney,
    CreditCardNumber,
    Distance,
    Duration,
    Email,
    Numeral,
    Ordinal,
    PhoneNumber,
    Quantity,
    Temperature,
    Time,
    TimeGrain,
    Url,
    Volume,
}

impl Dim {
    pub const ALL_PUBLIC: &'static [Dim] = &[
        Dim::AmountOfMoney,
        Dim::CreditCardNumber,
        Dim::Distance,
        Dim::Duration,
        Dim::Email,
        Dim::Numeral,
        Dim::Ordinal,
        Dim::PhoneNumber,
        Dim::Quantity,
        Dim::Temperature,
        Dim::Time,
        Dim::TimeGrain,
        Dim::Url,
        Dim::Volume,
    ];

    /// `Duckling.Dimensions.Types.toName`.
    pub fn name(self) -> &'static str {
        match self {
            Dim::RegexMatch => "regex",
            Dim::CreditCardNumber => "credit-card-number",
            Dim::Distance => "distance",
            Dim::Duration => "duration",
            Dim::Email => "email",
            Dim::AmountOfMoney => "amount-of-money",
            Dim::Numeral => "number",
            Dim::Ordinal => "ordinal",
            Dim::PhoneNumber => "phone-number",
            Dim::Quantity => "quantity",
            Dim::Temperature => "temperature",
            Dim::Time => "time",
            Dim::TimeGrain => "time-grain",
            Dim::Url => "url",
            Dim::Volume => "volume",
        }
    }

    /// `Duckling.Dimensions.Types.fromName` (RegexMatch is not exposed).
    pub fn from_name(name: &str) -> Option<Dim> {
        Dim::ALL_PUBLIC.iter().copied().find(|d| d.name() == name)
    }

    /// `Duckling.Dimensions.dependents`.
    pub fn dependents(self) -> &'static [Dim] {
        match self {
            Dim::CreditCardNumber | Dim::Numeral | Dim::Email | Dim::PhoneNumber => &[],
            Dim::RegexMatch | Dim::TimeGrain | Dim::Url => &[],
            Dim::Distance
            | Dim::AmountOfMoney
            | Dim::Ordinal
            | Dim::Quantity
            | Dim::Temperature
            | Dim::Volume => &[Dim::Numeral],
            Dim::Duration => &[Dim::Numeral, Dim::TimeGrain],
            Dim::Time => &[Dim::Numeral, Dim::Duration, Dim::Ordinal, Dim::TimeGrain],
        }
    }
}

/// Captured groups of a regex match (`GroupMatch`). Unset groups are empty strings.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GroupMatch(pub Vec<Rc<str>>);

/// A typed intermediate value attached to a node.
#[derive(Clone, Debug)]
pub enum Token {
    RegexMatch(GroupMatch),
    AmountOfMoney(AmountOfMoneyData),
    CreditCardNumber(CreditCardNumberData),
    Distance(DistanceData),
    Duration(DurationData),
    Email(EmailData),
    Numeral(NumeralData),
    Ordinal(OrdinalData),
    PhoneNumber(PhoneNumberData),
    Quantity(QuantityData),
    Temperature(TemperatureData),
    Time(TimeData),
    TimeGrain(Grain),
    Url(UrlData),
    Volume(VolumeData),
}

impl Token {
    pub fn dim(&self) -> Dim {
        match self {
            Token::RegexMatch(_) => Dim::RegexMatch,
            Token::AmountOfMoney(_) => Dim::AmountOfMoney,
            Token::CreditCardNumber(_) => Dim::CreditCardNumber,
            Token::Distance(_) => Dim::Distance,
            Token::Duration(_) => Dim::Duration,
            Token::Email(_) => Dim::Email,
            Token::Numeral(_) => Dim::Numeral,
            Token::Ordinal(_) => Dim::Ordinal,
            Token::PhoneNumber(_) => Dim::PhoneNumber,
            Token::Quantity(_) => Dim::Quantity,
            Token::Temperature(_) => Dim::Temperature,
            Token::Time(_) => Dim::Time,
            Token::TimeGrain(_) => Dim::TimeGrain,
            Token::Url(_) => Dim::Url,
            Token::Volume(_) => Dim::Volume,
        }
    }

    /// `Resolve.resolve`: the JSON value plus a latent flag, or `None` when unresolvable.
    pub fn resolve(&self, ctx: &Context, opts: &Options) -> Option<(Json, bool)> {
        match self {
            Token::RegexMatch(_) => None,
            Token::AmountOfMoney(d) => d.resolve(ctx, opts),
            Token::CreditCardNumber(d) => Some((d.resolve(), false)),
            Token::Distance(d) => d.resolve(ctx, opts),
            Token::Duration(d) => Some((d.resolve(), false)),
            Token::Email(d) => Some((d.resolve(), false)),
            Token::Numeral(d) => Some((d.resolve(), false)),
            Token::Ordinal(d) => Some((d.resolve(), false)),
            Token::PhoneNumber(d) => Some((d.resolve(), false)),
            Token::Quantity(d) => d.resolve(ctx, opts),
            Token::Temperature(d) => d.resolve(ctx, opts),
            Token::Time(d) => d.resolve(ctx, opts),
            Token::TimeGrain(_) => None,
            Token::Url(d) => Some((d.resolve(), false)),
            Token::Volume(d) => d.resolve(ctx, opts),
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        match (self, other) {
            (Token::RegexMatch(a), Token::RegexMatch(b)) => a == b,
            (Token::AmountOfMoney(a), Token::AmountOfMoney(b)) => a == b,
            (Token::CreditCardNumber(a), Token::CreditCardNumber(b)) => a == b,
            (Token::Distance(a), Token::Distance(b)) => a == b,
            (Token::Duration(a), Token::Duration(b)) => a == b,
            (Token::Email(a), Token::Email(b)) => a == b,
            (Token::Numeral(a), Token::Numeral(b)) => a == b,
            (Token::Ordinal(a), Token::Ordinal(b)) => a == b,
            (Token::PhoneNumber(a), Token::PhoneNumber(b)) => a == b,
            (Token::Quantity(a), Token::Quantity(b)) => a == b,
            (Token::Temperature(a), Token::Temperature(b)) => a == b,
            (Token::Time(a), Token::Time(b)) => a == b,
            (Token::TimeGrain(a), Token::TimeGrain(b)) => a == b,
            (Token::Url(a), Token::Url(b)) => a == b,
            (Token::Volume(a), Token::Volume(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for Token {}

impl Hash for Token {
    fn hash<H: Hasher>(&self, h: &mut H) {
        std::mem::discriminant(self).hash(h);
        match self {
            Token::RegexMatch(a) => a.hash(h),
            Token::AmountOfMoney(a) => a.hash(h),
            Token::CreditCardNumber(a) => a.hash(h),
            Token::Distance(a) => a.hash(h),
            Token::Duration(a) => a.hash(h),
            Token::Email(a) => a.hash(h),
            Token::Numeral(a) => a.hash(h),
            Token::Ordinal(a) => a.hash(h),
            Token::PhoneNumber(a) => a.hash(h),
            Token::Quantity(a) => a.hash(h),
            Token::Temperature(a) => a.hash(h),
            Token::Time(a) => a.hash(h),
            Token::TimeGrain(a) => a.hash(h),
            Token::Url(a) => a.hash(h),
            Token::Volume(a) => a.hash(h),
        }
    }
}

/// Hashes an `f64` consistently with `==` (so `0.0` and `-0.0` collide).
pub fn hash_f64<H: Hasher>(x: f64, h: &mut H) {
    let x = if x == 0.0 { 0.0 } else { x };
    x.to_bits().hash(h);
}

/// A node of the parse forest.
#[derive(Debug)]
pub struct Node {
    pub range: Range,
    pub token: Token,
    pub children: Vec<Rc<Node>>,
    pub rule: Option<Arc<str>>,
    hash: u64,
}

impl Node {
    pub fn new(range: Range, token: Token, children: Vec<Rc<Node>>, rule: Option<Arc<str>>) -> Node {
        let mut h = DefaultHasher::new();
        range.hash(&mut h);
        token.hash(&mut h);
        rule.hash(&mut h);
        for c in &children {
            c.hash.hash(&mut h);
        }
        Node { range, token, children, rule, hash: h.finish() }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.hash == other.hash
            && self.range == other.range
            && self.rule == other.rule
            && self.token == other.token
            && self.children.len() == other.children.len()
            && self.children.iter().zip(&other.children).all(|(a, b)| Rc::ptr_eq(a, b) || a == b)
    }
}
impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.hash.hash(h)
    }
}

/// A resolved token together with its JSON value (`ResolvedToken`).
#[derive(Debug, Clone)]
pub struct Resolved {
    pub range: Range,
    pub node: Rc<Node>,
    pub dim: Dim,
    pub value: Json,
    /// Cached `toJText value`, used for ordering.
    pub value_text: String,
    pub is_latent: bool,
}

impl PartialEq for Resolved {
    fn eq(&self, other: &Resolved) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}
impl Eq for Resolved {}

impl PartialOrd for Resolved {
    fn partial_cmp(&self, other: &Resolved) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Haskell's `Ord ResolvedToken`: by range, then the encoded JSON text, then latency.
impl Ord for Resolved {
    fn cmp(&self, other: &Resolved) -> std::cmp::Ordering {
        self.range
            .cmp(&other.range)
            .then_with(|| self.value_text.cmp(&other.value_text))
            .then_with(|| self.is_latent.cmp(&other.is_latent))
    }
}

pub type Production = Box<dyn Fn(&[&Token]) -> Option<Token> + Send + Sync>;
pub type Predicate = Box<dyn Fn(&Token) -> bool + Send + Sync>;

pub enum PatternItem {
    Regex(Re),
    Predicate(Predicate),
}

pub struct Rule {
    pub name: Arc<str>,
    pub pattern: Vec<PatternItem>,
    pub prod: Production,
}

impl std::fmt::Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rule({:?})", self.name)
    }
}

/// `Duckling.Types.regex`.
pub fn regex(pattern: &str) -> PatternItem {
    PatternItem::Regex(Re::new(pattern))
}

/// `Duckling.Types.dimension`.
pub fn dimension(d: Dim) -> PatternItem {
    PatternItem::Predicate(Box::new(move |t| t.dim() == d))
}

/// Builds a predicate pattern item from a closure.
pub fn pred(f: impl Fn(&Token) -> bool + Send + Sync + 'static) -> PatternItem {
    PatternItem::Predicate(Box::new(f))
}

/// Builds a rule.
pub fn rule(
    name: &str,
    pattern: Vec<PatternItem>,
    prod: impl Fn(&[&Token]) -> Option<Token> + Send + Sync + 'static,
) -> Rule {
    Rule { name: Arc::from(name), pattern, prod: Box::new(prod) }
}

/// `singleStringLookupRule`: a rule matching any key of `map` (longest first), mapped through `prod`.
pub fn single_string_lookup_rule<T: Clone + Send + Sync + 'static>(
    map: &[(&str, T)],
    name: &str,
    prod: impl Fn(&T) -> Option<Token> + Send + Sync + 'static,
) -> Rule {
    let mut keys: Vec<&str> = map.iter().map(|(k, _)| *k).collect();
    // sortOn (negate . length) is stable: ties keep the (HashMap) key order, which is
    // unspecified; the regex alternation order only matters for equal-length keys, which
    // cannot both match the same text.
    keys.sort_by_key(|k| std::cmp::Reverse(k.chars().count()));
    let re = format!("({})", keys.join("|"));
    let table: std::collections::HashMap<String, T> =
        map.iter().map(|(k, v)| (k.to_lowercase(), v.clone())).collect();
    rule(name, vec![regex(&re)], move |ts| match ts.first() {
        Some(Token::RegexMatch(GroupMatch(g))) => {
            let m = g.first()?;
            table.get(&m.to_lowercase()).and_then(&prod)
        }
        _ => None,
    })
}

/// Convenience accessor used by productions: the first regex group at index `i`.
pub fn group(ts: &[&Token], i: usize) -> Option<Rc<str>> {
    match ts.first() {
        Some(Token::RegexMatch(GroupMatch(g))) => g.get(i).cloned(),
        _ => None,
    }
}

/// All groups of the first (regex) token.
pub fn groups<'a>(ts: &'a [&Token]) -> Option<&'a [Rc<str>]> {
    match ts.first() {
        Some(Token::RegexMatch(GroupMatch(g))) => Some(g.as_slice()),
        _ => None,
    }
}
