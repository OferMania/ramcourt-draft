use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};

use crate::kv::KeyValue;
use crate::kv::KeyValueRecord;
use crate::kv::now_secs;

use crate::ServerState;

fn identifier_is_valid(key: &str) -> bool {
    let expected_regex = regex::Regex::new(r"^([a-zA-Z0-9]|_|\-|\.|:)+$").unwrap();
    expected_regex.is_match(key)
}

fn identifier_to_table_and_key(identifier: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = identifier.splitn(2, ':').collect();
    if parts.len() < 1 {
        return None;
    }
    let table = parts[0].to_string();
    if parts.len() < 2 {
        return Some((table, "".to_string()));
    }
    let key = parts[1].to_string();
    Some((table, key))
}

pub async fn http_set_empty_key() -> impl IntoResponse {
    (StatusCode::BAD_REQUEST, Json("key cannot be empty")).into_response()
}

pub async fn http_unsupported() -> impl IntoResponse {
    (StatusCode::METHOD_NOT_ALLOWED, Json("method not allowed")).into_response()
}

pub async fn http_set(State(state): State<ServerState>, Path(identifier): Path<String>, Json(payload): Json<KeyValueRecord>) -> impl IntoResponse {
    if !identifier_is_valid(&identifier) {
        return (StatusCode::BAD_REQUEST, Json("key or table is invalid")).into_response();
    }
    let (table, key) = match identifier_to_table_and_key(&identifier) {
        Some((table, key)) => (table, key),
        None => return (StatusCode::BAD_REQUEST, Json("key or table is invalid")).into_response(),
    };
    let current_time_secs = now_secs();
    let mut store = state.lock_tables_for_write();
    if key.is_empty() {
        if store.contains_key(&table) {
            return (StatusCode::BAD_REQUEST, Json("table already exists")).into_response();
        } else {
            store.insert(table.clone(), KeyValue::new());
            match store[&table].get_table_version() {
                Ok(table_version) => return (StatusCode::OK, Json(table_version)).into_response(),
                Err(e) => return (StatusCode::BAD_REQUEST, Json(e)).into_response(),
            }
        }
    }

    if !store.contains_key(&table) {
        return (StatusCode::BAD_REQUEST, Json("table does not exist")).into_response();
    }
    match store.get_mut(&table).unwrap().set(key, &payload, current_time_secs) {
        Ok(record) => (StatusCode::OK, Json(record)).into_response(),
        Err(e) => {
            if payload.version != uuid::Uuid::nil() {
                return (StatusCode::CONFLICT, Json(e)).into_response();
            }
            (StatusCode::BAD_REQUEST, Json(e)).into_response()
        },
    }
}

pub async fn http_get(State(state): State<ServerState>, Path(identifier): Path<String>) -> impl IntoResponse {
    if !identifier_is_valid(&identifier) {
        return (StatusCode::BAD_REQUEST, Json("key or table is invalid")).into_response();
    }
    let (table, key) = match identifier_to_table_and_key(&identifier) {
        Some((table, key)) => (table, key),
        None => return (StatusCode::BAD_REQUEST, Json("key or table is invalid")).into_response(),
    };
    let current_time_secs = now_secs();
    let store = state.lock_tables_for_read();
    if !store.contains_key(&table) {
        return (StatusCode::BAD_REQUEST, Json("table does not exist")).into_response();
    }
    if key.is_empty() {
        match store[&table].get_table_version() {
            Ok(table_version) => return (StatusCode::OK, Json(table_version)).into_response(),
            Err(e) => return (StatusCode::BAD_REQUEST, Json(e)).into_response(),
        }
    }
    match store[&table].get(&key, current_time_secs) {
        Ok(record) => (StatusCode::OK, Json(record)).into_response(),
        Err(_e) => (StatusCode::NOT_FOUND, Json("key not found")).into_response(),
    }
}

pub async fn http_delete(State(state): State<ServerState>, Path(identifier): Path<String>, Json(payload): Json<KeyValueRecord>) -> impl IntoResponse {
    if !identifier_is_valid(&identifier) {
        return (StatusCode::BAD_REQUEST, Json("key or table is invalid")).into_response();
    }
    let (table, key) = match identifier_to_table_and_key(&identifier) {
        Some((table, key)) => (table, key),
        None => return (StatusCode::BAD_REQUEST, Json("key or table is invalid")).into_response(),
    };
    let current_time_secs = now_secs();
    let mut store = state.lock_tables_for_write();
    if !store.contains_key(&table) {
        return (StatusCode::BAD_REQUEST, Json("table does not exist")).into_response();
    }
    if key.is_empty() {
        match store.get_mut(&table).unwrap().clear(&payload) {
            Ok(record) => {
                store.remove(&table);
                return (StatusCode::OK, Json(record)).into_response()
            },
            Err(e) => return (StatusCode::CONFLICT, Json(e)).into_response(),
        }
    }
    match store.get_mut(&table).unwrap().delete(&key, &payload, current_time_secs) {
        Ok(record) => (StatusCode::OK, Json(record)).into_response(),
        Err(e) => {
            if payload.version != uuid::Uuid::nil() {
                return (StatusCode::CONFLICT, Json(e)).into_response();
            }
            (StatusCode::BAD_REQUEST, Json(e)).into_response()
        }
    }
}

// pub async fn http_clear(State(state): State<ServerState>, Json(payload): Json<KeyValueRecord>) -> impl IntoResponse {
//     let mut store = state.lock_tables_for_write();
//     match store.clear(&payload) {
//         Ok(record) => (StatusCode::OK, Json(record)).into_response(),
//         Err(e) => (StatusCode::CONFLICT, Json(e)).into_response(),
//     }
// }

// pub async fn http_get_table_version(State(state): State<ServerState>) -> impl IntoResponse {
//     let store = state.lock_tables_for_read();
//     match store.get_table_version() {
//         Ok(table_version) => (StatusCode::OK, Json(table_version)).into_response(),
//         Err(e) => (StatusCode::BAD_REQUEST, Json(e)).into_response(),
//     }
// }

pub async fn get_server_info(State(state): State<ServerState>) -> impl IntoResponse {
    let registry = state.lock_registry().clone();
    (StatusCode::OK, Json(registry)).into_response()
}
