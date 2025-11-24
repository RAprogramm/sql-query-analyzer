use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{LazyLock, RwLock}
};

use crate::query::Query;

/// Global query cache
static QUERY_CACHE: LazyLock<RwLock<QueryCache>> =
    LazyLock::new(|| RwLock::new(QueryCache::new(1000)));

/// LRU-like cache for parsed queries
pub struct QueryCache {
    cache:    HashMap<u64, Vec<Query>>,
    max_size: usize
}

impl QueryCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(max_size),
            max_size
        }
    }

    fn hash_key(sql: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get(&self, sql: &str) -> Option<Vec<Query>> {
        let key = Self::hash_key(sql);
        self.cache.get(&key).cloned()
    }

    pub fn insert(&mut self, sql: &str, queries: Vec<Query>) {
        // Simple eviction: clear half when full
        if self.cache.len() >= self.max_size {
            let keys: Vec<_> = self.cache.keys().take(self.max_size / 2).copied().collect();
            for key in keys {
                self.cache.remove(&key);
            }
        }

        let key = Self::hash_key(sql);
        self.cache.insert(key, queries);
    }
}

/// Get cached queries or None
pub fn get_cached(sql: &str) -> Option<Vec<Query>> {
    QUERY_CACHE.read().ok()?.get(sql)
}

/// Cache parsed queries
pub fn cache_queries(sql: &str, queries: Vec<Query>) {
    if let Ok(mut cache) = QUERY_CACHE.write() {
        cache.insert(sql, queries);
    }
}
