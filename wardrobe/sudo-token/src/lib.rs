//! Some functionality in a Runtime needs to be gated behind some form of on-chain
//! governance. This pallet implements a token-based solution to restrict access to
//! sensitive transactions to callers who have access to a specific token.
//! 
//! One simple way to manage this token is to lock it behind a signature check verifier,
//! Or some other private ownership verifier. In this configuration it is similar to
//! FRAME's pallet sudo. One advantage over pallet sudo is that 
//! 
//! You could achieve basic council-like governance by locking the token behind a
//! multisig verifier, or composing it with an on-chain stateful multisig.
//! 
//! Currently using the token requires consuming it and recreating it. But in the
//! future, peeks may also allow verifiers, and then peeking would be sufficient.
//! 
//! More complex governance like token voting is not yet in scope for consideration.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData}, ensure, ConstraintChecker, SimpleConstraintChecker
};

#[cfg(test)]
mod tests;

/// A simple one-off token that represents the ability to access elevated privledges.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct SudoToken;

impl UtxoData for SudoToken {
    const TYPE_ID: [u8; 4] = *b"sudo";
}

/// Reasons that the sudo token constraint checkers may fail
#[derive(Debug, Eq, PartialEq)]
pub enum ConstraintCheckerError {
    /// No inputs were presented in the transaction. But the sudo token must be consumed.
    NoInputs,
    /// The first input to the transaction must be the sudo token, but it was not.
    FirstInputIsNotSudoToken,
    /// 
    NoOutput,
    ///
    FirstOutputIsNotSudoToken,
}

/// Call some transaction with escalated privledges
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct SudoCall<Inner>;

impl<Inner: SimpleChecker> SimpleConstraintChecker for SudoCall<Inner> {
    type Error = ConstraintCheckerError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, ConstraintCheckerError> {
        // Make sure the first input is the sudo token.
        // If the caller is able to consume this token, they may have the elevated access.
        ensure!();
        
        // Make sure the first output is the same sudo token.
        ensure!();

        //
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
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
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
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
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
