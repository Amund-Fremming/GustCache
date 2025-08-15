use std::{collections::HashMap, hash::Hash};

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::generate_hash;

#[derive(Debug, Clone)]
struct CacheEntry<T: Clone> {
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

    pub async fn invalidate(&mut self) {
        *self = GustCache {
            cache: RwLock::new(HashMap::new()),
            ttl: chrono::Duration::minutes(2),
        };
    }

    pub async fn get_or<F, TKey, E>(&self, req: &TKey, on_failure: F) -> Result<T, E>
    where
        F: AsyncFnOnce() -> Result<T, E>,
        TKey: Hash,
    {
        let key = generate_hash(req);
        let mut map = self.cache.write().await;

        if let Some(entry) = map.get_mut(&key) {
            if entry.timestamp + self.ttl > Utc::now() {
                entry.timestamp = Utc::now();
            }
        };

        // Release lock while db operation finishes
        drop(map);

        let data = on_failure().await?;
        let cache_entry = CacheEntry {
            data: data.clone(),
            timestamp: Utc::now(),
        };

        let mut map = self.cache.write().await;
        map.insert(key, cache_entry.clone());

        Ok(data)
    }

    pub async fn try_get() {
        todo!();
    }
}
