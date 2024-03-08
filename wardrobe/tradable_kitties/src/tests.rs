//! Tests for the Tradable Kitty Piece

use super::*;
use kitties::DadKittyStatus;
use kitties::KittyDNA;
use kitties::MomKittyStatus;
use kitties::Parent;
use sp_runtime::testing::H256;
use tuxedo_core::dynamic_typing::testing::Bogus;

impl TradableKittyData {
    pub fn default_kitty() -> KittyData {
        KittyData {
            parent: Parent::Dad(DadKittyStatus::RearinToGo),
            ..Default::default()
        }
    }

    pub fn default_tradable_kitty() -> Self {
        let kitty_basic = KittyData {
            parent: Parent::Dad(DadKittyStatus::RearinToGo),
            ..Default::default()
        };
        TradableKittyData {
            kitty_basic_data: kitty_basic,
            price: 100,
            ..Default::default()
        }
    }
}
// listKittyForSale UT startes from here.
#[test]
fn list_kitty_for_sale_happy_path_works() {
    let result = TradableKittyConstraintChecker::<0>::ListKittyForSale.check(
        &[TradableKittyData::default_kitty().into()],
        &[],
        &[TradableKittyData::default_tradable_kitty().into()],
    );
    assert!(result.is_ok());
}

#[test]
fn list_kitty_for_sale_multiple_input_happy_path_works() {
    let input1 = TradableKittyData::default_kitty();
    let mut input2 = TradableKittyData::default_kitty();
    input2.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadola"));
    let mut output1 = TradableKittyData::default_tradable_kitty();
    let mut output2 = TradableKittyData::default_tradable_kitty();
    output1.kitty_basic_data = input1.clone();
    output2.kitty_basic_data = input2.clone();

    let result = TradableKittyConstraintChecker::<0>::ListKittyForSale.check(
        &[input1.into(), input2.into()],
        &[],
        &[output1.into(), output2.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn list_kitty_for_sale_multiple_out_of_order_input_fails() {
    let input1 = TradableKittyData::default_kitty();
    let mut input2 = TradableKittyData::default_kitty();
    input2.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadola"));
    let mut output1 = TradableKittyData::default_tradable_kitty();
    let mut output2 = TradableKittyData::default_tradable_kitty();
    output1.kitty_basic_data = input1.clone();
    output2.kitty_basic_data = input2.clone();

    let result = TradableKittyConstraintChecker::<0>::ListKittyForSale.check(
        &[input1.into(), input2.into()],
        &[],
        &[output2.into(), output1.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyBasicPropertiesAltered)
    );
}

#[test]
fn list_kitty_for_sale_different_num_of_input_output_path_fails() {
    let mut input1 = TradableKittyData::default_kitty();
    let input2 = TradableKittyData::default_kitty();
    input1.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoca"));
    let mut output1 = TradableKittyData::default_tradable_kitty();
    output1.kitty_basic_data = input2.clone();

    let result = TradableKittyConstraintChecker::<0>::ListKittyForSale.check(
        &[input1.into(), input2.into()],
        &[],
        &[output1.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::NumberOfInputOutputMismatch)
    );
}

#[test]
fn list_kitty_for_sale_input_missing_path_fails() {
    let output = TradableKittyData::default_tradable_kitty();

    let result =
        TradableKittyConstraintChecker::<0>::ListKittyForSale.check(&[], &[], &[output.into()]);
    assert_eq!(
        result,
        Err(TradeableKittyError::NumberOfInputOutputMismatch)
    );
}

#[test]
fn list_kitty_for_sale_out_put_missing_path_fails() {
    let input1 = TradableKittyData::default_kitty();
    let result =
        TradableKittyConstraintChecker::<0>::ListKittyForSale.check(&[input1.into()], &[], &[]);
    assert_eq!(
        result,
        Err(TradeableKittyError::NumberOfInputOutputMismatch)
    );
}

#[test]
fn list_kitty_for_sale_with_input_missing_fails() {
    let input1 = TradableKittyData::default_kitty();

    let mut output1 = TradableKittyData::default_tradable_kitty();
    output1.kitty_basic_data = input1.clone();
    let mut output2 = TradableKittyData::default_tradable_kitty();
    output2.kitty_basic_data.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoca"));
    let result = TradableKittyConstraintChecker::<0>::ListKittyForSale.check(
        &[],
        &[],
        &[output1.into(), output2.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::NumberOfInputOutputMismatch)
    );
}

#[test]
fn list_for_sale_with_wrong_output_type_amoung_valid_output_fails() {
    let input1 = TradableKittyData::default_kitty();
    let mut output1 = TradableKittyData::default_tradable_kitty();
    output1.kitty_basic_data = input1.clone();
    let result = TradableKittyConstraintChecker::<0>::ListKittyForSale.check(
        &[input1.into()],
        &[],
        &[Bogus.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped));
}

#[test]
fn list_kitty_for_sale_with_wrong_input_type_fails() {
    let result = TradableKittyConstraintChecker::<0>::ListKittyForSale.check(
        &[Bogus.into()],
        &[],
        &[TradableKittyData::default_tradable_kitty().into()],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped));
}

#[test]
fn list_for_sale_multiple_inputs_with_basic_property_changed_fails() {
    let input1 = TradableKittyData::default_kitty();
    let input2 = TradableKittyData::default_kitty();

    let mut output1 = TradableKittyData::default_tradable_kitty();
    output1.kitty_basic_data = input1.clone();
    let mut output2 = TradableKittyData::default_tradable_kitty();
    output2.kitty_basic_data.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoca"));
    let result = TradableKittyConstraintChecker::<0>::ListKittyForSale.check(
        &[input1.into(), input2.into()],
        &[],
        &[output1.clone().into(), output2.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyBasicPropertiesAltered)
    );
}

#[test]
fn list_for_sale_with_basic_property_changed_fails() {
    let input = TradableKittyData::default_kitty();
    let mut output = TradableKittyData::default_tradable_kitty();
    output.kitty_basic_data = input.clone();
    output.kitty_basic_data.free_breedings += 1;
    let result = TradableKittyConstraintChecker::<0>::ListKittyForSale.check(
        &[input.into()],
        &[],
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyBasicPropertiesAltered)
    );
}

#[test]
fn list_for_sale_with_price_zero_fails() {
    let input1 = TradableKittyData::default_kitty();
    let mut output1 = TradableKittyData::default_tradable_kitty();
    output1.kitty_basic_data = input1.clone();
    output1.price = 0;
    let result = TradableKittyConstraintChecker::<0>::ListKittyForSale.check(
        &[input1.into()],
        &[],
        &[output1.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::KittyPriceCantBeZero));
}

// delistKittyFromSale UT starts from here.
#[test]
fn delist_kitty_from_sale_happy_path_works() {
    let result = TradableKittyConstraintChecker::<0>::DelistKittyFromSale.check(
        &[TradableKittyData::default_tradable_kitty().into()],
        &[],
        &[TradableKittyData::default_kitty().into()],
    );
    assert!(result.is_ok());
}

#[test]
fn delist_kitty_from_sale_multiple_input_happy_path_works() {
    let output1 = TradableKittyData::default_kitty();
    let mut output2 = TradableKittyData::default_kitty();
    output2.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoca"));
    let input1 = TradableKittyData::default_tradable_kitty();
    let mut input2 = TradableKittyData::default_tradable_kitty();
    input2.kitty_basic_data = output2.clone();

    let result = TradableKittyConstraintChecker::<0>::DelistKittyFromSale.check(
        &[input1.into(), input2.into()],
        &[],
        &[output1.into(), output2.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn delist_kitty_from_sale_multiple_input_out_of_order_path_fails() {
    let output1 = TradableKittyData::default_kitty();
    let mut output2 = TradableKittyData::default_kitty();
    output2.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoca"));
    let input1 = TradableKittyData::default_tradable_kitty();
    let mut input2 = TradableKittyData::default_tradable_kitty();
    input2.kitty_basic_data = output2.clone();

    let result = TradableKittyConstraintChecker::<0>::DelistKittyFromSale.check(
        &[input1.into(), input2.into()],
        &[],
        &[output2.into(), output1.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyBasicPropertiesAltered)
    );
}

#[test]
fn delist_kitty_from_sale_different_num_of_input_output_fails() {
    let output1 = TradableKittyData::default_kitty();
    let mut output2 = TradableKittyData::default_kitty();
    output2.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoca"));
    let input1 = TradableKittyData::default_tradable_kitty();
    let mut input2 = TradableKittyData::default_tradable_kitty();
    input2.kitty_basic_data = output2;

    let result = TradableKittyConstraintChecker::<0>::DelistKittyFromSale.check(
        &[input1.into(), input2.into()],
        &[],
        &[output1.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::NumberOfInputOutputMismatch)
    );
}

#[test]
fn delist_kitty_from_sale_input_missing_fails() {
    let output = TradableKittyData::default_kitty();
    let result =
        TradableKittyConstraintChecker::<0>::DelistKittyFromSale.check(&[], &[], &[output.into()]);
    assert_eq!(
        result,
        Err(TradeableKittyError::NumberOfInputOutputMismatch)
    );
}
#[test]
fn delist_kitty_from_sale_out_put_missing_path_fails() {
    let result = TradableKittyConstraintChecker::<0>::DelistKittyFromSale.check(
        &[TradableKittyData::default_tradable_kitty().into()],
        &[],
        &[],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::NumberOfInputOutputMismatch)
    );
}

#[test]
fn delist_kitty_from_sale_with_wrong_output_type_ampoung_valid_output_fails() {
    let output1 = TradableKittyData::default_kitty();
    let mut output2 = TradableKittyData::default_kitty();
    output2.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoca"));
    let mut input1 = TradableKittyData::default_tradable_kitty();
    let mut input2 = TradableKittyData::default_tradable_kitty();
    input1.kitty_basic_data = output1.clone();
    input2.kitty_basic_data = output2.clone();

    let result = TradableKittyConstraintChecker::<0>::DelistKittyFromSale.check(
        &[input1.into(), input2.into()],
        &[],
        &[output1.into(), Bogus.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped));
}

#[test]
fn delist_from_sale_with_wrong_input_type_fails() {
    let result = TradableKittyConstraintChecker::<0>::DelistKittyFromSale.check(
        &[Bogus.into()],
        &[],
        &[TradableKittyData::default_kitty().into()],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped));
}

#[test]
fn delist_from_sale_with_basic_property_update_fails() {
    let output1 = TradableKittyData::default_kitty();
    let mut output2 = TradableKittyData::default_kitty();
    output2.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadocz"));
    let mut input1 = TradableKittyData::default_tradable_kitty();
    let mut input2 = TradableKittyData::default_tradable_kitty();
    input1.kitty_basic_data = output1.clone();
    input2.kitty_basic_data = output2.clone();
    let result = TradableKittyConstraintChecker::<0>::DelistKittyFromSale.check(
        &[input1.clone().into(), input2.into()],
        &[],
        &[output1.clone().into(), output1.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyBasicPropertiesAltered)
    );
}

// From below update tradable kitty name test cases starts
#[test]
fn update_name_happy_path_works() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.kitty_basic_data.name = *b"tdkt";

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyName.check(
        &[input.into()],
        &[],
        &[output.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn update_name_invalid_type_in_input_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.kitty_basic_data.name = *b"tdkt";

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyName.check(
        &[input.into(), Bogus.into()],
        &[],
        &[output.clone().into(), output.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped));
}

#[test]
fn update_name_invalid_type_in_output_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.kitty_basic_data.name = *b"tdkt";

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyName.check(
        &[input.clone().into(), input.into()],
        &[],
        &[output.into(), Bogus.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped));
}

#[test]
fn update_name_dna_update_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.kitty_basic_data.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoca"));
    output.kitty_basic_data.name = *b"tdkt";

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyName.check(
        &[input.into()],
        &[],
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyError(
            kitties::ConstraintCheckerError::DnaMismatchBetweenInputAndOutput
        ))
    );
}

#[test]
fn update_name_free_breeding_update_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.kitty_basic_data.name = *b"kty1";
    output.kitty_basic_data.free_breedings += 1;

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyName.check(
        &[input.into()],
        &[],
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyError(
            kitties::ConstraintCheckerError::FreeBreedingCannotBeUpdated
        ))
    );
}

#[test]
fn update_name_num_of_breeding_updated_path_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.kitty_basic_data.name = *b"kty1";
    output.kitty_basic_data.num_breedings += 1;

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyName.check(
        &[input.into()],
        &[],
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyError(
            kitties::ConstraintCheckerError::NumOfBreedingCannotBeUpdated
        ))
    );
}

#[test]
fn update_name_gender_updated_path_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = TradableKittyData::default_tradable_kitty();
    output.kitty_basic_data.name = *b"tdk1";
    output.kitty_basic_data.parent = Parent::Mom(MomKittyStatus::RearinToGo);

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyName.check(
        &[input.into()],
        &[],
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyError(
            kitties::ConstraintCheckerError::KittyGenderCannotBeUpdated
        ))
    );
}

//// Price update UT starts from here.
#[test]
fn update_price_happy_path_works() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.price = 500;

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyPrice.check(
        &[input.into()],
        &[],
        &[output.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn update_price_multiple_input_happy_path_works() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.price = 500;
    let mut input1 = TradableKittyData::default_tradable_kitty();
    input1.kitty_basic_data.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoai"));
    let mut output1 = input1.clone();
    output1.price = 700;

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyPrice.check(
        &[input.into(), input1.into()],
        &[],
        &[output.into(), output1.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn update_price_multiple_input_out_of_order_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.price = 500;
    let mut input1 = TradableKittyData::default_tradable_kitty();
    input1.kitty_basic_data.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoai"));
    let mut output1 = input1.clone();
    output1.price = 700;

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyPrice.check(
        &[input.into(), input1.into()],
        &[],
        &[output1.into(), output.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyBasicPropertiesAltered)
    );
}

#[test]
fn update_price_output_missing_path_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.price = 500;
    let mut input1 = TradableKittyData::default_tradable_kitty();
    input1.kitty_basic_data.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoai"));

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyPrice.check(
        &[input.into(), input1.into()],
        &[],
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::NumberOfInputOutputMismatch)
    );
}

#[test]
fn update_price_input_missing_path_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.price = 500;
    let mut input1 = TradableKittyData::default_tradable_kitty();
    input1.kitty_basic_data.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoai"));
    let mut output1 = input1.clone();
    output1.price = 700;

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyPrice.check(
        &[input.into()],
        &[],
        &[output.into(), output1.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::NumberOfInputOutputMismatch)
    );
}

#[test]
fn update_price_bad_input_path_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.price = 500;

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyPrice.check(
        &[Bogus.into()],
        &[],
        &[output.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped));
}

#[test]
fn update_price_bad_output_path_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.price = 500;
    let mut input1 = TradableKittyData::default_tradable_kitty();
    input1.kitty_basic_data.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoai"));

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyPrice.check(
        &[input.into(), input1.into()],
        &[],
        &[output.into(), Bogus.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped));
}

#[test]
fn update_price_different_dna_path_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.price = 500;

    output.kitty_basic_data.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoai"));

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyPrice.check(
        &[input.clone().into()],
        &[],
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyBasicPropertiesAltered)
    );
}

#[test]
fn update_price_basic_properties_updated_path_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.price = 500;
    output.kitty_basic_data.free_breedings += 1;

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyPrice.check(
        &[input.into()],
        &[],
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::KittyBasicPropertiesAltered)
    );
}

#[test]
fn update_price_not_updated_path_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let output = input.clone();

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyPrice.check(
        &[input.into()],
        &[],
        &[output.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::KittyPriceUnaltered));
}

#[test]
fn update_price_to_zero_updated_path_fails() {
    let input = TradableKittyData::default_tradable_kitty();
    let mut output = input.clone();
    output.price = 0;

    let result = TradableKittyConstraintChecker::<0>::UpdateKittyPrice.check(
        &[input.into()],
        &[],
        &[output.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::KittyPriceCantBeZero));
}

// Buy tradable Kitty UT starts from here.

#[test]
fn buy_happy_path_single_input_coinworks() {
    let mut input_kitty = TradableKittyData::default_tradable_kitty();
    input_kitty.price = 100;
    let output_kitty = input_kitty.clone();

    let input_coin = Coin::<0>(100);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin.into()],
        &[],
        &[output_kitty.into(), output_coin.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn buy_happy_path_multiple_input_coinworks() {
    let mut input_kitty = TradableKittyData::default_tradable_kitty();
    input_kitty.price = 100;
    let output_kitty = input_kitty.clone();

    let input_coin1 = Coin::<0>(10);
    let input_coin2 = Coin::<0>(90);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin1.into(), input_coin2.into()],
        &[],
        &[output_kitty.into(), output_coin.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn buy_path_multiple_kitty_fails() {
    let mut input_kitty = TradableKittyData::default_tradable_kitty();
    input_kitty.price = 100;
    let output_kitty = input_kitty.clone();

    let mut input_kitty1 = TradableKittyData::default_tradable_kitty();
    input_kitty1.kitty_basic_data.dna =
        KittyDNA(H256::from_slice(b"superkalifragislisticexpialadolx"));
    input_kitty1.price = 1;
    let output_kitty1 = input_kitty1.clone();

    let input_coin1 = Coin::<0>(10);
    let input_coin2 = Coin::<0>(90);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[
            input_kitty.into(),
            input_kitty1.into(),
            input_coin1.into(),
            input_coin2.into(),
        ],
        &[],
        &[
            output_kitty.into(),
            output_kitty1.into(),
            output_coin.into(),
        ],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped));
}

#[test]
fn buy_kityy_with_price_none_fails() {
    let mut input_kitty = TradableKittyData::default_tradable_kitty();
    input_kitty.price = 0;
    let output_kitty = input_kitty.clone();

    let input_coin = Coin::<0>(100);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin.into()],
        &[],
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::KittyPriceCantBeZero));
}

#[test]
fn buy_kityy_wrong_input_type_fails() {
    let mut input_kitty = TradableKittyData::default_tradable_kitty();

    input_kitty.price = 101;
    let output_kitty = input_kitty.clone();

    let input_coin = Coin::<0>(100);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin.into(), Bogus.into()],
        &[],
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped));
}

#[test]
fn buy_kityy_wrong_output_type_fails() {
    let mut input_kitty = TradableKittyData::default_tradable_kitty();
    input_kitty.price = 101;
    let output_kitty = input_kitty.clone();

    let input_coin = Coin::<0>(100);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin.into()],
        &[],
        &[output_kitty.into(), output_coin.into(), Bogus.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped));
}

#[test]
fn buy_kitty_less_money_than_price_of_kityy_fails() {
    let mut input_kitty = TradableKittyData::default_tradable_kitty();
    input_kitty.price = 101;
    let output_kitty = input_kitty.clone();

    let input_coin1 = Coin::<0>(100);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin1.into()],
        &[],
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::InsufficientCollateralToBuyKitty)
    );
}

#[test]
fn buy_kitty_coin_output_value_exceeds_input_coin_value_fails() {
    let mut input_kitty = TradableKittyData::default_tradable_kitty();
    input_kitty.price = 101;
    let output_kitty = input_kitty.clone();

    let input_coin1 = Coin::<0>(100);
    let input_coin2 = Coin::<0>(90);
    let output_coin = Coin::<0>(300);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin1.into(), input_coin2.into()],
        &[],
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::MoneyError(
            MoneyError::OutputsExceedInputs
        ))
    )
}

#[test]
fn buy_kitty_input_zero_coin_value_fails() {
    let mut input_kitty = TradableKittyData::default_tradable_kitty();
    input_kitty.price = 101;
    let output_kitty = input_kitty.clone();

    let input_coin1 = Coin::<0>(0);
    let input_coin2 = Coin::<0>(90);
    let output_coin = Coin::<0>(300);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin1.into(), input_coin2.into()],
        &[],
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::MoneyError(MoneyError::ZeroValueCoin))
    )
}

#[test]
fn buy_kitty_output_zero_coin_value_fails() {
    let mut input_kitty = TradableKittyData::default_tradable_kitty();
    input_kitty.price = 101;
    let output_kitty = input_kitty.clone();

    let input_coin1 = Coin::<0>(100);
    let input_coin2 = Coin::<0>(90);
    let output_coin = Coin::<0>(0);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin1.into(), input_coin2.into()],
        &[],
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(
        result,
        Err(TradeableKittyError::MoneyError(MoneyError::ZeroValueCoin))
    )
}

#[test]
fn buy_kitty_basic_kitty_fails() {
    let input_kitty = TradableKittyData::default_kitty();
    let output_kitty = input_kitty.clone();

    let input_coin1 = Coin::<0>(100);
    let input_coin2 = Coin::<0>(90);
    let output_coin = Coin::<0>(0);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin1.into(), input_coin2.into()],
        &[],
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(result, Err(TradeableKittyError::BadlyTyped))
}
