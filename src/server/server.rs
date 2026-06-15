use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    response::Response,
};
use std::path::PathBuf;

fn mime_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "application/octet-stream"
    }
}

#[derive(Clone)]
struct AppState {
    serve_dir: PathBuf,
}

pub async fn serve(port: u16, serve_dir: PathBuf) {
    let app = Router::new()
        .fallback(handler)
        .with_state(AppState { serve_dir });

    let addr = format!("0.0.0.0:{}", port);
    println!("Chess server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind address");

    axum::serve(listener, app).await.expect("server error");
}

async fn handler(State(state): State<AppState>, req: Request) -> Response<Body> {
    let path = req.uri().path();

    let normalized = if path == "/" || path.is_empty() {
        "index.html"
    } else {
        &path[1..]
    };

    let file_path = state.serve_dir.join(normalized);

    match tokio::fs::read(&file_path).await {
        Ok(data) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type(normalized))
            .body(Body::from(data))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from("404 Not Found"))
            .unwrap(),
    }
}
