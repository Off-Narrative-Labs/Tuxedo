//! This is a small pallet that handles runtime upgrades in chains that want
//! to support them.
//!
//! Right now this method is entirely unprotected (except by the redeemer) which
//! may not be realistic enough for public production chains. It should be composed
//! with some governance mechanism when one is available.
//!
//! It is not possible to adhere perfectly to the UTXO model here, because the
//! wasm code must be stored in the well-known `:code` key. We stick as closely
//! as possible to the UTXO model by having a UTXO that holds a hash of the current
//! wasm code. Then we pass the full wasm code as part of the verifier and write
//! it to the well-known key as a side effect.

use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::vec::Vec;
use sp_storage::well_known_keys::CODE;
use tuxedo_core::{
    types::{Input, Output},
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, Verifier, SimpleVerifier, utxo_set::TransparentUtxoSet, Redeemer,
};

/// A reference to a runtime wasm blob. It is just a hash.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
struct RuntimeRef {
    hash: [u8; 32],
}

impl UtxoData for RuntimeRef {
    const TYPE_ID: [u8; 4] = *b"upgd";
}

/// Reasons that the RuntimeUpgrade Verifier may fail
#[derive(Debug)]
pub enum VerifierError {
    // Again we're duplicating these common errors. Probably going to want a
    // better way to handle these.
    /// Wrong number of inputs were provided to the verifier.
    WrongNumberInputs,
    /// Wrong number of outputs were provided to the verifier.
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

/// The sole verifier for the runtime upgrade. It confirms that the UTXO
/// being consumed points to the correct current wasm and creates a new
/// UTXO for the new wasm.
///
/// This verifier is somewhat non-standard in that it has a side-effect that
/// writes the full wasm code to the well-known `:code` storage key. This is
/// necessary to satisfy Substrate's assumptions that this will happen.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct RuntimeUpgrade {
    full_wasm: Vec<u8>,
}

impl SimpleVerifier for RuntimeUpgrade {
    type Error = VerifierError;

    fn verify(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there is a single input that matches the hash of the previous runtime logic
        ensure!(input_data.len() == 1, VerifierError::WrongNumberInputs);
        let consumed = input_data[0]
            .extract::<RuntimeRef>()
            .map_err(|_| VerifierError::BadlyTypedInput)?;
        let outgoing_runtime =
            sp_io::storage::get(CODE).expect("Some runtime code should always be stored");
        let outgoing_hash = sp_io::hashing::blake2_256(&outgoing_runtime);
        ensure!(consumed.hash == outgoing_hash, VerifierError::InputMismatch);

        // Make sure there is a single output that matches the has of the incoming runtime logic
        ensure!(output_data.len() == 1, VerifierError::WrongNumberOutputs);
        let created = output_data[0]
            .extract::<RuntimeRef>()
            .map_err(|_| VerifierError::BadlyTypedOutput)?;
        let incoming_hash = sp_io::hashing::blake2_256(&self.full_wasm);
        ensure!(created.hash == incoming_hash, VerifierError::OutputMismatch);

        // SIDE EFFECT: Write the new wasm to storage
        sp_io::storage::set(CODE, &self.full_wasm);

        //TODO Figure out a better priority
        Ok(0)
    }
}

impl Verifier for RuntimeUpgrade {
    type Error = VerifierError;

    fn verify<R: Redeemer>(
        &self,
        inputs: &[Input],
        outputs: &[Output<R>],
    ) -> Result<TransactionPriority, Self::Error> {
        let input_data: Vec<DynamicallyTypedData> = inputs
            .iter()
            .map(|i| {
                TransparentUtxoSet::<R>::peek_utxo(&i.output_ref)
                    .expect("We just checked that all inputs were present.")
                    .payload
            })
            .collect();
        let output_data: Vec<DynamicallyTypedData> = outputs
            .iter()
            .map(|o| o.payload.clone())
            .collect();

        <RuntimeUpgrade as SimpleVerifier>::verify(self, &input_data, &output_data)
    }
}
