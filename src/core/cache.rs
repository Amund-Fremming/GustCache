use std::{collections::HashMap, hash::Hash, sync::Arc};

use chrono::{DateTime, Utc};
use tokio::{sync::RwLock, task::JoinHandle};

use crate::{CacheEntry, ChronoTime, MAX_BYTE_SIZE, TokioTime, core::common::generate_hash};

#[derive(Debug)]
pub struct GustCache<T: Clone + Send + Sync + 'static> {
    cache: Arc<RwLock<HashMap<u64, CacheEntry<T>>>>,
    ttl: ChronoTime,
    cleanup_task: Option<JoinHandle<()>>,
    eviction_task: Option<JoinHandle<()>>,
}

impl<T: Clone + Send + Sync> GustCache<T> {
    pub fn new() -> Self {
        Self::setup(ChronoTime::minutes(2))
    }

    pub fn from_ttl(ttl: chrono::Duration) -> Self {
        Self::setup(ttl)
    }

    pub async fn size(&self) -> usize {
        let lock = self.cache.read().await;
        lock.len()
    }

    fn setup(ttl: ChronoTime) -> Self {
        let mut cache = Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            cleanup_task: None,
            eviction_task: None,
        };

        cache.spawn_cleanup();
        cache.spawn_eviction();
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
        self.spawn_eviction();
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
        let cache_entry = CacheEntry::new(data.clone());
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
        let cache_entry = CacheEntry::new(value);
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
        let mut interval_seconds = (self.ttl.num_seconds() / 2) as u64;
        if interval_seconds == 0 {
            interval_seconds = 1;
        }
        let interval = TokioTime::from_secs(interval_seconds);
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

    fn spawn_eviction(&mut self) {
        let interval = TokioTime::from_secs(60 * 10);
        let mut ticker = tokio::time::interval(interval);
        let cache_pointer = self.cache.clone();

        self.eviction_task = Some(tokio::spawn(async move {
            loop {
                ticker.tick().await;

                let lock = cache_pointer.read().await;
                let cache_byte_size: usize = lock
                    .values()
                    .map(|entry| std::mem::size_of_val(entry))
                    .sum();

                if cache_byte_size < MAX_BYTE_SIZE {
                    drop(lock);
                    continue;
                }

                let num_evictions = lock.len() * 70 / 100;
                let mut entries: Vec<(u64, DateTime<Utc>)> =
                    lock.iter().map(|(k, v)| (*k, v.timestamp)).collect();

                drop(lock);

                entries.sort_by_key(|(_, ts)| std::cmp::Reverse(*ts));
                let mut overflow: Vec<u64> = Vec::new();

                for _ in 0..num_evictions {
                    match entries.pop() {
                        None => break,
                        Some((key, _)) => overflow.push(key),
                    };
                }

                let mut lock = cache_pointer.write().await;
                for key in overflow {
                    lock.remove(&key);
                }

                drop(lock);
            }
        }));
    }
}
