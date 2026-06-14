use axum::{body::Body, extract::Request, http::{header, StatusCode}, response::Response, Router};
use clap::Parser;
use std::collections::HashMap;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));

static ASSETS: LazyLock<HashMap<&'static str, &'static [u8]>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for a in ALL_ASSETS {
        m.insert(a.path, a.data);
    }
    m
});

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

#[derive(Parser)]
#[command(name = "chess-server", about = "Serves the Rust Chess WASM app")]
struct Args {
    #[arg(short, long, default_value = "8080", env = "PORT")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let app = Router::new().fallback(handler);

    let addr = format!("0.0.0.0:{}", args.port);
    println!("Chess server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("server error");
}

async fn handler(req: Request) -> Response<Body> {
    let path = req.uri().path();

    let normalized = if path == "/" || path.is_empty() {
        "/index.html"
    } else {
        path
    };

    if let Some(data) = ASSETS.get(normalized) {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type(normalized))
            .body(Body::from(*data))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from("404 Not Found"))
            .unwrap()
    }
}
