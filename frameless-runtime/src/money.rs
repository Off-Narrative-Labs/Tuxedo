//! An implementation of a simple fungible token.


use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{
	H256,
	H512,
	sr25519::{Public, Signature},
};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
	transaction_validity::{TransactionLongevity, ValidTransaction},
};
use sp_std::marker::PhantomData;

use log::info;
use crate::utxo::{TuxedoPiece, TypeId, Transaction, PreValidator, PieceExtracter, UtxoSet, UtxoRef, Utxo};


#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum MoneyPiece { // Decodes Value -> u128
    /// A typical spend transaction where some coins are consumed and others are created.
    Spend,
    /// A mint transaction that creates no coins out of the void. In a real-world chain,
    /// this should be protected somehow, or not included at all. For now it is publicly
    /// available. I'm adding it to explore multiple validation paths in a single piece.
    Mint,
}

impl UtxoSet for MoneyPiece {
    fn contains(utxo_ref: UtxoRef) -> bool {
        sp_io::storage::exists(&utxo_ref.encode())
    }

    fn insert(utxo_ref: UtxoRef, utxo: &Utxo) -> bool {
        sp_io::storage::set(&utxo_ref.encode(), &utxo.encode());
        true
    }

    /// For the Money UTXO we just want to consume it.
    fn nullify(utxo_ref: UtxoRef) -> Option<Utxo> {
        let encoded_utxo = sp_io::storage::get(&utxo_ref.encode())?;
        sp_io::storage::clear(&utxo_ref.encode());
        match Utxo::decode(&mut &encoded_utxo[..]) {
            Ok(utxo) => Some(utxo),
            Err(_) => None,
        }
    }
}

impl TuxedoPiece for MoneyPiece {
    type Data = u128;
    const TYPE_ID: TypeId = *b"1111";
    type Error = ();

    fn validate(&self, transaction: Transaction) -> Result<(), Self::Error> {
        
        // Always pre-validate before every validate
        PreValidator::<Self>::pre_validate(&transaction)?;

        match &self {
            Self::Spend => {

                let mut total_input_value: Self::Data = 0;
                let mut total_output_value: Self::Data = 0;

                // Check that sum of input values < output values
                for input in transaction.inputs.iter() {
                    let utxo_value = PieceExtracter::<Self>::extract(input.output)?;
                    total_input_value.checked_add(utxo_value).ok_or(())?;
                }

                for utxo in transaction.outputs.iter() {
                    let utxo_value = PieceExtracter::<Self>::extract_from_output(&utxo)?;
                    if utxo_value <= 0 {
                        return Err(Self::Error::default());
                    }
                    total_output_value.checked_add(utxo_value).ok_or(())?;
                }

                if total_output_value > total_input_value {
                    return Err(Self::Error::default());
                }

                // Update storage
                let mut transaction = transaction;
                for input in transaction.inputs.iter_mut() {
                    let _ = Self::nullify(input.output);
                    // strip the signature off the transaction for deterministic keys
                    input.witness = vec![];
                }

                let mut output_index: u32 = 0;
                for utxo in transaction.outputs.iter() {
                    let new_utxo_ref = BlakeTwo256::hash_of(&(&transaction, output_index));
                    let _ = Self::insert(new_utxo_ref, utxo);
                    output_index.checked_add(1).ok_or(())?;
                }
            },
            Self::Mint => {
                // Make sure there are no inputs being consumed
                if !transaction.inputs.is_empty() {
                    return Err(());
                }

                let mut output_index: u32 = 0;
                for utxo in transaction.outputs.iter() {
                    let new_utxo_ref = BlakeTwo256::hash_of(&(&transaction, output_index));
                    let _ = Self::insert(new_utxo_ref, utxo);
                    output_index.checked_add(1).ok_or(())?;
                }
            }
        };


        // TODO: construct 'ValidTransaction and return'
        Ok(())
    }
}
