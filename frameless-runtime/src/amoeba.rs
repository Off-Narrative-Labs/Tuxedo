//! This file represents a simple example Tuxedo piece that tracks amoeba populations.
//! Amoeba's can be affected in three ways throughout their lifecycle.
//! 1. A new amoeba can be created by a creator. This is analogous to divine
//!    creation of a new species, and is currently not feature-gated, which
//!    is not very realistic. Ideally there would be a simple genesis config.
//! 2. An existing amoeba can die. When an amoeba dies, the utxo that represents it
//!    is consumed, and nothing new is created.
//! 3. An existing amoeba can undergo mitosis. Mitosis is a process that consumes the
//!    mother amoeba and creates, in its place two new daughter amoebas.

use crate::Verifier;
use crate::{ensure, fail};
use crate::tuxedo_types::{TypedData, UtxoData};
use parity_scale_codec::{Encode, Decode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;

/// An amoeba tracked by our simple Amoeba APP
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AmoebaDetails {
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

/// Things that can go wrong in the amoeba lifecycle
pub enum AmoebaError {

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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AmoebaMitosis;

impl Verifier for AmoebaMitosis {

    type Error = AmoebaError;

    fn verify(&self, input_data: &[TypedData], output_data: &[TypedData]) -> Result<TransactionPriority, AmoebaError> {
        // Make sure there is exactly one mother.
        ensure!(input_data.len() == 1, AmoebaError::WrongNumberInputs);
        let mother = input_data[0].extract::<AmoebaDetails>().map_err(|_| AmoebaError::BadlyTypedInput)?;

        // Make sure there are exactly two daughters.
        ensure!(output_data.len() == 2, AmoebaError::WrongNumberOutputs);
        let first_daughter = output_data[0].extract::<AmoebaDetails>().map_err(|_| AmoebaError::BadlyTypedOutput)?;
        let second_daughter = output_data[1].extract::<AmoebaDetails>().map_err(|_| AmoebaError::BadlyTypedOutput)?;

        // Make sure the generations are correct
        ensure!(first_daughter.generation == mother.generation + 1, AmoebaError::WrongGeneration);
        ensure!(second_daughter.generation == mother.generation + 1, AmoebaError::WrongGeneration);
        
        //TODO Figure out how to calculate priority.
        // Best priority idea so far. We have a verifier, PriorityVerifierWrapper<Inner: Verifier>(u8)
        // where you pass it the a number of inputs. It will take those first n inputs for itself, and assume
        // they are coins in some native currency. Then it will call the inner verifier with the remaining input
        // and if the inner verifier succeeds, it will prioritize based on the tip given in the first few inputs.
        // Such a wrapper should live with the money piece, and thus returning 0 here is fine.
        Ok(0)
    }
}