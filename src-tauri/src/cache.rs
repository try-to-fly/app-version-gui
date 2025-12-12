use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub latest_version: String,
    pub published_at: Option<DateTime<Utc>>,
    pub cached_at: DateTime<Utc>,
    pub ttl_minutes: i64,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let cache_duration = Duration::minutes(self.ttl_minutes);
        now - self.cached_at > cache_duration
    }
}

pub struct CacheManager {
    entries: RwLock<HashMap<String, CacheEntry>>,
    default_ttl: i64,
}

impl CacheManager {
    pub fn new(default_ttl_minutes: i64) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            default_ttl: default_ttl_minutes,
        }
    }

    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        let entries = self.entries.read().ok()?;
        let entry = entries.get(key)?;

        if entry.is_expired() {
            return None;
        }

        Some(entry.clone())
    }

    pub fn set(&self, key: &str, latest_version: String, published_at: Option<DateTime<Utc>>) {
        if let Ok(mut entries) = self.entries.write() {
            let entry = CacheEntry {
                latest_version,
                published_at,
                cached_at: Utc::now(),
                ttl_minutes: self.default_ttl,
            };
            entries.insert(key.to_string(), entry);
        }
    }

    pub fn invalidate(&self, key: &str) {
        if let Ok(mut entries) = self.entries.write() {
            entries.remove(key);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }

    pub fn set_ttl(&self, ttl_minutes: i64) {
        // Note: This doesn't affect existing entries
        // In a production app, you might want to update existing entries too
        let _ = ttl_minutes;
    }
}

pub type CacheState = CacheManager;
