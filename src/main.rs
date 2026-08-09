use axum::{Router, routing::{get, put, delete, post}, response::Html, http::StatusCode};
use std::sync::{Arc, Mutex};

mod kv;
mod http_kv;
use kv::KeyValue;
use http_kv::{http_get, http_set, http_delete, http_clear, http_set_empty_key, http_unsupported};

pub type SharedStore = Arc<Mutex<KeyValue>>;

#[tokio::main]
async fn main() {
    let store: SharedStore = Arc::new(Mutex::new(KeyValue::new()));
    let app = Router::new()
        .route("/", get(root))
        .route("/kv/:key", get(http_get))
        .route("/kv/:key", put(http_set))
        .route("/kv/:key", delete(http_delete))
        .route("/kv/:key", post(http_unsupported).patch(http_unsupported))
        .route("/kv/", get(http_set_empty_key).put(http_set_empty_key).delete(http_set_empty_key).post(http_unsupported).patch(http_unsupported))
        .route("/clear", delete(http_clear))
        .route("/clear", get(http_unsupported).put(http_unsupported).post(http_unsupported).patch(http_unsupported))
        .route("/health", get(healthcheck))
        .route("/health", put(http_unsupported).post(http_unsupported).patch(http_unsupported).delete(http_unsupported))
        .with_state(store);

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn healthcheck() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Healthy")
}
