use std::{collections::HashMap, hash::Hash, sync::Arc};

use chrono::{DateTime, Utc};
use tokio::{sync::RwLock, task::JoinHandle};

use crate::generate_hash;

#[derive(Debug, Clone)]
struct CacheEntry<T: Clone + Sync + 'static> {
    timestamp: DateTime<Utc>,
    value: T,
}

#[derive(Debug)]
pub struct GustCache<T: Clone + Send + Sync + 'static> {
    cache: Arc<RwLock<HashMap<u64, CacheEntry<T>>>>,
    ttl: chrono::Duration,
    cleanup_task: Option<JoinHandle<()>>,
}

impl<T: Clone + Send + Sync> GustCache<T> {
    pub fn new() -> Self {
        let mut cache = Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: chrono::Duration::minutes(2),
            cleanup_task: None,
        };
        cache.spawn_cleanup();
        cache
    }

    pub fn from_ttl(ttl: chrono::Duration) -> Self {
        let mut cache = Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            cleanup_task: None,
        };
        cache.spawn_cleanup();
        cache
    }

    pub async fn invalidate(&mut self) {
        if let Some(task) = self.cleanup_task.take() {
            task.abort();
        }

        {
            let mut lock = self.cache.write().await;
            lock.clear();
        }

        self.spawn_cleanup();
    }

    pub async fn get_or<K, F, E>(&self, key: &K, on_failure: F) -> Result<T, E>
    where
        F: AsyncFnOnce() -> Result<T, E>,
        K: Hash,
    {
        let key = generate_hash(key);
        let mut lock = self.cache.write().await;
        if let Some(entry) = lock.get_mut(&key) {
            if entry.timestamp + self.ttl > Utc::now() {
                entry.timestamp = Utc::now();
                return Ok(entry.value.clone());
            }
        };

        // Release lock while db operation finishes
        drop(lock);

        let data = on_failure().await?;
        let cache_entry = CacheEntry {
            value: data.clone(),
            timestamp: Utc::now(),
        };

        let mut map = self.cache.write().await;
        map.insert(key, cache_entry.clone());

        Ok(data)
    }

    pub async fn insert<K>(&self, key: &K, value: T)
    where
        K: Hash,
    {
        let key = generate_hash(key);
        let mut lock = self.cache.write().await;
        let cache_entry = CacheEntry {
            timestamp: Utc::now(),
            value,
        };
        lock.insert(key, cache_entry);
    }

    pub async fn try_get<K>(&self, key: &K) -> Option<T>
    where
        K: Hash,
    {
        let key = generate_hash(key);
        let lock = self.cache.read().await;
        return match lock.get(&key).cloned() {
            Some(cache_entry) => Some(cache_entry.value),
            None => None,
        };
    }

    fn spawn_cleanup(&mut self) {
        let interval_seconds = self.ttl.num_seconds() as u64 * 10;
        let interval = tokio::time::Duration::from_secs(interval_seconds);
        let mut ticker = tokio::time::interval(interval);
        let cache_pointer = self.cache.clone();

        let offset = self.ttl.clone();

        self.cleanup_task = Some(tokio::spawn(async move {
            loop {
                ticker.tick().await;
                let mut lock = cache_pointer.write().await;
                let now = Utc::now();
                lock.retain(|_, value| now < value.timestamp + offset);

                // Drop lock to avoid deadlocks
                drop(lock);
            }
        }));
    }

    pub async fn size(&self) -> usize {
        let lock = self.cache.read().await;
        lock.len()
    }
}
