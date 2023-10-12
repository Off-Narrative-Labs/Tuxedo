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

    let old_best: DynamicallyTypedData = BestTimestamp(100).into();
    let new_best: DynamicallyTypedData = BestTimestamp(400).into();
    let new_noted: DynamicallyTypedData = NotedTimestamp(400).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![old_best.into()];
    let output_data: Vec<Output<UpForGrabs>> = vec![new_best.into(), new_noted.into()];

    assert_eq!(checker.check(&input_data, &[], &output_data), Ok(0),);
}

#[test]
fn update_timestamp_bogus_input() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best: DynamicallyTypedData = Bogus.into();
    let new_best: DynamicallyTypedData = BestTimestamp(400).into();
    let new_noted: DynamicallyTypedData = NotedTimestamp(400).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![old_best.into()];
    let output_data: Vec<Output<UpForGrabs>> = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(BadlyTyped)
    );
}

#[test]
fn update_timestamp_input_noted_not_best() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best: DynamicallyTypedData = NotedTimestamp(100).into();
    let new_best: DynamicallyTypedData = BestTimestamp(400).into();
    let new_noted: DynamicallyTypedData = NotedTimestamp(400).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![old_best.into()];
    let output_data: Vec<Output<UpForGrabs>> = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(BadlyTyped)
    );
}

#[test]
fn update_timestamp_no_input() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let new_best: DynamicallyTypedData = BestTimestamp(400).into();
    let new_noted: DynamicallyTypedData = NotedTimestamp(400).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![];
    let output_data: Vec<Output<UpForGrabs>> = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(MissingPreviousBestTimestamp),
    );
}

#[test]
fn update_timestamp_output_earlier_than_input() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best: DynamicallyTypedData = BestTimestamp(500).into();
    let new_best: DynamicallyTypedData = BestTimestamp(400).into();
    let new_noted: DynamicallyTypedData = NotedTimestamp(400).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![old_best.into()];
    let output_data: Vec<Output<UpForGrabs>> = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(TimestampTooOld)
    );
}

#[test]
fn update_timestamp_output_newer_than_previous_best_nut_not_enough_to_meet_threshold() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best: DynamicallyTypedData = BestTimestamp(100).into();
    let new_best: DynamicallyTypedData = BestTimestamp(200).into();
    let new_noted: DynamicallyTypedData = NotedTimestamp(200).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![old_best.into()];
    let output_data: Vec<Output<UpForGrabs>> = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(TimestampTooOld)
    );
}

#[test]
fn update_timestamp_too_many_inputs() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best: DynamicallyTypedData = BestTimestamp(100).into();
    let new_best: DynamicallyTypedData = BestTimestamp(400).into();
    let new_noted: DynamicallyTypedData = NotedTimestamp(400).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![old_best.clone().into(), old_best.into()];
    let output_data: Vec<Output<UpForGrabs>> = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(TooManyInputsWhileSettingTimestamp)
    );
}

#[test]
fn update_timestamp_new_best_and_new_noted_inconsistent() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best: DynamicallyTypedData = BestTimestamp(100).into();
    let new_best: DynamicallyTypedData = BestTimestamp(400).into();
    let new_noted: DynamicallyTypedData = NotedTimestamp(401).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![old_best.into()];
    let output_data: Vec<Output<UpForGrabs>> = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(InconsistentBestAndNotedTimestamps)
    );
}

#[test]
fn update_timestamp_no_outputs() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best: DynamicallyTypedData = BestTimestamp(100).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![old_best.into()];
    let output_data: Vec<Output<UpForGrabs>> = vec![];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(MissingNewBestTimestamp)
    );
}

#[test]
fn update_timestamp_no_new_best() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best: DynamicallyTypedData = BestTimestamp(100).into();
    let new_noted: DynamicallyTypedData = NotedTimestamp(400).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![old_best.into()];
    let output_data: Vec<Output<UpForGrabs>> = vec![new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(BadlyTyped)
    );
}

#[test]
fn update_timestamp_no_new_noted() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best: DynamicallyTypedData = BestTimestamp(100).into();
    let new_best: DynamicallyTypedData = BestTimestamp(400).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![old_best.into()];
    let output_data: Vec<Output<UpForGrabs>> = vec![new_best.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(MissingNewNotedTimestamp)
    );
}

#[test]
fn update_timestamp_too_many_outputs() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best: DynamicallyTypedData = BestTimestamp(100).into();
    let new_best: DynamicallyTypedData = BestTimestamp(400).into();
    let new_noted: DynamicallyTypedData = NotedTimestamp(400).into();
    let input_data: Vec<Output<UpForGrabs>> = vec![old_best.into()];
    let output_data: Vec<Output<UpForGrabs>> =
        vec![new_best.into(), new_noted.clone().into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(TooManyOutputsWhileSettingTimestamp)
    );
}
