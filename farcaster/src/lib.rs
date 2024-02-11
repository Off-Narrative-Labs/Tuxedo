//! The [Farcaster project](https://github.com/farcaster-project) s a cross-chain atomic swap protocol
//! that allows swaps with Monero despite its lack of any on chain logic.
//!
//! This crate contains several Tuxedo `Verifier`s that implement the same logic as Farcaster's
//! original Bitcoin scripts the allowing any Tuxedo chain to act on the arbitrating side of a
//! Farcaster protocol swap.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::sr25519::{Public, Signature};
use tuxedo_core::Verifier;

/// Allows coins on the arbitrating Tuxedo chain to be bought or moved into in intermediate
/// utxo that represents the cancellation phase.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct SwapLock {
    recipient: Public,
    canceller: Public,
    cancelation_timeout: u32,
}

impl Verifier for SwapLock {
    type Redeemer = UnlockSwap;

    fn verify(&self, simplified_tx: &[u8], block_height: u32, redeemer: &Self::Redeemer) -> bool {
        todo!()
    }
}

/// Satisfies a `SwapLock` verifier.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum UnlockSwap {
    ///
    Buy,
    ///
    Cancel,
}

/// Allows coins in the intermediate cancellation state to be refunded to the original owner
/// or for an alternate punish path. TODO better docs after I fully understand the punish path
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct SwapCancellation {}

impl Verifier for SwapCancellation {
    type Redeemer = CompleteCancellation;

    fn verify(&self, simplified_tx: &[u8], block_height: u32, redeemer: &Self::Redeemer) -> bool {
        todo!()
    }
}

///
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum CompleteCancellation {
    ///
    Refund,
    ///
    Punish,
}
