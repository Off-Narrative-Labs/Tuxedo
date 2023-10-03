//! Unit tests for the Timestamp piece.
//! This module tests the secondary flow of cleaning up old timestamps.

use super::*;
use tuxedo_core::dynamic_typing::testing::Bogus;
use TimestampError::*;

#[test]
fn cleanup_timestamp_happy_path() {
    let old = NotedTimestamp(100);
    let newer = NotedTimestamp(2 * CLEANUP_AGE);

    let input_data = vec![old.into()];
    let peek_data = vec![newer.into()];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &[]), Ok(0),);
}

// Cleanup happy case
// Cleanup input is best, not noted
// Cleanup, input is newer than reference
// Cleanup input is older than reference, but not by enough
// Cleanup missing input
// Cleanup missing reference
// Cleanup input is wrong type
// Reference is wrong type
// Cleanup output is wrong type
