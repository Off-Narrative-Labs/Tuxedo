//! This file represents a simple example Tuxedo piece that tracks amoeba populations.
//! Amoeba's can be affected in three ways throughout their lifecycle.
//! 1. A new amoeba can be created by a creator. This is analogous to divine
//!    creation of a new species, and is currently not feature-gated, which
//!    is not very realistic. Ideally there would be a simple genesis config.
//! 2. An existing amoeba can die. When an amoeba dies, the utxo that represents it
//!    is consumed, and nothing new is created.
//! 3. An existing amoeba can undergo mitosis. Mitosis is a process that consumes the
//!    mother amoeba and creates, in its place two new daughter amoebas.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, SimpleConstraintChecker,
};

#[cfg(test)]
mod tests;

/// An amoeba tracked by our simple Amoeba APP
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
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

/// Reasons that the amoeba constraint checkers may fail
#[derive(Debug, Eq, PartialEq)]
pub enum ConstraintCheckerError {
    /// An input data has the wrong type.
    BadlyTypedInput,
    /// An output data has the wrong type.
    BadlyTypedOutput,
    /// The Amoeba piece does not allow any evictions at all.
    NoEvictionsAllowed,

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

/// A constraint checker for the process of amoeba mitosis
/// The mitosis is valid is the following criteria are met
/// 1. There is exactly one mother amoeba.
/// 2. There are exactly two daughter amoebas
/// 3. Each Daughter amoeba has a generation one higher than its mother.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct AmoebaMitosis;

impl SimpleConstraintChecker for AmoebaMitosis {
    type Error = ConstraintCheckerError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        evicted_input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, ConstraintCheckerError> {
        // Can't evict anything
        ensure!(
            evicted_input_data.is_empty(),
            ConstraintCheckerError::NoEvictionsAllowed
        );

        // Make sure there is exactly one mother.
        ensure!(
            input_data.len() == 1,
            ConstraintCheckerError::WrongNumberOfMothers
        );
        let mother = input_data[0]
            .extract::<AmoebaDetails>()
            .map_err(|_| ConstraintCheckerError::BadlyTypedInput)?;

        // Make sure there are exactly two daughters.
        ensure!(
            output_data.len() == 2,
            ConstraintCheckerError::WrongNumberOfDaughters
        );
        let first_daughter = output_data[0]
            .extract::<AmoebaDetails>()
            .map_err(|_| ConstraintCheckerError::BadlyTypedOutput)?;
        let second_daughter = output_data[1]
            .extract::<AmoebaDetails>()
            .map_err(|_| ConstraintCheckerError::BadlyTypedOutput)?;

        // Make sure the generations are correct
        ensure!(
            first_daughter.generation == mother.generation + 1,
            ConstraintCheckerError::WrongGeneration
        );
        ensure!(
            second_daughter.generation == mother.generation + 1,
            ConstraintCheckerError::WrongGeneration
        );

        //TODO Figure out how to calculate priority.
        // Best priority idea so far. We have a constraint checker,
        // PriorityConstraintCheckerWrapper<Inner: ConstraintChecker>(u8)
        // where you pass it the a number of inputs. It will take those first n inputs for itself, and assume
        // they are coins in some native currency. Then it will call the inner constraint checker with the remaining input
        // and if the inner constraint checker succeeds, it will prioritize based on the tip given in the first few inputs.
        // Such a wrapper should live with the money piece, and thus returning 0 here is fine.
        Ok(0)
    }
}

/// A constraint checker for simple death of an amoeba.
///
/// Any amoeba can be killed by providing it as the sole input to this constraint checker. No
/// new outputs are ever created.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct AmoebaDeath;

impl SimpleConstraintChecker for AmoebaDeath {
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
            ConstraintCheckerError::NoEvictionsAllowed
        );

        // Make sure there is a single victim
        ensure!(!input_data.is_empty(), ConstraintCheckerError::NoVictim);
        ensure!(
            input_data.len() == 1,
            ConstraintCheckerError::TooManyVictims
        );

        // We don't actually need to check any details of the victim, but we do need to make sure
        // we have the correct type.
        let _victim = input_data[0]
            .extract::<AmoebaDetails>()
            .map_err(|_| ConstraintCheckerError::BadlyTypedInput)?;

        // Make sure there are no outputs
        ensure!(
            output_data.is_empty(),
            ConstraintCheckerError::DeathMayNotCreate
        );

        Ok(0)
    }
}

/// A constraint checker for simple creation of an amoeba.
///
/// A new amoeba can be created by providing it as the sole output to this constraint checker. No
/// inputs are ever consumed.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct AmoebaCreation;

impl SimpleConstraintChecker for AmoebaCreation {
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
            ConstraintCheckerError::NoEvictionsAllowed
        );

        // Make sure there is a single created amoeba
        ensure!(
            !output_data.is_empty(),
            ConstraintCheckerError::CreatedNothing
        );
        ensure!(
            output_data.len() == 1,
            ConstraintCheckerError::CreatedTooMany
        );
        let eve = output_data[0]
            .extract::<AmoebaDetails>()
            .map_err(|_| ConstraintCheckerError::BadlyTypedOutput)?;

        // Make sure the newly created amoeba has generation 0
        ensure!(eve.generation == 0, ConstraintCheckerError::WrongGeneration);

        // Make sure there are no inputs
        ensure!(
            input_data.is_empty(),
            ConstraintCheckerError::CreationMayNotConsume
        );

        Ok(0)
    }
}
