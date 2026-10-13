use axum::{Router, routing::{get, put, delete, patch}, response::Html, http::StatusCode};
use std::sync::{Arc, Mutex, MutexGuard};

mod kv;
mod http_kv;
use kv::KeyValue;
use http_kv::{http_get, http_set, http_delete, http_clear, http_get_table_version, http_set_empty_key, http_unsupported};
use kv::now_secs;

pub type SharedStore = Arc<Mutex<KeyValue>>;

// Poison policy: recover and continue (log once). Safe today because every
// mutation computes first, then does insert + table_version bump, with nothing
// that can panic in between. Once replicas/recovery exist, revisit: fail-stop
// and recover from a replica is the more honest policy for a store whose point
// is never acting on stale state.
pub fn lock_store(store: &SharedStore) -> MutexGuard<'_, KeyValue> {
    store.lock().unwrap_or_else(|poisoned| {
        eprintln!("WARNING: store mutex poisoned by an earlier panic; recovering");
        store.clear_poison(); // log once, not on every request (Rust 1.77+)
        poisoned.into_inner()
    })
}

async fn clean_expired(store: SharedStore) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        interval.tick().await;
        let current_time_secs = now_secs();
        {
            let mut store = lock_store(&store);
            store.clean(current_time_secs);
        }
    }
}

#[tokio::main]
async fn main() {
    let store: SharedStore = Arc::new(Mutex::new(KeyValue::new()));
    let store_clone = Arc::clone(&store);
    tokio::spawn(clean_expired(store_clone));

    let app = Router::new()
        .route("/", get(root))
        .route("/kv/:key", get(http_get))
        .route("/kv/:key", put(http_set).post(http_set))
        .route("/kv/:key", delete(http_delete))
        .route("/kv/:key", patch(http_unsupported))
        .route("/kv/", get(http_set_empty_key).put(http_set_empty_key).delete(http_set_empty_key).post(http_unsupported).patch(http_unsupported))
        .route("/clear", delete(http_clear))
        .route("/clear", get(http_unsupported).put(http_unsupported).post(http_unsupported).patch(http_unsupported))
        .route("/table", delete(http_clear))
        .route("/table", get(http_get_table_version))
        .route("/table", put(http_unsupported).post(http_unsupported).patch(http_unsupported))
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
