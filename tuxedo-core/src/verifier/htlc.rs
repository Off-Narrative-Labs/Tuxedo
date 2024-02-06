//! This module contains `Verifier` implementations related to Hash Time Lock Contracts.
//! It contains a simple hash lock, a simple time lock, and a hash time lock.
//! 
//! These could be used as the base of an atomic swap protocol with a similarly expressive
//! utxo chain like Bitcoin. For atomic swaps with less expressive counter party chains,
//! such as Monero, see the Farcaster protocol.

use super::Verifier;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// Allows UTXOs to be spent after a certain block height has been reached.
/// This is useful for locking up tokens as a future investment. Timelocking
/// also form the basis of timeout paths in swapping protocols.
///
/// This verifier is unlike many others because it requires some environmental information,
/// namely the current block number. So there is a decision to be made:
/// * Allow the verifier to take come config and grab that info by calling a function given in the config.
///   This is what we do with constraint checker.
/// * Modify the `Verifier` trait to pass along the block number.
///
/// On the one hand the block number seems like a pretty fundamental and basic thing to add. On the other
/// hand there could be many more things to pass. For example, the timestamp.
/// However any more complex information would require coupling with Constraint Checkers and it is not
/// easy to red state like in accounts.
///
/// Decision: I will add block number to the signature. And remain open to adding more blockchain-level
/// fundamental things. Especially if they are available in bitcoin script.
///
/// Regarding the verifier constraint checker separation, perhaps the right line to be drawn is
/// that verifiers are useful in a lot of places, but perhaps not expressive enough in others.
/// When they are not expressive enough, just use `UpForGrabs` and rely on the constraint checker,
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct TimeLock {
    pub unlock_block_height: u32,
}

impl Verifier for TimeLock {
    fn verify(&self, simplified_tx: &[u8], block_height: u32, redeemer: &[u8]) -> bool {
        block_height >= self.unlock_block_height
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Time lock too soon
    // Timelock exactly equal to threshold
    // Timelock past threshold

    // Hashlock wrong secret
    // Hashlock right secret

}