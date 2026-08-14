
use std::collections::HashMap;
use std::time::{ SystemTime };
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KeyValueRecord {
    pub key: String,
    pub value: String,
    pub duration_secs: u64,
    pub expiration_secs: u64,
    pub version: Uuid,
    pub table_version: Uuid,
}

pub struct ValueRecord {
    pub duration_secs: u64,
    pub expiration_secs: u64,
    pub version: Uuid,
    pub value: String,
}

pub struct KeyValue {

    storage: HashMap<String, ValueRecord>,

    // represents the whole store today; will map 1:1 to a table once multi-table support lands
    table_version: Uuid,
}

impl KeyValue {
    pub fn new() -> Self {
        KeyValue {
            storage: HashMap::new(),
            table_version: Uuid::new_v4(),
        }
    }

    fn verify_version(&self, key: &str, version: Uuid) -> Result<(), String> {
        match self.storage.get(key) {
            Some(record) => {
                if record.version != version {
                    return Err(format!("{}", record.version));
                }
                Ok(())
            }
            None => {
                if version != Uuid::nil() {
                    return Err(format!("{}", Uuid::nil()));
                } else {
                    return Ok(());
                }
            }
        }
    }

    pub fn set(&mut self, key: String, value_record: &KeyValueRecord) -> Result<KeyValueRecord, String> {
        let verification = self.verify_version(&key, value_record.version);
        if !verification.is_ok() {
            return Err(format!("version mismatch for key '{}': expected {}, got {}", key, verification.unwrap_err(), value_record.version));
        }

        // Compute everything the response needs BEFORE key/record get moved into storage.
        let expiration_secs = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + value_record.duration_secs;
        let new_version = Uuid::new_v4();
        let key_for_response = key.clone();

        let record = ValueRecord {
            duration_secs: value_record.duration_secs,
            expiration_secs,
            version: new_version,
            value: value_record.value.clone(),
        };
        self.storage.insert(key, record);
        self.table_version = Uuid::new_v4();
        let external_record = KeyValueRecord {
            key: key_for_response,
            value: value_record.value.clone(),
            duration_secs: value_record.duration_secs,
            expiration_secs,
            version: new_version,
            table_version: self.table_version,
        };
        return Ok(external_record);
    }

    pub fn get(&self, key: &str) -> Result<KeyValueRecord, String> {
        if ! self.storage.contains_key(key) {
            return Err(format!("key '{}' not found", key));
        }
        let record = self.storage.get(key).unwrap();
        let external_record = KeyValueRecord {
            key: key.to_string(),
            value: record.value.clone(),
            duration_secs: record.duration_secs,
            expiration_secs: record.expiration_secs,
            version: record.version,
            table_version: self.table_version,
        };
        Ok(external_record)
    }

    pub fn delete(&mut self, key: &str, value_record: &KeyValueRecord) -> Result<KeyValueRecord, String> {
        let verification = self.verify_version(&key, value_record.version);
        if !verification.is_ok() {
            return Err(format!("version mismatch for key '{}': expected {}, got {}", key, verification.unwrap_err(), value_record.version));
        }
        self.storage.remove(key);
        self.table_version = Uuid::new_v4();
        let external_record = KeyValueRecord {
            key: key.to_string(),
            value: "".to_string(),
            duration_secs: 0,
            expiration_secs: 0,
            version: Uuid::nil(),
            table_version: self.table_version,
        };
        Ok(external_record)
    }

    pub fn clear(&mut self, value_record: &KeyValueRecord) -> Result<KeyValueRecord, String> {
        if value_record.table_version != self.table_version {
            return Err(format!("table_version mismatch: expected {}, got {}", self.table_version, value_record.table_version));
        }
        self.storage.clear();
        self.table_version = Uuid::new_v4();
        let external_record = KeyValueRecord {
            key: "".to_string(),
            value: "".to_string(),
            duration_secs: 0,
            expiration_secs: 0,
            version: Uuid::nil(),
            table_version: self.table_version,
        };
        Ok(external_record)
    }

    pub fn get_table_version(&self) -> Result<KeyValueRecord, String> {
        let external_record = KeyValueRecord {
            key: "".to_string(),
            value: "".to_string(),
            duration_secs: 0,
            expiration_secs: 0,
            version: Uuid::nil(),
            table_version: self.table_version,
        };
        Ok(external_record)
    }
}
