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

use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::transaction_validity::TransactionPriority;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, SimpleConstraintChecker,
};

// Notice this type doesn't have to be public. Cool.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
struct ClaimData {
    /// The hash of the data whose existence is being proven.
    claim: H256,
    /// The time (in block height) at which the claim becomes valid.
    effective_height: u32, //TODO get the generic block height type
}

impl UtxoData for ClaimData {
    const TYPE_ID: [u8; 4] = *b"poe_";
}

/// Errors that can occur when checking PoE Transactions
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
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

    /// Disputes should not have any normal inputs.
    DisputeWithInput,
    /// Disputes should not have any normal outputs; they only clean up storage; not create it.
    DisputeWithOutput,
    /// Disputes need to have exactly one peek which is the winner of the dispute.
    /// This dispute did not have that peek.
    DisputeWithNoPeek,
    /// Disputes need to have exactly one peek which is the winner of the dispute.
    /// This dispute had multiple such peeks
    DisputeWithMultiplePeeks,
    /// Disputes should have at least one eviction which are the losers of the dispute.
    /// This dispute did not have that eviction.
    DisputeWithNoEvictions,
    /// This dispute tries to evict claims that are not for the same hash as the winner.
    DisputedClaimsNotForSameHash,
    /// This dispute does not have the oldest claim as the winner and is, therefore, invalid.
    DisputeSettledIncorrectly,
}

/// A constraint checker to create claims.
///
/// This constraint checker allows the creation of many claims in a single operation
/// It also allows the creation of zero claims, although such a transaction is useless and is simply a
/// waste of caller fees.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PoeClaim;

impl SimpleConstraintChecker for PoeClaim {
    type Error = ConstraintCheckerError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        _evictions: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there are no inputs
        ensure!(
            input_data.is_empty(),
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
                output.effective_height >= crate::Executive::block_height(),
                ConstraintCheckerError::EffectiveHeightInPast
            );
        }

        Ok(0)
    }
}

/// A constraint checker to revoke claims.
///
/// Like the creation constraint checker, this allows batch revocation.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PoeRevoke;

impl SimpleConstraintChecker for PoeRevoke {
    type Error = ConstraintCheckerError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        _evictions: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there are no outputs
        ensure!(
            output_data.is_empty(),
            ConstraintCheckerError::WrongNumberOutputs
        );

        // Make sure the inputs are properly typed. We don't need to check anything else about them.
        for untyped_input in input_data {
            let _ = untyped_input
                .extract::<ClaimData>()
                .map_err(|_| ConstraintCheckerError::BadlyTypedInput);
        }

        Ok(0)
    }
}

/// A constraint checker that resolves claim disputes by keeping whichever claim came first.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PoeDispute;

impl SimpleConstraintChecker for PoeDispute {
    type Error = ConstraintCheckerError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        peek_data: &[DynamicallyTypedData],
        eviction_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there are no inputs to be normally consumed
        ensure!(
            input_data.is_empty(),
            ConstraintCheckerError::DisputeWithInput
        );

        // Make sure there are no outputs to be created
        ensure!(
            output_data.is_empty(),
            ConstraintCheckerError::DisputeWithOutput
        );

        // Make sure there is exactly one peek: the claim that will be retained.
        ensure!(
            !peek_data.is_empty(),
            ConstraintCheckerError::DisputeWithNoPeek
        );
        ensure!(
            peek_data.len() == 1,
            ConstraintCheckerError::DisputeWithMultiplePeeks
        );
        let winner: ClaimData = peek_data[1]
            .extract()
            .map_err(|_| ConstraintCheckerError::BadlyTypedInput)?;

        // Check the winner against the losers.
        // 1. All losers claim the same hash as the winner.
        // 2. All losers have effective block heights strictly greater than the winner.
        for untyped_loser in eviction_data {
            let loser: ClaimData = untyped_loser
                .extract()
                .map_err(|_| ConstraintCheckerError::BadlyTypedInput)?;
            ensure!(
                winner.claim == loser.claim,
                ConstraintCheckerError::DisputedClaimsNotForSameHash
            );
            ensure!(
                winner.effective_height > loser.effective_height,
                ConstraintCheckerError::DisputeSettledIncorrectly
            );
        }

        Ok(0)
    }
}
