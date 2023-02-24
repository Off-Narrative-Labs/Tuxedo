//! This crate is the core of the Tuxedo runtime framework.
//!
//! All Tuxedo runtimes will use this machinery and plug in their specific
//! Tuxedo piece(s)

// TODO Maybe this doesn't even need to be conditional. Just always build to no_std.
#![cfg_attr(not(feature = "std"), no_std)]

mod executive;
pub mod redeemer;
pub mod types;
mod utxo_set;
mod verifier;

pub use executive::Executive;
pub use redeemer::Redeemer;
pub use verifier::Verifier;

// TODO These are copied from frame_support. We should PR Substrate to move them
// somewhere better and less frame-specific because they are more broadly useful.
pub mod support_macros;

/// A Tuxedo-specific target for diagnostic node log messages
const LOG_TARGET: &'static str = "tuxedo-core";

/// A transient storage key that will hold the partial header while a block is being built.
/// This key is cleared before the end of the block.
const HEADER_KEY: &[u8] = b"header"; // 686561646572

/// A transient storage key that will hold the list of extrinsics that have been applied so far.
/// This key is cleared before the end of the block.
const EXTRINSIC_KEY: &[u8] = b"extrinsics";
