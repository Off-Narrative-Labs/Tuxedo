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
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};

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
    fn verify(&self, _: &[u8], block_height: u32, _: &[u8]) -> bool {
        block_height >= self.unlock_block_height
    }
}

/// Allows UTXOs to be spent when a preimage to a recorded hash is provided.
/// This could be used as a puzzle (although a partial preimage search would be better)
/// or a means of sharing a password, or as part of a simple atomic swapping protocol.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct BlakeTwoHashLock {
    pub hash_lock: H256,
}

impl BlakeTwoHashLock {
    pub fn new_from_secret<T: Encode>(secret: T) -> Self {
        Self {
            hash_lock: BlakeTwo256::hash(&secret.encode()),
        }
    }
}

impl Verifier for BlakeTwoHashLock {
    fn verify(&self, _: &[u8], _: u32, redeemer: &[u8]) -> bool {
        BlakeTwo256::hash(redeemer) == self.hash_lock
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn time_lock_too_soon() {
        let time_lock = TimeLock {
            unlock_block_height: 100,
        };
        assert!(!time_lock.verify(&[], 10, &[]));
    }

    #[test]
    fn time_lock_exactly_on_time() {
        let time_lock = TimeLock {
            unlock_block_height: 100,
        };
        assert!(time_lock.verify(&[], 100, &[]));
    }

    #[test]
    fn time_lock_past_threshold() {
        let time_lock = TimeLock {
            unlock_block_height: 100,
        };
        assert!(time_lock.verify(&[], 200, &[]));
    }

    #[test]
    fn hash_lock_correct_secret() {
        let secret = "htlc ftw";

        let hash_lock = BlakeTwoHashLock::new_from_secret(secret);
        assert!(hash_lock.verify(&[], 0, &secret.encode()));
    }

    #[test]
    fn hash_lock_wrong_secret() {
        let secret = "htlc ftw";
        let incorrect = "there is no second best";

        let hash_lock = BlakeTwoHashLock::new_from_secret(secret);
        assert!(!hash_lock.verify(&[], 0, &incorrect.encode()));
    }
}
