//! Unit tests for the Timestamp piece.
//! This module tests the "hack / workaround" where we allow setting a timestamp in block #1
//! without consuming any previous one. I hope to remove this hack by including a timestamp extrinsic
//! in the genesis block. I've asked for some background about that in
//! https://substrate.stackexchange.com/questions/10105/extrinsics-in-genesis-block

use super::*;
use tuxedo_core::dynamic_typing::testing::Bogus;

// Set timestamp happy case
// Set timestamp no outputs
// Set timestamp no new best
// Set timestamp no new noted
// Set timestamp noted not equal to best
// Set timestamp too many outputs
