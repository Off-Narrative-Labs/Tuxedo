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
use crate::tuxedo_types::{TypedData, UtxoData};
use parity_scale_codec::{Encode, Decode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

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
    /// No mother provided to mitosis process.
    NoMother,
    /// Too many inputs were provided to the verifier.
    ExtraInputs,
    /// The daughters did not have to right generation based on the mother.
    WrongGeneration,
    /// Too many outputs were provided to the verifier.
    ExtraOutputs,
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

    //TODO consider associated error type here.

    fn verify(&self, input_data: &[TypedData], output_data: &[TypedData]) -> bool {
        // Make sure there is exactly one mother.
        // assert!(input_data.len() == 1, )
        if input_data.is_empty() {
            return false;
        }
        if input_data.len() > 2 {
            return false;
        }
        let mother = match input_data[0].extract::<AmoebaDetails>() {
            Ok(a) => a,
            Err(_) => return false,
        };

        // Make sure there are exactly two daughters.
        if output_data.len() != 2 {
            return false;
        }
        let first_daughter = match output_data[0].extract::<AmoebaDetails>() {
            Ok(a) => a,
            Err(_) => return false,
        };
        let second_daughter = match output_data[1].extract::<AmoebaDetails>() {
            Ok(a) => a,
            Err(_) => return false,
        };

        // Make sure the generations are correct
        if first_daughter.generation != mother.generation + 1 {
            false
        }
        else if second_daughter.generation != mother.generation + 1 {
            false
        }
        else {
            true
        }
    }
}