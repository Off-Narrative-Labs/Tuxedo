//! The common types that will be used across a Tuxedo runtime, and not specific to any one piece

// My IDE added this at some point. I'll leave it here as a reminder that maybe I don't need to
// re-invent the type-id wheel;
// use core::any::TypeId;

use sp_std::vec::Vec;
use parity_scale_codec::{Encode, Decode};
use sp_core::H256;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// A reference to a output that is expected to exist in the state.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct OutputRef {
    /// A hash of the transaction that created this output
    pub tx_hash: H256,
    /// The index of this output among all outputs created by the same transaction
    pub index: u32,
}

/// A UTXO Transaction
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Transaction<R, V> {
    pub inputs: Vec<Input>,
    //Todo peeks: Vec<Input>,
    pub outputs: Vec<Output<R>>,
    pub verifier: V,
}


/// A reference the a utxo that will be consumed along with proof that it may be consumed
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Input {
    /// a reference to the output being consumed
    pub output_ref: OutputRef,
    // Eg the signature
    pub witness: Vec<u8>,
}

/// An opaque piece of Transaction output data. This is how the data appears at the Runtime level. After
/// the redeemer is checked, strongly typed data will be extracted and passed to the verifier.
/// In a cryptocurrency, the data represents a single coin. In Tuxedo, the type of
/// the contained data is generic.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Output<R> {
    pub payload: TypedData,
    pub redeemer: R,
}

/// A piece of encoded data with a type id associated
/// Strongly typed data can be extracted
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct TypedData {
    pub data: Vec<u8>,
    pub type_id: [u8; 4],
}

/// A trait that must be implemented for any data that can be contained in a UTXO.
/// It is not recommended to implement this trait directly for primitive types, but rather to
/// use the newtype pattern: https://doc.rust-lang.org/book/ch19-04-advanced-types.html.
/// Using a new type allows strong type disambiguation between bespoke use-cases in which
/// the same primitive may be stored.
pub trait UtxoData: Encode + Decode {
    //TODO this is ugly. But at least I'm not stuck anymore.
    /// A unique identifier for this type. For now choosing this value and making sure it
    /// really is unique is the problem of the developer. Ideally this would be better.
    /// Maybe macros... Doesn't frame somehow pass info about the string in construct runtime to the pallet-level storage items?
    const TYPE_ID: [u8; 4];
}

impl TypedData {
    /// Extracts strongly typed data from an Output, iff the output contains the type of data
    /// specified. If the contained data is not the specified type, or decoding fails, this errors.
    pub fn extract<T: UtxoData>(&self) -> Result<T, ()> {
        
        // The first four bytes represent the type id that that was encoded. If they match the type
        // we are trying to decode into, we continue, otherwise we error out.
        if self.type_id == <T as UtxoData>::TYPE_ID {
            T::decode(&mut &self.data[..]).map_err(|_| ())
        } else {
            Err(())
        }
    }
}
