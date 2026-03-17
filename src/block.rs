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

/// Returns true if the first `bits` bits of `hash` are all zero.
pub fn meets_difficulty(hash: &[u8], bits: u32) -> bool {
    let full_bytes = (bits / 8) as usize;
    let extra_bits = bits % 8;
    let min_len = full_bytes + if extra_bits > 0 { 1 } else { 0 };
    hash.len() >= min_len
        && hash[..full_bytes].iter().all(|&b| b == 0)
        && (extra_bits == 0 || hash[full_bytes] >> (8 - extra_bits) == 0)
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
