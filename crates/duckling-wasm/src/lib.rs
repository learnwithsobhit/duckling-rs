//! WebAssembly bindings. The surface mirrors the HTTP server: `parse` takes the same fields as
//! `POST /parse` (as a JSON object) and returns the identical JSON text, `targets` mirrors
//! `GET /targets`.
//!
//! ```js
//! const d = new Duckling();
//! d.addTimeZone("America/Los_Angeles", tzifBytes); // optional; unknown zones behave like UTC
//! d.parse(JSON.stringify({ text: "tomorrow at 5pm", dims: ["time"], tz: "America/Los_Angeles" }));
//! ```

use std::collections::HashMap;

use duckling::service::{handle_parse, handle_targets, ParseParams};
use duckling::time_zone::TimeZoneSeries;
use wasm_bindgen::prelude::*;

/// A parser instance holding the registered time zones.
#[wasm_bindgen]
pub struct Duckling {
    tzs: HashMap<String, TimeZoneSeries>,
}

#[wasm_bindgen]
impl Duckling {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Duckling {
        Duckling { tzs: HashMap::new() }
    }

    /// Registers a time zone from the bytes of a compiled Olson (`TZif`) file.
    #[wasm_bindgen(js_name = addTimeZone)]
    pub fn add_time_zone(&mut self, name: &str, tzif: &[u8]) -> Result<(), JsError> {
        let series = duckling::tzif::parse_series(tzif).map_err(|e| JsError::new(&e))?;
        self.tzs.insert(name.to_string(), series);
        Ok(())
    }

    /// Parses text. `request` is a JSON object with the fields of `POST /parse`
    /// (`text`, `lang`, `locale`, `dims`, `tz`, `reftime`, `latent`, `location_tz`); `reftime` defaults to now.
    /// Returns the JSON array of entities as text.
    pub fn parse(&self, request: &str) -> Result<String, JsError> {
        let v: serde_json::Value = serde_json::from_str(request).map_err(|e| JsError::new(&e.to_string()))?;
        let params = ParseParams::from_json(&v).map_err(|e| JsError::new(&e))?;
        handle_parse(&params, &self.tzs, js_sys::Date::now() as i64).map_err(|e| JsError::new(&e.body))
    }

    /// `GET /targets`: which dimensions each language supports, as JSON text.
    pub fn targets(&self) -> String {
        handle_targets()
    }
}

impl Default for Duckling {
    fn default() -> Self {
        Self::new()
    }
}
