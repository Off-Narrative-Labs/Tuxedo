//! This crate is the core of the Tuxedo runtime framework.
//!
//! All Tuxedo runtimes will use this machinery and plug in their specific
//! Tuxedo piece(s)

// TODO Maybe this doesn't even need to be conditional. Just always build to no_std.
#![cfg_attr(not(feature = "std"), no_std)]

pub mod dynamic_typing;
mod executive;

pub mod constraint_checker;
pub mod inherents;
pub mod support_macros;
pub mod traits;
pub mod types;
pub mod utxo_set;
pub mod verifier;

// TODO feature gate this
pub mod validate_block;

pub use aggregator::{aggregate, tuxedo_constraint_checker, tuxedo_verifier};
pub use constraint_checker::{ConstraintChecker, SimpleConstraintChecker};
pub use executive::Executive;
pub use verifier::Verifier;

/// A Tuxedo-specific target for diagnostic node log messages
const LOG_TARGET: &str = "tuxedo-core";

/// A transient storage key that will hold the partial header while a block is being built.
/// This key is cleared before the end of the block.
const HEADER_KEY: &[u8] = b"header"; // 686561646572

/// A transient storage key that will hold the list of extrinsics that have been applied so far.
/// This key is cleared before the end of the block.
const EXTRINSIC_KEY: &[u8] = b"extrinsics";
