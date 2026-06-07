//! Optimal sizing utilities for Bloom filters.
//!
//! Given an expected number of items `n` and a desired false positive rate `p`:
//! ```text
//! m = -n · ln(p) / (ln 2)²    (optimal number of bits)
//! k = (m/n) · ln 2             (optimal number of hash functions)
//! ```

/// Compute the optimal number of bits `m`.
pub fn optimal_m(n: usize, p: f64) -> usize {
    let m = -(n as f64) * p.ln() / (std::f64::consts::LN_2.powi(2));
    m.ceil() as usize
}

/// Compute the optimal number of hash functions `k`.
pub fn optimal_k(m: usize, n: usize) -> usize {
    let k = (m as f64 / n as f64) * std::f64::consts::LN_2;
    k.round().max(1.0) as usize
}

/// Estimate the false positive rate for given `m`, `n`, `k`.
pub fn estimate_fpr(m: usize, n: usize, k: usize) -> f64 {
    (1.0 - (-(k as f64 * n as f64 / m as f64)).exp()).powi(k as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimal_m_reasonable() {
        let m = optimal_m(1000, 0.01);
        assert!(m > 5000 && m < 20000, "m={m}");
    }

    #[test]
    fn optimal_k_reasonable() {
        let m = optimal_m(1000, 0.01);
        let k = optimal_k(m, 1000);
        assert!(k >= 3 && k <= 15, "k={k}");
    }

    #[test]
    fn fpr_decreases_with_m() {
        let f1 = estimate_fpr(1000, 100, 3);
        let f2 = estimate_fpr(10000, 100, 3);
        assert!(f2 < f1, "f2={f2}, f1={f1}");
    }

    #[test]
    fn fpr_increases_with_n() {
        let f1 = estimate_fpr(10000, 100, 5);
        let f2 = estimate_fpr(10000, 1000, 5);
        assert!(f2 > f1, "f2={f2}, f1={f1}");
    }

    #[test]
    fn tighter_p_more_bits() {
        let m1 = optimal_m(1000, 0.1);
        let m2 = optimal_m(1000, 0.001);
        assert!(m2 > m1, "m2={m2}, m1={m1}");
    }

    #[test]
    fn round_trip_fpr() {
        let n = 1000;
        let p = 0.01;
        let m = optimal_m(n, p);
        let k = optimal_k(m, n);
        let actual_p = estimate_fpr(m, n, k);
        assert!(actual_p <= p * 2.0, "actual_p={actual_p}, target={p}");
    }
}
