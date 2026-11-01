//! A Rust port of Meta's [Duckling](https://github.com/facebook/duckling): a parser that turns
//! text such as "tomorrow at 5pm" or "twenty-one" into structured entities.

pub mod api;
pub mod classifier_data;
pub mod dims;
pub mod document;
pub mod engine;
pub mod json;
pub mod lang_dims;
pub mod langs;
pub mod locale;
pub mod ported;
pub mod ranking;
pub mod regex_prefilter;
pub mod regex_tr;
pub mod rules;
pub mod service;
pub mod time_zone;
pub mod tzif;
pub mod types;

pub use api::{all_dimensions, analyze, format_token, parse, Entity};
pub use locale::{Lang, Locale, Region};
pub use time_zone::{DucklingTime, TimeZone, TimeZoneSeries, UtcTime};
pub use types::{Context, Dim, Options};
