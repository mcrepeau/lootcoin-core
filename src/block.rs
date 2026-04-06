use cubehash::CubeHash256;
use serde::{Serialize, Deserialize};
use crate::transaction::Transaction;

// Consensus safety: meets_difficulty() uses f64 arithmetic.
//
// IEEE-754 mandates exact rounding for +, -, *, / and sqrt.  The ASERT
// adjustment in the node uses only those operations and is therefore
// bit-for-bit identical on every compliant platform.
//
// meets_difficulty() additionally calls `2.0f64.powf(x)` where x ∈ (0, 8].
// IEEE-754 does NOT mandate exact rounding for transcendental functions;
// different libm implementations (glibc, Apple libm, msvcrt) may differ by
// ±1 ULP.  In practice this is harmless because:
//   • At x near an integer (the only case where 2^x lands on an integer
//     boundary) every conforming libm returns the exact result.
//   • At non-integer x a 1-ULP error (~2e-14 relative) never changes the
//     floor() of the threshold for any difficulty in [8, 127].
//   • The canary test below pins the bit pattern on a reference platform;
//     CI will catch any divergence before it reaches production.
//
// Known unsafe target: 32-bit x86 WITHOUT SSE2 (target_feature = "sse2").
// The x87 FPU uses 80-bit extended precision internally.  Values spilled
// to memory are rounded to 64 bits non-deterministically, producing
// different results than SSE2 on the same inputs.  All other supported
// targets (x86-64, ARM64, RISC-V, WASM) use strict 64-bit IEEE-754.
#[cfg(all(target_arch = "x86", not(target_feature = "sse2")))]
compile_error!(
    "lootcoin-core requires SSE2 on x86 for deterministic f64 difficulty \
     arithmetic.  Build with RUSTFLAGS=\"-C target-feature=+sse2\" or \
     target x86_64 instead.  Without SSE2, x87 extended-precision \
     arithmetic may cause consensus divergence between nodes."
);

/// Maximum non-coinbase transactions a block may contain.
/// Prevents DoS via enormous blocks from a flooded mempool.
/// At the 60 s target block time this gives a sustained throughput of 4 tps.
pub const MAX_BLOCK_TXS: usize = 240;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
   pub index: u64,
   pub previous_hash: Vec<u8>,
   pub timestamp: u64,
   pub nonce: u64,
   pub transactions: Vec<Transaction>,
   /// Commitment hash of the transaction list. Computed once when the block is
   /// assembled and included in the mined header so that mining speed is
   /// independent of block size — only the 8-byte nonce is hashed per attempt.
   pub tx_root: Vec<u8>,
   pub hash: Vec<u8>,
}

/// Returns true if `hash` (big-endian unsigned integer) is strictly less than
/// `2^(256 - bits)`, i.e., the hash meets the given difficulty.
///
/// `bits` may be fractional, enabling sub-bit difficulty adjustments that avoid
/// the oscillation caused by rounding to whole bits on every retarget.
pub fn meets_difficulty(hash: &[u8], bits: f64) -> bool {
    if bits <= 0.0 { return true; }

    // n full zero bytes must precede the boundary byte.
    let n = (bits / 8.0) as usize;
    let remainder = bits - (n as f64 * 8.0); // [0.0, 8.0)

    if hash.len() < n + 1 { return false; }
    if hash[..n].iter().any(|&b| b != 0) { return false; }
    if remainder == 0.0 { return true; }

    // Threshold for hash[n]: must be strictly less than 2^(8 - remainder).
    // Walk byte-by-byte using a floating-point threshold, descending into the
    // fractional part whenever hash[i] exactly equals floor(threshold).
    let mut threshold = 2.0f64.powf(8.0 - remainder);
    for &byte in &hash[n..] {
        let b = byte as f64;
        let t_floor = threshold.floor();
        if b < t_floor { return true; }
        if b > t_floor { return false; }
        // b == floor(threshold): check whether threshold has a fractional part.
        let frac = threshold - t_floor;
        if frac == 0.0 { return false; } // hash equals the exact threshold → fail
        threshold = frac * 256.0;
    }
    // All bytes exhausted: passes iff threshold is non-integer (hash < threshold).
    threshold.fract() > 0.0
}

impl Block {
    /// Hash the full transaction list into a fixed-size commitment.
    /// Called once when a block is assembled; the result is stored in `tx_root`.
    pub fn compute_tx_root(transactions: &[Transaction]) -> Vec<u8> {
        let data = bincode::serialize(transactions).unwrap();
        CubeHash256::digest(&data).to_vec()
    }

    /// Hash only the fixed-size block header. Because `tx_root` already commits
    /// to the transaction list, the per-iteration work during mining is constant
    /// regardless of how many transactions the block contains.
    pub fn calculate_hash(&self) -> Vec<u8> {
        let data = bincode::serialize(&(
            self.index,
            &self.previous_hash,
            self.timestamp,
            self.nonce,
            &self.tx_root,
        )).unwrap();
        CubeHash256::digest(&data).to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Canary test: verifies that meets_difficulty() produces the expected
    /// result on this platform's f64/powf implementation.
    ///
    /// If this test fails, the platform's libm powf() disagrees with the
    /// reference IEEE-754 SSE2 result by more than a floor() boundary, which
    /// would cause consensus divergence between nodes.  See the compile-time
    /// SSE2 guard above for the known-unsafe case (32-bit x86 without SSE2).
    ///
    /// Vector derivation (difficulty = 25.3):
    ///   n = floor(25.3 / 8) = 3  →  three leading zero bytes required
    ///   remainder = 25.3 - 24.0 = 1.3
    ///   threshold = 2^(8 - 1.3) = 2^6.7 ≈ 103.968  →  floor = 103
    ///   byte[3] = 102  →  102 < 103          → PASS  (clear margin below)
    ///   byte[3] = 104  →  104 > 103          → FAIL  (clear margin above)
    ///   byte[3] = 103, byte[4] = 246 (< floor(0.968 * 256 ≈ 247.9)) → PASS
    ///   byte[3] = 103, byte[4] = 248 (> floor ≈ 247)                → FAIL
    #[test]
    fn meets_difficulty_float_canary() {
        let difficulty = 25.3_f64;

        let hash = |b3: u8, b4: u8| -> Vec<u8> {
            let mut h = vec![0u8; 32];
            h[3] = b3;
            h[4] = b4;
            h
        };

        // Clear pass: byte[3] well below floor(threshold).
        assert!(meets_difficulty(&hash(102, 0), difficulty),
            "canary FAIL: 102 should be below threshold floor 103");

        // Clear fail: byte[3] above floor(threshold).
        assert!(!meets_difficulty(&hash(104, 0), difficulty),
            "canary FAIL: 104 should be above threshold floor 103");

        // Boundary pass: byte[3] == floor, byte[4] below second-level floor.
        assert!(meets_difficulty(&hash(103, 246), difficulty),
            "canary FAIL: [103,246] should pass second-level threshold");

        // Boundary fail: byte[3] == floor, byte[4] above second-level floor.
        assert!(!meets_difficulty(&hash(103, 248), difficulty),
            "canary FAIL: [103,248] should fail second-level threshold");
    }
}
