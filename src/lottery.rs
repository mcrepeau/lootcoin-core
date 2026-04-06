/// Number of consecutive block hashes accumulated as lottery randomness,
/// starting from the block that issued the ticket.
///
/// These blocks serve as both the maturity delay and the entropy source:
/// settlement fires at `created_height + REVEAL_BLOCKS` using blocks
/// `[created_height, created_height + REVEAL_BLOCKS)` as randomness.
///
/// An attacker needs to control all REVEAL_BLOCKS consecutive blocks to
/// steer the outcome; at 30% hashrate the probability is 0.3^100 ≈ 10^-52.
///
/// Must not exceed REORG_WINDOW in lootcoin-node (enforced by a compile-time
/// assertion there) — the full reveal window must fit in the in-memory block
/// cache when settlement fires.
pub const REVEAL_BLOCKS: u64 = 100;

/// Number of equally-likely outcome buckets per lottery draw.
pub const PPM: u32 = 1_000_000;

/// Pot payout divisors per tier.
///
/// Payout formula: `pot / DIVISOR`
///
/// Every winning ticket receives the same flat fraction of the pot regardless
/// of how many transactions were in the block. Per-transaction miner incentives
/// are provided by the 50/50 fee split instead.
///
///   Tier    │ Probability │  Divisor  │ Prize at pot=99M  │ Expected frequency
///   ────────┼─────────────┼───────────┼───────────────────┼───────────────────
///   SMALL   │    36.25%   │ 400,000   │        ~247 coins │ every ~3 blocks
///   MEDIUM  │     1.67%   │  30,000   │      ~3,300 coins │ every ~60 blocks (~1 h)
///   LARGE   │     0.07%   │   2,000   │     ~49,500 coins │ every ~1,440 blocks (~1 day)
///   JACKPOT │     0.01%   │     500   │    ~198,000 coins │ every ~10,080 blocks (~1 week)
///
/// No-win probability: 62.00% (bucket 0..=619,999 out of PPM=1,000,000).
///
/// Total payouts decrease across tiers (small dominates aggregate payout)
/// matching real-lottery conventions while keeping jackpots exciting.
///
/// Expected value per ticket decreases across tiers (p/D): each higher tier
/// is a strictly worse bet, with the excitement premium compensating for EV.
///
/// Rewards are a fraction of the current pot rather than fixed amounts.
/// This prevents the pot from ever fully draining (asymptotic decay) and
/// naturally dampens whale creation: each successive winner gets less as
/// the pot shrinks, while fees continuously replenish it.
pub const SMALL_DIVISOR:   u64 = 400_000;
pub const MEDIUM_DIVISOR:  u64 =  30_000;
pub const LARGE_DIVISOR:   u64 =   2_000;
pub const JACKPOT_DIVISOR: u64 =     500;

/// Lottery outcome bucket boundaries (out of PPM = 1,000,000).
///
/// Buckets [0, SMALL_BUCKET_START)  → no-win  (62.00%)
/// Buckets [SMALL_BUCKET_START,  MEDIUM_BUCKET_START)  → small   (36.25%)
/// Buckets [MEDIUM_BUCKET_START, LARGE_BUCKET_START)   → medium  ( 1.67%)
/// Buckets [LARGE_BUCKET_START,  JACKPOT_BUCKET_START) → large   ( 0.07%)
/// Buckets [JACKPOT_BUCKET_START, PPM)                 → jackpot ( 0.01%)
pub const SMALL_BUCKET_START:   u32 = 620_000;
pub const MEDIUM_BUCKET_START:  u32 = 982_500;
pub const LARGE_BUCKET_START:   u32 = 999_200;
pub const JACKPOT_BUCKET_START: u32 = 999_900;

/// Minimum fee required for a non-coinbase transaction.
///
/// A fee of at least 2 ensures the 50/50 fee split always gives at least 1 coin
/// to the miner AND 1 coin to the pot. A fee of 1 would be rounded down to 0 for
/// one side under integer division, effectively removing the lottery pot incentive
/// or the miner incentive. Transactions with fee < MIN_TX_FEE are rejected.
pub const MIN_TX_FEE: u64 = 2;

/// Fee-based inclusion delay constant.
///
/// `eligible_after(fee) = (GUARANTEE_AFTER / fee).saturating_sub(1)` blocks.
///
/// Examples (1 block ≈ 60 s):
///   fee = 2   →  59 blocks (~1 h)   ← minimum fee
///   fee = 12  →   9 blocks (~9 min)
///   fee ≥ 120 →   0 blocks (next block)
///
/// Any transaction with fee ≥ MIN_TX_FEE is guaranteed inclusion within
/// GUARANTEE_AFTER blocks regardless of congestion.
pub const GUARANTEE_AFTER: u64 = 120;
