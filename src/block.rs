use cubehash::CubeHash256;
use serde::{Serialize, Deserialize};
use crate::transaction::Transaction;

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
