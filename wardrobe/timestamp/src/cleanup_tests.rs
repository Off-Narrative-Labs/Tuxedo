//! Unit tests for the Timestamp piece.
//! This module tests the secondary flow of cleaning up old timestamps.

use super::*;
use tuxedo_core::dynamic_typing::testing::Bogus;

// Cleanup happy case
// Cleanup input is best, not noted
// Cleanup, input is newer than reference
// Cleanup input is older than reference, but not by enough
// Cleanup missing input
// Cleanup missing reference
// Cleanup input is wrong type
// Cleanup output is wrong type
