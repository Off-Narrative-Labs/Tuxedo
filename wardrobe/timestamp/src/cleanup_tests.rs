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
fn cleanup_timestamp_multiple_happy_path() {
    let old1 = NotedTimestamp(100);
    let old2 = NotedTimestamp(200);
    let newer = NotedTimestamp(2 * CLEANUP_AGE);

    let input_data = vec![old1.into(), old2.into()];
    let peek_data = vec![newer.into()];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &[]), Ok(0),);
}

#[test]
fn cleanup_timestamp_missing_input() {
    // The logic allows cleaning up "multiple", or more precisely, zero or more,
    // stale inputs. This test ensures that cleaning up zero is considered valid.
    // Of course there is little reason to do this in real life; it only wastes resources.

    let newer = NotedTimestamp(100);

    let input_data = vec![];
    let peek_data = vec![newer.into()];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &[]), Ok(0));
}

#[test]
fn cleanup_timestamp_multiple_first_valid_second_invalid() {
    let old = NotedTimestamp(100);
    let supposedly_old = NotedTimestamp(2 * CLEANUP_AGE);
    let newer = NotedTimestamp(2 * CLEANUP_AGE);

    let input_data = vec![old.into(), supposedly_old.into()];
    let peek_data = vec![newer.into()];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &[]), Err(DontBeSoHasty));
}

#[test]
fn cleanup_timestamp_input_is_wong_type() {
    let old = Bogus;
    let newer = NotedTimestamp(2 * CLEANUP_AGE);

    let input_data = vec![old.into()];
    let peek_data = vec![newer.into()];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &[]), Err(BadlyTyped));
}

#[test]
fn cleanup_timestamp_reference_is_wong_type() {
    let old = NotedTimestamp(100);
    let newer = Bogus;

    let input_data = vec![old.into()];
    let peek_data = vec![newer.into()];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &[]), Err(BadlyTyped));
}

#[test]
fn cleanup_timestamp_rcannot_create_state() {
    let old = NotedTimestamp(100);
    let newer = NotedTimestamp(2 * CLEANUP_AGE);
    let new = NotedTimestamp(CLEANUP_AGE);

    let input_data = vec![old.into()];
    let peek_data = vec![newer.into()];
    let output_data = vec![new.into()];

    assert_eq!(CleanUpTimestamp.check(&input_data, &peek_data, &output_data), Err(CleanupCannotCreateState));
}
