//! An implementation that supports multiple simple fungible tokens.
//! 
//! Each token behaves similarly to the the single token from the Money,
//! piece as both implement the `Fungible` trait.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::prelude::*;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, SimpleConstraintChecker,
};

/// The main constraint checker for the multitoken. It allows sending tokens of a single type.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct SpendToken;

// Idea, we could have a spend-multi where tokens of all kinds can be passed in
// and the input > output constraint is enforced for each token.

/// A development constraint checker that allows minting tokens of a single type.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct MintToken;

/// A single Token in the fungible multitoken system.
/// We know what type of token this is at the type level through the generic const.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Token {
    /// The unique ID for this kind of token.
    pub id: u8,
    /// The value of this token.
    pub value: u128,
}

impl UtxoData for Token {
    const TYPE_ID: [u8; 4] = *b"tokn";
}

/// Errors that can occur when checking multimoney transactions.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum MultiTokenError {
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
    /// This transaction contains tokens of different ids.
    MixedTokenIDs,
}

impl SimpleConstraintChecker for SpendToken {
    type Error = MultiTokenError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        
        // Check that we are consuming at least one input
        ensure!(
            !input_data.is_empty(),
            MultiTokenError::SpendingNothing
        );

        let mut token_id = None;
        let mut total_input_value: u128 = 0;
        let mut total_output_value: u128 = 0;

        // Check that sum of input values < output values
        // and every token is of the same id
        for input in input_data {
            let token = input
                .extract::<Token>()
                .map_err(|_| MultiTokenError::BadlyTyped)?;
            if token_id.is_none() {
                token_id = Some(token.id);
            } else {
                ensure!(token_id == Some(token.id), MultiTokenError::MixedTokenIDs);
            }
            total_input_value = total_input_value
                .checked_add(token.value)
                .ok_or(MultiTokenError::ValueOverflow)?;
        }

        for utxo in output_data {
            let token = utxo
                .extract::<Token>()
                .map_err(|_| MultiTokenError::BadlyTyped)?;
            ensure!(token.value > 0, MultiTokenError::ZeroValueCoin);
            ensure!(token_id == Some(token.id), MultiTokenError::MixedTokenIDs);
            total_output_value = total_output_value
                .checked_add(token.value)
                .ok_or(MultiTokenError::ValueOverflow)?;
        }

        ensure!(
            total_output_value <= total_input_value,
            MultiTokenError::OutputsExceedInputs
        );

        // Priority is based on how many token are burned
        // Type stuff is kinda ugly. Maybe division would be better?
        // TODO add a configuration item that specifies the relative priority
        // for each transaction type.
        let burned = total_input_value - total_output_value;
        Ok(if burned < u64::max_value() as u128 {
            burned as u64
        } else {
            u64::max_value()
        })
    }
}

/// Unit tests for the Money piece
#[cfg(test)]
mod test {
    use super::*;
    use tuxedo_core::dynamic_typing::testing::Bogus;

    /// Helper function to create a new token of id 0
    fn token_0(value: u128) -> Token {
        Token {
            id: 0,
            value,
        }
    }

    /// Helper function to create a new token of id 1
    fn token_1(value: u128) -> Token {
        Token {
            id: 1,
            value,
        }
    }

    #[test]
    fn spend_valid_transaction_work() {
        let input_data = vec![token_0(5).into(), token_0(7).into()]; // total 12
        let output_data = vec![token_0(10).into(), token_0(1).into()]; // total 11
        let expected_priority = 1u64;

        assert_eq!(
            SpendToken.check(&input_data, &output_data),
            Ok(expected_priority),
        );
    }

    #[test]
    fn spend_with_zero_value_output_fails() {
        let input_data = vec![token_0(5).into(), token_0(7).into()]; // total 12
        let output_data = vec![token_0(10).into(), token_0(1).into(), token_0(0).into()]; // total 1164;

        assert_eq!(
            SpendToken.check(&input_data, &output_data),
            Err(MultiTokenError::ZeroValueCoin),
        );
    }

    #[test]
    fn spend_no_outputs_is_a_burn() {
        let input_data = vec![token_0(5).into(), token_0(7).into()]; // total 12
        let output_data = vec![];
        let expected_priority = 12u64;

        assert_eq!(
            SpendToken.check(&input_data, &output_data),
            Ok(expected_priority),
        );
    }

    #[test]
    fn spend_no_inputs_fails() {
        let input_data = vec![];
        let output_data = vec![token_0(10).into(), token_0(1).into()];

        assert_eq!(
            SpendToken.check(&input_data, &output_data),
            Err(MultiTokenError::SpendingNothing),
        );
    }

    #[test]
    fn spend_wrong_input_type_fails() {
        let input_data = vec![Bogus.into()];
        let output_data = vec![token_0(10).into(), token_0(1).into()];

        assert_eq!(
            SpendToken.check(&input_data, &output_data),
            Err(MultiTokenError::BadlyTyped),
        );
    }

    #[test]
    fn spend_wrong_output_type_fails() {
        let input_data = vec![token_0(5).into(), token_0(7).into()]; // total 12
        let output_data = vec![Bogus.into()];

        assert_eq!(
            SpendToken.check(&input_data, &output_data),
            Err(MultiTokenError::BadlyTyped),
        );
    }

    #[test]
    fn spend_output_value_exceeds_input_value_fails() {
        let input_data = vec![token_0(10).into(), token_0(1).into()]; // total 11
        let output_data = vec![token_0(5).into(), token_0(7).into()]; // total 12

        assert_eq!(
            SpendToken.check(&input_data, &output_data),
            Err(MultiTokenError::OutputsExceedInputs),
        );
    }

    #[test]
    fn spend_mixed_input_types_fails() {
        let input_data = vec![token_0(10).into(), token_1(1).into()];
        let output_data = vec![token_0(5).into(), token_0(7).into()];

        assert_eq!(
            SpendToken.check(&input_data, &output_data),
            Err(MultiTokenError::MixedTokenIDs),
        );
    }

    #[test]
    fn spend_mixed_output_types_fails() {
        let input_data = vec![token_0(10).into(), token_0(1).into()];
        let output_data = vec![token_0(5).into(), token_1(7).into()];

        assert_eq!(
            SpendToken.check(&input_data, &output_data),
            Err(MultiTokenError::MixedTokenIDs),
        );
    }

    // #[test]
    // fn mint_valid_transaction_works() {
    //     let input_data = vec![];
    //     let output_data = vec![token_0(10).into(), token_0(1).into()];

    //     assert_eq!(
    //         MoneyConstraintChecker::Mint.check(&input_data, &output_data),
    //         Ok(0),
    //     );
    // }

    // #[test]
    // fn mint_with_zero_value_output_fails() {
    //     let input_data = vec![];
    //     let output_data = vec![token_0(0).into()];

    //     assert_eq!(
    //         MoneyConstraintChecker::Mint.check(&input_data, &output_data),
    //         Err(MultiTokenError::ZeroValueCoin),
    //     );
    // }

    // #[test]
    // fn mint_with_inputs_fails() {
    //     let input_data = vec![token_0(5).into()];
    //     let output_data = vec![token_0(10).into(), token_0(1).into()];

    //     assert_eq!(
    //         MoneyConstraintChecker::Mint.check(&input_data, &output_data),
    //         Err(MultiTokenError::MintingWithInputs),
    //     );
    // }

    // #[test]
    // fn mint_with_no_outputs_fails() {
    //     let input_data = vec![];
    //     let output_data = vec![];

    //     assert_eq!(
    //         MoneyConstraintChecker::Mint.check(&input_data, &output_data),
    //         Err(MultiTokenError::MintingNothing),
    //     );
    // }

    // #[test]
    // fn mint_wrong_output_type_fails() {
    //     let input_data = vec![];
    //     let output_data = vec![token_0(10).into(), Bogus.into()];

    //     assert_eq!(
    //         MoneyConstraintChecker::Mint.check(&input_data, &output_data),
    //         Err(MultiTokenError::BadlyTyped),
    //     );
    // }
}
