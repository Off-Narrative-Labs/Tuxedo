//! This file represents a simple Proof of Existence application, identical in behavior
//! to the tutorial https://docs.substrate.io/tutorials/work-with-pallets/use-macros-in-a-custom-pallet/
//! Of course, this implementation is based on UTXOs and works with Tuxedo rather than FRAME.
//!
//! The application allows users to claim the existence of a preimage for a particular hash with a
//! transaction. Thus, the blockchain network acts as a decentralized notary service. Claims are
//! stored in the state, and can be "revoked" from the state later, although the redeemer to the original
//! claim will always remain in the history of the blockchain.
//!
//! The main design deviation from the FRAME PoE pallet is the means by which redundant claims are settled.
//! In FRAME, the exact storage location of each claim is known globally, whereas in the UTXO model, all state
//! is local. This means that when a new claim is registered, it is not possible to efficiently check that the
//! same claim has not already been registered. Instead there is a constraint checker
//! to boot subsequent redundant claims when they are discovered. This difference is analogous to
//! the difference between recorded and registered land
//! https://cannerlaw.com/blog/the-difference-of-recorded-and-registered-land/

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::fmt::Debug;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure,
    support_macros::{CloneNoBound, DebugNoBound, DefaultNoBound},
    SimpleConstraintChecker,
};

#[cfg(test)]
mod tests;

// Notice this type doesn't have to be public. Cool.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
struct ClaimData {
    /// The hash of the data whose existence is being proven.
    claim: H256,
    /// The time (in block height) at which the claim becomes valid.
    effective_height: u32,
}

impl UtxoData for ClaimData {
    const TYPE_ID: [u8; 4] = *b"poe_";
}

/// Errors that can occur when checking PoE Transactions
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum ConstraintCheckerError {
    // Ughhh again with these common errors.
    /// Wrong number of inputs were provided to the constraint checker.
    WrongNumberInputs,
    /// Wrong number of outputs were provided to the constraint checker.
    WrongNumberOutputs,
    /// An input data has the wrong type.
    BadlyTypedInput,
    /// An output data has the wrong type.
    BadlyTypedOutput,

    // Now we get on to the actual amoeba-specific errors
    /// The effective height of this claim is in the past,
    /// So the claim cannot be created.
    EffectiveHeightInPast,

    /// Claims under dispute do not have the same hash, but they must.
    DisputingMismatchedClaims,
    /// The winner of a dispute must be the oldest claim (the lowest block number)
    IncorrectDisputeWinner,
}

/// Configuration items for the Proof of Existence piece when it is
/// instantiated in a concrete runtime.
pub trait PoeConfig {
    /// A means of getting the current block height.
    /// Probably this will be the Tuxedo Executive
    fn block_height() -> u32;
}

/// A constraint checker to create claims.
///
/// This constraint checker allows the creation of many claims in a single operation
/// It also allows the creation of zero claims, although such a transaction is useless and is simply a
/// waste of caller fees.
#[derive(
    Serialize,
    Deserialize,
    Encode,
    Decode,
    DebugNoBound,
    DefaultNoBound,
    CloneNoBound,
    PartialEq,
    Eq,
    TypeInfo,
)]
pub struct PoeClaim<T>(PhantomData<T>);

impl<T: PoeConfig> SimpleConstraintChecker for PoeClaim<T> {
    type Error = ConstraintCheckerError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        evicted_input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there are no inputs or evictions
        ensure!(
            input_data.is_empty(),
            ConstraintCheckerError::WrongNumberInputs
        );
        ensure!(
            evicted_input_data.is_empty(),
            ConstraintCheckerError::WrongNumberInputs
        );

        // For each output, make sure the claimed block height is >= the current block height.
        // If we required exact equality, this would mean that transactors needed to get their transactions
        // in exactly the next block which is challenging in times of network congestion. Relaxing the
        // requirement allows the caller to make a somewhat weaker claim with the advantage that they have a longer
        // period of time during which their transaction is valid.
        for untyped_output in output_data {
            let output = untyped_output
                .extract::<ClaimData>()
                .map_err(|_| ConstraintCheckerError::BadlyTypedOutput)?;
            ensure!(
                //TODO we're grabbing the block height function directly from
                // the runtime level. This needs to be made available through some
                // kind of config.
                output.effective_height >= T::block_height(),
                ConstraintCheckerError::EffectiveHeightInPast
            );
        }

        Ok(0)
    }
}

/// A constraint checker to revoke claims.
///
/// Like the creation constraint checker, this allows batch revocation.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct PoeRevoke;

impl SimpleConstraintChecker for PoeRevoke {
    type Error = ConstraintCheckerError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        evicted_input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Can't evict anything
        ensure!(
            evicted_input_data.is_empty(),
            ConstraintCheckerError::WrongNumberInputs
        );

        // Make sure there are no outputs
        ensure!(
            output_data.is_empty(),
            ConstraintCheckerError::WrongNumberOutputs
        );

        // Make sure the inputs are properly typed. We don't need to check anything else about them.
        for untyped_input in input_data {
            let _ = untyped_input
                .extract::<ClaimData>()
                .map_err(|_| ConstraintCheckerError::BadlyTypedInput)?;
        }

        Ok(0)
    }
}

/// A constraint checker that resolves claim disputes by keeping whichever claim came first.
///
/// Any user may submit a transaction reporting conflicting claims, and the oldest one will be kept.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct PoeDispute;

impl SimpleConstraintChecker for PoeDispute {
    type Error = ConstraintCheckerError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        evicted_input_data: &[DynamicallyTypedData],
        peek_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there are no normal inputs or outputs
        ensure!(
            input_data.is_empty(),
            ConstraintCheckerError::WrongNumberInputs
        );
        ensure!(
            output_data.is_empty(),
            ConstraintCheckerError::WrongNumberOutputs
        );

        // Make sure there is exactly one peek (the oldest, winning claim)
        let winner = peek_data
            .first()
            .ok_or(ConstraintCheckerError::WrongNumberInputs)?
            .extract::<ClaimData>()
            .map_err(|_| ConstraintCheckerError::BadlyTypedInput)?;

        // Make sure that all evicted inputs, claim the same hash as the winner
        // and have block heights strictly greater than the winner.
        for untyped_loser in evicted_input_data {
            let loser = untyped_loser
                .extract::<ClaimData>()
                .map_err(|_| ConstraintCheckerError::BadlyTypedInput)?;
            ensure!(
                winner.claim == loser.claim,
                Self::Error::DisputingMismatchedClaims
            );
            ensure!(
                winner.effective_height < loser.effective_height,
                Self::Error::IncorrectDisputeWinner
            );
        }

        Ok(0)
    }
}
