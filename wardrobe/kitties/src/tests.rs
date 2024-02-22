//! Tests for the Crypto Kitties Piece

use super::*;
/// A bogus data type used in tests for type validation
#[derive(Encode, Decode)]
struct Bogus;

impl UtxoData for Bogus {
    const TYPE_ID: [u8; 4] = *b"bogs";
}

impl KittyData {
    pub fn default_dad() -> Self {
        KittyData {
            parent: Parent::Dad(DadKittyStatus::RearinToGo),
            ..Default::default()
        }
    }

    pub fn default_child() -> Self {
        let mom = Self::default();
        let dad = Self::default_dad();

        KittyData {
            parent: Parent::Mom(MomKittyStatus::RearinToGo),
            free_breedings: 2,
            name: *b"bkty",
            dna: KittyDNA(BlakeTwo256::hash_of(&(
                mom.dna,
                dad.dna,
                mom.num_breedings + 1,
                dad.num_breedings + 1,
            ))),
            num_breedings: 0,
        }
    }

    pub fn default_family() -> Box<Vec<Self>> {
        let mut new_mom: KittyData = KittyData::default();
        new_mom.parent = Parent::Mom(MomKittyStatus::HadBirthRecently);
        new_mom.num_breedings += 1;
        new_mom.free_breedings -= 1;

        let mut new_dad = KittyData::default_dad();
        new_dad.parent = Parent::Dad(DadKittyStatus::Tired);
        new_dad.num_breedings += 1;
        new_dad.free_breedings -= 1;

        let child = KittyData::default_child();

        Box::new(vec![new_mom, new_dad, child])
    }
}

#[test]
fn create_happy_path_works() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Create,
        &[],
        &[], 
        &[KittyData::default().into(), KittyData::default_dad().into()],
    );
    assert!(result.is_ok());
}

#[test]
fn create_with_input_fails() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Create,
        &[KittyData::default().into()],
        &[], 
        &[],
    );
    assert_eq!(result, Err(ConstraintCheckerError::CreatingWithInputs));
}
#[test]
fn create_without_output_fails() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Create,
        &[],
        &[], 
        &[],
    );
    assert_eq!(result, Err(ConstraintCheckerError::CreatingNothing));
}
#[test]
fn create_with_wrong_output_type_fails() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Create,
        &[],
        &[], 
        &[
            Bogus.into(),
            KittyData::default().into(),
            KittyData::default_dad().into(),
        ],
    );
    assert_eq!(result, Err(ConstraintCheckerError::BadlyTyped));
}

#[test]
fn breed_happy_path_works() {
    let new_family = KittyData::default_family();
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
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
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[Bogus.into(), Bogus.into()],
        &[],
        &[],
    );
    assert_eq!(result, Err(ConstraintCheckerError::BadlyTyped));
}

#[test]
fn breed_wrong_output_type_fails() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[Bogus.into(), Bogus.into(), Bogus.into()],
    );
    assert_eq!(result, Err(ConstraintCheckerError::BadlyTyped));
}

#[test]
fn inputs_dont_contain_two_parents_fails() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into()],
        &[],
        &[],
    );
    assert_eq!(result, Err(ConstraintCheckerError::TwoParentsDoNotExist));
}

#[test]
fn outputs_dont_contain_all_family_members_fails() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[KittyData::default().into()],
    );
    assert_eq!(result, Err(ConstraintCheckerError::NotEnoughFamilyMembers));
}

#[test]
fn breed_two_dads_fails() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[
            KittyData::default_dad().into(),
            KittyData::default_dad().into(),
        ],
        &[],
        &[KittyData::default().into()],
    );
    assert_eq!(result, Err(ConstraintCheckerError::TwoDadsNotValid));
}

#[test]
fn breed_two_moms_fails() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default().into()],
        &[],
        &[KittyData::default().into()],
    );
    assert_eq!(result, Err(ConstraintCheckerError::TwoMomsNotValid));
}

#[test]
fn first_input_not_mom_fails() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default_dad().into(), KittyData::default().into()],
        &[],
        &[],
    );
    assert_eq!(result, Err(ConstraintCheckerError::TwoDadsNotValid))
}

#[test]
fn first_output_not_mom_fails() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            KittyData::default_dad().into(),
            KittyData::default().into(),
            KittyData::default_child().into(),
        ],
    );
    assert_eq!(result, Err(ConstraintCheckerError::TwoDadsNotValid));
}

#[test]
fn breed_mom_when_she_gave_birth_recently_fails() {
    let mut new_momma = KittyData::default();
    new_momma.parent = Parent::Mom(MomKittyStatus::HadBirthRecently);

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[new_momma.into(), KittyData::default_dad().into()],
        &[],
        &[],
    );
    assert_eq!(result, Err(ConstraintCheckerError::MomNotReadyYet));
}

#[test]
fn breed_dad_when_he_is_tired_fails() {
    let mut tired_dadda = KittyData::default_dad();
    tired_dadda.parent = Parent::Dad(DadKittyStatus::Tired);

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), tired_dadda.into()],
        &[],
        &[],
    );
    assert_eq!(result, Err(ConstraintCheckerError::DadTooTired));
}

#[test]
fn check_mom_breedings_overflow_fails() {
    let mut test_mom = KittyData::default();
    test_mom.num_breedings = u128::MAX;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[test_mom.into(), KittyData::default_dad().into()],
        &[],
        &[],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::TooManyBreedingsForKitty)
    );
}

#[test]
fn check_dad_breedings_overflow_fails() {
    let mut test_dad = KittyData::default_dad();
    test_dad.num_breedings = u128::MAX;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), test_dad.into()],
        &[],
        &[],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::TooManyBreedingsForKitty)
    );
}

#[test]
fn check_mom_free_breedings_zero_fails() {
    let mut test_mom = KittyData::default();
    test_mom.free_breedings = 0;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[test_mom.into(), KittyData::default_dad().into()],
        &[],
        &[],
    );
    assert_eq!(result, Err(ConstraintCheckerError::NotEnoughFreeBreedings));
}

#[test]
fn check_dad_free_breedings_zero_fails() {
    let mut test_dad = KittyData::default_dad();
    test_dad.free_breedings = 0;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), test_dad.into()],
        &[],
        &[],
    );
    assert_eq!(result, Err(ConstraintCheckerError::NotEnoughFreeBreedings));
}

#[test]
fn check_new_mom_free_breedings_incorrect_fails() {
    let new_family = KittyData::default_family();
    let mut new_mom = new_family[0].clone();
    new_mom.free_breedings = 2;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            new_mom.into(),
            new_family[1].clone().into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::NewParentFreeBreedingsIncorrect)
    );
}

#[test]
fn check_new_dad_free_breedings_incorrect_fails() {
    let new_family = KittyData::default_family();
    let mut new_dad = new_family[1].clone();
    new_dad.free_breedings = 2;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            new_family[0].clone().into(),
            new_dad.into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::NewParentFreeBreedingsIncorrect)
    );
}

#[test]
fn check_new_mom_num_breedings_incorrect_fails() {
    let new_family = KittyData::default_family();
    let mut new_mom = new_family[0].clone();
    new_mom.num_breedings = 0;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            new_mom.into(),
            new_family[1].clone().into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::NewParentNumberBreedingsIncorrect)
    );
}

#[test]
fn check_new_dad_num_breedings_incorrect_fails() {
    let new_family = KittyData::default_family();
    let mut new_dad = new_family[1].clone();
    new_dad.num_breedings = 0;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            new_family[0].clone().into(),
            new_dad.into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::NewParentNumberBreedingsIncorrect)
    );
}

#[test]
fn check_new_mom_dna_doesnt_match_old_fails() {
    let new_family = KittyData::default_family();
    let mut new_mom = new_family[0].clone();
    new_mom.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoci"));

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            new_mom.into(),
            new_family[1].clone().into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::NewParentDnaDoesntMatchOld)
    );
}

#[test]
fn check_new_dad_dna_doesnt_match_old_fails() {
    let new_family = KittyData::default_family();
    let mut new_dad = new_family[1].clone();
    new_dad.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoci"));

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            new_family[0].clone().into(),
            new_dad.into(),
            new_family[2].clone().into(),
        ],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::NewParentDnaDoesntMatchOld)
    );
}

#[test]
fn check_child_dna_incorrect_fails() {
    let new_family = KittyData::default_family();
    let mut new_child = new_family[2].clone();
    new_child.dna = KittyDNA(H256::zero());

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            new_family[0].clone().into(),
            new_family[1].clone().into(),
            new_child.into(),
        ],
    );
    assert_eq!(result, Err(ConstraintCheckerError::NewChildDnaIncorrect));
}

#[test]
fn check_child_dad_parent_tired_fails() {
    let new_family = KittyData::default_family();
    let mut new_child = new_family[2].clone();
    new_child.parent = Parent::Dad(DadKittyStatus::Tired);

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            new_family[0].clone().into(),
            new_family[1].clone().into(),
            new_child.into(),
        ],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::NewChildIncorrectParentInfo)
    );
}

#[test]
fn check_child_mom_parent_recently_gave_birth_fails() {
    let new_family = KittyData::default_family();
    let mut new_child = new_family[2].clone();
    new_child.parent = Parent::Mom(MomKittyStatus::HadBirthRecently);

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            new_family[0].clone().into(),
            new_family[1].clone().into(),
            new_child.into(),
        ],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::NewChildIncorrectParentInfo)
    );
}

#[test]
fn check_child_free_breedings_incorrect_fails() {
    let new_family = KittyData::default_family();
    let mut new_child = new_family[2].clone();
    new_child.free_breedings = KittyHelpers::NUM_FREE_BREEDINGS + 1;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            new_family[0].clone().into(),
            new_family[1].clone().into(),
            new_child.into(),
        ],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::NewChildFreeBreedingsIncorrect)
    );
}

#[test]
fn check_child_num_breedings_non_zero_fails() {
    let new_family = KittyData::default_family();
    let mut new_child = new_family[2].clone();
    new_child.num_breedings = 42;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::Breed,
        &[KittyData::default().into(), KittyData::default_dad().into()],
        &[],
        &[
            new_family[0].clone().into(),
            new_family[1].clone().into(),
            new_child.into(),
        ],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::NewChildHasNonZeroBreedings)
    );
}

#[test]
fn update_name_happy_path_works() {
    let input = KittyData::default_dad();
    let mut output = KittyData::default_dad();
    output.name = *b"kty1";

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::UpdateKittyName,
        &[input.into()],
        &[], 
        &[output.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn update_name_happy_path_with_multiple_input_sworks() {
    let input1 = KittyData::default_dad();
    let mut input2 = KittyData::default();
    input2.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialaroci"));
    let mut output1 = input1.clone();
    let mut output2 = input2.clone();

    output1.name = *b"kty1";
    output2.name = *b"kty2";

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::UpdateKittyName,
        &[input1.into(), input2.into()],
        &[], 
        &[output1.into(), output2.into()],
    );
    assert!(result.is_ok());
}

#[test]
fn update_name_inputs_and_outputs_number_mismatch_fails() {
    let input1 = KittyData::default_dad();
    let input2 = KittyData::default_dad();
    let mut output1 = input1.clone();
    let mut output2 = input2.clone();

    output1.name = *b"kty1";
    output2.name = *b"kty2";

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::UpdateKittyName,
        &[input1.into(), input2.into()],
        &[], 
        &[output1.into()],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::InvalidNumberOfInputOutput)
    );
}
#[test]
fn update_name_no_inputs_fails() {
    let output = KittyData::default_dad();

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::UpdateKittyName,
        &[],
        &[], 
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::InvalidNumberOfInputOutput)
    );
}

#[test]
fn update_name_no_output_fails() {
    let input = KittyData::default_dad();

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::UpdateKittyName,
        &[input.into()],
        &[], 
        &[],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::InvalidNumberOfInputOutput)
    );
}
#[test]
fn update_name_dna_update_fails() {
    let input = KittyData::default_dad();
    let mut output = input.clone();
    output.dna = KittyDNA(H256::from_slice(b"superkalifragislisticexpialadoca"));
    output.name = *b"kty1";

    let input1 = KittyData::default_dad();
    let mut output1 = input1.clone();
    output1.name = *b"kty2";

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::UpdateKittyName,
        &[input1.into(), input.into()],
        &[], 
        &[output1.into(), output.into()],
    );
    assert_eq!(result, Err(ConstraintCheckerError::OutputUtxoMissingError));
}

#[test]
fn update_name_name_unupdated_path_fails() {
    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::UpdateKittyName,
        &[KittyData::default_dad().into()],
        &[], 
        &[KittyData::default_dad().into()],
    );
    assert_eq!(result, Err(ConstraintCheckerError::KittyNameUnAltered));
}

#[test]
fn update_name_free_breeding_updated_path_fails() {
    let mut output = KittyData::default_dad();
    output.name = *b"kty1";
    output.free_breedings += 1;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::UpdateKittyName,
        &[KittyData::default().into()],
        &[], 
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::FreeBreedingCannotBeUpdated)
    );
}

#[test]
fn update_name_num_of_breeding_updated_path_fails() {
    let mut output = KittyData::default_dad();
    output.name = *b"kty1";
    output.num_breedings += 1;

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::UpdateKittyName,
        &[KittyData::default().into()],
        &[], 
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::NumOfBreedingCannotBeUpdated)
    );
}

#[test]
fn update_name_gender_updated_path_fails() {
    let input = KittyData::default();
    let mut output = KittyData::default_dad();
    output.name = *b"kty1";

    let result = FreeKittyConstraintChecker::check(
        &FreeKittyConstraintChecker::UpdateKittyName,
        &[input.into()],
        &[], 
        &[output.into()],
    );
    assert_eq!(
        result,
        Err(ConstraintCheckerError::KittyGenderCannotBeUpdated)
    );
}
