use ed25519_dalek::{SigningKey, Signature, Signer};
use rand::rngs::OsRng;
use cubehash::CubeHash256;
use bech32::{ToBase32, FromBase32, Variant};

/// The human-readable part for all lootcoin addresses.
pub const ADDRESS_HRP: &str = "loot";

/// Encode a 32-byte CubeHash digest as a bech32m address (e.g. `loot1q…`).
pub fn encode_address(hash: &[u8; 32]) -> String {
    bech32::encode(ADDRESS_HRP, hash.to_base32(), Variant::Bech32m)
        .expect("valid bech32m encoding")
}

/// Decode a bech32m address back to its 32-byte hash, or `None` if the
/// string is not a well-formed lootcoin address.
pub fn decode_address(addr: &str) -> Option<[u8; 32]> {
    let (hrp, data, variant) = bech32::decode(addr).ok()?;
    if hrp != ADDRESS_HRP || variant != Variant::Bech32m {
        return None;
    }
    let bytes = Vec::<u8>::from_base32(&data).ok()?;
    bytes.try_into().ok()
}

pub struct Wallet {
    pub keypair: SigningKey,
}

impl Wallet {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let keypair = SigningKey::generate(&mut csprng);
        Self { keypair }
    }

    pub fn from_secret_key_bytes(bytes: [u8; 32]) -> Self {
        Self { keypair: SigningKey::from_bytes(&bytes) }
    }

    pub fn secret_key_bytes(&self) -> [u8; 32] {
        self.keypair.to_bytes()
    }

    pub fn get_address(&self) -> String {
        let vk = self.keypair.verifying_key();
        let hash: [u8; 32] = CubeHash256::digest(vk.as_bytes()).into();
        encode_address(&hash)
    }

    pub fn sign(&self, data: &[u8]) -> Signature {
        self.keypair.sign(data)
    }

    pub fn get_public_key_bytes(&self) -> [u8; 32] {
        *self.keypair.verifying_key().as_bytes()
    }
}
