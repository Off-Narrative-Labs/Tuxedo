//! This file represents a simple example Tuxedo piece that tracks amoeba populations.
//! Amoeba's can be affected in three ways throughout their lifecycle.
//! 1. A new amoeba can be created by a creator. This is analogous to divine
//!    creation of a new species, and is currently not feature-gated, which
//!    is not very realistic. Ideally there would be a simple genesis config.
//! 2. An existing amoeba can die. When an amoeba dies, the utxo that represents it
//!    is consumed, and nothing new is created.
//! 3. An existing amoeba can undergo mitosis. Mitosis is a process that consumes the
//!    mother amoeba and creates, in its place two new daughter amoebas.

use crate::tuxedo_types::{TypedData, UtxoData};
use crate::Verifier;
use crate::{ensure, fail};
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;

/// An amoeba tracked by our simple Amoeba APP
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
struct AmoebaDetails {
    /// How many generations after the original Eve Amoeba this one is.
    /// When going through mitosis, this number must increase by 1 each time.
    generation: u32,
    /// Four totally arbitrary bytes that each amoeba has. There is literally no
    /// validation on this field whatsoever. I just had an instinct to include a second field.
    four_bytes: [u8; 4],
}

impl UtxoData for AmoebaDetails {
    const TYPE_ID: [u8; 4] = *b"amoe";
}

/// Reasons that the amoeba verifiers may fail
#[derive(Debug)]
pub enum VerifierError {
    // TODO In the current design, this will need to be repeated in every piece. This is not nice.
    // Well, actually, no. Some pieces will be flexible about how many inputs and outputs they can take.
    // For example, the money piece can take as many input or output coins as it wants.
    // Similarly, in kitties, we may allow breeding via cat orgies with arbitrarily many parents providing genetic source material.
    /// Wrong number of inputs were provided to the verifier.
    WrongNumberInputs,
    /// Wrong number of outputs were provided to the verifier.
    WrongNumberOutputs,
    /// An input data has the wrong type.
    BadlyTypedInput,
    /// An output data has the wrong type.
    BadlyTypedOutput,

    // Now we get on to the actual amoeba-specific errors
    /// The daughters did not have to right generation based on the mother.
    WrongGeneration,
}

/// A verifier for the process of amoeba mitosis
/// The mitosis is valid is the following criteria are met
/// 1. There is exactly one mother amoeba.
/// 2. There are exactly two daughter amoebas
/// 3. Each Daughter amoeba has a generation one higher than its mother.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AmoebaMitosis;

impl Verifier for AmoebaMitosis {
    type Error = VerifierError;

    fn verify(
        &self,
        input_data: &[TypedData],
        output_data: &[TypedData],
    ) -> Result<TransactionPriority, VerifierError> {
        // Make sure there is exactly one mother.
        ensure!(input_data.len() == 1, VerifierError::WrongNumberInputs);
        let mother = input_data[0]
            .extract::<AmoebaDetails>()
            .map_err(|_| VerifierError::BadlyTypedInput)?;

        // Make sure there are exactly two daughters.
        ensure!(output_data.len() == 2, VerifierError::WrongNumberOutputs);
        let first_daughter = output_data[0]
            .extract::<AmoebaDetails>()
            .map_err(|_| VerifierError::BadlyTypedOutput)?;
        let second_daughter = output_data[1]
            .extract::<AmoebaDetails>()
            .map_err(|_| VerifierError::BadlyTypedOutput)?;

        // Make sure the generations are correct
        ensure!(
            first_daughter.generation == mother.generation + 1,
            VerifierError::WrongGeneration
        );
        ensure!(
            second_daughter.generation == mother.generation + 1,
            VerifierError::WrongGeneration
        );

        //TODO Figure out how to calculate priority.
        // Best priority idea so far. We have a verifier, PriorityVerifierWrapper<Inner: Verifier>(u8)
        // where you pass it the a number of inputs. It will take those first n inputs for itself, and assume
        // they are coins in some native currency. Then it will call the inner verifier with the remaining input
        // and if the inner verifier succeeds, it will prioritize based on the tip given in the first few inputs.
        // Such a wrapper should live with the money piece, and thus returning 0 here is fine.
        Ok(0)
    }
}

/// A verifier for simple death of an amoeba.
///
/// Any amoeba can be killed by providing it as the sole input to this verifier. No
/// new outputs are ever created.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AmoebaDeath;

impl Verifier for AmoebaDeath {
    type Error = VerifierError;

    fn verify(
        &self,
        input_data: &[TypedData],
        output_data: &[TypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there is a single victim
        // Another valid design choice would be to allow killing many amoebas at once
        // but that is not the choice I've made here.
        ensure!(input_data.len() == 1, VerifierError::WrongNumberInputs);

        // We don't actually need to check any details of the victim, but we do need to make sure
        // we have the correct type.
        let _victim = input_data[0]
            .extract::<AmoebaDetails>()
            .map_err(|_| VerifierError::BadlyTypedInput)?;

        // Make sure there are no outputs
        ensure!(output_data.is_empty(), VerifierError::WrongNumberOutputs);

        Ok(0)
    }
}

/// A verifier for simple creation of an amoeba.
///
/// A new amoeba can be created by providing it as the sole output to this verifier. No
/// inputs are ever consumed.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AmoebaCreation;

impl Verifier for AmoebaCreation {
    type Error = VerifierError;

    fn verify(
        &self,
        input_data: &[TypedData],
        output_data: &[TypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there is a single created amoeba
        ensure!(output_data.len() == 1, VerifierError::WrongNumberOutputs);
        let eve = output_data[0]
            .extract::<AmoebaDetails>()
            .map_err(|_| VerifierError::BadlyTypedInput)?;

        // Make sure the newly created amoeba has generation 0
        ensure!(eve.generation == 0, VerifierError::WrongGeneration);

        // Make sure there are no inputs
        ensure!(input_data.is_empty(), VerifierError::WrongNumberInputs);

        Ok(0)
    }
}
