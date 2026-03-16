// Core library root for shared protocol types and logic.
// Expose only what other crates need.

pub mod wallet;
pub mod transaction;
pub mod block;
pub mod lottery;

pub use crate::wallet::Wallet;
pub use crate::transaction::Transaction;
pub use crate::block::Block;