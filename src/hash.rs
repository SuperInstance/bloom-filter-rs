//! Hashing utilities for Bloom filters.
//!
//! Uses the FNV-1a hash as a base and generates `k` independent hash values
//! via the technique of Kirschner and Mitzenmacher:
//! ```text
//! hᵢ(x) = h₁(x) + i · h₂(x)
//! ```

/// FNV-1a hash of a byte slice (64-bit).
pub fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Generate `k` hash indices in the range `[0, m)` from a byte slice.
///
/// Uses two base hashes (FNV-1a with different seeds) and combines them.
pub fn hash_indices(data: &[u8], k: usize, m: usize) -> Vec<usize> {
    let h1 = fnv1a(data);
    // Second hash: FNV-1a with inverted bits
    let h2 = fnv1a(&[data, &[0xFF]].concat());
    (0..k).map(|i| {
        let h = h1.wrapping_add((i as u64).wrapping_mul(h2));
        (h % m as u64) as usize
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv_deterministic() {
        assert_eq!(fnv1a(b"hello"), fnv1a(b"hello"));
    }

    #[test]
    fn fnv_different_inputs() {
        assert_ne!(fnv1a(b"hello"), fnv1a(b"world"));
    }

    #[test]
    fn hash_indices_in_range() {
        let indices = hash_indices(b"test", 5, 1000);
        assert_eq!(indices.len(), 5);
        for &idx in &indices {
            assert!(idx < 1000, "idx={idx}");
        }
    }

    #[test]
    fn hash_indices_deterministic() {
        let a = hash_indices(b"test", 10, 1000);
        let b = hash_indices(b"test", 10, 1000);
        assert_eq!(a, b);
    }

    #[test]
    fn hash_indices_different_for_different_inputs() {
        let a = hash_indices(b"foo", 5, 1000);
        let b = hash_indices(b"bar", 5, 1000);
        assert_ne!(a, b);
    }
}
