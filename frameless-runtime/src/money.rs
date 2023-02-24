//! An implementation of a simple fungible token.

use tuxedo_core::{ensure, Verifier, types::{TypedData, UtxoData}};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;
use sp_runtime::{
	transaction_validity::{TransactionPriority},
};

// use log::info;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
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
                Ok(if burned < u64::max_value() as u128 {burned as u64} else {u64::max_value()})
            },
            Self::Mint => {
                // Make sure there are no inputs being consumed
                if !input_data.is_empty() {
                    return Err(());
                }

                // No priority for minting
                Ok(0)
            }
        }
    }
}
