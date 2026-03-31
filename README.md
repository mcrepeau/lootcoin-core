# lootcoin-core

Shared protocol types, cryptographic primitives, and constants used by all Lootcoin crates. Not meant to be run directly — it is a library dependency of `lootcoin-node`, `lootcoin-miner`, and `lootcoin-wallet`.

---

## Contents

### `block`

The fundamental unit of the chain. Fields: `index`, `previous_hash`, `timestamp`, `nonce`, `transactions`, `hash`.

- `calculate_hash()` serialises `(index, previous_hash, timestamp, nonce, tx_root)` with `bincode` and hashes the result with CubeHash-256. The `hash` field is excluded from the input so it can be set after mining. Using `tx_root` rather than the full transaction list means the mining loop only hashes the fixed-size header — transaction count does not affect mining performance.
- `meets_difficulty(hash, bits: f64)` returns `true` if the hash meets the given difficulty target. Accepts fractional bit values — see [Fractional difficulty](#fractional-difficulty) below.

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
| `MIN_TX_FEE` | 2 | Minimum fee for non-coinbase transactions; ensures 50/50 fee split gives ≥1 coin to each side |
| `SMALL_DIVISOR` | 400,000 | 36.25% tier — `pot / 400,000` |
| `MEDIUM_DIVISOR` | 30,000 | 1.67% tier — `pot / 30,000` (~hourly) |
| `LARGE_DIVISOR` | 2,000 | 0.07% tier — `pot / 2,000` (~daily) |
| `JACKPOT_DIVISOR` | 500 | 0.01% tier — `pot / 500` (~weekly) |
| `GUARANTEE_AFTER` | 120 blocks | Fee eligibility formula: `eligible_after = (GUARANTEE_AFTER / fee) - 1` |

---

## Cryptography

| Primitive | Algorithm | Used for |
|---|---|---|
| Block hashing | CubeHash-256 | PoW, chain linkage, address derivation |
| Transaction signing | Ed25519 (ed25519-dalek) | Authorising transfers |
| Key generation | OS CSPRNG (rand) | Wallet creation |

Ed25519 was chosen for compact 64-byte signatures, fast verification, and strong security margins. CubeHash-256 was chosen as a NIST SHA-3 finalist that is simple to implement and has no known length-extension vulnerabilities.

---

## Fractional difficulty

Most PoW chains store difficulty as an integer number of leading zero bits. Lootcoin aims for 60-second blocks and adjusts the difficulty accordingly.
The problem is that each bit represents a 2× change in expected work, so rounding to the nearest integer on every retarget window produces oscillation: if blocks are only slightly too fast, difficulty rounds up by one full bit (halving the block rate), then rounds back down next window, and so on indefinitely.

Lootcoin stores difficulty as a `f64` fractional bit count. A value like `26.47` is perfectly valid and processed by `meets_difficulty` using a numeric threshold on the first non-zero byte rather than a pure bit mask.

**What fractional bits mean conceptually:**

Difficulty `D` means the block hash, treated as a big-endian 256-bit integer, must be strictly less than `2^(256 − D)`.

| Difficulty | Leading zero bytes | Boundary byte must be less than | Values of that byte that pass |
|---|---|---|---|
| `26.0` | 3 | `2^6.0` = 64.00 | `0x00`–`0x3F` (64 values) |
| `26.5` | 3 | `2^5.5` ≈ 45.25 | `0x00`–`0x2C` (45 values) |
| `27.0` | 3 | `2^5.0` = 32.00 | `0x00`–`0x1F` (32 values) |

A fractional difficulty sits between two integers on a logarithmic scale: `26.5` is exactly halfway in expected hash count between `26` and `27`.

**Implementation:**

`meets_difficulty` walks the hash byte-by-byte using a floating-point threshold. When a byte exactly equals `floor(threshold)`, it descends into the fractional part for the next byte, giving a correct comparison without any 256-bit integer arithmetic:

```
threshold = 2^(8 − remainder)   // e.g. 2^5.5 ≈ 45.25 for D = 26.5

for each byte b (starting after the mandatory zero bytes):
    if b < floor(threshold)  → pass
    if b > floor(threshold)  → fail
    // b == floor(threshold): the answer depends on the bytes that follow
    threshold = frac(threshold) × 256
```

The miner is unaffected: it calls `meets_difficulty(hash, difficulty)` and gets a boolean back. The fractional threshold is invisible at the mining loop level.
