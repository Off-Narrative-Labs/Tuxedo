//! General-purpose runtime traits for describing common types of on-chain logic.
//! Tuxedo piece implementations may loosely couple through these traits.

/// A trait for UTXOs that can act like coins, or bank notes.
pub trait Cash {
    /// Get the value of this token.
    fn value(&self) -> u128;

    /// A 1-byte unique identifier for this coin.
    /// Might need more than 1 byte eventually...
    const ID: u8;
}
