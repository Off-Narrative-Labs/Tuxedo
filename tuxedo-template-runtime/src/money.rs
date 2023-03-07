//! An implementation of a simple fungible token.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::prelude::*;
use tuxedo_core::{
    types::{Input, Output},
    dynamic_typing::{DynamicallyTypedData, UtxoData}, utxo_set::TransparentUtxoSet,
    ensure, Verifier, SimpleVerifier, Redeemer,
};

// use log::info;

/// The main verifier for the money piece. Allows spending and minting tokens.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum MoneyVerifier {
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
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Coin(pub u128);

impl Coin {
    pub fn new(amt: u128) -> Self {
        Coin(amt)
    }
}

impl UtxoData for Coin {
    const TYPE_ID: [u8; 4] = *b"coin";
}

/// Errors that can occur when verifying money transactions.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum VerifierError {
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

impl SimpleVerifier for MoneyVerifier {
    type Error = VerifierError;

    fn verify(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        match &self {
            Self::Spend => {
                // Check that we are consuming at least one input
                ensure!(!input_data.is_empty(), VerifierError::SpendingNothing);

                let mut total_input_value: u128 = 0;
                let mut total_output_value: u128 = 0;

                // Check that sum of input values < output values
                for input in input_data {
                    let utxo_value = input
                        .extract::<Coin>()
                        .map_err(|_| VerifierError::BadlyTyped)?
                        .0;
                    total_input_value = total_input_value
                        .checked_add(utxo_value)
                        .ok_or(VerifierError::ValueOverflow)?;
                }

                for utxo in output_data {
                    let utxo_value = utxo
                        .extract::<Coin>()
                        .map_err(|_| VerifierError::BadlyTyped)?
                        .0;
                    ensure!(utxo_value > 0, VerifierError::ZeroValueCoin);
                    total_output_value = total_output_value
                        .checked_add(utxo_value)
                        .ok_or(VerifierError::ValueOverflow)?;
                }

                ensure!(
                    total_output_value <= total_input_value,
                    VerifierError::OutputsExceedInputs
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
                ensure!(input_data.is_empty(), VerifierError::MintingWithInputs);

                // Make sure there is at least one output being minted
                ensure!(!output_data.is_empty(), VerifierError::MintingNothing);

                // Make sure the outputs are the right type
                for utxo in output_data {
                    let utxo_value = utxo
                        .extract::<Coin>()
                        .map_err(|_| VerifierError::BadlyTyped)?
                        .0;
                    ensure!(utxo_value > 0, VerifierError::ZeroValueCoin);
                }

                // No priority for minting
                Ok(0)
            }
        }
    }
}

impl Verifier for MoneyVerifier {
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

        <MoneyVerifier as SimpleVerifier>::verify(self, &input_data, &output_data)
    }
}

/// Unit tests for the Money piece
#[cfg(test)]
mod test {
    use super::*;
    use tuxedo_core::dynamic_typing::testing::Bogus;

    #[test]
    fn spend_valid_transaction_work() {
        let input_data = vec![Coin(5).into(), Coin(7).into()]; // total 12
        let output_data = vec![Coin(10).into(), Coin(1).into()]; // total 11
        let expected_priority = 1u64;

        assert_eq!(
            MoneyVerifier::Spend.verify(&input_data, &output_data),
            Ok(expected_priority),
        );
    }

    #[test]
    fn spend_with_zero_value_output_fails() {
        let input_data = vec![Coin(5).into(), Coin(7).into()]; // total 12
        let output_data = vec![Coin(10).into(), Coin(1).into(), Coin(0).into()]; // total 1164;

        assert_eq!(
            MoneyVerifier::Spend.verify(&input_data, &output_data),
            Err(VerifierError::ZeroValueCoin),
        );
    }

    #[test]
    fn spend_no_outputs_is_a_burn() {
        let input_data = vec![Coin(5).into(), Coin(7).into()]; // total 12
        let output_data = vec![];
        let expected_priority = 12u64;

        assert_eq!(
            MoneyVerifier::Spend.verify(&input_data, &output_data),
            Ok(expected_priority),
        );
    }

    #[test]
    fn spend_no_inputs_fails() {
        let input_data = vec![];
        let output_data = vec![Coin(10).into(), Coin(1).into()];

        assert_eq!(
            MoneyVerifier::Spend.verify(&input_data, &output_data),
            Err(VerifierError::SpendingNothing),
        );
    }

    #[test]
    fn spend_wrong_input_type_fails() {
        let input_data = vec![Bogus.into()];
        let output_data = vec![Coin(10).into(), Coin(1).into()];

        assert_eq!(
            MoneyVerifier::Spend.verify(&input_data, &output_data),
            Err(VerifierError::BadlyTyped),
        );
    }

    #[test]
    fn spend_wrong_output_type_fails() {
        let input_data = vec![Coin(5).into(), Coin(7).into()]; // total 12
        let output_data = vec![Bogus.into()];

        assert_eq!(
            MoneyVerifier::Spend.verify(&input_data, &output_data),
            Err(VerifierError::BadlyTyped),
        );
    }

    #[test]
    fn spend_output_value_exceeds_input_value_fails() {
        let input_data = vec![Coin(10).into(), Coin(1).into()]; // total 11
        let output_data = vec![Coin(5).into(), Coin(7).into()]; // total 12

        assert_eq!(
            MoneyVerifier::Spend.verify(&input_data, &output_data),
            Err(VerifierError::OutputsExceedInputs),
        );
    }

    #[test]
    fn mint_valid_transaction_works() {
        let input_data = vec![];
        let output_data = vec![Coin(10).into(), Coin(1).into()];

        assert_eq!(MoneyVerifier::Mint.verify(&input_data, &output_data), Ok(0),);
    }

    #[test]
    fn mint_with_zero_value_output_fails() {
        let input_data = vec![];
        let output_data = vec![Coin(0).into()];

        assert_eq!(
            MoneyVerifier::Mint.verify(&input_data, &output_data),
            Err(VerifierError::ZeroValueCoin),
        );
    }

    #[test]
    fn mint_with_inputs_fails() {
        let input_data = vec![Coin(5).into()];
        let output_data = vec![Coin(10).into(), Coin(1).into()];

        assert_eq!(
            MoneyVerifier::Mint.verify(&input_data, &output_data),
            Err(VerifierError::MintingWithInputs),
        );
    }

    #[test]
    fn mint_with_no_outputs_fails() {
        let input_data = vec![];
        let output_data = vec![];

        assert_eq!(
            MoneyVerifier::Mint.verify(&input_data, &output_data),
            Err(VerifierError::MintingNothing),
        );
    }

    #[test]
    fn mint_wrong_output_type_fails() {
        let input_data = vec![];
        let output_data = vec![Coin(10).into(), Bogus.into()];

        assert_eq!(
            MoneyVerifier::Mint.verify(&input_data, &output_data),
            Err(VerifierError::BadlyTyped),
        );
    }
}
