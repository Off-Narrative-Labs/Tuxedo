//! Unit tests for the Parachain Info inherent piece

use super::*;
use tuxedo_core::dynamic_typing::{testing::Bogus, DynamicallyTypedData};
use ParachainError::*;

/// The mock config always says the block number is two.
pub struct AlwaysBlockTwo;

impl ParachainPieceConfig for AlwaysBlockTwo {
    fn block_height() -> u32 {
        2
    }
}

#[test]
fn update_parachain_info_happy_path() {
    sp_io::TestExternalities::new_empty().execute_with(|| {
        let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
        let inputs: Vec<Output<UpForGrabs>> = vec![old.into()];
        let new: DynamicallyTypedData = new_data_from_relay_parent_number(4).into();
        let outputs: Vec<Output<UpForGrabs>> = vec![new.into()];

        assert_eq!(
            SetParachainInfo::<AlwaysBlockTwo>(Default::default()).check(&inputs, &[], &outputs),
            Ok(0),
        );
    });
}

#[test]
fn update_parachain_info_relay_block_not_increasing() {
    sp_io::TestExternalities::new_empty().execute_with(|| {
        let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
        let inputs: Vec<Output<UpForGrabs>> = vec![old.into()];
        let new: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
        let outputs: Vec<Output<UpForGrabs>> = vec![new.into()];

        assert_eq!(
            SetParachainInfo::<AlwaysBlockTwo>(Default::default()).check(&inputs, &[], &outputs),
            Err(RelayBlockNotIncreasing),
        );
    });
}

#[test]
fn update_parachain_info_extra_inputs() {
    sp_io::TestExternalities::new_empty().execute_with(|| {
        let old1: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
        let old2: DynamicallyTypedData = Bogus.into();
        let inputs: Vec<Output<UpForGrabs>> = vec![old1.into(), old2.into()];
        let new: DynamicallyTypedData = new_data_from_relay_parent_number(4).into();
        let outputs: Vec<Output<UpForGrabs>> = vec![new.into()];

        assert_eq!(
            SetParachainInfo::<AlwaysBlockTwo>(Default::default()).check(&inputs, &[], &outputs),
            Err(ExtraInputs)
        );
    });
}

#[test]
fn update_parachain_info_missing_input() {
    sp_io::TestExternalities::new_empty().execute_with(|| {
        let inputs: Vec<Output<UpForGrabs>> = vec![];
        let new: DynamicallyTypedData = new_data_from_relay_parent_number(4).into();
        let outputs: Vec<Output<UpForGrabs>> = vec![new.into()];

        assert_eq!(
            SetParachainInfo::<AlwaysBlockTwo>(Default::default()).check(&inputs, &[], &outputs),
            Err(MissingPreviousInfo)
        );
    });
}

#[test]
fn update_parachain_info_bogus_input() {
    sp_io::TestExternalities::new_empty().execute_with(|| {
        let old: DynamicallyTypedData = Bogus.into();
        let inputs: Vec<Output<UpForGrabs>> = vec![old.into()];
        let new: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
        let outputs: Vec<Output<UpForGrabs>> = vec![new.into()];

        assert_eq!(
            SetParachainInfo::<AlwaysBlockTwo>(Default::default()).check(&inputs, &[], &outputs),
            Err(BadlyTyped)
        );
    });
}

#[test]
fn update_parachain_info_extra_outputs() {
    sp_io::TestExternalities::new_empty().execute_with(|| {
        let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
        let inputs: Vec<Output<UpForGrabs>> = vec![old.into()];
        let new1: DynamicallyTypedData = new_data_from_relay_parent_number(4).into();
        let new2: DynamicallyTypedData = Bogus.into();
        let outputs: Vec<Output<UpForGrabs>> = vec![new1.into(), new2.into()];

        assert_eq!(
            SetParachainInfo::<AlwaysBlockTwo>(Default::default()).check(&inputs, &[], &outputs),
            Err(ExtraOutputs)
        );
    });
}

#[test]
fn update_parachain_info_missing_output() {
    sp_io::TestExternalities::new_empty().execute_with(|| {
        let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
        let inputs: Vec<Output<UpForGrabs>> = vec![old.into()];
        let outputs: Vec<Output<UpForGrabs>> = vec![];

        assert_eq!(
            SetParachainInfo::<AlwaysBlockTwo>(Default::default()).check(&inputs, &[], &outputs),
            Err(MissingNewInfo)
        );
    });
}

#[test]
fn update_parachain_info_bogus_output() {
    sp_io::TestExternalities::new_empty().execute_with(|| {
        let old: DynamicallyTypedData = new_data_from_relay_parent_number(3).into();
        let inputs: Vec<Output<UpForGrabs>> = vec![old.into()];
        let new: DynamicallyTypedData = Bogus.into();
        let outputs: Vec<Output<UpForGrabs>> = vec![new.into()];

        assert_eq!(
            SetParachainInfo::<AlwaysBlockTwo>(Default::default()).check(&inputs, &[], &outputs),
            Err(BadlyTyped)
        );
    });
}
