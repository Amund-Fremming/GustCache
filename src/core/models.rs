use chrono::{DateTime, Utc};

pub type TokioTime = tokio::time::Duration;
pub type ChronoTime = chrono::Duration;

// 20MB
pub static MAX_BYTE_SIZE: usize = 20_971_520;

#[derive(Debug, Clone)]
pub struct CacheEntry<T: Clone + Sync + 'static> {
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) value: T,
}

impl<T: Clone + Sync + 'static> CacheEntry<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            timestamp: Utc::now(),
            value,
        }
    }
}
