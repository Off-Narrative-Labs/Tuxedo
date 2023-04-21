//! Verifier Abstractions provide users with a UX-improved alternative to attaching a
//! verifier (eg a public key) directly to a UTXO. With Verifier Abstractions, users
//! instead point to a Verifier abstraction which is stored in state. The Abstraction
//! contains an inner verifier which will actually be used to redeem inputs. The big
//! advantage is that the user can swap out the inner verifier in a single transaction
//! and affect all UTXOs guarded by it.
//!
//! This is analogous to account abstraction in accounts-based systems.
//!
//! The motivating use case is when a user has a possible key exposure. Using
//! traditional verifiers, the only way to protect all assets guarded by the compromised
//! key is to spend each and every one of them which will be expensive, monotonous, and
//! error-prone. With verifier abstractions, the compromised key can be taken out of
//! production with a single transaction.
//!
//! This moves the task of validating a single input into the constraint checker.
//! An alternative approach would be to allow the verifier peek-access to state.

use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::vec::Vec;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    SimpleConstraintChecker, Verifier,
};

/// A unique identifier for each abstraction.
/// TODO this is NOT SECURE. We just let any user create an abstraction with
/// any id. That means there can be collisions. We need a more unique type.
/// For example it could be the hash of the transaction that creates it.
/// But if we do that, we have to watch out for batching to come later in which case a single transaction
/// could create multiple.
type AbstractionId = u32;

/// A wrapper around a UTXO that can only be spent by calling the abstract verifier
/// constraint checker
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AbstractlyGuardedUtxo {
    /// The inner UTXO is stored dynamically typed. Casting to a type
    /// will be done downstream in the inner constraint checker.
    pub contents: DynamicallyTypedData,
    /// Which abstraction this utxo is guarded by.
    pub abstraction: AbstractionId,
}

impl UtxoData for AbstractlyGuardedUtxo {
    const TYPE_ID: [u8; 4] = *b"abst";
}

/// A layer of indirection around a verifier. Rather than tying a verifier directly to
/// a UTXO, the UTXO is points to this abstraction which is then used to unlock it.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Abstraction<V: Verifier> {
    /// The unique identifier of this abstraction
    pub id: AbstractionId,
    /// The actual verifier that should be used to unlock outputs associated with
    /// this abstraction. This verifier can be swapped by a transaction.
    pub inner_verifier: V,
}

/// The various errors that can occur when using verifier abstractions
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum VerificationAbstractionError {
    /// I haven't actually written the logic yet, so I haven't written the error variants either.
    Todo,
}

/// Wrap some UTXOs in an abstraction.
///
/// When wrapping a UTXO in an abstraction, users typically guard the new
/// AbstractlyGuardedUtxo with `UpForGrabs` or something similar that allows anyone
/// to consume it. This is because the constraint checker will be performing the actual
/// inner verification logic. However, the piece does not enforce this. It is possible
/// to have an AbstractlyGuardedUtxo that is itself guarded by a normal verifier even though
/// no use case for this is immediately obvious.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Wrap;

impl SimpleConstraintChecker for Wrap {
    type Error = VerificationAbstractionError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!("Actually implement this pseudocode")

        // Make sure that there are the same number of inputs and outputs

        // Make sure that each output:
        // * Is of type AbstractlyGuardedUtxo
        // * Contains the same dynamically typed data from the original
    }
}

/// Unwrap some UTXOs from their abstraction returning it to the utxo set with a normal verifier
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct UnWrap {
    /// The redeemers that will be used to satisfy the current inner verifier against the inputs
    pub redeemers: Vec<Vec<u8>>,
}

impl SimpleConstraintChecker for UnWrap {
    type Error = VerificationAbstractionError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!("Actually implement this pseudocode")

        // Make sure that there is one more output than input.
        // The one extra input is the abstraction. Later this should be made into a peek.
        // Make sure the abstraction is properly typed.

        // Make sure that each remaining (non-abstraction) input:
        // * Is of type AbstractlyGuardedUtxo
        // * Points to the abstraction being supplied
        // * Verifies properly according to the abstraction's current inner verifier

        // Make sure each output
        // * Contains the same dynamically typed data that was wrapped
    }
}

/// Create a new verifier abstraction
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct NewAbstraction;

impl SimpleConstraintChecker for NewAbstraction {
    type Error = VerificationAbstractionError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!("Actually implement this pseudocode")

        // Make sure there are no inputs

        // Make sure there is a single output of type Abstraction

        // Make sure the abstraction's id is correct.
        // Currently I'm just making the user supply it , so there is no notion of "correct"
        // but that needs to change to be production ready
    }
}

/// Change the inner verifier associated with a Verifier Abstraction
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct UpdateAbstraction {
    pub redeemer: Vec<u8>,
}

impl SimpleConstraintChecker for UpdateAbstraction {
    type Error = VerificationAbstractionError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!("Actually implement this pseudocode")

        // Make sure there is a single input of type Abstraction

        // Design decision: Make sure the input satisfies its own inner verifier.
        // This is not strictly necessary since a normal verifier can still be used.
        // Notice it isn't a good idea to wrap your own abstraction with itself. Doing so makes it
        // forever unusable. Because the raw abstraction is no longer in the utxo set (its only there in wrapped form)
        // It can't be used to unwrap anything. I prefer not requiring this. Users can use the same verifier as the inner and the normal.

        // Make sure there is a single output of type Abstraction.

        // Make sure the output and the input have the same id.
    }
}

// Extra constraint checkers that would improve ux
// * Delete an abstraction
// * Allow unwrapping some utxos, and consuming them in the same transaction, by calling some inner constraint checker".
//   Ideally it would allow unwrapping multiple utxos in a single transaction and mixing them with normally-verified utxos
// * Allow newly created UTXOs to be wrapped in the same tx before being stored (compliment to the previous)
