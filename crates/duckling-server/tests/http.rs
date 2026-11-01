use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use duckling_server::{app, load_time_zone_series, AppState};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn state() -> Arc<AppState> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../duckling/tests/tz/tzdata");
    Arc::new(AppState { tzs: load_time_zone_series(&base).unwrap() })
}

async fn call(req: Request<Body>) -> (StatusCode, String) {
    let resp = app(state()).oneshot(req).await.unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

fn post(body: &str) -> Request<Body> {
    Request::post("/parse")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn root_quacks() {
    let (s, b) = call(Request::get("/").body(Body::empty()).unwrap()).await;
    assert_eq!((s, b.as_str()), (StatusCode::OK, "quack!"));
}

#[tokio::test]
async fn targets_lists_languages() {
    let (s, b) = call(Request::get("/targets").body(Body::empty()).unwrap()).await;
    assert_eq!(s, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&b).unwrap();
    assert_eq!(v.as_object().unwrap().len(), 49);
    assert!(v["en"].as_array().unwrap().iter().any(|d| d == "time"));
    assert!(v["en"].as_array().unwrap().iter().all(|d| d != "timezone"));
}

#[tokio::test]
async fn missing_text_is_422() {
    let (s, b) = call(post("lang=en")).await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(b, "Need a 'text' parameter to parse");
}

#[tokio::test]
async fn wrong_method_is_404() {
    let (s, _) = call(Request::get("/parse").body(Body::empty()).unwrap()).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn parses_number() {
    let (s, b) = call(post("text=twenty+one&lang=en&dims=%5B%22number%22%5D")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        b,
        r#"[{"body":"twenty one","dim":"number","end":10,"latent":false,"start":0,"value":{"type":"value","value":21}}]"#
    );
}

#[tokio::test]
async fn time_in_zone_with_reftime() {
    // 2023-11-14T22:13:20Z is 14:13:20 in Los Angeles (PST, UTC-8).
    let (s, b) = call(post("text=tomorrow+at+5pm&lang=en&dims=%5B%22time%22%5D&tz=America%2FLos_Angeles&reftime=1700000000000")).await;
    assert_eq!(s, StatusCode::OK);
    assert!(b.contains(r#""value":"2023-11-15T17:00:00.000-08:00""#), "{b}");
}

#[tokio::test]
async fn unknown_zone_falls_back_to_utc() {
    let (_, b) = call(post("text=at+5pm&dims=%5B%22time%22%5D&tz=Nowhere%2FLand&reftime=1700000000000")).await;
    assert!(b.contains("+00:00"), "{b}");
}

#[tokio::test]
async fn location_tz_opt_in() {
    let off = call(post("text=now+in+delhi&dims=%5B%22time%22%5D&tz=America%2FLos_Angeles")).await.1;
    assert!(!off.contains("Asia/Kolkata"), "{off}");
    let on = call(post(
        "text=now+in+delhi&dims=%5B%22time%22%5D&tz=America%2FLos_Angeles&location_tz=true",
    ))
    .await
    .1;
    assert!(on.contains("Asia/Kolkata"), "{on}");
    assert!(on.contains(r#""dim":"timezone""#), "{on}");
}
