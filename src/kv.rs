
use std::collections::HashMap;
use std::time::{ SystemTime, Duration };
use uuid::Uuid;

pub struct ValueRecord {
    pub expiration: SystemTime,
    pub version: Uuid,
    pub value: String,
}

pub struct KeyValue {
    storage: HashMap<String, ValueRecord>,
}

impl KeyValue {
    pub fn new() -> Self {
        KeyValue {
            storage: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Result<Option<ValueRecord>, String> {
        let record = ValueRecord {
            expiration: SystemTime::now() + Duration::from_secs(3600),
            version: Uuid::new_v4(),
            value,
        };
        return Ok(self.storage.insert(key, record));
    }

    pub fn get(&self, key: &str) -> Result<&ValueRecord, String> {
        if ! self.storage.contains_key(key) {
            return Err(format!("Key '{}' not found", key));
        }
        Ok(self.storage.get(key).unwrap())
    }

    pub fn delete(&mut self, key: &str) -> Result<ValueRecord, String> {
        if ! self.storage.contains_key(key) {
            return Err(format!("Key '{}' not found", key));
        }
        Ok(self.storage.remove(key).unwrap())
    }

    pub fn clear(&mut self) {
        self.storage.clear();
    }
}
