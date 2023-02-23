//! An NFT game inspired by cryptokitties.

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

// Api USER defined
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum DadKittyStatus {
    #[default]
    RearinToGo,
    Tired,
}

// Api USER defined
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum MomKittyStatus {
    #[default]
    RearinToGo,
    HadBirthRecently,
}

// Api USER defined
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct KittyData {
    dad: DadKittyStatus,
    mom: MomKittyStatus,
}

pub struct KittiesPiece; // Decodes Value -> KittyData
impl TuxedoPiece for KittiesPiece {
    type Data = KittyData; // This is API user Defined
    const TYPE_ID: TypeId = *b"2222";
    type Error = ();

    fn validate(transaction: Transaction) -> Result<(), Self::Error> {
        PreValidator::<Self>::pre_validate(&transaction)?;
        // TODO: Implement Kitty Logic scenario

        // 1.) If you want to breed Mom cannot have given birth before
        // 2.) If you want to breed Dad cannot be too tired
        for input in transaction.inputs.iter() {
            let parents = PieceExtracter::<Self>::extract(input.output)?;
            Self::mom_and_dad_ready(&parents)?;
        }
        Ok(())
    }
}

impl UtxoSet for KittiesPiece {
    /// Check whether a given utxo exists in the current set
    fn contains(utxo_ref: UtxoRef) -> bool {
        sp_io::storage::exists(&utxo_ref.encode())
    }

    /// Insert the given utxo into the state storing it with the given ref
    /// The ref is probably the hash of a tx that created it and its index in that tx, but this decision is opaque to this trait
    /// Return whether the operation is successful (It can fail if the ref is already present)
    fn insert(utxo_ref: UtxoRef, utxo: &Utxo) -> bool {
        sp_io::storage::set(&utxo_ref.encode(), &utxo.encode());
        true
    }

    ///
    /// nullify the utxo by either:
    /// - Consuming it entirely
    /// - Putting it on "Timeout"
    /// - Not consuming it but marking it as spent
    ///
    fn nullify(utxo_ref: UtxoRef) -> Option<Utxo> {
        // let parents = PieceExtracter::<Self>::extract(utxo_ref)?;

        // if Self::mom_ready(&parents.mom) {

        // }
        // let encoded_utxo = sp_io::storage::get(&utxo_ref.encode())?;
        // match Utxo::decode(&mut &encoded_utxo[..]) {
        //     Ok(utxo) => Some(utxo),
        //     Err(_) => None,
        // }
        // PieceExtractor::<Self>::extract(utxo_ref)
        None
    }
}

impl KittiesPiece {
    fn mom_and_dad_ready(parents: &<Self as TuxedoPiece>::Data) -> Result<(), ()> {
        Self::mom_ready(&parents.mom).then_some(()).ok_or(())?;
        Self::dad_ready(&parents.dad).then_some(()).ok_or(())?;
        Ok(())
    }

    fn mom_ready(mom_status: &MomKittyStatus) -> bool {
        match mom_status {
            MomKittyStatus::RearinToGo => true,
            MomKittyStatus::HadBirthRecently => false,
        }
    }

    fn dad_ready(dad_status: &DadKittyStatus) -> bool {
        match dad_status {
            DadKittyStatus::RearinToGo => true,
            DadKittyStatus::Tired => false,
        }
    }
}