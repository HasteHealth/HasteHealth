//! The cache behind parsed expressions and compiled patterns. Both are keyed by text that
//! can come from a request, so what the cache holds is bounded.

use dashmap::DashMap;
use std::sync::{Mutex, PoisonError};

/// Values by the text they were built from, each with a weight that is roughly its size in
/// bytes. The weights never add up to more than `capacity`: an insert that would exceed it
/// empties the cache first, and a value heavier than `capacity` is not kept at all.
///
/// Emptying takes no bookkeeping on a lookup, which stays one map read, and costs what is
/// still in use one rebuild each.
pub(crate) struct WeightedCache<V> {
    entries: DashMap<String, (V, usize)>,
    capacity: usize,
    /// Total weight of `entries`. Held for the length of an insert, so it stays exact.
    weight: Mutex<usize>,
}

impl<V: Clone> WeightedCache<V> {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            entries: DashMap::new(),
            capacity,
            weight: Mutex::new(0),
        }
    }

    pub(crate) fn get(&self, key: &str) -> Option<V> {
        self.entries.get(key).map(|entry| entry.0.clone())
    }

    pub(crate) fn insert(&self, key: &str, value: V, weight: usize) {
        if weight > self.capacity {
            return;
        }

        let mut total = self.weight.lock().unwrap_or_else(PoisonError::into_inner);
        if *total + weight > self.capacity {
            self.entries.clear();
            *total = 0;
        }
        // Two callers can build the value for one key at the same time.
        if let Some((_, replaced)) = self.entries.insert(key.to_string(), (value, weight)) {
            *total -= replaced;
        }
        *total += weight;
    }

    #[cfg(test)]
    pub(crate) fn weight(&self) -> usize {
        *self.weight.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_values_within_capacity() {
        let cache = WeightedCache::new(10);
        cache.insert("a", 1, 4);
        cache.insert("b", 2, 6);

        assert_eq!(cache.get("a"), Some(1));
        assert_eq!(cache.get("b"), Some(2));
        assert_eq!(cache.weight(), 10);
    }

    #[test]
    fn empties_before_exceeding_capacity() {
        let cache = WeightedCache::new(10);
        cache.insert("a", 1, 4);
        cache.insert("b", 2, 6);
        cache.insert("c", 3, 1);

        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get("b"), None);
        assert_eq!(cache.get("c"), Some(3));
        assert_eq!(cache.weight(), 1);
    }

    #[test]
    fn replacing_a_value_replaces_its_weight() {
        let cache = WeightedCache::new(10);
        cache.insert("a", 1, 4);
        cache.insert("a", 2, 6);
        cache.insert("b", 3, 4);

        assert_eq!(cache.get("a"), Some(2));
        assert_eq!(cache.get("b"), Some(3));
        assert_eq!(cache.weight(), 10);
    }

    #[test]
    fn skips_a_value_heavier_than_capacity() {
        let cache = WeightedCache::new(10);
        cache.insert("a", 1, 4);
        cache.insert("b", 2, 11);

        assert_eq!(cache.get("a"), Some(1));
        assert_eq!(cache.get("b"), None);
        assert_eq!(cache.weight(), 4);
    }
}
