//! An implementation of a simple fungible token.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::prelude::*;
use tuxedo_core::{
    ensure,
    types::{TypedData, UtxoData},
    Verifier,
};

// use log::info;

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum MoneyVerifier {
    /// A typical spend transaction where some coins are consumed and others are created.
    Spend,
    /// A mint transaction that creates no coins out of the void. In a real-world chain,
    /// this should be protected somehow, or not included at all. For now it is publicly
    /// available. I'm adding it to explore multiple validation paths in a single piece.
    Mint,
}

/// A single coin in the fungible money system.
/// A new type wrapper around a u128 value.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Coin(u128);

impl UtxoData for Coin {
    const TYPE_ID: [u8; 4] = *b"coin";
}

/// TODO better error type
pub type VerifierError = ();
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
// #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
// pub enum VerifierError{
//     BadlyType
//     OutputsExceedInputs
// }

impl Verifier for MoneyVerifier {
    type Error = VerifierError;

    fn verify(
        &self,
        input_data: &[TypedData],
        output_data: &[TypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        match &self {
            Self::Spend => {
                let mut total_input_value: u128 = 0;
                let mut total_output_value: u128 = 0;

                // Check that sum of input values < output values
                for input in input_data {
                    let utxo_value = input.extract::<Coin>()?.0;
                    total_input_value = total_input_value.checked_add(utxo_value).ok_or(())?;
                }

                for utxo in output_data {
                    let utxo_value = utxo.extract::<Coin>()?.0;
                    if utxo_value <= 0 {
                        return Err(Self::Error::default());
                    }
                    total_output_value = total_output_value.checked_add(utxo_value).ok_or(())?;
                }

                if total_output_value > total_input_value {
                    return Err(Self::Error::default());
                }

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
                if !input_data.is_empty() {
                    return Err(());
                }

                // Make sure there is at least one output being minted
                if output_data.is_empty() {
                    return Err(());
                }

                // Make sure the outputs are the right type
                for utxo in output_data {
                    let utxo_value = utxo.extract::<Coin>()?.0;
                    if utxo_value <= 0 {
                        return Err(Self::Error::default());
                    }
                }

                // No priority for minting
                Ok(0)
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    /// A bogus data type used in tests for type validation
    #[derive(Encode, Decode)]
    struct Bogus;

    impl UtxoData for Bogus {
        const TYPE_ID: [u8; 4] = *b"bogs";
    }

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
            Err(()),
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
            Err(()),
        );
    }

    #[test]
    fn spend_wrong_input_type_fails() {
        let input_data = vec![Bogus.into()];
        let output_data = vec![Coin(10).into(), Coin(1).into()];

        assert_eq!(
            MoneyVerifier::Spend.verify(&input_data, &output_data),
            Err(()),
        );
    }

    #[test]
    fn spend_wrong_output_type_fails() {
        let input_data = vec![Coin(5).into(), Coin(7).into()]; // total 12
        let output_data = vec![Bogus.into()];

        assert_eq!(
            MoneyVerifier::Spend.verify(&input_data, &output_data),
            Err(()),
        );
    }

    #[test]
    fn spend_output_value_exceeds_input_value_fails() {
        let input_data = vec![Coin(10).into(), Coin(1).into()]; // total 11
        let output_data = vec![Coin(5).into(), Coin(7).into()]; // total 12

        assert_eq!(
            MoneyVerifier::Spend.verify(&input_data, &output_data),
            Err(()),
        );
    }

    #[test]
    fn mint_valid_transaction_works() {
        let input_data = vec![];
        let output_data = vec![Coin(10).into(), Coin(1).into()];

        assert_eq!(
            MoneyVerifier::Mint.verify(&input_data, &output_data),
            Ok(0),
        );
    }

    #[test]
    fn mint_with_zero_value_output_fails() {
        let input_data = vec![];
        let output_data = vec![Coin(0).into()];

        assert_eq!(
            MoneyVerifier::Mint.verify(&input_data, &output_data),
            Err(()),
        );
    }

    #[test]
    fn mint_with_inputs_fails() {
        let input_data = vec![Coin(5).into()];
        let output_data = vec![Coin(10).into(), Coin(1).into()];

        assert_eq!(
            MoneyVerifier::Mint.verify(&input_data, &output_data),
            Err(()),
        );
    }

    #[test]
    fn mint_with_no_outputs_fails() {
        let input_data = vec![];
        let output_data = vec![];

        assert_eq!(
            MoneyVerifier::Mint.verify(&input_data, &output_data),
            Err(()),
        );
    }

    #[test]
    fn mint_wrong_output_type_fails() {
        let input_data = vec![];
        let output_data = vec![Coin(10).into(), Bogus.into()];

        assert_eq!(
            MoneyVerifier::Mint.verify(&input_data, &output_data),
            Err(()),
        );
    }
}