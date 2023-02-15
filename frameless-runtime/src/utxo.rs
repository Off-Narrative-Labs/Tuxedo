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

use log::info;

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

// pub type DispatchResult = Result<(), sp_runtime::DispatchError>;
// Temporary should probably move to something like this above ^^
pub type DispatchResult = Result<(), ()>;

/// A single input references the output to be consumed or peeked at and provides some witness data, possibly a signature.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Input {
    /// A previously created output that will be consumed by the transaction containing this input.
    output: OutputRef,
    /// A witness proving that the output can be consumed by this input. In many cases including that of a basic cryptocurrency, this will be a digital signature.
    redeemer: Sig,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Output {
    /// The address that owns this output. Based on either a public key or a Tuxedo Piece
    pub owner: Address,
    /// The data associated with this output. In the simplest case, this will be a token balance, but could be arbitrarily rich state.
    pub data: Value,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct Transaction {
    /// The inputs refer to currently existing unspent outputs that will be consumed by this transaction
    pub inputs: BTreeSet<Input>,
    /// Similar to inputs, Peeks refer to currently existing utxos, but they will be read only, and not consumed
    pub peeks: BTreeSet<Input>,
    /// The new outputs to be created by this transaction.
    pub outputs: Vec<Output>,
}

pub type Utxo = Output;
pub type UtxoRef = OutputRef;

// TODO: Implement this on Each Tuxedo Piece??
pub trait UtxoSet {
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
}

/// The API of a Tuxedo Piece
pub trait TuxedoPiece {
    /// The type of data stored in Outputs associated with this Piece
    type Data: Encode + Decode;

    /// The validation function to determine whether a given input can be consumed.
    fn validate(transaction: Transaction) -> bool;
}

// User defined logic below for the STF..

pub struct MoneyPiece; // Decodes Value -> u128
impl TuxedoPiece for MoneyPiece {
    type Data = u128;

    fn validate(transaction: Transaction) -> bool {
        // decode the transaction output data as type `Data`
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

    fn validate(transaction: Transaction) -> bool {
        // decode transaction output data as type 'Data'
        // if it fails then return early
        // TODO: Implement Kitty Logic scenario
        true
    }
}

pub struct ExistencePiece; // Decodes Value -> H256
impl TuxedoPiece for ExistencePiece {
    type Data = H256;

    fn validate(transaction: Transaction) -> bool {
        // decode transaction output data as type 'Data'
        // if it fails then return early
        // TODO: Implement Proof of existence Logic scenario
        true
    }
}







