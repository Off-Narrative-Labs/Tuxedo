//! This crate is the core of the Tuxedo runtime framework.
//! 
//! All Tuxedo runtimes will use this machinery and plug in their specific
//! Tuxedo piece(s)

// TODO Maybe this doesn't even need to be conditional. Just always build to no_std.
#![cfg_attr(not(feature = "std"), no_std)]

pub mod types;
pub mod executive;
pub mod redeemer;
pub mod verifier;
pub mod utxo_set;

// TODO These are copied from frame_support. We should PR Substrate to move them
// somewhere better and less frame-specific because they are more broadly useful.
mod support_macros;