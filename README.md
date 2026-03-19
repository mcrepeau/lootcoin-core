# lootcoin-core

Shared protocol types, cryptographic primitives, and constants used by all Lootcoin crates. Not meant to be run directly — it is a library dependency of `lootcoin-node`, `lootcoin-miner`, and `lootcoin-wallet`.

---

## Contents

### `block`

The fundamental unit of the chain. Fields: `index`, `previous_hash`, `timestamp`, `nonce`, `transactions`, `hash`.

- `calculate_hash()` serialises `(index, previous_hash, timestamp, nonce, tx_root)` with `bincode` and hashes the result with CubeHash-256. The `hash` field is excluded from the input so it can be set after mining. Using `tx_root` rather than the full transaction list means the mining loop only hashes the fixed-size header — transaction count does not affect mining performance.
- `meets_difficulty(hash, bits)` returns `true` if the first `bits` bits of `hash` are all zero. Used by both the node (block validation) and the miner (PoW loop).

### `transaction`

Represents a transfer of coins between two addresses. Fields: `sender`, `receiver`, `amount`, `fee`, `nonce`, `public_key`, `signature`.

- Coinbase transactions have an empty `sender` and carry no signature.
- Non-coinbase transactions are signed with Ed25519 over `bincode(sender, receiver, amount, fee, nonce)`.
- `new_signed(wallet, receiver, amount, fee)` constructs and signs a transaction, generating a random 53-bit nonce for uniqueness (53 bits keeps the value within JavaScript's safe integer range).
- `verify()` checks that `public_key` hashes to `sender` and that the Ed25519 signature is valid.

### `wallet`

Ed25519 keypair wrapper.

- `Wallet::new()` generates a fresh keypair using the OS CSPRNG.
- `Wallet::from_secret_key_bytes(bytes)` restores a wallet from a 32-byte seed.
- `secret_key_bytes()` exports the 32-byte seed.
- `get_address()` returns the 64-char hex-encoded CubeHash-256 digest of the public key.
- `get_public_key_bytes()` returns the raw 32-byte Ed25519 public key.
- `sign(data)` signs arbitrary bytes with the private key.

### `lottery`

Protocol constants shared across all crates. Any tool that needs to reason about lottery timing or payouts should import from here rather than hardcoding values.

| Constant | Value | Meaning |
|---|---|---|
| `TICKET_MATURITY` | 100 blocks | Blocks before a mined ticket enters the reveal window |
| `REVEAL_BLOCKS` | 10 blocks | Block hashes accumulated as lottery entropy |
| `PPM` | 1,000,000 | Outcome buckets per draw |
| `SMALL_DIVISOR` | 100,000 | 98.00% tier — pot / 100,000 |
| `MEDIUM_DIVISOR` | 10,000 | 1.90% tier — pot / 10,000 |
| `LARGE_DIVISOR` | 1,000 | 0.09% tier — pot / 1,000 |
| `JACKPOT_DIVISOR` | 200 | 0.01% tier — pot / 200 |
| `GUARANTEE_AFTER` | 120 blocks | Fee eligibility formula: `eligible_after = (GUARANTEE_AFTER / fee) - 1` |

---

## Cryptography

| Primitive | Algorithm | Used for |
|---|---|---|
| Block hashing | CubeHash-256 | PoW, chain linkage, address derivation |
| Transaction signing | Ed25519 (ed25519-dalek) | Authorising transfers |
| Key generation | OS CSPRNG (rand) | Wallet creation |

Ed25519 was chosen for compact 64-byte signatures, fast verification, and strong security margins. CubeHash-256 was chosen as a NIST SHA-3 finalist that is simple to implement and has no known length-extension vulnerabilities.
