use std::sync::Arc;

use duckling_server::{app, load_default_time_zones, AppState};

fn port_from_args() -> Option<u16> {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--port" => return args.get(i + 1).and_then(|p| p.parse().ok()),
            a if a.starts_with("--port=") => return a["--port=".len()..].parse().ok(),
            _ => i += 1,
        }
    }
    None
}

#[tokio::main]
async fn main() {
    let tzs = tokio::task::spawn_blocking(load_default_time_zones).await.expect("loading time zones");
    let port = port_from_args()
        .or_else(|| std::env::var("PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(8000);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.expect("bind");
    eprintln!("duckling-server listening on port {port} ({} time zones)", tzs.len());
    axum::serve(listener, app(Arc::new(AppState { tzs }))).await.expect("serve");
}
