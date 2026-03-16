use ed25519_dalek::{SigningKey, Signature, Signer};
use rand::rngs::OsRng;
use cubehash::CubeHash256;

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
        let pubkey_bytes = vk.as_bytes();
        let hash = CubeHash256::digest(pubkey_bytes);
        hex::encode(hash)
    }

    pub fn sign(&self, data: &[u8]) -> Signature {
        self.keypair.sign(data)
    }

    pub fn get_public_key_bytes(&self) -> [u8; 32] {
        *self.keypair.verifying_key().as_bytes()
    }
}
