use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};

use serde_json::json;

use crate::kv::KeyValue;
use crate::kv::KeyValueRecord;
use crate::kv::TableRecord;
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
        store.insert(table.clone(), KeyValue::new());
        match store[&table].get_table_version() {
            Err(e) => return (StatusCode::BAD_REQUEST, Json(e)).into_response(),
            _ => {
                // table created, continue to set the key
            },
        }
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
        return (StatusCode::NOT_FOUND, Json("table not found")).into_response();
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
        return (StatusCode::NOT_FOUND, Json("table not found")).into_response();
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

pub async fn get_server_info(State(state): State<ServerState>) -> impl IntoResponse {
    let registry = state.lock_registry().clone();
    (StatusCode::OK, Json(registry)).into_response()
}

pub async fn enumerate_tables(State(state): State<ServerState>) -> impl IntoResponse {
    let store = state.lock_tables_for_read();
    let mut table_names_versions: Vec<TableRecord> = Vec::new();
    for (table_name, table) in store.iter() {
        table_names_versions.push(TableRecord {
            table_name: table_name.clone(),
            table_version: table.get_table_version().unwrap_or_else(|_| KeyValueRecord::default()).table_version,
        });
    }
    let result = json!(
        {
            "tables": table_names_versions,
        }
    );
    (StatusCode::OK, Json(result)).into_response()
}

pub async fn enumerate_table(State(state): State<ServerState>, Path(table): Path<String>) -> impl IntoResponse {
    let store = state.lock_tables_for_read();
    if !store.contains_key(&table) {
        return (StatusCode::NOT_FOUND, Json("table not found")).into_response();
    }
    let current_time_secs = now_secs();
    let mut key_values: Vec<KeyValueRecord> = Vec::new();
    let table_version = store[&table].get_table_version().unwrap_or_else(|_| KeyValueRecord::default()).table_version;
    for key in store[&table].keys() {
        match store[&table].get(&key, current_time_secs) {
            Ok(record) => {
                key_values.push(KeyValueRecord {
                    key: key.clone(),
                    value: record.value.clone(),
                    duration_secs: record.duration_secs,
                    expiration_secs: record.expiration_secs,
                    version: record.version,
                    table_version,
                });
            },
            Err(_e) => {
                // skip expired keys
            },
        }
    }
    let result = json!(
        {
            "keys": key_values,
        }
    );
    (StatusCode::OK, Json(result)).into_response()
}
