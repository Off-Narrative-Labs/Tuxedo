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

#[test]
fn cleanup_timestamp_input_is_best_not_noted() {
    let old = BestTimestamp(100);
    let newer = NotedTimestamp(2 * CLEANUP_AGE);

    let input_data = vec![old.into()];
    let peek_data = vec![newer.into()];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &[]), Err(BadlyTyped));
}

#[test]
fn cleanup_timestamp_input_newer_than_reference() {
    let old = NotedTimestamp(200);
    let newer = NotedTimestamp(100);

    let input_data = vec![old.into()];
    let peek_data = vec![newer.into()];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &[]), Err(DontBeSoHasty));
}

#[test]
fn cleanup_timestamp_input_not_yet_ripe_for_cleaning() {
    let old = NotedTimestamp(200);
    let newer = NotedTimestamp(300);

    let input_data = vec![old.into()];
    let peek_data = vec![newer.into()];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &[]), Err(DontBeSoHasty));
}

#[test]
fn cleanup_timestamp_missing_reference() {
    let old = NotedTimestamp(200);

    let input_data = vec![old.into()];
    let peek_data = vec![];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &[]), Err(CleanupRequiresOneReference));
}

#[test]
fn cleanup_timestamp_missing_input() {
    // 

    let newer = NotedTimestamp(100);

    let input_data = vec![];
    let peek_data = vec![newer.into()];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &[]), Ok(0));
}

// cleanup multiple happy path

// cleanup multiple, first is valid, second is not

// Cleanup input is wrong type
// Reference is wrong type
// Cleanup output is wrong type
