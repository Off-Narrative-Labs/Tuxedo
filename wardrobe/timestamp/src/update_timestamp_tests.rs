//! Unit tests for the Timestamp piece.
//! This module tests the primary flow of updating the timestamp via an inherent after it has been initialized.

use super::*;
use tuxedo_core::dynamic_typing::testing::Bogus;
use TimestampError::*;

/// The mock config always says the block number is two.
pub struct AlwaysBlockTwo;

impl TimestampConfig for AlwaysBlockTwo {
    fn block_height() -> u32 {
        2
    }
}

#[test]
fn update_timestamp_happy_path() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old: DynamicallyTypedData = Timestamp::new(1_000, 1).into();
    let peek: Vec<Output<UpForGrabs>> = vec![old.into()];
    let new: DynamicallyTypedData = Timestamp::new(3_000, 2).into();
    let out: Vec<Output<UpForGrabs>> = vec![new.into()];

    assert_eq!(checker.check(&[], &peek, &out), Ok(0));
}

#[test]
fn update_timestamp_with_input() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let bogus: DynamicallyTypedData = Bogus.into();
    let inp: Vec<Output<UpForGrabs>> = vec![bogus.into()];
    let old: DynamicallyTypedData = Timestamp::new(1_000, 1).into();
    let peek: Vec<Output<UpForGrabs>> = vec![old.into()];
    let new: DynamicallyTypedData = Timestamp::new(3_000, 2).into();
    let out: Vec<Output<UpForGrabs>> = vec![new.into()];

    assert_eq!(
        checker.check(&inp, &peek, &out),
        Err(InputsWhileSettingTimestamp)
    );
}

#[test]
fn update_timestamp_bogus_peek() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old: DynamicallyTypedData = Bogus.into();
    let peek: Vec<Output<UpForGrabs>> = vec![old.into()];
    let new: DynamicallyTypedData = Timestamp::new(3_000, 2).into();
    let out: Vec<Output<UpForGrabs>> = vec![new.into()];

    assert_eq!(checker.check(&[], &peek, &out), Err(BadlyTyped));
}

#[test]
fn update_timestamp_no_peek() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let new: DynamicallyTypedData = Timestamp::new(3_000, 2).into();
    let out: Vec<Output<UpForGrabs>> = vec![new.into()];

    assert_eq!(checker.check(&[], &[], &out), Err(MissingPreviousTimestamp));
}

#[test]
fn update_timestamp_no_output() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old: DynamicallyTypedData = Timestamp::new(1_000, 1).into();
    let peek: Vec<Output<UpForGrabs>> = vec![old.into()];

    assert_eq!(checker.check(&[], &peek, &[]), Err(MissingNewTimestamp));
}

#[test]
fn update_timestamp_too_many_outputs() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old: DynamicallyTypedData = Timestamp::new(1_000, 1).into();
    let peek: Vec<Output<UpForGrabs>> = vec![old.into()];
    let new: DynamicallyTypedData = Timestamp::new(3_000, 2).into();
    let bogus: DynamicallyTypedData = Bogus.into();
    let out: Vec<Output<UpForGrabs>> = vec![new.into(), bogus.into()];

    assert_eq!(
        checker.check(&[], &peek, &out),
        Err(TooManyOutputsWhileSettingTimestamp)
    );
}

#[test]
fn update_timestamp_wrong_height() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old: DynamicallyTypedData = Timestamp::new(1_000, 1).into();
    let peek: Vec<Output<UpForGrabs>> = vec![old.into()];
    let new: DynamicallyTypedData = Timestamp::new(5_000, 3).into();
    let out: Vec<Output<UpForGrabs>> = vec![new.into()];

    assert_eq!(
        checker.check(&[], &peek, &out),
        Err(NewTimestampWrongHeight)
    );
}

#[test]
fn update_timestamp_output_earlier_than_input() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old: DynamicallyTypedData = Timestamp::new(2_000, 1).into();
    let peek: Vec<Output<UpForGrabs>> = vec![old.into()];
    let new: DynamicallyTypedData = Timestamp::new(1_000, 2).into();
    let out: Vec<Output<UpForGrabs>> = vec![new.into()];

    assert_eq!(checker.check(&[], &peek, &out), Err(TimestampTooOld));
}

#[test]
fn update_timestamp_output_newer_than_previous_best_nut_not_enough_to_meet_threshold() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old: DynamicallyTypedData = Timestamp::new(1_000, 1).into();
    let peek: Vec<Output<UpForGrabs>> = vec![old.into()];
    let new: DynamicallyTypedData = Timestamp::new(2_000, 2).into();
    let out: Vec<Output<UpForGrabs>> = vec![new.into()];

    assert_eq!(checker.check(&[], &peek, &out), Err(TimestampTooOld));
}

#[test]
fn update_timestamp_previous_timestamp_wrong_height() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old: DynamicallyTypedData = Timestamp::new(0, 0).into();
    let peek: Vec<Output<UpForGrabs>> = vec![old.into()];
    let new: DynamicallyTypedData = Timestamp::new(2_000, 2).into();
    let out: Vec<Output<UpForGrabs>> = vec![new.into()];

    assert_eq!(
        checker.check(&[], &peek, &out),
        Err(PreviousTimestampWrongHeight)
    );
}
