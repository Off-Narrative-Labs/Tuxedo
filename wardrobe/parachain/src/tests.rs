//! Unit tests for the Parachain Info inherent piece

use super::*;
use tuxedo_parachain_core::{
    tuxedo_core::dynamic_typing::{testing::Bogus, DynamicallyTypedData},
    MockRelayParentNumberStorage,
};
use ParachainError::*;

/// The mock config ignores the set relay parent storage number.
pub struct MockConfig;

impl ParachainPieceConfig for MockConfig {
    type SetRelayParentNumberStorage = MockRelayParentNumberStorage;
}

#[test]
fn update_parachain_info_happy_path() {
    let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
    let new: DynamicallyTypedData = new_data_from_relay_parent_number(4).into();

    assert_eq!(
        SetParachainInfo::<MockConfig>(Default::default()).check(&[], &[old], &[], &[new]),
        Ok(0),
    );
}

#[test]
fn update_parachain_info_relay_block_not_increasing() {
    let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
    let new: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();

    assert_eq!(
        SetParachainInfo::<MockConfig>(Default::default()).check(&[], &[old], &[], &[new]),
        Err(RelayBlockNotIncreasing),
    );
}

#[test]
fn update_parachain_info_extra_eviction() {
    let old1: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
    let old2: DynamicallyTypedData = Bogus.into();
    let new: DynamicallyTypedData = new_data_from_relay_parent_number(4).into();

    assert_eq!(
        SetParachainInfo::<MockConfig>(Default::default()).check(&[], &[old1, old2], &[], &[new]),
        Err(ExtraInputs)
    );
}

#[test]
fn update_parachain_info_missing_eviction() {
    let new: DynamicallyTypedData = new_data_from_relay_parent_number(4).into();

    assert_eq!(
        SetParachainInfo::<MockConfig>(Default::default()).check(&[], &[], &[], &[new]),
        Err(MissingPreviousInfo)
    );
}

#[test]
fn update_parachain_info_bogus_eviction() {
    let old: DynamicallyTypedData = Bogus.into();
    let new: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();

    assert_eq!(
        SetParachainInfo::<MockConfig>(Default::default()).check(&[], &[old], &[], &[new]),
        Err(BadlyTyped)
    );
}

#[test]
fn update_parachain_info_with_unexpected_normal_input() {
    let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
    let bogus: DynamicallyTypedData = Bogus.into();
    let new: DynamicallyTypedData = new_data_from_relay_parent_number(4).into();

    assert_eq!(
        SetParachainInfo::<MockConfig>(Default::default()).check(&[bogus], &[old], &[], &[new]),
        Err(ExtraInputs),
    );
}

#[test]
fn update_parachain_info_with_otherwise_valid_old_info_as_normal_input() {
    let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
    let new: DynamicallyTypedData = new_data_from_relay_parent_number(4).into();

    assert_eq!(
        SetParachainInfo::<MockConfig>(Default::default()).check(&[old], &[], &[], &[new]),
        Err(ExtraInputs),
    );
}

#[test]
fn update_parachain_info_extra_outputs() {
    let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
    let new1: DynamicallyTypedData = new_data_from_relay_parent_number(4).into();
    let new2: DynamicallyTypedData = Bogus.into();

    assert_eq!(
        SetParachainInfo::<MockConfig>(Default::default()).check(&[old], &[], &[], &[new1, new2]),
        Err(ExtraOutputs)
    );
}

#[test]
fn update_parachain_info_missing_output() {
    let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
    let inputs = vec![old];
    let outputs = vec![];

    assert_eq!(
        SetParachainInfo::<MockConfig>(Default::default()).check(&inputs, &[], &[], &outputs),
        Err(MissingNewInfo)
    );
}

#[test]
fn update_parachain_info_bogus_output() {
    let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
    let inputs = vec![old];
    let new: DynamicallyTypedData = Bogus.into();
    let outputs = vec![new];

    assert_eq!(
        SetParachainInfo::<MockConfig>(Default::default()).check(&inputs, &[], &[], &outputs),
        Err(BadlyTyped)
    );
}
