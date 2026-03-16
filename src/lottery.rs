/// Number of blocks a ticket must wait before the reveal window opens.
pub const TICKET_MATURITY: u64 = 100;

/// Number of consecutive block hashes accumulated as lottery randomness.
/// An attacker needs to control all REVEAL_BLOCKS blocks to manipulate the
/// outcome; at 30% hashrate the probability of doing so is 0.3^10 ≈ 0.000006.
pub const REVEAL_BLOCKS: u64 = 10;

/// Number of equally-likely outcome buckets per lottery draw.
pub const PPM: u32 = 1_000_000;

/// Pot payout divisors per tier.
///
/// Rewards are a fraction of the current pot rather than fixed amounts.
/// This prevents the pot from ever fully draining (asymptotic decay) and
/// naturally dampens whale creation: each successive winner gets less as
/// the pot shrinks, while fees continuously replenish it.
///
///   Tier    │ Divisor  │  % of pot  │ Initial value (99.9M pot)
///   ────────┼──────────┼────────────┼──────────────────────────
///   SMALL   │ 100,000  │  0.001 %   │        ~999
///   MEDIUM  │  10,000  │  0.01  %   │       ~9,990
///   LARGE   │   1,000  │  0.1   %   │      ~99,900
///   JACKPOT │     200  │  0.5   %   │     ~499,500
pub const SMALL_DIVISOR:   u64 = 100_000;
pub const MEDIUM_DIVISOR:  u64 =  10_000;
pub const LARGE_DIVISOR:   u64 =   1_000;
pub const JACKPOT_DIVISOR: u64 =     200;

/// Fee-based inclusion delay constant.
///
/// `eligible_after(fee) = (GUARANTEE_AFTER / fee).saturating_sub(1)` blocks.
///
/// Examples (1 block ≈ 60 s):
///   fee = 1   → 119 blocks (~2 h)
///   fee = 12  →   9 blocks (~9 min)
///   fee ≥ 120 →   0 blocks (next block)
///
/// Any transaction with fee > 0 is guaranteed inclusion within GUARANTEE_AFTER
/// blocks regardless of congestion. Transactions with fee = 0 are never included.
pub const GUARANTEE_AFTER: u64 = 120;
