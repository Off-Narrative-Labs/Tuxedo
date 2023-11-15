//! This module provides the majority of Tuxedo's parachain support.
//! It's primary jobs are to recieve the parachain inehrent,
//! provide collation information to the client side collator service,
//! and implement the `validate_block` funtion required by relay chain validators
//!
//! This is mostly copied and stripped down from cumulus pallet parachain system
//! https://paritytech.github.io/polkadot-sdk/master/cumulus_pallet_parachain_system/index.html
//!
//! Better docs coming after this takes shape. For now it is hack'n'slash.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;
#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub mod validate_block;

mod collation_api;
mod relay_state_snapshot;
pub use collation_api::ParachainExecutiveExtension;
use parity_scale_codec::{Decode, Encode};

#[cfg(not(feature = "std"))]
#[doc(hidden)]
mod trie_cache;

#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub use bytes;
#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub use parity_scale_codec::decode_from_bytes;
#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub use polkadot_parachain_primitives;
#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub use sp_runtime::traits::GetRuntimeBlockType;
#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub use sp_std;

/// Re-export of the Tuxedo-core crates. This allows parachain-specific
/// Tuxedo-pieces to depend only on tuxedo-parachain-core without worrying about
/// accidental version mismatches.
pub use tuxedo_core;

use cumulus_primitives_parachain_inherent::ParachainInherentData;
use tuxedo_core::{
    dynamic_typing::UtxoData,
    support_macros::{CloneNoBound, DebugNoBound},
};

/// A transient storage key that will hold the block number of the relay chain parent
/// that is associated with the current parachain block. This data enters the parachain
/// through the parachain inherent
const RELAY_PARENT_NUMBER_KEY: &[u8] = b"relay_parent_number";

/// A public interface for accessing and mutating the relay parent number. This is
/// expected to be called from the parachain piece
pub enum RelayParentNumberStorage {}

/// An abstraction over reading the ambiently available relay parent number.
/// This allows it to be mocked during tests and require actual externalities.
pub trait GetRelayParentNumberStorage {
    fn get() -> u32;
}

impl GetRelayParentNumberStorage for RelayParentNumberStorage {
    fn get() -> u32 {
        let encoded = sp_io::storage::get(RELAY_PARENT_NUMBER_KEY)
            .expect("Some relay parent number should always be stored");
        Decode::decode(&mut &encoded[..])
            .expect("properly encoded relay parent number should have been stored.")
    }
}

/// An abstraction over setting the ambiently available relay parent number.
/// This allows it to be mocked during tests and require actual externalities.
pub trait SetRelayParentNumberStorage {
    fn set(new_parent_number: u32);
}

impl SetRelayParentNumberStorage for RelayParentNumberStorage {
    fn set(new_parent_number: u32) {
        sp_io::storage::set(RELAY_PARENT_NUMBER_KEY, &new_parent_number.encode());
    }
}

/// Basically the same as
/// [`ValidationParams`](polkadot_parachain_primitives::primitives::ValidationParams), but a little
/// bit optimized for our use case here.
///
/// `block_data` and `head_data` are represented as [`bytes::Bytes`] to make them reuse
/// the memory of the input parameter of the exported `validate_blocks` function.
///
/// The layout of this type must match exactly the layout of
/// [`ValidationParams`](polkadot_parachain_primitives::primitives::ValidationParams) to have the
/// same SCALE encoding.
#[derive(parity_scale_codec::Decode)]
#[cfg_attr(feature = "std", derive(parity_scale_codec::Encode))]
#[doc(hidden)]
pub struct MemoryOptimizedValidationParams {
    pub parent_head: bytes::Bytes,
    pub block_data: bytes::Bytes,
    pub relay_parent_number: cumulus_primitives_core::relay_chain::BlockNumber,
    pub relay_parent_storage_root: cumulus_primitives_core::relay_chain::Hash,
}

/// Register the `validate_block` function that is used by parachains to validate blocks on a
/// validator.
///
/// Does *nothing* when `std` feature is enabled.
///
/// Expects as parameters the Block type, the OuterVerifier, and the OuterConstraintChecker.
pub use tuxedo_register_validate_block::register_validate_block;

/// A wrapper type around Cumulus's ParachainInherentData ype that can be stored.
/// Having to do this wrapping is one more reason to abandon this UtxoData trait,
/// and go for a more strongly typed aggregate type approach.
#[derive(Encode, Decode, DebugNoBound, CloneNoBound, scale_info::TypeInfo)]

/// A wrapper type around Cumulus's ParachainInherentData type.
/// This type is convertable Into and From the inner type.
/// This is necessary so that we can implement the `UtxoData` trait.
pub struct ParachainInherentDataUtxo(ParachainInherentData);

impl UtxoData for ParachainInherentDataUtxo {
    const TYPE_ID: [u8; 4] = *b"para";
}

impl From<ParachainInherentDataUtxo> for ParachainInherentData {
    fn from(val: ParachainInherentDataUtxo) -> Self {
        val.0
    }
}

impl From<ParachainInherentData> for ParachainInherentDataUtxo {
    fn from(value: ParachainInherentData) -> Self {
        Self(value)
    }
}
