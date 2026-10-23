
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KeyValueRecord {
    pub key: String,
    pub value: Option<String>,
    pub duration_secs: Option<u64>,
    pub expiration_secs: Option<u64>,
    pub version: Uuid,
    pub table_version: Uuid,
}

pub struct ValueRecord {
    pub duration_secs: u64,
    pub expiration_secs: u64,
    pub version: Uuid,
    pub value: String,
}

const DEFAULT_TTL_SECS: u64 = 300; // 5 minutes

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
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

    fn is_expired(record: &ValueRecord, now: u64) -> bool {
        now >= record.expiration_secs
    }

    fn live(&self, key: &str, now: u64) -> Option<&ValueRecord> {
        self.storage.get(key).filter(|r| !Self::is_expired(r, now))
    }

    pub fn clean(&mut self, current_time_secs: u64) {
        self.storage.retain(|_, r| !Self::is_expired(r, current_time_secs));
    }

    fn verify_empty(&self, version: Uuid) -> Result<(), String> {
        if version != Uuid::nil() {
            return Err(format!("{}", Uuid::nil()));
        } else {
            return Ok(());
        }
    }

    fn verify_version(&self, key: &str, version: Uuid, current_time_secs: u64) -> Result<(), String> {
        match self.live(key, current_time_secs) {
            Some(record) if record.version != version => Err(format!("{}", record.version)),
            Some(_) => Ok(()),
            None => self.verify_empty(version),
        }
    }

    pub fn set(&mut self, key: String, value_record: &KeyValueRecord, current_time_secs: u64) -> Result<KeyValueRecord, String> {
        let verification = self.verify_version(&key, value_record.version, current_time_secs);
        if !verification.is_ok() {
            return Err(format!("version mismatch for key '{}': expected {}, got {}", key, verification.unwrap_err(), value_record.version));
        }

        // Compute everything the response needs BEFORE key/record get moved into storage.
        let prior_record = self.live(&key, current_time_secs);
        if prior_record.is_none() && value_record.value.is_none() {
            return Err(format!("new key '{}' requires a value on initialization", key));
        }
        let value = if value_record.value.is_some() {
            value_record.value.clone().unwrap()
        } else {
            prior_record.unwrap().value.clone()
        };

        let duration_secs = match value_record.duration_secs {
            None if prior_record.is_some() => prior_record.unwrap().duration_secs,
            None => DEFAULT_TTL_SECS,
            Some(d) => d,
        };
        let expiration_secs = if duration_secs == 0 {u64::MAX} else { 
            current_time_secs.saturating_add(duration_secs)
        };
        let new_version = Uuid::new_v4();
        let key_for_response = key.clone();

        let record = ValueRecord {
            duration_secs,
            expiration_secs,
            version: new_version,
            value: value.clone(),
        };
        self.storage.insert(key, record);
        self.table_version = Uuid::new_v4();
        let external_record = KeyValueRecord {
            key: key_for_response,
            value: Some(value),
            duration_secs: Some(duration_secs),
            expiration_secs: Some(expiration_secs),
            version: new_version,
            table_version: self.table_version,
        };
        return Ok(external_record);
    }

    pub fn get(&self, key: &str, current_time_secs: u64) -> Result<KeyValueRecord, String> {
        let record = self.live(key, current_time_secs)
            .ok_or_else(|| format!("key '{}' not found", key))?;
        Ok(KeyValueRecord {
            key: key.to_string(),
            value: Some(record.value.clone()),
            duration_secs: Some(record.duration_secs),
            expiration_secs: Some(record.expiration_secs),
            version: record.version,
            table_version: self.table_version,
        })
    }

    pub fn delete(&mut self, key: &str, value_record: &KeyValueRecord, current_time_secs: u64) -> Result<KeyValueRecord, String> {
        if self.live(key, current_time_secs).is_none() {
            return Err(format!("key '{}' not found", key));
        }
        let verification = self.verify_version(&key, value_record.version, current_time_secs);
        if !verification.is_ok() {
            return Err(format!("version mismatch for key '{}': expected {}, got {}", key, verification.unwrap_err(), value_record.version));
        }
        self.storage.remove(key);
        self.table_version = Uuid::new_v4();
        let external_record = KeyValueRecord {
            key: key.to_string(),
            value: None,
            duration_secs: None,
            expiration_secs: None,
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
            value: None,
            duration_secs: None,
            expiration_secs: None,
            version: Uuid::nil(),
            table_version: self.table_version,
        };
        Ok(external_record)
    }

    pub fn get_table_version(&self) -> Result<KeyValueRecord, String> {
        let external_record = KeyValueRecord {
            key: "".to_string(),
            value: None,
            duration_secs: None,
            expiration_secs: None,
            version: Uuid::nil(),
            table_version: self.table_version,
        };
        Ok(external_record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(value: Option<String>, duration: Option<u64>, version: Option<Uuid>) -> KeyValueRecord {
        KeyValueRecord { value: value, duration_secs: duration, version: version.unwrap_or_else(Uuid::nil), ..Default::default() }
    }

    #[test]
    fn expired_key_is_absent_before_sweeper_runs() {
        let mut kv = KeyValue::new();
        kv.set("k".into(), &rec(Some("v".into()), Some(2), None), 1_000).unwrap();
        assert!(kv.get("k", 1_001).is_ok());
        assert!(kv.get("k", 1_002).is_err()); // expires exactly at 1002
        // recreate with a nil version without any clean() call
        assert!(kv.set("k".into(), &rec(Some("v2".into()), Some(2), None), 1_002).is_ok());
    }

    #[test]
    fn zero_duration_never_expires() {
        let mut kv = KeyValue::new();
        kv.set("k".into(), &rec(Some("v".into()), Some(0), None), 1_000).unwrap();
        assert!(kv.get("k", u64::MAX - 1).is_ok());
    }

    #[test]
    fn omitted_duration_nonexistent_uses_default_ttl() {
        let mut kv = KeyValue::new();
        let r = kv.set("k".into(), &rec(Some("v".into()), None, None), 1_000).unwrap();
        assert_eq!(r.duration_secs, Some(DEFAULT_TTL_SECS));
        assert!(kv.get("k", 1_000 + DEFAULT_TTL_SECS - 1).is_ok());
        assert!(kv.get("k", 1_000 + DEFAULT_TTL_SECS).is_err());
    }

    #[test]
    fn omitted_duration_existent_uses_prior_ttl() {
        let mut kv = KeyValue::new();
        let r0 = kv.set("k".into(), &rec(Some("v".into()), Some(100), None), 1_000).unwrap();
        let r = kv.set("k".into(), &rec(Some("v2".into()), None, Some(r0.version)), 1_005).unwrap();
        assert_eq!(r.duration_secs, Some(100));
        assert!(kv.get("k", 1_005 + 100 - 1).is_ok());
        assert!(kv.get("k", 1_005 + 100).is_err());
    }

    #[test]
    fn huge_duration_saturates_instead_of_overflowing() {
        let mut kv = KeyValue::new();
        let r = kv.set("k".into(), &rec(Some("v".into()), Some(u64::MAX), None), 1_000).unwrap();
        assert_eq!(r.expiration_secs, Some(u64::MAX));
    }

    #[test]
    fn omitted_value_on_create_is_error() {
        let mut kv = KeyValue::new();
        let r0 = kv.set("k".into(), &rec(None, Some(100), None), 1_000);
        assert!(r0.is_err());
    }

    #[test]
    fn omitted_value_on_update_preserves_value() {
        let mut kv = KeyValue::new();
        let r0 = kv.set("k".into(), &rec(Some("v".into()), Some(100), None), 1_000).unwrap();
        let r = kv.set("k".into(), &rec(None, Some(200), Some(r0.version)), 1_005).unwrap();
        assert_eq!(r.duration_secs, Some(200));
        assert_eq!(r.value, Some("v".into()));
        assert!(kv.get("k", 1_005 + 200 - 1).is_ok());
        assert!(kv.get("k", 1_005 + 200).is_err());
    }

    #[test]
    fn omitted_value_and_duration_on_update_renews() {
        let mut kv = KeyValue::new();
        let r0 = kv.set("k".into(), &rec(Some("v".into()), Some(100), None), 1_000).unwrap();
        let r = kv.set("k".into(), &rec(None, None, Some(r0.version)), 1_005).unwrap();
        assert_eq!(r.duration_secs, Some(100));
        assert_eq!(r.value, Some("v".into()));
        assert!(kv.get("k", 1_005 + 100 - 1).is_ok());
        assert!(kv.get("k", 1_005 + 100).is_err());
    }
}
