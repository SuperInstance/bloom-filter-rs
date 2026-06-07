//! Counting Bloom filter.
//!
//! A Bloom filter variant that uses counters instead of bits, enabling
//! deletion of items. Each bucket is a `u8` counter.

use crate::hash::hash_indices;
use crate::optimal::{optimal_k, optimal_m};

/// Counting Bloom filter with `u8` counters.
pub struct CountingBloomFilter {
    counters: Vec<u8>,
    num_buckets: usize,
    num_hashes: usize,
    count: usize,
}

impl CountingBloomFilter {
    /// Create a new counting Bloom filter.
    pub fn new(expected_items: usize, fp_rate: f64) -> Self {
        let num_buckets = optimal_m(expected_items, fp_rate);
        let num_hashes = optimal_k(num_buckets, expected_items);
        Self {
            counters: vec![0u8; num_buckets],
            num_buckets,
            num_hashes,
            count: 0,
        }
    }

    /// Insert an item (increments counters).
    pub fn insert(&mut self, item: &[u8]) {
        for idx in hash_indices(item, self.num_hashes, self.num_buckets) {
            self.counters[idx] = self.counters[idx].saturating_add(1);
        }
        self.count += 1;
    }

    /// Remove an item (decrements counters). Saturates to 0.
    pub fn remove(&mut self, item: &[u8]) {
        for idx in hash_indices(item, self.num_hashes, self.num_buckets) {
            self.counters[idx] = self.counters[idx].saturating_sub(1);
        }
        self.count = self.count.saturating_sub(1);
    }

    /// Check if an item is possibly in the set.
    pub fn contains(&self, item: &[u8]) -> bool {
        for idx in hash_indices(item, self.num_hashes, self.num_buckets) {
            if self.counters[idx] == 0 {
                return false;
            }
        }
        true
    }

    /// Number of items currently in the filter.
    pub fn count(&self) -> usize { self.count }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_contains() {
        let mut bf = CountingBloomFilter::new(100, 0.01);
        bf.insert(b"hello");
        assert!(bf.contains(b"hello"));
        assert!(!bf.contains(b"world"));
    }

    #[test]
    fn delete_works() {
        let mut bf = CountingBloomFilter::new(100, 0.01);
        bf.insert(b"hello");
        assert!(bf.contains(b"hello"));
        bf.remove(b"hello");
        assert!(!bf.contains(b"hello"));
    }

    #[test]
    fn delete_one_of_duplicates() {
        let mut bf = CountingBloomFilter::new(100, 0.01);
        bf.insert(b"hello");
        bf.insert(b"hello");
        bf.remove(b"hello");
        assert!(bf.contains(b"hello"));
        bf.remove(b"hello");
        assert!(!bf.contains(b"hello"));
    }

    #[test]
    fn no_false_negatives() {
        let mut bf = CountingBloomFilter::new(100, 0.01);
        bf.insert(b"alpha");
        bf.insert(b"beta");
        bf.insert(b"gamma");
        assert!(bf.contains(b"alpha"));
        assert!(bf.contains(b"beta"));
        assert!(bf.contains(b"gamma"));
    }

    #[test]
    fn count_after_operations() {
        let mut bf = CountingBloomFilter::new(100, 0.01);
        bf.insert(b"a");
        bf.insert(b"b");
        assert_eq!(bf.count(), 2);
        bf.remove(b"a");
        assert_eq!(bf.count(), 1);
        bf.remove(b"b");
        assert_eq!(bf.count(), 0);
    }

    #[test]
    fn false_positive_rate() {
        let n = 5000;
        let target_fpr = 0.05;
        let mut bf = CountingBloomFilter::new(n, target_fpr);
        for i in 0..n {
            bf.insert(format!("item_{}", i).as_bytes());
        }
        let test_count = 50_000;
        let mut fp = 0;
        for i in n..(n + test_count) {
            if bf.contains(format!("item_{}", i).as_bytes()) {
                fp += 1;
            }
        }
        let actual_fpr = fp as f64 / test_count as f64;
        assert!(actual_fpr < target_fpr * 5.0, "actual_fpr={actual_fpr}");
    }

    #[test]
    fn remove_doesnt_affect_others() {
        let mut bf = CountingBloomFilter::new(100, 0.01);
        bf.insert(b"a");
        bf.insert(b"b");
        bf.insert(b"c");
        bf.remove(b"b");
        assert!(bf.contains(b"a"));
        assert!(bf.contains(b"c"));
    }

    #[test]
    fn empty_after_all_removals() {
        let mut bf = CountingBloomFilter::new(100, 0.01);
        bf.insert(b"x");
        bf.insert(b"y");
        bf.remove(b"x");
        bf.remove(b"y");
        assert!(!bf.contains(b"x"));
        assert!(!bf.contains(b"y"));
        assert!(!bf.contains(b"z"));
    }

    #[test]
    fn saturating_counters() {
        let mut bf = CountingBloomFilter::new(10, 0.01);
        for _ in 0..300 {
            bf.insert(b"test");
        }
        assert!(bf.contains(b"test"));
    }

    #[test]
    fn remove_nonexistent_doesnt_panic() {
        let mut bf = CountingBloomFilter::new(100, 0.01);
        bf.remove(b"ghost");
    }
}
