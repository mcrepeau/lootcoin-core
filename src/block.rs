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
    pub fn calculate_hash(&self) -> Vec<u8> {
        let data = bincode::serialize(&(
            self.index,
            &self.previous_hash,
            self.timestamp,
            self.nonce,
            &self.transactions
            )).unwrap();

        let hash = CubeHash256::digest(&data);
        hash.to_vec()
    }
}
