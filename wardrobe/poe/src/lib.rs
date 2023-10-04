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
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::fmt::Debug;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure,
    support_macros::{CloneNoBound, DebugNoBound},
    SimpleConstraintChecker, Verifier,
};

#[cfg(test)]
mod tests;

// Notice this type doesn't have to be public. Cool.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
}

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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, DebugNoBound, CloneNoBound, PartialEq, Eq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct PoeClaim<T>(PhantomData<T>);

impl<T: PoeConfig + 'static, V: Verifier> SimpleConstraintChecker<V> for PoeClaim<T> {
    type Error = ConstraintCheckerError;
    type InherentHooks = ();

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct PoeRevoke;

impl<V: Verifier> SimpleConstraintChecker<V> for PoeRevoke {
    type Error = ConstraintCheckerError;
    type InherentHooks = ();
    

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
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
///
/// TODO this will work much more elegantly once peek is implemented. We only need to peek at the
/// older winning claim because it will remain in state afterwards.
///
/// TODO what shall we do about the verifier? Each claimer may have given their claim a verifier
/// such that their own private signature. Perhaps there should be a way for a constraint checker to override
/// the verifier logic? This is a concrete case where the constraint checker verifier separation is not ideal.
/// Another, weaker example, is when trying o implement something like sudo. Where we want a signature,
/// but we want to authorized signer to come from the a different part of state.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct PoeDispute;

impl<V: Verifier> SimpleConstraintChecker<V> for PoeDispute {
    type Error = ConstraintCheckerError;
    type InherentHooks = ();

    fn check(
        &self,
        _input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        _output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!("implement this once we have at least peeks and maybe evictions")

        // Make sure there is at least one input (once peek is ready, it will become a peek)
        // This first input (or only peek) is the claim that will be retained.

        // Make sure that all other inputs, claim the same hash as the winner.

        // Make sure that all other claims have block heights strictly greater than the winner.

        //TODO what to do about the verifiers on those losing claims.
    }
}

#[allow(dead_code)]
mod brainstorm {
    /// One workable solution to the problem above is modifying the core transaction structure to something like this
    struct Transaction {
        /// A classic input that is consumed from the utxo set. Its verifier must be satisfied for the tx to be valid
        redemptions: Vec<InputRef>,
        /// Similar to a redemption, this is an input that is consumed from the utxo set, but its verifier need not be satisfied
        /// In the Poe case above, the losing claims that came later would be evictions.
        evictions: Vec<InputRef>,
        /// Similar to an input, but it is not consumed. This is a way to read pre-existing state without removing it from the utxo set
        /// this also indicates when transaction are not competing for state despite reading the same state, and thus commute.
        /// TBD whether it makes sense to have a verifier check. My gut instinct is no verifier check, but it needs more careful thought.
        peeks: Vec<InputRef>,
        /// Newly created pieces of state to be added to the utxo set.
        outputs: Vec<Output>,
    }

    // Aliases so my sketch above compiles
    type InputRef = ();
    type Output = ();
    use sp_std::vec::Vec;
}
