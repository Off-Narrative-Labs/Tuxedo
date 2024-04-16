//! An implementation of a simple fungible token.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::prelude::*;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure,
    traits::Cash,
    types::Transaction,
    SimpleConstraintChecker, Verifier,
};

#[cfg(test)]
mod tests;

impl<const ID: u8> Cash for Coin<ID> {
    fn value(&self) -> u128 {
        self.0
    }

    const ID: u8 = ID;
}

// use log::info;

/// The main constraint checker for the money piece. Allows spending and minting tokens.
#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
    Hash,
    Debug,
    TypeInfo,
)]
pub enum MoneyConstraintChecker<const ID: u8> {
    /// A typical spend transaction where some coins are consumed and others are created.
    /// Input value must exceed output value. The difference is burned and reflected in the
    /// transaction's priority.
    Spend,
    /// A mint transaction that creates no coins out of the void. In a real-world chain,
    /// this should be protected somehow, or not included at all. For now it is publicly
    /// available. I'm adding it to explore multiple validation paths in a single piece.
    Mint,
}

/// A single coin in the fungible money system.
/// A new-type wrapper around a `u128` value.
#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
    Hash,
    Debug,
    TypeInfo,
)]
pub struct Coin<const ID: u8>(pub u128);

impl<const ID: u8> Coin<ID> {
    pub fn new(amt: u128) -> Self {
        Coin(amt)
    }

    /// Create a mint transaction for a single Coin.
    pub fn mint<V, OV, OC>(amt: u128, v: V) -> Transaction<OV, OC>
    where
        V: Verifier,
        OV: Verifier + From<V>,
        OC: tuxedo_core::ConstraintChecker<OV> + From<MoneyConstraintChecker<ID>>,
    {
        Transaction {
            inputs: vec![],
            peeks: vec![],
            outputs: vec![(Self::new(amt), v).into()],
            checker: MoneyConstraintChecker::Mint.into(),
        }
    }
}

impl<const ID: u8> UtxoData for Coin<ID> {
    const TYPE_ID: [u8; 4] = [b'c', b'o', b'i', ID];
}

/// Errors that can occur when checking money transactions.
#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
    Hash,
    Debug,
    TypeInfo,
)]
pub enum ConstraintCheckerError {
    /// Dynamic typing issue.
    /// This error doesn't discriminate between badly typed inputs and outputs.
    BadlyTyped,
    /// The transaction attempts to consume inputs while minting. This is not allowed.
    MintingWithInputs,
    /// The transaction attempts to mint zero coins. This is not allowed.
    MintingNothing,
    /// The transaction attempts to spend without consuming any inputs.
    /// Either the output value will exceed the input value, or if there are no outputs,
    /// it is a waste of processing power, so it is not allowed.
    SpendingNothing,
    /// The value of the spent input coins is less than the value of the newly created
    /// output coins. This would lead to money creation and is not allowed.
    OutputsExceedInputs,
    /// The value consumed or created by this transaction overflows the value type.
    /// This could lead to problems like https://bitcointalk.org/index.php?topic=823.0
    ValueOverflow,
    /// The transaction attempted to create a coin with zero value. This is not allowed
    /// because it wastes state space.
    ZeroValueCoin,
}

impl<const ID: u8> SimpleConstraintChecker for MoneyConstraintChecker<ID> {
    type Error = ConstraintCheckerError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        match &self {
            Self::Spend => {
                // Check that we are consuming at least one input
                ensure!(
                    !input_data.is_empty(),
                    ConstraintCheckerError::SpendingNothing
                );

                let mut total_input_value: u128 = 0;
                let mut total_output_value: u128 = 0;

                // Check that sum of input values < output values
                for input in input_data {
                    let utxo_value = input
                        .extract::<Coin<ID>>()
                        .map_err(|_| ConstraintCheckerError::BadlyTyped)?
                        .0;
                    total_input_value = total_input_value
                        .checked_add(utxo_value)
                        .ok_or(ConstraintCheckerError::ValueOverflow)?;
                }

                for utxo in output_data {
                    let utxo_value = utxo
                        .extract::<Coin<ID>>()
                        .map_err(|_| ConstraintCheckerError::BadlyTyped)?
                        .0;
                    ensure!(utxo_value > 0, ConstraintCheckerError::ZeroValueCoin);
                    total_output_value = total_output_value
                        .checked_add(utxo_value)
                        .ok_or(ConstraintCheckerError::ValueOverflow)?;
                }

                ensure!(
                    total_output_value <= total_input_value,
                    ConstraintCheckerError::OutputsExceedInputs
                );

                // Priority is based on how many token are burned
                // Type stuff is kinda ugly. Maybe division would be better?
                let burned = total_input_value - total_output_value;
                Ok(if burned < u64::max_value() as u128 {
                    burned as u64
                } else {
                    u64::max_value()
                })
            }
            Self::Mint => {
                // Make sure there are no inputs being consumed
                ensure!(
                    input_data.is_empty(),
                    ConstraintCheckerError::MintingWithInputs
                );

                // Make sure there is at least one output being minted
                ensure!(
                    !output_data.is_empty(),
                    ConstraintCheckerError::MintingNothing
                );

                // Make sure the outputs are the right type
                for utxo in output_data {
                    let utxo_value = utxo
                        .extract::<Coin<ID>>()
                        .map_err(|_| ConstraintCheckerError::BadlyTyped)?
                        .0;
                    ensure!(utxo_value > 0, ConstraintCheckerError::ZeroValueCoin);
                }

                // No priority for minting
                Ok(0)
            }
        }
    }
}
