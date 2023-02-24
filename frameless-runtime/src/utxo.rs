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

pub struct PreValidator<Piece>(PhantomData<Piece>);
impl<Piece: UtxoSet> PreValidator<Piece> {
    pub fn pre_validate(transaction: &Transaction) -> Result<(), ()> {
        {
            let input_set: BTreeSet<_> = transaction.inputs.iter().collect();
            if input_set.len() < transaction.inputs.len() {
                return Err(());
            }
        }

        for input in transaction.inputs.iter() {
            if let Some(utxo) = <Piece as UtxoSet>::peak(input.output) {
                utxo.redeemer.redeem(&transaction.encode(), &input.witness).then_some(()).ok_or(())?;
            }
            else {
                // Not handling any utxo races just fail this transaction
                return Err(())
            }
        }
        Ok(())
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
    fn insert(utxo_ref: UtxoRef, utxo: &Utxo) -> bool;

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
    type Error: Default;

    /// The validation function to determine whether a given input can be consumed.
    fn validate(transaction: Transaction) -> Result<(), Self::Error>;
}

pub struct PieceExtracter<Piece>(PhantomData<Piece>);
impl<Piece: TuxedoPiece> PieceExtracter<Piece> {
    pub fn extract(key: UtxoRef) -> Result<Piece::Data, ()> {
        let encoded_utxo = sp_io::storage::get(&key.encode()).ok_or(())?;
        let utxo = Utxo::decode(&mut &encoded_utxo[..]).map_err(|_| ())?;
        Self::extract_from_output(&utxo)
    }

    pub fn extract_from_output(utxo: &Utxo) -> Result<Piece::Data, ()> {
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

pub trait MoneyConfig {
    type Existence;
}

impl MoneyConfig for MoneyPiece {
    type Existence = ExistencePiece;
}

pub trait MoneyData {
    type Currency: Encode + Decode;
    type Existence: Encode + Decode;
}

impl MoneyData for Money {
    type Currency = u128;
    type Existence = H256;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum Money {
    Currency(u128),
    Existence(H256),
}

impl Default for Money {
    fn default() -> Self {
        Money::Currency(0)
    }
}

impl TuxedoPiece for MoneyPiece {
    type Data = Money;
    const TYPE_ID: TypeId = *b"1111";
    type Error = ();

    fn validate(transaction: Transaction) -> Result<(), Self::Error> {
        // Always pre-validate before every validate
        PreValidator::<Self>::pre_validate(&transaction)?;

        let mut total_input_value: <Self::Data as MoneyData>::Currency = 0;
        let mut total_output_value: <Self::Data as MoneyData>::Currency = 0;

        // Check that sum of input values < output values
        for input in transaction.inputs.iter() {
            let money_type = PieceExtracter::<Self>::extract(input.output)?;
            match money_type {
                Money::Currency(value) => {
                    total_input_value.checked_add(value).ok_or(())?;
                },
                Money::Existence(existence_value) => {
                    // <<Self as MoneyConfig>::Existence as Verify>::verify_input(&existence_value)
                    // TODO: Could add some helpers such that you can easily access
                    //       The state of an existence piece and its validation logic
                    // Check state of existence Piece? and validate? some glue code?
                    // Such as verify input is required for every Piece???
                },
            }
        }

        for utxo in transaction.outputs.iter() {
            let money_type = PieceExtracter::<Self>::extract_from_output(&utxo)?;
            match money_type {
                Money::Currency(value) => {
                    if value <= 0 {
                        return Err(Self::Error::default());
                    }
                    total_output_value.checked_add(value).ok_or(())?;
                },
                Money::Existence(existence_value) => {
                    // TODO: Could add some helpers such that you can easily access
                    //       The state of an existence piece and its validation logic
                    // Check state of existence Piece? and validate? some glue code?
                    // Such as verify output is required for every Piece???
                },
            }
        }

        if total_output_value < total_input_value {
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

        // TODO: construct 'ValidTransaction and return'
        Ok(())
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

pub struct ExistencePiece; // Decodes Value -> H256
impl TuxedoPiece for ExistencePiece {
    type Data = H256;
    const TYPE_ID: TypeId = *b"3333";
    type Error = ();

    fn validate(transaction: Transaction) -> Result<(), Self::Error> {
        // Check that the input is unique and a set
        // if it fails then return early
        // TODO: Implement Proof of existence Logic scenario
        Ok(())
    }
}
