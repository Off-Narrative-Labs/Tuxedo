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
/// Expects as parameters the runtime, a block executor and an inherent checker.
///
/// # Example
///
/// ```
///     struct BlockExecutor;
///     struct Runtime;
///     struct CheckInherents;
///
///     cumulus_pallet_parachain_system::register_validate_block! {
///         Runtime = Runtime,
///         BlockExecutor = Executive,
///         CheckInherents = CheckInherents,
///     }
///
/// # fn main() {}
/// ```
pub use tuxedo_register_validate_block::register_validate_block;