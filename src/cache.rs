use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::error::CacheError;

#[derive(Debug, Clone)]
pub struct CacheEntry<T: Clone> {
    timestamp: DateTime<Utc>,
    data: T,
}

#[derive(Debug)]
pub struct GustCache<T: Clone> {
    cache: RwLock<HashMap<u64, CacheEntry<T>>>,
    ttl: chrono::Duration,
}

impl<T: Clone> GustCache<T> {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl: chrono::Duration::minutes(2),
        }
    }

    pub fn from_ttl(ttl: chrono::Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    fn generate_hash<TKey>(&self, value: &TKey) -> u64
    where
        TKey: Hash,
    {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    pub async fn invalidate(&mut self) {
        *self = GustCache {
            cache: RwLock::new(HashMap::new()),
            ttl: chrono::Duration::minutes(2),
        };
    }

    pub async fn get<F, TKey>(&self, req: &TKey, db_fn: F) -> Result<T, CacheError>
    where
        F: AsyncFnOnce() -> Result<T, CacheError>,
        TKey: Hash,
    {
        let key = self.generate_hash(req);
        let mut map = self.cache.write().await;

        if let Some(entry) = map.get_mut(&key) {
            if entry.timestamp + self.ttl > Utc::now() {
                entry.timestamp = Utc::now();
            }
        };

        // Release lock while db operation finishes
        drop(map);

        let data = db_fn().await?;
        let cache_entry = CacheEntry {
            data: data.clone(),
            timestamp: Utc::now(),
        };

        let mut map = self.cache.write().await;
        map.insert(key, cache_entry.clone());

        Ok(data)
    }
}
