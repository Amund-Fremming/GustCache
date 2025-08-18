#[cfg(test)]
mod test {
    use tokio::time::sleep;

    use crate::{CacheError, ChronoTime, GustCache, TokioTime};

    async fn failure_fn() -> Result<String, CacheError> {
        tokio::time::sleep(TokioTime::from_nanos(1)).await;
        Ok("failure-fn".to_string())
    }

    async fn failed_insert_fn() -> Result<String, CacheError> {
        tokio::time::sleep(TokioTime::from_nanos(1)).await;
        Ok("failure-insert".to_string())
    }

    #[tokio::test]
    async fn invalidate_successfull() {
        let mut cache: GustCache<String> = GustCache::from_ttl(ChronoTime::seconds(1));

        cache.insert(&1, "value 1".into()).await;
        cache.insert(&2, "value 2".into()).await;
        cache.insert(&3, "value 3".into()).await;

        cache.invalidate().await;
        let size = cache.size().await;

        assert_eq!(0, size);
    }

    #[tokio::test]
    async fn get_or_executes_failure_fn() {
        let cache: GustCache<String> = GustCache::from_ttl(ChronoTime::seconds(1));
        cache.insert(&2, "cache-value".into()).await;
        let result = cache.get_or(&1, || failure_fn()).await.unwrap();

        assert_eq!("failure-fn".to_string(), result);
    }

    #[tokio::test]
    async fn get_or_hits_cache() {
        // Manual insert
        let cache: GustCache<String> = GustCache::from_ttl(ChronoTime::seconds(1));
        cache.insert(&1, "manual-insert".to_string()).await;
        let result = cache.get_or(&1, || failure_fn()).await.unwrap();

        assert_eq!("manual-insert".to_string(), result);

        // Insert by failed hit
        cache.get_or(&10, || failed_insert_fn()).await.unwrap();
        let result = cache.get_or(&10, || failure_fn()).await.unwrap();

        assert_eq!("failure-insert".to_string(), result);
    }

    #[tokio::test]
    async fn cleanup_task_should_be_empty() {
        let cache: GustCache<String> = GustCache::from_ttl(ChronoTime::seconds(1));

        cache.insert(&1, "value 1".into()).await;
        cache.insert(&2, "value 2".into()).await;
        cache.insert(&3, "value 3".into()).await;

        sleep(tokio::time::Duration::from_secs(2)).await;
        let size = cache.size().await;

        assert_eq!(0, size);
    }

    #[tokio::test]
    async fn cleanup_task_should_not_empty() {
        let cache: GustCache<String> = GustCache::from_ttl(chrono::Duration::seconds(2));

        cache.insert(&1, "value 1".into()).await;
        cache.insert(&2, "value 2".into()).await;
        cache.insert(&3, "value 3".into()).await;
        let size = cache.size().await;

        assert_eq!(3, size);
    }

    #[tokio::test]
    async fn insert_and_get_successfull() {
        // Insert
        let cache: GustCache<String> = GustCache::new();
        cache.insert(&1, "manual-insert".to_string()).await;
        let size = cache.size().await;

        assert_eq!(1, size);

        // Get
        let result = cache.try_get(&1).await.unwrap();
        assert_eq!(result, "manual-insert".to_string());
    }

    #[tokio::test]
    async fn get_should_be_none() {
        let cache: GustCache<String> = GustCache::new();
        let result = cache.try_get(&1).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn eviction_task_should_evict() {
        todo!();
    }

    #[tokio::test]
    async fn eviction_task_should_not_evict() {
        todo!();
    }
}
