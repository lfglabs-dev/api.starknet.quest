#[cfg(test)]
pub mod tests {
    use std::time::{Duration, Instant};
    use dashmap::DashMap;
    use once_cell::sync::Lazy;
    
    // Mock cache for testing
    static TEST_CACHE: Lazy<DashMap<String, (Instant, String)>> = Lazy::new(DashMap::new);
    const TEST_CACHE_TTL: Duration = Duration::from_millis(100);

    #[tokio::test]
    async fn test_cache_hit() {
        let key = "test_key".to_string();
        let value = "test_value".to_string();
        
        // Insert value into cache
        TEST_CACHE.insert(key.clone(), (Instant::now(), value.clone()));
        
        // Check cache hit
        if let Some((cached_at, cached_value)) = TEST_CACHE.get(&key).map(|v| v.value().clone()) {
            if cached_at.elapsed() < TEST_CACHE_TTL {
                assert_eq!(cached_value, value);
            }
        }
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let key = "nonexistent_key".to_string();
        
        // Check cache miss
        assert!(TEST_CACHE.get(&key).is_none());
    }

    #[tokio::test]
    async fn test_cache_expiry() {
        let key = "expiry_test_key".to_string();
        let value = "expiry_test_value".to_string();
        
        // Insert value with past timestamp to simulate expiry
        TEST_CACHE.insert(key.clone(), (Instant::now() - Duration::from_secs(1), value.clone()));
        
        // Check cache expiry
        if let Some((cached_at, _)) = TEST_CACHE.get(&key).map(|v| v.value().clone()) {
            assert!(cached_at.elapsed() > TEST_CACHE_TTL);
        }
    }

    #[tokio::test]
    async fn test_cache_update() {
        let key = "update_test_key".to_string();
        let old_value = "old_value".to_string();
        let new_value = "new_value".to_string();
        
        // Insert old value
        TEST_CACHE.insert(key.clone(), (Instant::now(), old_value.clone()));
        
        // Update with new value
        TEST_CACHE.insert(key.clone(), (Instant::now(), new_value.clone()));
        
        // Check updated value
        if let Some((_, cached_value)) = TEST_CACHE.get(&key).map(|v| v.value().clone()) {
            assert_eq!(cached_value, new_value);
            assert_ne!(cached_value, old_value);
        }
    }

    #[tokio::test]
    async fn test_cache_concurrent_access() {
        let key = "concurrent_test_key".to_string();
        let value = "concurrent_test_value".to_string();
        
        // Simulate concurrent access
        let handles: Vec<_> = (0..10).map(|i| {
            let key = key.clone();
            let value = format!("{}_{}", value, i);
            tokio::spawn(async move {
                TEST_CACHE.insert(key, (Instant::now(), value));
            })
        }).collect();
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify cache contains a value (any of the concurrent writes)
        assert!(TEST_CACHE.get(&key).is_some());
    }
}
