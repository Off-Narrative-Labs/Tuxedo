//! This is a small pallet that handles runtime upgrades in chains that want
//! to support them.
//!
//! Right now this method is entirely unprotected (except by the verifier) which
//! may not be realistic enough for public production chains. It should be composed
//! with some governance mechanism when one is available.
//!
//! It is not possible to adhere perfectly to the UTXO model here, because the
//! wasm code must be stored in the well-known `:code` key. We stick as closely
//! as possible to the UTXO model by having a UTXO that holds a hash of the current
//! wasm code. Then we pass the full wasm code as part of the constraint checker and write
//! it to the well-known key as a side effect.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::vec::Vec;
use sp_storage::well_known_keys::CODE;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, SimpleConstraintChecker, Verifier,
};

#[cfg(test)]
mod tests;

/// A reference to a runtime wasm blob. It is just a hash.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
struct RuntimeRef {
    hash: [u8; 32],
}

impl UtxoData for RuntimeRef {
    const TYPE_ID: [u8; 4] = *b"upgd";
}

/// Reasons that the RuntimeUpgrade constraint checker may fail
#[derive(Debug)]
pub enum ConstraintCheckerError {
    // Again we're duplicating these common errors. Probably going to want a
    // better way to handle these.
    /// Wrong number of inputs were provided to the constraint checker.
    WrongNumberInputs,
    /// Wrong number of outputs were provided to the constraint checker.
    WrongNumberOutputs,
    /// An input data has the wrong type.
    BadlyTypedInput,
    /// An output data has the wrong type.
    BadlyTypedOutput,

    // Now we get on to the actual upgrade-specific errors
    /// The consumed input does not match the current wasm. This should never happen
    /// and is indicative of inconsistent state. Perhaps another piece interfered?
    InputMismatch,
    /// The created output does not match the provided new runtime wasm.
    OutputMismatch,
}

/// The sole constraint checker for the runtime upgrade. It confirms that the UTXO
/// being consumed points to the correct current wasm and creates a new
/// UTXO for the new wasm.
///
/// This constraint checker is somewhat non-standard in that it has a side-effect that
/// writes the full wasm code to the well-known `:code` storage key. This is
/// necessary to satisfy Substrate's assumptions that this will happen.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct RuntimeUpgrade {
    full_wasm: Vec<u8>,
}

impl<V: Verifier> SimpleConstraintChecker<V> for RuntimeUpgrade {
    type Error = ConstraintCheckerError;
    type InherentHooks = ();

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there is a single input that matches the hash of the previous runtime logic
        ensure!(
            input_data.len() == 1,
            ConstraintCheckerError::WrongNumberInputs
        );
        let consumed = input_data[0]
            .extract::<RuntimeRef>()
            .map_err(|_| ConstraintCheckerError::BadlyTypedInput)?;
        let outgoing_runtime =
            sp_io::storage::get(CODE).expect("Some runtime code should always be stored");
        let outgoing_hash = sp_io::hashing::blake2_256(&outgoing_runtime);
        ensure!(
            consumed.hash == outgoing_hash,
            ConstraintCheckerError::InputMismatch
        );

        // Make sure there is a single output that matches the has of the incoming runtime logic
        ensure!(
            output_data.len() == 1,
            ConstraintCheckerError::WrongNumberOutputs
        );
        let created = output_data[0]
            .extract::<RuntimeRef>()
            .map_err(|_| ConstraintCheckerError::BadlyTypedOutput)?;
        let incoming_hash = sp_io::hashing::blake2_256(&self.full_wasm);
        ensure!(
            created.hash == incoming_hash,
            ConstraintCheckerError::OutputMismatch
        );

        // SIDE EFFECT: Write the new wasm to storage
        sp_io::storage::set(CODE, &self.full_wasm);

        //TODO Figure out a better priority
        Ok(0)
    }
}
