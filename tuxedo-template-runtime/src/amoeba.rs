//! This file represents a simple example Tuxedo piece that tracks amoeba populations.
//! Amoeba's can be affected in three ways throughout their lifecycle.
//! 1. A new amoeba can be created by a creator. This is analogous to divine
//!    creation of a new species, and is currently not feature-gated, which
//!    is not very realistic. Ideally there would be a simple genesis config.
//! 2. An existing amoeba can die. When an amoeba dies, the utxo that represents it
//!    is consumed, and nothing new is created.
//! 3. An existing amoeba can undergo mitosis. Mitosis is a process that consumes the
//!    mother amoeba and creates, in its place two new daughter amoebas.

use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::vec::Vec;
use tuxedo_core::{
    types::{Input, Output},
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, Verifier, SimpleVerifier, utxo_set::TransparentUtxoSet, Redeemer,
};

/// An amoeba tracked by our simple Amoeba APP
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AmoebaDetails {
    /// How many generations after the original Eve Amoeba this one is.
    /// When going through mitosis, this number must increase by 1 each time.
    pub generation: u32,
    /// Four totally arbitrary bytes that each amoeba has. There is literally no
    /// validation on this field whatsoever. I just had an instinct to include a second field.
    pub four_bytes: [u8; 4],
}

impl UtxoData for AmoebaDetails {
    const TYPE_ID: [u8; 4] = *b"amoe";
}

/// Reasons that the amoeba verifiers may fail
#[derive(Debug, Eq, PartialEq)]
pub enum VerifierError {
    /// An input data has the wrong type.
    BadlyTypedInput,
    /// An output data has the wrong type.
    BadlyTypedOutput,

    /// Amoeba creation requires a new amoeba to be created, but none was provided.
    CreatedNothing,
    /// Amoeba creation is not a mass operation. Only one new amoeba can be created.
    /// If you need to create multiple amoebas, you must submit multiple transactions.
    CreatedTooMany,
    /// No input may be consumed by amoeba creation.
    CreationMayNotConsume,

    /// Amoeba death requires a "victim" amoeba that will be consumed
    /// but noe was provided.
    NoVictim,
    /// Amoeba death is not a mass operation. Only one "victim" may be specified.
    /// If you need to kill off multiple amoebas, you must submit multiple transactions.
    TooManyVictims,
    /// No output may be created by amoeba death.
    DeathMayNotCreate,
    
    /// Amoeba mitosis requires exactly two daughter amoebas to be created.
    // Creating more or fewer than that is invalid.
    WrongNumberOfDaughters,
    /// Amoeba mitosis requires exactly one mother amoeba to be consumed.
    /// Consuming any more or fewer than that is invalid.
    WrongNumberOfMothers,
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

impl SimpleVerifier for AmoebaMitosis {
    type Error = VerifierError;

    fn verify(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, VerifierError> {
        // Make sure there is exactly one mother.
        ensure!(input_data.len() == 1, VerifierError::WrongNumberOfMothers);
        let mother = input_data[0]
            .extract::<AmoebaDetails>()
            .map_err(|_| VerifierError::BadlyTypedInput)?;

        // Make sure there are exactly two daughters.
        ensure!(output_data.len() == 2, VerifierError::WrongNumberOfDaughters);
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

impl SimpleVerifier for AmoebaDeath {
    type Error = VerifierError;

    fn verify(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there is a single victim
        ensure!(!input_data.is_empty(), VerifierError::NoVictim);
        ensure!(input_data.len() == 1, VerifierError::TooManyVictims);

        // We don't actually need to check any details of the victim, but we do need to make sure
        // we have the correct type.
        let _victim = input_data[0]
            .extract::<AmoebaDetails>()
            .map_err(|_| VerifierError::BadlyTypedInput)?;

        // Make sure there are no outputs
        ensure!(output_data.is_empty(), VerifierError::DeathMayNotCreate);

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

impl SimpleVerifier for AmoebaCreation {
    type Error = VerifierError;

    fn verify(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there is a single created amoeba
        ensure!(!output_data.is_empty(), VerifierError::CreatedNothing);
        ensure!(output_data.len() == 1, VerifierError::CreatedTooMany);
        let eve = output_data[0]
            .extract::<AmoebaDetails>()
            .map_err(|_| VerifierError::BadlyTypedOutput)?;

        // Make sure the newly created amoeba has generation 0
        ensure!(eve.generation == 0, VerifierError::WrongGeneration);

        // Make sure there are no inputs
        ensure!(input_data.is_empty(), VerifierError::CreationMayNotConsume);

        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tuxedo_core::dynamic_typing::testing::Bogus;

    #[test]
    fn creation_valid_transaction_works() {
        let to_spawn = AmoebaDetails {
            generation: 0,
            four_bytes: *b"test",
        };
        let input_data = Vec::new();
        let output_data = vec![to_spawn.into()];

        assert_eq!(
            AmoebaCreation.verify(&input_data, &output_data),
            Ok(0),
        );
    }

    #[test]
    fn creation_invalid_generation_fails() {
        let to_spawn = AmoebaDetails {
            generation: 100,
            four_bytes: *b"test",
        };
        let input_data = Vec::new();
        let output_data = vec![to_spawn.into()];

        assert_eq!(
            AmoebaCreation.verify(&input_data, &output_data),
            Err(VerifierError::WrongGeneration),
        );
    }

    #[test]
    fn creation_with_inputs_fails() {
        let example = AmoebaDetails {
            generation: 0,
            four_bytes: *b"test",
        };
        let input_data = vec![example.clone().into()];
        let output_data = vec![example.into()];

        assert_eq!(
            AmoebaCreation.verify(&input_data, &output_data),
            Err(VerifierError::CreationMayNotConsume),
        );
    }

    #[test]
    fn creation_with_badly_typed_output_fails() {
        let input_data = Vec::new();
        let output_data = vec![Bogus.into()];

        assert_eq!(
            AmoebaCreation.verify(&input_data, &output_data),
            Err(VerifierError::BadlyTypedOutput),
        );
    }

    #[test]
    fn creation_multiple_fails() {
        let to_spawn = AmoebaDetails {
            generation: 0,
            four_bytes: *b"test",
        };
        let input_data = Vec::new();
        let output_data = vec![to_spawn.clone().into(), to_spawn.into()];

        assert_eq!(
            AmoebaCreation.verify(&input_data, &output_data),
            Err(VerifierError::CreatedTooMany),
        );
    }

    #[test]
    fn creation_with_no_output_fails() {
        let input_data = Vec::new();
        let output_data = Vec::new();

        assert_eq!(
            AmoebaCreation.verify(&input_data, &output_data),
            Err(VerifierError::CreatedNothing),
        );
    }

    #[test]
    fn mitosis_valid_transaction_works() {
        let mother = AmoebaDetails {
            generation: 1,
            four_bytes: *b"test",
        };
        let d1 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        let d2 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        let input_data = vec![mother.into()];
        let output_data = vec![d1.into(), d2.into()];

        assert_eq!(
            AmoebaMitosis.verify(&input_data, &output_data),
            Ok(0),
        );
    }

    #[test]
    fn mitosis_wrong_generation() {
        let mother = AmoebaDetails {
            generation: 1,
            four_bytes: *b"test",
        };
        let d1 = AmoebaDetails {
            generation: 3, // This daughter has the wrong generation
            four_bytes: *b"test",
        };
        let d2 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        let input_data = vec![mother.into()];
        let output_data = vec![d1.into(), d2.into()];

        assert_eq!(
            AmoebaMitosis.verify(&input_data, &output_data),
            Err(VerifierError::WrongGeneration),
        );
    }

    #[test]
    fn mitosis_badly_typed_input() {
        let mother = Bogus;
        let d1 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        let d2 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        let input_data = vec![mother.into()];
        let output_data = vec![d1.into(), d2.into()];

        assert_eq!(
            AmoebaMitosis.verify(&input_data, &output_data),
            Err(VerifierError::BadlyTypedInput),
        );
    }

    #[test]
    fn mitosis_no_input() {
        let d1 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        let d2 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        let input_data = Vec::new();
        let output_data = vec![d1.into(), d2.into()];

        assert_eq!(
            AmoebaMitosis.verify(&input_data, &output_data),
            Err(VerifierError::WrongNumberOfMothers),
        );
    }

    #[test]
    fn mitosis_badly_typed_output() {
        let mother = AmoebaDetails {
            generation: 1,
            four_bytes: *b"test",
        };
        let d1 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        let d2 = Bogus;
        let input_data = vec![mother.into()];
        let output_data = vec![d1.into(), d2.into()];

        assert_eq!(
            AmoebaMitosis.verify(&input_data, &output_data),
            Err(VerifierError::BadlyTypedOutput),
        );
    }

    #[test]
    fn mitosis_only_one_output() {
        let mother = AmoebaDetails {
            generation: 1,
            four_bytes: *b"test",
        };
        let d1 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        let input_data = vec![mother.into()];
        // There is only one daughter when there should be two
        let output_data = vec![d1.into()];

        assert_eq!(
            AmoebaMitosis.verify(&input_data, &output_data),
            Err(VerifierError::WrongNumberOfDaughters),
        );
    }

    #[test]
    fn mitosis_too_many_outputs() {
        let mother = AmoebaDetails {
            generation: 1,
            four_bytes: *b"test",
        };
        let d1 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        let d2 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        // Mitosis requires exactly two daughters. There should not be a third one.
        let d3 = AmoebaDetails {
            generation: 2,
            four_bytes: *b"test",
        };
        let input_data = vec![mother.into()];
        let output_data = vec![d1.into(), d2.into(), d3.into()];

        assert_eq!(
            AmoebaMitosis.verify(&input_data, &output_data),
            Err(VerifierError::WrongNumberOfDaughters),
        );
    }

    #[test]
    fn death_valid_transaction_works() {
        let example = AmoebaDetails {
            generation: 1,
            four_bytes: *b"test",
        };
        let input_data = vec![example.into()];
        let output_data = vec![];

        assert_eq!(
            AmoebaDeath.verify(&input_data, &output_data),
            Ok(0),
        );
    }

    #[test]
    fn death_no_input() {
        let input_data = vec![];
        let output_data = vec![];

        assert_eq!(
            AmoebaDeath.verify(&input_data, &output_data),
            Err(VerifierError::NoVictim),
        );
    }

    #[test]
    fn death_multiple_inputs() {
        let a1 = AmoebaDetails {
            generation: 1,
            four_bytes: *b"test",
        };
        let a2 = AmoebaDetails {
            generation: 4,
            four_bytes: *b"test",
        };
        let input_data = vec![a1.into(), a2.into()];
        let output_data = vec![];

        assert_eq!(
            AmoebaDeath.verify(&input_data, &output_data),
            Err(VerifierError::TooManyVictims),
        );
    }

    #[test]
    fn death_with_output() {
        let example = AmoebaDetails {
            generation: 1,
            four_bytes: *b"test",
        };
        let input_data = vec![example.clone().into()];
        let output_data = vec![example.into()];

        assert_eq!(
            AmoebaDeath.verify(&input_data, &output_data),
            Err(VerifierError::DeathMayNotCreate),
        );
    }

    #[test]
    fn death_badly_typed_input() {
        let example = Bogus;
        let input_data = vec![example.into()];
        let output_data = vec![];

        assert_eq!(
            AmoebaDeath.verify(&input_data, &output_data),
            Err(VerifierError::BadlyTypedInput),
        );
    }


}