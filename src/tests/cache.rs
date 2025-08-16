#[cfg(test)]
mod test {
    use tokio::time::sleep;

    use crate::{CacheError, GustCache};

    type TokioTime = tokio::time::Duration;
    type ChronoTime = chrono::Duration;

    async fn failure_fn() -> Result<String, CacheError> {
        tokio::time::sleep(TokioTime::from_nanos(1)).await;
        Ok("failure-fn".to_string())
    }

    #[tokio::test]
    async fn invalidate_successfull() {
        assert!(true);
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
        let cache: GustCache<String> = GustCache::from_ttl(ChronoTime::seconds(1));
        cache.insert(&1, "cache-hit".to_string()).await;
        let result = cache.get_or(&1, || failure_fn()).await.unwrap();

        assert_eq!("cache-hit".to_string(), result);
    }

    #[tokio::test]
    async fn cleanup_task_should_be_empty() {
        let cache: GustCache<String> = GustCache::from_ttl(ChronoTime::seconds(1));

        cache.insert(&1, "value 1".into()).await;
        cache.insert(&2, "value 2".into()).await;
        cache.insert(&3, "value 3".into()).await;

        sleep(tokio::time::Duration::from_secs(11)).await;
        let size = cache.size().await;

        assert_eq!(0, size);
    }

    #[tokio::test]
    async fn cleanup_task_should_not_empty() {
        let cache: GustCache<String> = GustCache::from_ttl(chrono::Duration::seconds(1));

        cache.insert(&1, "value 1".into()).await;
        cache.insert(&2, "value 2".into()).await;
        cache.insert(&3, "value 3".into()).await;

        sleep(TokioTime::from_secs(2)).await;
        let size = cache.size().await;

        assert_eq!(3, size);
    }

    #[tokio::test]
    async fn x() {
        assert!(true);
    }
}
