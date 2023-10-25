//! This module provides the majority of Tuxedo's parachain support.
//! It's primary jobs are to recieve the parachain inehrent,
//! provide collation information to the client side collator service,
//! and implement the `validate_block` funtion required by relay chain validators
//!
//! This is mostly copied and stripped down from cumulus pallet parachain system
//! https://paritytech.github.io/polkadot-sdk/master/cumulus_pallet_parachain_system/index.html
//!
//! Better docs coming after this takes shape. For now it is hack'n'slash.

#[cfg(not(feature = "std"))]
#[doc(hidden)]
pub mod implementation;
#[cfg(test)]
mod tests;

mod relay_state_snapshot;
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

use cumulus_primitives_parachain_inherent::ParachainInherentData;

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

use crate::dynamic_typing::UtxoData;

/// A wrapper type around Cumulus's ParachainInherentData ype that can be stored.
/// Having to do this wrapping is one more reason to abandon this UtxoData trait,
/// and go for a more strongly typed aggregate type approach.
#[derive(
    // Serialize,
    // Deserialize,
    Encode,
    Decode,
    derive_no_bound::DebugNoBound,
    // DefaultNoBound,
    // PartialEq,
    // Eq,
    derive_no_bound::CloneNoBound,
    scale_info::TypeInfo,
)]
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
