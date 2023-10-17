//! Dynamic Typing utilities for Tuxedo runtimes
//!
//! # Motivation
//!
//! In Tuxedo, UTXOs are like envelopes that store to later be opened and computed on.
//! These data can be of many different types depending on which Tuxedo pieces may
//! operate on them. And different Tuxedo runtimes may support different data types.
//!
//! In order to support type safety as well as serialization, we must do some dynamic
//! type checking when a UTXO is taken out of storage to ensure that the data is decoded
//! to the same type from which it was encoded. This is important because it will occasionally
//! be the case that serialized data may validly decode to more than one type.
//!
//! # Example Attack
//!
//! To understand the importance of the type checking, consider a concrete Tuxedo runtime that
//! has both a bitcoin-like cryptocurrency and a crypto-kitties-like NFT game. Imagine a user
//! spends a moderate amount of money breeding cryptokitties until they have an Attack Kitty.
//! And Attack Kitty is one whose serialized data has the property that it would also validly
//! decode into a coin in the cryptocurrency and spent accordingly. In the worst case (best for
//! the attacker) the value of the coin would exceed the value spent breeding the cryptokitties.
//!
//! # Methodology
//!
//! To solve this problem we associate a four-byte type identifier with each data type that can
//! be stored in a UTXO. When a UTXO is stored, the type identifier is stored along with the
//! serialized data. When the UTXO is later read from storage, the type identifier is checked
//! against the type into which the data is being decoded. Currently this read-time checking
//! is the job of the piece developer, although that may be able to improve in the future.
//!
//! # Comparison with `sp_std::any`
//!
//! The Rust standard library, and also the `sp-std` crate offer utilities for dynamic typing
//! as well. We have considered and are still considering using that crate instead of these
//! custom utilities.
//!
//! ## In favor of `sp_std::any`
//!
//! * The compiler guarantees unique type ids for every type, whereas this utility
//!   requires the developer to avoid collisions (hopefully macros can improve this slightly)
//! * Using that crate would be less code for Tuxedo developers to maintain
//!
//! ## In favor of this custom utility
//!
//! * `sp_std::any` does not make the type_id public. This makes it impossible to encode it
//!   and store it along with the serialized data which is the whole point. This could be _hacked_
//!   around by, for example, hashing the Debug strong, but that is ugly

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

/// A piece of encoded data with a type id associated
/// Strongly typed data can be extracted
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct DynamicallyTypedData {
    pub data: Vec<u8>,
    pub type_id: [u8; 4],
}

/// A trait that must be implemented for any data that can be contained in a UTXO.
/// It is not recommended to implement this trait directly for primitive types, but rather to
/// use the newtype pattern: https://doc.rust-lang.org/book/ch19-04-advanced-types.html.
/// Using a new type allows strong type disambiguation between bespoke use-cases in which
/// the same primitive may be stored.
pub trait UtxoData: Encode + Decode {
    //TODO Not great that it is up to the runtime dev to enforce uniqueness
    // Maybe macros can help... Doesn't frame somehow pass info about the string in construct runtime to the pallet-level storage items?
    /// A unique identifier for this type. For now choosing this value and making sure it
    /// really is unique is the problem of the developer. Ideally this would be better.
    const TYPE_ID: [u8; 4];
}

impl DynamicallyTypedData {
    /// Extracts strongly typed data from an Output, iff the output contains the type of data
    /// specified. If the contained data is not the specified type, or decoding fails, this errors.
    pub fn extract<T: UtxoData>(&self) -> Result<T, DynamicTypingError> {
        // The first four bytes represent the type id that that was encoded. If they match the type
        // we are trying to decode into, we continue, otherwise we error out.
        if self.type_id == <T as UtxoData>::TYPE_ID {
            T::decode(&mut &self.data[..]).map_err(|_| DynamicTypingError::DecodingFailed)
        } else {
            Err(DynamicTypingError::WrongType)
        }
    }
}

/// Errors that can occur when casting dynamically typed data into strongly typed data.
#[derive(Debug, PartialEq, Eq)]
pub enum DynamicTypingError {
    /// The data provided was not of the target decoding type.
    WrongType,
    /// Although the types matched, the data could not be decoded with the SCALE codec.
    DecodingFailed,
}

impl sp_std::fmt::Display for DynamicTypingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::WrongType => write!(f, "dynamic type does not match extraction target"),
            Self::DecodingFailed => write!(
                f,
                "failed to decode dynamically typed data with scale codec"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DynamicTypingError {}

//TODO, I tried replacing the extract method above with this impl,
// but it conflicts with something in core, that I don't understand.
// Extracts strongly typed data from dynamically typed data.
// impl<U: UtxoData> TryInto<U> for TypedData {
//     type Error = DynamicTypingError;

//     fn try_into(self) -> Result<U, Self::Error> {
//         todo!()
//     }
// }

// Packages strongly typed data with a dynamic typing tag
// probably for storage in a UTXO `Output`.
impl<T: UtxoData> From<T> for DynamicallyTypedData {
    fn from(value: T) -> Self {
        Self {
            data: value.encode(),
            type_id: T::TYPE_ID,
        }
    }
}

pub mod testing {
    use super::*;

    /// A bogus data type for use in tests.
    ///
    /// When writing tests for individual Tuxedo pieces, developers
    /// need to make sure that the piece properly sanitizes the dynamically
    /// typed data that is passed into its verifiers.
    /// This type is used to represent incorrectly typed data.
    #[derive(Encode, Decode, PartialEq, Eq, Debug)]
    pub struct Bogus;

    impl UtxoData for Bogus {
        const TYPE_ID: [u8; 4] = *b"bogs";
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testing::Bogus;

    /// A simple type that implements UtxoData and just wraps a single u8.
    /// Used to test the extraction logic.
    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
    struct Byte(u8);

    impl UtxoData for Byte {
        const TYPE_ID: [u8; 4] = *b"byte";
    }

    #[test]
    fn extract_works() {
        let original_b = Byte(4);
        let dynamically_typed_b: DynamicallyTypedData = original_b.clone().into();

        let extracted_b = dynamically_typed_b.extract::<Byte>();

        assert_eq!(extracted_b, Ok(original_b));
    }

    #[test]
    fn extract_wrong_type() {
        let original_b = Byte(4);
        let dynamically_typed_b: DynamicallyTypedData = original_b.clone().into();

        let extracted_b = dynamically_typed_b.extract::<Bogus>();

        assert_eq!(extracted_b, Err(DynamicTypingError::WrongType));
    }

    #[test]
    fn extract_decode_fails() {
        let original_b = Byte(4);
        let mut dynamically_typed_b: DynamicallyTypedData = original_b.clone().into();

        // Change the encoded bytes so they no longer decode correctly.
        dynamically_typed_b.data = Vec::new();

        let extracted_b = dynamically_typed_b.extract::<Byte>();

        assert_eq!(extracted_b, Err(DynamicTypingError::DecodingFailed));
    }

    #[test]
    fn display_wrong_type_error() {
        let actual = format!("{}", DynamicTypingError::WrongType);
        let expected = String::from("dynamic type does not match extraction target");

        assert_eq!(actual, expected);
    }

    #[test]
    fn display_decoding_failed_error() {
        let actual = format!("{}", DynamicTypingError::DecodingFailed);
        let expected = String::from("failed to decode dynamically typed data with scale codec");

        assert_eq!(actual, expected);
    }
}
