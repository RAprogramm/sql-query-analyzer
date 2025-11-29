// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

use sql_query_analyzer::{
    cache::{QueryCache, cache_queries, get_cached},
    query::{SqlDialect, parse_queries}
};

#[test]
fn test_query_cache_new() {
    let cache = QueryCache::new(100);
    assert!(cache.get("SELECT 1").is_none());
}

#[test]
fn test_query_cache_insert_and_get() {
    let mut cache = QueryCache::new(100);
    let queries = parse_queries("SELECT id FROM users", SqlDialect::Generic).unwrap();
    cache.insert("SELECT id FROM users", queries.clone());
    let cached = cache.get("SELECT id FROM users");
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().len(), queries.len());
}

#[test]
fn test_query_cache_miss() {
    let cache = QueryCache::new(100);
    assert!(cache.get("SELECT * FROM nonexistent").is_none());
}

#[test]
fn test_query_cache_eviction() {
    let mut cache = QueryCache::new(3);
    let q1 = parse_queries("SELECT 1", SqlDialect::Generic).unwrap();
    let q2 = parse_queries("SELECT 2", SqlDialect::Generic).unwrap();
    let q3 = parse_queries("SELECT 3", SqlDialect::Generic).unwrap();
    let q4 = parse_queries("SELECT 4", SqlDialect::Generic).unwrap();
    cache.insert("SELECT 1", q1);
    cache.insert("SELECT 2", q2);
    cache.insert("SELECT 3", q3);
    cache.insert("SELECT 4", q4);
    let cached4 = cache.get("SELECT 4");
    assert!(cached4.is_some());
}

#[test]
fn test_global_cache_queries() {
    let queries = parse_queries("SELECT global FROM test", SqlDialect::Generic).unwrap();
    cache_queries("SELECT global FROM test", queries.clone());
    let cached = get_cached("SELECT global FROM test");
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().len(), queries.len());
}

#[test]
fn test_global_get_cached_miss() {
    let cached = get_cached("SELECT random_unique_query_xyz_123");
    assert!(cached.is_none());
}

#[test]
fn test_cache_different_queries() {
    let mut cache = QueryCache::new(100);
    let q1 = parse_queries("SELECT a FROM t1", SqlDialect::Generic).unwrap();
    let q2 = parse_queries("SELECT b FROM t2", SqlDialect::Generic).unwrap();
    cache.insert("SELECT a FROM t1", q1.clone());
    cache.insert("SELECT b FROM t2", q2.clone());
    let cached1 = cache.get("SELECT a FROM t1");
    let cached2 = cache.get("SELECT b FROM t2");
    assert!(cached1.is_some());
    assert!(cached2.is_some());
}

#[test]
fn test_cache_overwrite() {
    let mut cache = QueryCache::new(100);
    let q1 = parse_queries("SELECT 1", SqlDialect::Generic).unwrap();
    let q2 = parse_queries("SELECT 1; SELECT 2", SqlDialect::Generic).unwrap();
    cache.insert("SELECT 1", q1);
    cache.insert("SELECT 1", q2.clone());
    let cached = cache.get("SELECT 1").unwrap();
    assert_eq!(cached.len(), q2.len());
}
