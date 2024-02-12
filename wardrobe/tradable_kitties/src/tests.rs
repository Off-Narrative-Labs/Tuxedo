//! Tests for the Crypto Kitties Piece

use super::*;
use kitties::DadKittyStatus;
use kitties::KittyDNA;
use kitties::MomKittyStatus;
use kitties::Parent;
use sp_runtime::testing::H256;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::traits::Hash;

/// A bogus data type used in tests for type validation
#[derive(Encode, Decode)]
struct Bogus;

impl UtxoData for Bogus {
    const TYPE_ID: [u8; 4] = *b"bogs";
}

impl TradableKittyData {
    pub fn default_dad() -> Self {
        let kitty_basic = KittyData {
            parent: Parent::Dad(DadKittyStatus::RearinToGo),
            ..Default::default()
        };
        TradableKittyData {
            kitty_basic_data: kitty_basic,
            ..Default::default()
        }
    }

    pub fn default_child() -> Self {
        let mom = Self::default();
        let dad = Self::default_dad();

        let kitty_basic = KittyData {
            parent: Parent::Mom(MomKittyStatus::RearinToGo),
            free_breedings: 2,
            name: *b"tkty",
            dna: KittyDNA(BlakeTwo256::hash_of(&(
                mom.kitty_basic_data.dna,
                dad.kitty_basic_data.dna,
                mom.kitty_basic_data.num_breedings + 1,
                dad.kitty_basic_data.num_breedings + 1,
            ))),
            num_breedings: 0,
        };

        TradableKittyData {
            kitty_basic_data: kitty_basic,
            ..Default::default()
        }
    }

    pub fn default_family() -> Box<Vec<Self>> {
        let mut new_mom: TradableKittyData = TradableKittyData::default();
        new_mom.kitty_basic_data.parent = Parent::Mom(MomKittyStatus::HadBirthRecently);
        new_mom.kitty_basic_data.num_breedings += 1;
        new_mom.kitty_basic_data.free_breedings -= 1;

        let mut new_dad = TradableKittyData::default_dad();
        new_dad.kitty_basic_data.parent = Parent::Dad(DadKittyStatus::Tired);
        new_dad.kitty_basic_data.num_breedings += 1;
        new_dad.kitty_basic_data.free_breedings -= 1;

        let child = TradableKittyData::default_child();

        Box::new(vec![new_mom, new_dad, child])
    }

    pub fn default_updated() -> Self {
        let kitty_basic = KittyData {
            name: *b"tomy",
            ..Default::default()
        };
        TradableKittyData {
            kitty_basic_data: kitty_basic,
            is_available_for_sale: true,
            price: Some(200),
            ..Default::default()
        }
    }
}

// From below mint tradable kitty test cases start.

#[test]
fn mint_happy_path_works() {
    let result = TradableKittyConstraintChecker::<0>::Mint.check(
        &[],
        &[], // no peeks
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
    );
    assert!(result.is_ok());
}

#[test]
fn mint_with_input_fails() {
    let result = TradableKittyConstraintChecker::<0>::Mint.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::MintingWithInputs)
    );
}
#[test]
fn mint_without_output_fails() {
    let result = TradableKittyConstraintChecker::<0>::Mint.check(
        &[],
        &[], // no peeks
        &[],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::MintingNothing)
    );
}
#[test]
fn mint_with_wrong_output_type_fails() {
    let result = TradableKittyConstraintChecker::<0>::Mint.check(
        &[],
        &[], // no peeks
        &[
            Bogus.into(),
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
    );
    assert_eq!(result, Err(TradableKittyConstraintCheckerError::BadlyTyped));
}

// From below breed tradable kitty test cases start.

#[test]
fn breed_happy_path_works() {
    let new_family = TradableKittyData::default_family();
    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_family[0].clone().into(),
            new_family[1].clone().into(),
            new_family[2].clone().into(),
        ],
    );
    assert!(result.is_ok());
}

#[test]
fn breed_wrong_input_type_fails() {
    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[Bogus.into(), Bogus.into()],
        &[], // no peeks
        &[],
    );
    assert_eq!(result, Err(TradableKittyConstraintCheckerError::BadlyTyped));
}

#[test]
fn breed_wrong_output_type_fails() {
    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[Bogus.into(), Bogus.into(), Bogus.into()],
    );
    assert_eq!(result, Err(TradableKittyConstraintCheckerError::BadlyTyped));
}

#[test]
fn inputs_dont_contain_two_parents_fails() {
    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[TradableKittyData::default().into()],
        &[], // no peeks
        &[],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::TwoParentsDoNotExist)
    );
}

#[test]
fn outputs_dont_contain_all_family_members_fails() {
    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[TradableKittyData::default().into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NotEnoughFamilyMembers)
    );
}

#[test]
fn breed_two_dads_fails() {
    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default_dad().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[TradableKittyData::default().into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::TwoDadsNotValid)
    );
}

#[test]
fn breed_two_moms_fails() {
    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default().into(),
        ],
        &[], // no peeks
        &[TradableKittyData::default().into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::TwoMomsNotValid)
    );
}

#[test]
fn first_input_not_mom_fails() {
    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default_dad().into(),
            TradableKittyData::default().into(),
        ],
        &[], // no peeks
        &[],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::TwoDadsNotValid)
    )
}

#[test]
fn first_output_not_mom_fails() {
    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            TradableKittyData::default_dad().into(),
            TradableKittyData::default().into(),
            TradableKittyData::default_child().into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::TwoDadsNotValid)
    );
}

#[test]
fn breed_mom_when_she_gave_birth_recently_fails() {
    let mut new_momma = TradableKittyData::default();
    new_momma.kitty_basic_data.parent = Parent::Mom(MomKittyStatus::HadBirthRecently);

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[new_momma.into(), TradableKittyData::default_dad().into()],
        &[], // no peeks
        &[],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::MomNotReadyYet)
    );
}

#[test]
fn breed_dad_when_he_is_tired_fails() {
    let mut tired_dadda = TradableKittyData::default_dad();
    tired_dadda.kitty_basic_data.parent = Parent::Dad(DadKittyStatus::Tired);

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[TradableKittyData::default().into(), tired_dadda.into()],
        &[], // no peeks
        &[],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::DadTooTired)
    );
}

#[test]
fn check_mom_breedings_overflow_fails() {
    let mut test_mom = TradableKittyData::default();
    test_mom.kitty_basic_data.num_breedings = u128::MAX;

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[test_mom.into(), TradableKittyData::default_dad().into()],
        &[], // no peeks
        &[],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::TooManyBreedingsForKitty)
    );
}

#[test]
fn check_dad_breedings_overflow_fails() {
    let mut test_dad = TradableKittyData::default_dad();
    test_dad.kitty_basic_data.num_breedings = u128::MAX;

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[TradableKittyData::default().into(), test_dad.into()],
        &[], // no peeks
        &[],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::TooManyBreedingsForKitty)
    );
}

#[test]
fn check_mom_free_breedings_zero_fails() {
    let mut test_mom = TradableKittyData::default();
    test_mom.kitty_basic_data.free_breedings = 0;

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[test_mom.into(), TradableKittyData::default_dad().into()],
        &[], // no peeks
        &[],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NotEnoughFreeBreedings)
    );
}

#[test]
fn check_dad_free_breedings_zero_fails() {
    let mut test_dad = TradableKittyData::default_dad();
    test_dad.kitty_basic_data.free_breedings = 0;

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[TradableKittyData::default().into(), test_dad.into()],
        &[], // no peeks
        &[],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NotEnoughFreeBreedings)
    );
}

#[test]
fn check_new_mom_free_breedings_incorrect_fails() {
    let new_family = TradableKittyData::default_family();
    let mut new_mom = new_family[0].clone();
    new_mom.kitty_basic_data.free_breedings = 2;

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_mom.into(),
            new_family[1].clone().into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NewParentFreeBreedingsIncorrect)
    );
}

#[test]
fn check_new_dad_free_breedings_incorrect_fails() {
    let new_family = TradableKittyData::default_family();
    let mut new_dad = new_family[1].clone();
    new_dad.kitty_basic_data.free_breedings = 2;

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_family[0].clone().into(),
            new_dad.into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NewParentFreeBreedingsIncorrect)
    );
}

#[test]
fn check_new_mom_num_breedings_incorrect_fails() {
    let new_family = TradableKittyData::default_family();
    let mut new_mom = new_family[0].clone();
    new_mom.kitty_basic_data.num_breedings = 0;

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_mom.into(),
            new_family[1].clone().into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NewParentNumberBreedingsIncorrect)
    );
}

#[test]
fn check_new_dad_num_breedings_incorrect_fails() {
    let new_family = TradableKittyData::default_family();
    let mut new_dad = new_family[1].clone();
    new_dad.kitty_basic_data.num_breedings = 0;

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_family[0].clone().into(),
            new_dad.into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NewParentNumberBreedingsIncorrect)
    );
}

#[test]
fn check_new_mom_dna_doesnt_match_old_fails() {
    let new_family = TradableKittyData::default_family();
    let mut new_mom = new_family[0].clone();
    new_mom.kitty_basic_data.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoci"));

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_mom.into(),
            new_family[1].clone().into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NewParentDnaDoesntMatchOld)
    );
}

#[test]
fn check_new_dad_dna_doesnt_match_old_fails() {
    let new_family = TradableKittyData::default_family();
    let mut new_dad = new_family[1].clone();
    new_dad.kitty_basic_data.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoci"));

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_family[0].clone().into(),
            new_dad.into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NewParentDnaDoesntMatchOld)
    );
}

#[test]
fn check_child_dna_incorrect_fails() {
    let new_family = TradableKittyData::default_family();
    let mut new_child = new_family[2].clone();
    new_child.kitty_basic_data.dna = KittyDNA(H256::zero());

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_family[0].clone().into(),
            new_family[1].clone().into(),
            new_child.into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NewChildDnaIncorrect)
    );
}

#[test]
fn check_child_dad_parent_tired_fails() {
    let new_family = TradableKittyData::default_family();
    let mut new_child = new_family[2].clone();
    new_child.kitty_basic_data.parent = Parent::Dad(DadKittyStatus::Tired);

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_family[0].clone().into(),
            new_family[1].clone().into(),
            new_child.into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NewChildIncorrectParentInfo)
    );
}

#[test]
fn check_child_mom_parent_recently_gave_birth_fails() {
    let new_family = TradableKittyData::default_family();
    let mut new_child = new_family[2].clone();
    new_child.kitty_basic_data.parent = Parent::Mom(MomKittyStatus::HadBirthRecently);

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_family[0].clone().into(),
            new_family[1].clone().into(),
            new_child.into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NewChildIncorrectParentInfo)
    );
}

#[test]
fn check_child_free_breedings_incorrect_fails() {
    let new_family = TradableKittyData::default_family();
    let mut new_child = new_family[2].clone();
    new_child.kitty_basic_data.free_breedings = KittyHelpers::NUM_FREE_BREEDINGS + 1;

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_family[0].clone().into(),
            new_family[1].clone().into(),
            new_child.into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NewChildFreeBreedingsIncorrect)
    );
}

#[test]
fn check_child_num_breedings_non_zero_fails() {
    let new_family = TradableKittyData::default_family();
    let mut new_child = new_family[2].clone();
    new_child.kitty_basic_data.num_breedings = 42;

    let result = TradableKittyConstraintChecker::<0>::Breed.check(
        &[
            TradableKittyData::default().into(),
            TradableKittyData::default_dad().into(),
        ],
        &[], // no peeks
        &[
            new_family[0].clone().into(),
            new_family[1].clone().into(),
            new_child.into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NewChildHasNonZeroBreedings)
    );
}

// From below update tradable kitty properties test cases starts
#[test]
fn update_properties_happy_path_works() {
    let result = TradableKittyConstraintChecker::<0>::UpdateProperties.check(
        &[TradableKittyData::default().into()],
        &[], // no peeks
        &[TradableKittyData::default_updated().into()],
    );
    assert!(result.is_ok());
}

#[test]
fn update_properties_update_no_input_fails() {
    let result = TradableKittyConstraintChecker::<0>::UpdateProperties.check(
        &[],
        &[], // no peeks
        &[TradableKittyData::default_updated().into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::InputMissingUpdatingNothing)
    );
}

#[test]
fn update_properties_update_num_of_input_output_mismatch_fails() {
    let mut updated_kitty = TradableKittyData::default();
    updated_kitty.kitty_basic_data.dna =
        KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoci"));
    let result = TradableKittyConstraintChecker::<0>::UpdateProperties.check(
        &[TradableKittyData::default().into()],
        &[], // no peeks
        &[
            TradableKittyData::default_updated().into(),
            updated_kitty.into(),
        ],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::MismatchBetweenNumberOfInputAndUpdateUpadtingNothing)
    );
}

#[test]
fn update_properties_update_dna_fails() {
    let mut updated_kitty = TradableKittyData::default();
    updated_kitty.kitty_basic_data.dna =
        KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoci"));
    let result = TradableKittyConstraintChecker::<0>::UpdateProperties.check(
        &[TradableKittyData::default().into()],
        &[], // no peeks
        &[updated_kitty.into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::KittyDnaCannotBeUpdated)
    );
}

#[test]
fn update_properties_update_gender_fails() {
    let updated_kitty = TradableKittyData::default_dad();
    let result = TradableKittyConstraintChecker::<0>::UpdateProperties.check(
        &[TradableKittyData::default().into()],
        &[], // no peeks
        &[updated_kitty.into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::KittyGenderCannotBeUpdated)
    );
}

#[test]
fn update_properties_update_free_breedings_fails() {
    let mut updated_kitty = TradableKittyData::default();
    updated_kitty.kitty_basic_data.free_breedings = 5;
    let result = TradableKittyConstraintChecker::<0>::UpdateProperties.check(
        &[TradableKittyData::default().into()],
        &[], // no peeks
        &[updated_kitty.into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::FreeBreedingCannotBeUpdated)
    );
}

#[test]
fn update_properties_update_num_breedings_fails() {
    let mut updated_kitty = TradableKittyData::default();
    updated_kitty.kitty_basic_data.num_breedings = 5;
    let result = TradableKittyConstraintChecker::<0>::UpdateProperties.check(
        &[TradableKittyData::default().into()],
        &[], // no peeks
        &[updated_kitty.into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::NumOfBreedingCannotBeUpdated)
    );
}

#[test]
fn update_properties_non_none_price_when_is_avilable_for_sale_is_false_fails() {
    let mut updated_kitty = TradableKittyData::default();
    updated_kitty.price = Some(100);
    let result = TradableKittyConstraintChecker::<0>::UpdateProperties.check(
        &[TradableKittyData::default().into()],
        &[], // no peeks
        &[updated_kitty.into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::UpdatedKittyIncorrectPrice)
    );
}

#[test]
fn update_properties_none_price_when_is_avilable_for_sale_is_true_fails() {
    let mut updated_kitty = TradableKittyData::default();
    updated_kitty.is_available_for_sale = true;
    updated_kitty.price = None;
    let result = TradableKittyConstraintChecker::<0>::UpdateProperties.check(
        &[TradableKittyData::default().into()],
        &[], // no peeks
        &[updated_kitty.into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::UpdatedKittyIncorrectPrice)
    );
}

// From below buy tradable kitty test cases start.
#[test]
fn buy_happy_path_single_input_coinworks() {
    let mut input_kitty = TradableKittyData::default();
    input_kitty.is_available_for_sale = true;
    input_kitty.price = Some(100);
    let output_kitty = input_kitty.clone();

    let input_coin = Coin::<0>(100);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin.into()],
        &[], // no peeks
        &[output_kitty.into(), output_coin.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn buy_happy_path_multiple_input_coinworks() {
    let mut input_kitty = TradableKittyData::default();
    input_kitty.is_available_for_sale = true;
    input_kitty.price = Some(100);
    let output_kitty = input_kitty.clone();

    let input_coin1 = Coin::<0>(10);
    let input_coin2 = Coin::<0>(90);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin1.into(), input_coin2.into()],
        &[], // no peeks
        &[output_kitty.into(), output_coin.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn buy_kityy_is_available_for_sale_false_fails() {
    let mut input_kitty = TradableKittyData::default();
    input_kitty.price = Some(100);
    let output_kitty = input_kitty.clone();

    let input_coin = Coin::<0>(100);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin.into()],
        &[], // no peeks
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::KittyNotForSale)
    );
}
#[test]
fn buy_kityy_is_available_for_sale_true_price_none_fails() {
    let mut input_kitty = TradableKittyData::default();
    input_kitty.is_available_for_sale = true;
    let output_kitty = input_kitty.clone();
    let input_coin = Coin::<0>(100);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin.into()],
        &[], // no peeks
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::KittyPriceCantBeNone)
    );
}
#[test]
fn buy_kityy_wrong_input_type_fails() {
    let mut input_kitty = TradableKittyData::default();
    input_kitty.is_available_for_sale = true;
    input_kitty.price = Some(101);
    let output_kitty = input_kitty.clone();

    let input_coin = Coin::<0>(100);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin.into(), Bogus.into()],
        &[], // no peeks
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(result, Err(TradableKittyConstraintCheckerError::BadlyTyped));
}

#[test]
fn buy_kityy_wrong_output_type_fails() {
    let mut input_kitty = TradableKittyData::default();
    input_kitty.is_available_for_sale = true;
    input_kitty.price = Some(101);
    let output_kitty = input_kitty.clone();

    let input_coin = Coin::<0>(100);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin.into()],
        &[], // no peeks
        &[output_kitty.into(), output_coin.into(), Bogus.into()],
    );
    assert_eq!(result, Err(TradableKittyConstraintCheckerError::BadlyTyped));
}

#[test]
fn buy_kitty_less_money_than_price_of_kitty_fails() {
    let mut input_kitty = TradableKittyData::default();
    input_kitty.is_available_for_sale = true;
    input_kitty.price = Some(101);
    let output_kitty = input_kitty.clone();

    let input_coin1 = Coin::<0>(100);
    //  let input_coin2 = Coin::<0>(90);
    let output_coin = Coin::<0>(100);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin1.into()],
        &[], // no peeks
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::InsufficientCollateralToBuyKitty)
    );
}

#[test]
fn buy_kitty_coin_output_value_exceeds_input_coin_value_fails() {
    let mut input_kitty = TradableKittyData::default();
    input_kitty.is_available_for_sale = true;
    input_kitty.price = Some(101);
    let output_kitty = input_kitty.clone();

    let input_coin1 = Coin::<0>(100);
    let input_coin2 = Coin::<0>(90);
    let output_coin = Coin::<0>(300);

    let result = TradableKittyConstraintChecker::<0>::Buy.check(
        &[input_kitty.into(), input_coin1.into(), input_coin2.into()],
        &[], // no peeks
        &[output_kitty.into(), output_coin.into()],
    );
    assert_eq!(
        result,
        Err(TradableKittyConstraintCheckerError::OutputsExceedInputs)
    )
}
