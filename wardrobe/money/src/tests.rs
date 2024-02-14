//! Unit tests for the Money piece

use super::*;
use tuxedo_core::dynamic_typing::testing::Bogus;

#[test]
fn spend_valid_transaction_work() {
    let input_data = vec![Coin::<0>(5).into(), Coin::<0>(7).into()]; // total 12
    let output_data = vec![Coin::<0>(10).into(), Coin::<0>(1).into()]; // total 11
    let expected_priority = 1u64;

    assert_eq!(
        Spend::<0>.check(&input_data, &[], &output_data),
        Ok(expected_priority),
    );
}

#[test]
fn spend_with_zero_value_output_fails() {
    let input_data = vec![Coin::<0>(5).into(), Coin::<0>(7).into()]; // total 12
    let output_data = vec![
        Coin::<0>(10).into(),
        Coin::<0>(1).into(),
        Coin::<0>(0).into(),
    ]; // total 11

    assert_eq!(
        Spend::<0>.check(&input_data, &[], &output_data),
        Err(ConstraintCheckerError::ZeroValueCoin),
    );
}

#[test]
fn spend_no_outputs_is_a_burn() {
    let input_data = vec![Coin::<0>(5).into(), Coin::<0>(7).into()]; // total 12
    let output_data = vec![];
    let expected_priority = 12u64;

    assert_eq!(
        Spend::<0>.check(&input_data, &[], &output_data),
        Ok(expected_priority),
    );
}

#[test]
fn spend_no_inputs_fails() {
    let input_data = vec![];
    let output_data = vec![Coin::<0>(10).into(), Coin::<0>(1).into()];

    assert_eq!(
        Spend::<0>.check(&input_data, &[], &output_data),
        Err(ConstraintCheckerError::SpendingNothing),
    );
}

#[test]
fn spend_wrong_input_type_fails() {
    let input_data = vec![Bogus.into()];
    let output_data = vec![Coin::<0>(10).into(), Coin::<0>(1).into()];

    assert_eq!(
        Spend::<0>.check(&input_data, &[], &output_data),
        Err(ConstraintCheckerError::BadlyTyped),
    );
}

#[test]
fn spend_wrong_output_type_fails() {
    let input_data = vec![Coin::<0>(5).into(), Coin::<0>(7).into()]; // total 12
    let output_data = vec![Bogus.into()];

    assert_eq!(
        Spend::<0>.check(&input_data, &[], &output_data),
        Err(ConstraintCheckerError::BadlyTyped),
    );
}

#[test]
fn spend_output_value_exceeds_input_value_fails() {
    let input_data = vec![Coin::<0>(10).into(), Coin::<0>(1).into()]; // total 11
    let output_data = vec![Coin::<0>(5).into(), Coin::<0>(7).into()]; // total 12

    assert_eq!(
        Spend::<0>.check(&input_data, &[], &output_data),
        Err(ConstraintCheckerError::OutputsExceedInputs),
    );
}

#[test]
fn mint_valid_transaction_works() {
    let input_data = vec![];
    let output_data = vec![Coin::<0>(10).into(), Coin::<0>(1).into()];

    assert_eq!(Mint::<0>.check(&input_data, &[], &output_data), Ok(0),);
}

#[test]
fn mint_with_zero_value_output_fails() {
    let input_data = vec![];
    let output_data = vec![Coin::<0>(0).into()];

    assert_eq!(
        Mint::<0>.check(&input_data, &[], &output_data),
        Err(ConstraintCheckerError::ZeroValueCoin),
    );
}

#[test]
fn mint_with_inputs_fails() {
    let input_data = vec![Coin::<0>(5).into()];
    let output_data = vec![Coin::<0>(10).into(), Coin::<0>(1).into()];

    assert_eq!(
        Mint::<0>.check(&input_data, &[], &output_data),
        Err(ConstraintCheckerError::MintingWithInputs),
    );
}

#[test]
fn mint_with_no_outputs_fails() {
    let input_data = vec![];
    let output_data = vec![];

    assert_eq!(
        Mint::<0>.check(&input_data, &[], &output_data),
        Err(ConstraintCheckerError::MintingNothing),
    );
}

#[test]
fn mint_wrong_output_type_fails() {
    let input_data = vec![];
    let output_data = vec![Coin::<0>(10).into(), Bogus.into()];

    assert_eq!(
        Mint::<0>.check(&input_data, &[], &output_data),
        Err(ConstraintCheckerError::BadlyTyped),
    );
}
