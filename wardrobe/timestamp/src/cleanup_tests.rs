//! Unit tests for the Timestamp piece.
//! This module tests the secondary flow of cleaning up old timestamps.

use super::{
    CleanUpTimestamp, SimpleConstraintChecker, Timestamp, TimestampConfig, TimestampError,
};
use tuxedo_core::dynamic_typing::testing::Bogus;
use TimestampError::*;

/// The mock config always says the block number is one million.
pub struct AlwaysBlockMillion;

impl TimestampConfig for AlwaysBlockMillion {
    fn block_height() -> u32 {
        1_000_000
    }
}

#[test]
fn cleanup_timestamp_happy_path() {
    let old = Timestamp::new(1, 1);
    let newer = Timestamp::new(
        2 * AlwaysBlockMillion::MIN_TIME_BEFORE_CLEANUP,
        2 * AlwaysBlockMillion::MIN_BLOCKS_BEFORE_CLEANUP,
    );

    let evictions = vec![old.into()];
    let peek = vec![newer.into()];

    assert_eq!(
        CleanUpTimestamp::<AlwaysBlockMillion>::default().check(&[], &evictions, &peek, &[]),
        Ok(0),
    );
}

#[test]
fn cleanup_timestamp_no_peek() {
    let old = Timestamp::new(1, 1);
    let evictions = vec![old.into()];

    assert_eq!(
        CleanUpTimestamp::<AlwaysBlockMillion>::default().check(&[], &evictions, &[], &[]),
        Err(CleanupRequiresOneReference)
    );
}

#[test]
fn cleanup_timestamp_fails_when_target_newer_than_reference() {
    let old = Timestamp::new(1, 1);
    let newer = Timestamp::new(
        2 * AlwaysBlockMillion::MIN_TIME_BEFORE_CLEANUP,
        2 * AlwaysBlockMillion::MIN_BLOCKS_BEFORE_CLEANUP,
    );

    let evictions = vec![newer.into()];
    let peek = vec![old.into()];

    assert_eq!(
        CleanUpTimestamp::<AlwaysBlockMillion>::default().check(&[], &evictions, &peek, &[]),
        Err(DontBeSoHasty)
    );
}

#[test]
fn cleanup_timestamp_input_not_yet_ripe_for_cleaning() {
    let old = Timestamp::new(1, 1);
    let newer = Timestamp::new(
        AlwaysBlockMillion::MIN_TIME_BEFORE_CLEANUP / 2,
        AlwaysBlockMillion::MIN_BLOCKS_BEFORE_CLEANUP,
    );

    let evictions = vec![old.into()];
    let peek = vec![newer.into()];

    assert_eq!(
        CleanUpTimestamp::<AlwaysBlockMillion>::default().check(&[], &evictions, &peek, &[]),
        Err(DontBeSoHasty)
    );
}

#[test]
fn cleanup_timestamp_multiple_happy_path() {
    let old1 = Timestamp::new(AlwaysBlockMillion::MINIMUM_TIME_INTERVAL, 1);
    let old2 = Timestamp::new(2 * AlwaysBlockMillion::MINIMUM_TIME_INTERVAL, 2);
    let newer = Timestamp::new(
        2 * AlwaysBlockMillion::MIN_TIME_BEFORE_CLEANUP,
        2 * AlwaysBlockMillion::MIN_BLOCKS_BEFORE_CLEANUP,
    );

    let evictions = vec![old1.into(), old2.into()];
    let peek = vec![newer.into()];

    assert_eq!(
        CleanUpTimestamp::<AlwaysBlockMillion>::default().check(&[], &evictions, &peek, &[]),
        Ok(0),
    );
}

#[test]
fn cleanup_timestamp_missing_eviction() {
    // The logic allows cleaning up "multiple", or more precisely, zero or more,
    // stale inputs. This test ensures that cleaning up zero is considered valid.
    // Of course there is little reason to do this in real life; it only wastes resources.

    let newer = Timestamp::new(
        2 * AlwaysBlockMillion::MIN_TIME_BEFORE_CLEANUP,
        2 * AlwaysBlockMillion::MIN_BLOCKS_BEFORE_CLEANUP,
    );

    let peek = vec![newer.into()];

    assert_eq!(
        CleanUpTimestamp::<AlwaysBlockMillion>::default().check(&[], &[], &peek, &[]),
        Ok(0),
    );
}

#[test]
fn cleanup_timestamp_multiple_first_valid_second_invalid() {
    let old = Timestamp::new(AlwaysBlockMillion::MINIMUM_TIME_INTERVAL, 1);
    let supposedly_old = Timestamp::new(
        2 * AlwaysBlockMillion::MIN_TIME_BEFORE_CLEANUP,
        2 * AlwaysBlockMillion::MIN_BLOCKS_BEFORE_CLEANUP,
    );
    let newer = Timestamp::new(
        2 * AlwaysBlockMillion::MIN_TIME_BEFORE_CLEANUP,
        2 * AlwaysBlockMillion::MIN_BLOCKS_BEFORE_CLEANUP,
    );

    let evictions = vec![old.into(), supposedly_old.into()];
    let peek = vec![newer.into()];

    assert_eq!(
        CleanUpTimestamp::<AlwaysBlockMillion>::default().check(&[], &evictions, &peek, &[]),
        Err(DontBeSoHasty)
    );
}

#[test]
fn cleanup_timestamp_target_is_wrong_type() {
    let old = Bogus;
    let newer = Timestamp::new(
        2 * AlwaysBlockMillion::MIN_TIME_BEFORE_CLEANUP,
        2 * AlwaysBlockMillion::MIN_BLOCKS_BEFORE_CLEANUP,
    );

    let evictions = vec![old.into()];
    let peek = vec![newer.into()];

    assert_eq!(
        CleanUpTimestamp::<AlwaysBlockMillion>::default().check(&[], &evictions, &peek, &[]),
        Err(BadlyTyped)
    );
}

#[test]
fn cleanup_timestamp_reference_is_wrong_type() {
    let old = Timestamp::new(1, 1);

    let evictions = vec![old.into()];
    let peek = vec![Bogus.into()];

    assert_eq!(
        CleanUpTimestamp::<AlwaysBlockMillion>::default().check(&[], &evictions, &peek, &[]),
        Err(BadlyTyped)
    );
}

#[test]
fn cleanup_timestamp_cannot_create_state() {
    let old = Timestamp::new(1, 1);
    let newer = Timestamp::new(
        2 * AlwaysBlockMillion::MIN_TIME_BEFORE_CLEANUP,
        2 * AlwaysBlockMillion::MIN_BLOCKS_BEFORE_CLEANUP,
    );

    let evictions = vec![old.into()];
    let peek = vec![newer.into()];
    let out = vec![Bogus.into()];

    assert_eq!(
        CleanUpTimestamp::<AlwaysBlockMillion>::default().check(&[], &evictions, &peek, &out),
        Err(CleanupCannotCreateState)
    );
}

#[test]
fn cleanup_timestamp_with_stray_normal_redemptive_input() {
    let old = Timestamp::new(1, 1);
    let normal_input_old = Timestamp::new(2, 2);
    let newer = Timestamp::new(
        2 * AlwaysBlockMillion::MIN_TIME_BEFORE_CLEANUP,
        2 * AlwaysBlockMillion::MIN_BLOCKS_BEFORE_CLEANUP,
    );

    let inputs = vec![normal_input_old.into()];
    let evictions = vec![old.into()];
    let peek = vec![newer.into()];

    assert_eq!(
        CleanUpTimestamp::<AlwaysBlockMillion>::default().check(&inputs, &evictions, &peek, &[]),
        Err(CleanupEvictionsOnly)
    );
}
