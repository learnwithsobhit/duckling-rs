//! HTTP front-end mirroring upstream's `exe/ExampleMain.hs`:
//!
//! * `GET /` answers `quack!`
//! * `GET /targets` lists the dimensions supported per language
//! * `POST /parse` takes the form fields `text`, `lang`, `locale`, `dims`, `tz`, `reftime`,
//!   `latent` and answers a JSON array of entities.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use duckling::service::{handle_parse, handle_targets, HttpError, ParseParams};
use duckling::time_zone::TimeZoneSeries;

/// Shared, immutable server state: the time zone database.
pub struct AppState {
    pub tzs: HashMap<String, TimeZoneSeries>,
}

/// `loadTimeZoneSeries`: every Olson file below `base` (except `posix/` and `right/`), keyed by
/// its path relative to `base`; unparsable files are ignored.
pub fn load_time_zone_series(base: &Path) -> std::io::Result<HashMap<String, TimeZoneSeries>> {
    fn walk(dir: &Path, base: &Path, out: &mut HashMap<String, TimeZoneSeries>) -> std::io::Result<()> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let path = e.path();
            if path.is_dir() {
                if dir == base && matches!(e.file_name().to_str(), Some("posix") | Some("right")) {
                    continue;
                }
                walk(&path, base, out)?;
            } else if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(series) = duckling::tzif::parse_series(&bytes) {
                    let rel = path.strip_prefix(base).unwrap_or(&path).to_string_lossy().into_owned();
                    out.insert(rel, series);
                }
            }
        }
        Ok(())
    }
    let mut out = HashMap::new();
    walk(base, base, &mut out)?;
    Ok(out)
}

/// `loadTZs`: `/usr/share/zoneinfo/` with `/etc/zoneinfo/` as fallback. The directory can be
/// overridden with `DUCKLING_ZONEINFO` (useful to compare against a server running elsewhere).
pub fn load_default_time_zones() -> HashMap<String, TimeZoneSeries> {
    if let Some(dir) = std::env::var_os("DUCKLING_ZONEINFO") {
        return load_time_zone_series(Path::new(&dir)).unwrap_or_default();
    }
    load_time_zone_series(Path::new("/usr/share/zoneinfo/"))
        .or_else(|_| load_time_zone_series(Path::new("/etc/zoneinfo/")))
        .unwrap_or_default()
}

fn now_unix_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn json_response(body: String) -> Response {
    ([(header::CONTENT_TYPE, "application/json")], body).into_response()
}

async fn quack() -> Response {
    // Snap's `writeBS` sets no content type.
    Response::new(axum::body::Body::from("quack!"))
}

async fn targets() -> Response {
    json_response(handle_targets())
}

/// First value of a form field (Snap's `getPostParam`).
fn param(body: &[u8], key: &str) -> Option<String> {
    form_urlencoded::parse(body).find(|(k, _)| k == key).map(|(_, v)| v.into_owned())
}

async fn parse(State(state): State<Arc<AppState>>, body: Bytes) -> Response {
    let params = ParseParams {
        text: param(&body, "text"),
        lang: param(&body, "lang"),
        locale: param(&body, "locale"),
        dims: param(&body, "dims"),
        tz: param(&body, "tz"),
        reftime: param(&body, "reftime"),
        latent: param(&body, "latent"),
    };
    // Parsing is CPU-bound; keep it off the async workers' hot path.
    let result = tokio::task::spawn_blocking(move || handle_parse(&params, &state.tzs, now_unix_ms()))
        .await
        .unwrap_or_else(|_| Err(HttpError { status: 500, body: "internal error".into() }));
    match result {
        Ok(body) => json_response(body),
        Err(HttpError { status, body }) => {
            let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
        }
    }
}

async fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "Not found").into_response()
}

/// The application router.
pub fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(quack))
        .route("/targets", get(targets))
        .route("/parse", post(parse))
        .fallback(not_found)
        .method_not_allowed_fallback(not_found)
        .with_state(state)
}
