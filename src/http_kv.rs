use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::SharedStore;

fn key_is_valid(key: &String) -> bool {
    let expected_regex = regex::Regex::new(r"^([a-zA-Z0-9]|_|\-|\.|:)+$").unwrap();
    expected_regex.is_match(key)
}

pub async fn http_set_empty_key() -> impl IntoResponse {
    (StatusCode::BAD_REQUEST, "key cannot be empty".to_string())
}

pub async fn http_unsupported() -> impl IntoResponse {
    (StatusCode::METHOD_NOT_ALLOWED, "method not allowed".to_string())
}

pub async fn http_set(State(store): State<SharedStore>, Path(key): Path<String>, body: String) -> impl IntoResponse {
    if !key_is_valid(&key) {
        return (StatusCode::BAD_REQUEST, "key is invalid".to_string());
    }
    if body.is_empty() {
        return (StatusCode::BAD_REQUEST, "value cannot be empty".to_string());
    }
    let mut store = store.lock().unwrap();
    match store.set(key, body) {
        Ok(Some(_record)) => (StatusCode::OK, "Key updated successfully".to_string()),
        Ok(None) => (StatusCode::OK, "Key created successfully".to_string()),
        Err(e) => (StatusCode::BAD_REQUEST, e),
    }
}

pub async fn http_get(State(store): State<SharedStore>, Path(key): Path<String>) -> impl IntoResponse {
    if !key_is_valid(&key) {
        return (StatusCode::BAD_REQUEST, "key is invalid".to_string());
    }
    let store = store.lock().unwrap();
    match store.get(&key) {
        Ok(record) => (StatusCode::OK, record.value.clone()),
        Err(_e) => (StatusCode::NOT_FOUND, "key not found".to_string()),
    }
}

pub async fn http_delete(State(store): State<SharedStore>, Path(key): Path<String>) -> impl IntoResponse {
    if !key_is_valid(&key) {
        return (StatusCode::BAD_REQUEST, "key is invalid".to_string());
    }
    let mut store = store.lock().unwrap();
    match store.delete(&key) {
        Ok(Some(_record)) => (StatusCode::OK, "Key deleted successfully".to_string()),
        Ok(None) => (StatusCode::OK, "Key deleted successfully".to_string()),
        Err(_e) => (StatusCode::NOT_FOUND, "key not found".to_string()),
    }
}

pub async fn http_clear(State(store): State<SharedStore>) -> impl IntoResponse {
    let mut store = store.lock().unwrap();
    store.clear();
    (StatusCode::OK, "All keys cleared successfully".to_string())
}
