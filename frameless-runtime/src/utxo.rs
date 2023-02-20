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

/// TODO: Clean up this file and organize different parts into different modules for easier reading.

///
/// TODO: Something similar to construct_runtime! which will setup all the configurations for a UTXO runtime
/// construct_utxo_runtime!(
///     MoneyUTXO // Configuration for MoneyUTXO
///     KittiesUTXO // Configuration for KittiesUTXO
///     ExistenceUTXO // Configuration for ExistenceUTXO
/// )
///

// TODO: Configurable maybe when configuring overall UTXO Runtime?
// For now hardcoded
pub type OutputRef = H256;
pub type Address = H256;
pub type Value = Vec<u8>;
pub type Sig = Vec<u8>;
pub type TypeId = [u8; 4];
pub type Redeemer = sp_core::H256;

// pub type DispatchResult = Result<(), sp_runtime::DispatchError>;
// Temporary should probably move to something like this above ^^
pub type DispatchResult = Result<(), ()>;

/// A single input references the output to be consumed or peeked at and provides some witness data, possibly a signature.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Input {
    /// A previously created output that will be consumed by the transaction containing this input.
    pub output: OutputRef,
    /// A witness proving that the output can be consumed by this input. In many cases including that of a basic cryptocurrency, this will be a digital signature.
    pub witness: Sig,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Output {
    /// The address that owns this output. Based on either a public key or a Tuxedo Piece
    pub redeemer: Redeemer,
    /// The data associated with this output. In the simplest case, this will be a token balance, but could be arbitrarily rich state.
    pub data: Value,
    /// An Id for this type Such that we know how to encode or decode the 'data' field
    pub data_id: TypeId,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Transaction {
    /// The inputs refer to currently existing unspent outputs that will be consumed by this transaction
    pub inputs: Vec<Input>,
    /// Similar to inputs, Peeks refer to currently existing utxos, but they will be read only, and not consumed
    pub peeks: Option<Vec<Input>>,
    /// The new outputs to be created by this transaction.
    pub outputs: Vec<Output>,
}

pub type Utxo = Output;
pub type UtxoRef = OutputRef;

pub trait Redeem {
    fn redeem(self, tx: &[u8], witness: &[u8]) -> bool;
}

impl Redeem for Redeemer {
    fn redeem(self, tx: &[u8], witness: &[u8]) -> bool {
        let signature = match Signature::try_from(&witness[..]) {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        sp_io::crypto::sr25519_verify(&signature, &tx, &Public::from_h256(self))
    }
}

// TODO: Implement this for Each Tuxedo Piece
pub trait UtxoSet {

    /// TODO: Change these bool return types to Result types for more error propagation clarity

    /// Check whether a given utxo exists in the current set
    fn contains(utxo_ref: UtxoRef) -> bool;

    /// Insert the given utxo into the state storing it with the given ref
    /// The ref is probably the hash of a tx that created it and its index in that tx, but this decision is opaque to this trait
    /// Return whether the operation is successful (It can fail if the ref is already present)
    fn insert(utxo_ref: UtxoRef, utxo: Utxo) -> bool;

    ///
    /// nullify the utxo by either:
    /// - Consuming it entirely
    /// - Putting it on "Timeout"
    /// - Not consuming it but marking it as spent
    ///
    fn nullify(utxo_ref: UtxoRef) -> Option<Utxo>;

    fn peak(utxo_ref: UtxoRef) -> Option<Utxo> {
        let encoded_utxo = sp_io::storage::get(&utxo_ref.encode())?;
        match Utxo::decode(&mut &encoded_utxo[..]) {
            Ok(utxo) => Some(utxo),
            Err(_) => None,
        }
    }
}

pub trait Get<T> {
    fn get(&self) -> T;
}

/// The API of a Tuxedo Piece
pub trait TuxedoPiece {
    /// The type of data stored in Outputs associated with this Piece
    type Data: Encode + Decode;
    const TYPE_ID: TypeId;

    /// The validation function to determine whether a given input can be consumed.
    fn validate(transaction: Transaction) -> bool;
}

pub struct PieceExtracter<Piece>(PhantomData<Piece>);
impl<Piece: TuxedoPiece> PieceExtracter<Piece> {
    pub fn extract(key: UtxoRef) -> Result<Piece::Data, ()> {
        let encoded_utxo = sp_io::storage::get(&key.encode()).ok_or(())?;
        let utxo = Utxo::decode(&mut &encoded_utxo[..]).map_err(|_| ())?;
        if utxo.data_id != Piece::TYPE_ID {
            return Err(())
        }
        let piece_data = Piece::Data::decode(&mut &utxo.data[..]).map_err(|_| ())?;
        Ok(piece_data)
    }
}

// User defined logic below for the STF..

pub struct MoneyPiece; // Decodes Value -> u128
impl UtxoSet for MoneyPiece {
    fn contains(utxo_ref: UtxoRef) -> bool {
        sp_io::storage::exists(&utxo_ref.encode())
    }

    fn insert(utxo_ref: UtxoRef, utxo: Utxo) -> bool {
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

    fn validate(transaction: Transaction) -> bool {
        // Check that the input is unique and a set
        {
            let input_set: BTreeSet<_> = transaction.inputs.iter().collect();
            if input_set.len() < transaction.inputs.len() {
                return false;
            }
        }

        // 1.) Check that the outputs referenced by each input can be redeemed
        // 2.) Check that sum of input values < output values
        for input in transaction.inputs.iter() {
            if let Some(utxo) = <Self as UtxoSet>::peak(input.output) {
                let _ = utxo.redeemer.redeem(&transaction.encode(), &input.witness);
                // TODO: check that the redeem passes
                // Sum up inputs
            }
        }

        // if it fails then return early
        // TODO: Implement Money situation
        true
    }
}

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

    fn validate(transaction: Transaction) -> bool {
        // Check that the input is unique and a set
        // if it fails then return early
        // TODO: Implement Kitty Logic scenario
        true
    }
}

pub struct ExistencePiece; // Decodes Value -> H256
impl TuxedoPiece for ExistencePiece {
    type Data = H256;
    const TYPE_ID: TypeId = *b"3333";

    fn validate(transaction: Transaction) -> bool {
        // Check that the input is unique and a set
        // if it fails then return early
        // TODO: Implement Proof of existence Logic scenario
        true
    }
}
