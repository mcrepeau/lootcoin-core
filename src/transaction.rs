use crate::wallet::Wallet;
use cubehash::CubeHash256;
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use serde::{Serialize, Deserialize};

/// Domain separator included in every transaction's signed message.
/// Prevents cross-chain replay: a signature produced for lootcoin mainnet
/// is invalid on any chain that uses a different constant here, even if
/// the sender address and nonce match.
///
/// Changing this constant is a protocol-breaking change (all existing
/// signatures become invalid). It should only be changed for a deliberate
/// hard fork or a new network.
pub const CHAIN_ID: &[u8] = b"lootcoin-mainnet-1";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub fee: u64,
    /// Random nonce generated at signing time. Included in the signed message
    /// so that every transaction has a unique signature even when the other
    /// fields are identical. Zero for coinbase transactions.
    pub nonce: u64,
    pub public_key: [u8; 32],
    pub signature: Vec<u8>,
}

impl Transaction {
    pub fn new_signed(sender_wallet: &Wallet, receiver: String, amount: u64, fee: u64) -> Self {
        use rand::Rng;
        let nonce: u64 = rand::thread_rng().gen();
        let sender = sender_wallet.get_address();

        let mut temp_tx = Transaction {
            sender: sender.clone(),
            receiver,
            amount,
            fee,
            nonce,
            public_key: sender_wallet.get_public_key_bytes(),
            signature: vec![],
        };

        let tx_bytes = bincode::serialize(&(
            CHAIN_ID,
            &temp_tx.sender,
            &temp_tx.receiver,
            &temp_tx.amount,
            &temp_tx.fee,
            &temp_tx.nonce,
        ))
        .expect("bincode serialization of transaction fields is infallible");
        let signature = sender_wallet.sign(&tx_bytes);
        temp_tx.signature = signature.to_bytes().to_vec();
        temp_tx
    }

    pub fn verify(&self) -> bool {
        // Fix 1: public key must hash to sender address
        let hash: [u8; 32] = CubeHash256::digest(&self.public_key).into();
        let expected_addr = crate::wallet::encode_address(&hash);
        if expected_addr != self.sender {
            return false;
        }

        let pubkey = match VerifyingKey::from_bytes(&self.public_key) {
            Ok(pk) => pk,
            Err(_) => return false,
        };

        // Reconstruct the exact bytes that were signed: CHAIN_ID binds this
        // signature to lootcoin mainnet, preventing cross-chain replay.
        let Ok(tx_bytes) = bincode::serialize(&(
            CHAIN_ID,
            &self.sender,
            &self.receiver,
            &self.amount,
            &self.fee,
            &self.nonce,
        )) else {
            return false;
        };

        let sig_bytes: [u8; 64] = match self.signature.clone().try_into() {
            Ok(arr) => arr,
            Err(_) => return false,
        };
        let signature = Signature::from_bytes(&sig_bytes);

        pubkey.verify(&tx_bytes, &signature).is_ok()
    }

    /// Compute this transaction's identifier — a 32-byte CubeHash256 digest
    /// of all fields including the signature.
    ///
    /// For user transactions the txid is stable: it is determined at signing
    /// time and does not change when the transaction is included in a block.
    ///
    /// Coinbase transactions (empty sender, empty signature) that share the
    /// same receiver and reward amount will produce the same txid. Callers
    /// that need a globally unique identifier for coinbase entries should
    /// incorporate the block index at the point of indexing (e.g. by hashing
    /// `b"coinbase" || block_index_be`).
    pub fn txid(&self) -> [u8; 32] {
        let data = bincode::serialize(&(
            &self.sender,
            &self.receiver,
            &self.amount,
            &self.fee,
            &self.nonce,
            &self.public_key,
            &self.signature,
        ))
        .expect("bincode serialization of transaction fields is infallible");
        CubeHash256::digest(&data).into()
    }
}
