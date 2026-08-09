use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};

use crate::kv::KeyValueRecord;

use crate::SharedStore;

fn key_is_valid(key: &String) -> bool {
    let expected_regex = regex::Regex::new(r"^([a-zA-Z0-9]|_|\-|\.|:)+$").unwrap();
    expected_regex.is_match(key)
}

pub async fn http_set_empty_key() -> impl IntoResponse {
    (StatusCode::BAD_REQUEST, Json("key cannot be empty")).into_response()
}

pub async fn http_unsupported() -> impl IntoResponse {
    (StatusCode::METHOD_NOT_ALLOWED, Json("method not allowed")).into_response()
}

pub async fn http_set(State(store): State<SharedStore>, Path(key): Path<String>, Json(payload): Json<KeyValueRecord>) -> impl IntoResponse {
    if !key_is_valid(&key) {
        return (StatusCode::BAD_REQUEST, Json("key is invalid")).into_response();
    }
    if payload.value.is_empty() {
        return (StatusCode::BAD_REQUEST, Json("value cannot be empty")).into_response();
    }
    let mut store = store.lock().unwrap();
    match store.set(key, &payload) {
        Ok(record) => (StatusCode::OK, Json(record)).into_response(),
        Err(e) => {
            if payload.version != uuid::Uuid::nil() {
                return (StatusCode::CONFLICT, Json(e)).into_response();
            }
            (StatusCode::BAD_REQUEST, Json(e)).into_response()
        },
    }
}

pub async fn http_get(State(store): State<SharedStore>, Path(key): Path<String>) -> impl IntoResponse {
    if !key_is_valid(&key) {
        return (StatusCode::BAD_REQUEST, Json("key is invalid")).into_response();
    }
    let store = store.lock().unwrap();
    match store.get(&key) {
        Ok(record) => (StatusCode::OK, Json(record)).into_response(),
        Err(_e) => (StatusCode::NOT_FOUND, Json("key not found")).into_response(),
    }
}

pub async fn http_delete(State(store): State<SharedStore>, Path(key): Path<String>, Json(payload): Json<KeyValueRecord>) -> impl IntoResponse {
    if !key_is_valid(&key) {
        return (StatusCode::BAD_REQUEST, Json("key is invalid")).into_response();
    }
    let mut store = store.lock().unwrap();
    match store.delete(&key, &payload) {
        Ok(record) => (StatusCode::OK, Json(record)).into_response(),
        Err(e) => {
            if payload.version != uuid::Uuid::nil() {
                return (StatusCode::CONFLICT, Json(e)).into_response();
            }
            (StatusCode::BAD_REQUEST, Json(e)).into_response()
        }
    }
}

pub async fn http_clear(State(store): State<SharedStore>, Json(payload): Json<KeyValueRecord>) -> impl IntoResponse {
    let mut store = store.lock().unwrap();
    match store.clear(&payload) {
        Ok(record) => (StatusCode::OK, Json(record)).into_response(),
        Err(e) => (StatusCode::CONFLICT, Json(e)).into_response(),
    }
}
