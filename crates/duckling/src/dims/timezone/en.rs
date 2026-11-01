//! English location-timezone rules. Loaded only when `Dim::Timezone` is requested.

use once_cell::sync::Lazy;
use std::sync::Arc;

use super::{City, TimezoneData, CITIES};
use crate::dims::time::helpers::{cycle_nth, has_no_timezone, is_not_latent, now, today};
use crate::dims::time::types::ResolveZone;
use crate::dims::time_grain::Grain;
use crate::dims::util::to_lower;
use crate::types::{pred, regex, rule, Rule, Token};

fn re_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if ".+*?^$()[]{}|\\".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn alt(parts: &[&str]) -> String {
    let mut keys: Vec<String> = parts.iter().map(|p| re_escape(p)).collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.chars().count()));
    keys.join("|")
}

fn loc_token(city: &'static City, composed: bool) -> Token {
    Token::Timezone(TimezoneData { city, composed })
}

fn is_loc(t: &Token) -> bool {
    matches!(t, Token::Timezone(d) if !d.composed)
}

fn loc_of(t: &Token) -> Option<&TimezoneData> {
    match t {
        Token::Timezone(d) => Some(d),
        _ => None,
    }
}

fn usable_time(t: &Token) -> bool {
    is_not_latent(t) && has_no_timezone(t)
}

fn attach_zone(t: &Token, loc: &TimezoneData) -> Option<Token> {
    let Token::Time(td) = t else { return None };
    if td.has_timezone || td.resolve_zone.is_some() {
        return None;
    }
    let mut td = td.clone();
    td.has_timezone = true;
    td.resolve_zone = Some(Arc::new(ResolveZone {
        iana: loc.city.iana,
        city: loc.city.label,
        fallback_offset_min: loc.city.fallback_offset_min,
        rank: loc.city.rank,
    }));
    Some(Token::Time(td))
}

fn composed_tz(loc: &TimezoneData) -> Token {
    loc_token(loc.city, true)
}

fn time_word(s: &str) -> Option<crate::dims::time::types::TimeData> {
    match s {
        "time" => Some(now()),
        "date" => Some(today()),
        _ => None,
    }
}

fn day_word(s: &str) -> Option<crate::dims::time::types::TimeData> {
    match s {
        "today" | "tonight" => Some(today()),
        "tomorrow" => Some(cycle_nth(Grain::Day, 1)),
        _ => None,
    }
}

fn location_rules() -> Vec<Rule> {
    let mut v = Vec::new();
    for city in CITIES {
        let aliases = alt(city.aliases);
        v.push(rule(
            &format!("location:{}", city.label),
            vec![regex(&format!(r"\b(?:{aliases})\b(?!\s*,)"))],
            {
                let city: &'static City = city;
                move |_| Some(loc_token(city, false))
            },
        ));
        if !city.qualifiers.is_empty() {
            let quals = alt(city.qualifiers);
            v.push(rule(
                &format!("location:{} qualified", city.label),
                vec![regex(&format!(r"\b(?:{aliases}),?\s+(?:{quals})\b"))],
                {
                    let city: &'static City = city;
                    move |_| Some(loc_token(city, false))
                },
            ));
        }
    }
    v
}

fn composition_rules() -> Vec<Rule> {
    vec![
        rule(
            "<time> in <location>",
            vec![pred(usable_time), regex(r"\bin\b"), pred(is_loc)],
            |ts| attach_zone(ts[0], loc_of(ts[2])?),
        ),
        rule(
            "in <location> <time>",
            vec![regex(r"\bin\b"), pred(is_loc), pred(usable_time)],
            |ts| attach_zone(ts[2], loc_of(ts[1])?),
        ),
        rule(
            "time|date in <location>",
            vec![regex(r"\b(time|date)\b"), regex(r"\bin\b"), pred(is_loc)],
            |ts| {
                let w = match ts[0] {
                    Token::RegexMatch(g) => to_lower(g.0.first()?),
                    _ => return None,
                };
                let td = time_word(&w)?;
                attach_zone(&Token::Time(td), loc_of(ts[2])?)
            },
        ),
        rule(
            "date in <location> today|tomorrow",
            vec![
                regex(r"\b((?:the\s+)?date|time)\b"),
                regex(r"\bin\b"),
                pred(is_loc),
                regex(r"\b(today|tonight|tomorrow)\b"),
            ],
            |ts| {
                let w = match ts[3] {
                    Token::RegexMatch(g) => to_lower(g.0.first()?),
                    _ => return None,
                };
                let td = day_word(&w)?;
                attach_zone(&Token::Time(td), loc_of(ts[2])?)
            },
        ),
        rule(
            "<time> in <location> (timezone)",
            vec![pred(usable_time), regex(r"\bin\b"), pred(is_loc)],
            |ts| Some(composed_tz(loc_of(ts[2])?)),
        ),
        rule(
            "in <location> <time> (timezone)",
            vec![regex(r"\bin\b"), pred(is_loc), pred(usable_time)],
            |ts| Some(composed_tz(loc_of(ts[1])?)),
        ),
        rule(
            "time|date in <location> (timezone)",
            vec![regex(r"\b(time|date)\b"), regex(r"\bin\b"), pred(is_loc)],
            |ts| Some(composed_tz(loc_of(ts[2])?)),
        ),
        rule(
            "date in <location> today|tomorrow (timezone)",
            vec![
                regex(r"\b((?:the\s+)?date|time)\b"),
                regex(r"\bin\b"),
                pred(is_loc),
                regex(r"\b(today|tonight|tomorrow)\b"),
            ],
            |ts| Some(composed_tz(loc_of(ts[2])?)),
        ),
    ]
}

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    let mut v = location_rules();
    v.extend(composition_rules());
    v
});

pub fn rules() -> &'static [Rule] {
    &RULES
}
