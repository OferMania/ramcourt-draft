use crate::KeyValue;
use crate::now_secs;
use uuid::Uuid;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Mutex, MutexGuard};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct ServerState {
    registry: Arc<Mutex<ServerRegistry>>,
    tables: Arc<RwLock<HashMap<String, KeyValue>>>,
}

impl ServerState {
    pub fn new(name: String, rack: String) -> Self {
        ServerState {
            registry: Arc::new(Mutex::new(ServerRegistry::new(name, rack))),
            tables: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Poison policy: recover and continue (log once). Safe today because every
    // mutation computes first, then does insert + table_version bump, with nothing
    // that can panic in between. Once replicas/recovery exist, revisit: fail-stop
    // and recover from a replica is the more honest policy for a store whose point
    // is never acting on stale state.
    pub fn lock_tables_for_read(&self) -> RwLockReadGuard<'_, HashMap<String, KeyValue>> {
        self.tables.read().unwrap_or_else(|poisoned| {
            eprintln!("WARNING: store mutex poisoned by an earlier panic; recovering");
            self.tables.clear_poison(); // log once, not on every request (Rust 1.77+)
            poisoned.into_inner()
        })
    }

    pub fn lock_tables_for_write(&self) -> RwLockWriteGuard<'_, HashMap<String, KeyValue>> {
        self.tables.write().unwrap_or_else(|poisoned| {
            eprintln!("WARNING: store mutex poisoned by an earlier panic; recovering");
            self.tables.clear_poison(); // log once, not on every request (Rust 1.77+)
            poisoned.into_inner()
        })
    }

    pub fn lock_registry(&self) -> MutexGuard<'_, ServerRegistry> {
        self.registry.lock().unwrap_or_else(|poisoned| {
            eprintln!("WARNING: registry mutex poisoned by an earlier panic; recovering");
            self.registry.clear_poison(); // log once, not on every request (Rust 1.77+)
            poisoned.into_inner()
        })
    }
}

pub async fn clean_expired(state: ServerState) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        interval.tick().await;
        let current_time_secs = now_secs();
        {
            let mut store = state.lock_tables_for_write();
            for (_table_name, table) in store.iter_mut() {
                table.clean(current_time_secs);
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerRegistry {
    name: String,
    rack: String,
    version: Uuid,
}

impl ServerRegistry {
    pub fn new(name: String, rack: String) -> Self {
        ServerRegistry { name, rack, version: Uuid::new_v4() }
    }
}
