//! An NFT game inspired by cryptokitties.
//! This is a game which allows for kitties to be bred based on a few factors
//! 1.) Mom and Tired have to be in a state where they are ready to breed
//! 2.) Each Mom and Dad have some DNA and the child will have unique DNA combined from the both of them
//!     Linkable back to the Mom and Dad
//! 3.) The game also allows Kitties to have a cooling off period inbetween breeding before they can be bred again.
//! 4.) A rest operation allows for a Mom Kitty and a Dad Kitty to be cooled off
//!
//! In order to submit a valid transaction you must strutucture it as follows:
//! 1.) Input must contain 1 mom and 1 dad
//! 2.) Output must contain Mom, Dad, and newly created Child
//! 3.) A child's DNA is calculated by:
//!         BlakeTwo256::hash_of(MomDna, DadDna, MomCurrNumBreedings, DadCurrNumberBreedings)
//!
//! There are a only a finite amount of free breedings available before it starts to cost money
//! to breed kitties.
//!
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, Hash as HashT},
    transaction_validity::TransactionPriority,
};
use sp_std::prelude::*;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, SimpleConstraintChecker,
};

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct FreeKittyConstraintChecker;

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum DadKittyStatus {
    #[default]
    RearinToGo,
    Tired,
}

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum MomKittyStatus {
    #[default]
    RearinToGo,
    HadBirthRecently,
}

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum Parent {
    Mom(MomKittyStatus),
    Dad(DadKittyStatus),
}

impl Default for Parent {
    fn default() -> Self {
        Parent::Mom(MomKittyStatus::RearinToGo)
    }
}

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct KittyDNA(H256);

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct KittyData {
    pub parent: Parent,
    pub free_breedings: u64, // Ignore in breed for money case
    pub dna: KittyDNA,
    pub num_breedings: u128,
}

impl Default for KittyData {
    fn default() -> Self {
        Self {
            parent: Parent::Mom(MomKittyStatus::RearinToGo),
            free_breedings: 2,
            dna: KittyDNA(H256::from_slice(b"mom_kitty_1asdfasdfasdfasdfasdfa")),
            num_breedings: 3,
        }
    }
}

impl UtxoData for KittyData {
    const TYPE_ID: [u8; 4] = *b"Kitt";
}

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum ConstraintCheckerError {
    /// Dynamic typing issue.
    /// This error doesn't discriminate between badly typed inputs and outputs.
    BadlyTyped,
    /// Needed when spending for breeding.
    MinimumSpendAndBreedNotMet,
    /// Need two parents to breed.
    TwoParentsDoNotExist,
    /// Incorrect number of outputs when it comes to breeding.
    NotEnoughFamilyMembers,
    /// Mom has recently given birth and isnt ready to breed.
    MomNotReadyYet,
    /// Dad cannot breed because he is still too tired.
    DadTooTired,
    /// Cannot have two moms when breeding.
    TwoMomsNotValid,
    /// Cannot have two dads when breeding.
    TwoDadsNotValid,
    /// New Mom after breeding should be in HadBirthRecently state.
    NewMomIsStillRearinToGo,
    /// New Dad after breeding should be in Tired state.
    NewDadIsStillRearinToGo,
    /// Number of free breedings of new parent is not correct.
    NewParentFreeBreedingsIncorrect,
    /// New parents DNA does not match the old one parent has to still be the same kitty.
    NewParentDnaDoesntMatchOld,
    /// New parent Breedings has not incremented or is incorrect.
    NewParentNumberBreedingsIncorrect,
    /// New child DNA is not correct given the protocol.
    NewChildDnaIncorrect,
    /// New child doesnt have the correct number of free breedings.
    NewChildFreeBreedingsIncorrect,
    /// New child has non zero breedings which is impossible because it was just born.
    NewChildHasNonZeroBreedings,
    /// New child parent info is either in Tired state or HadBirthRecently state which is not possible.
    NewChildIncorrectParentInfo,
    /// Too many breedings for this kitty can no longer breed.
    TooManyBreedingsForKitty,
    /// Not enough free breedings available for these parents.
    NotEnoughFreeBreedings,
}

trait Breed {
    /// The Cost to breed a kitty if it is not free.
    const COST: u128;
    /// Number of free breedings a kitty will have.
    const NUM_FREE_BREEDINGS: u64;
    /// Error type for all Kitty errors.
    type Error: Into<ConstraintCheckerError>;
    /// Check if the two parents (Mom, Dad) proposed are capable of breeding.
    fn can_breed(mom: &KittyData, dad: &KittyData) -> Result<(), Self::Error>;
    /// Checks if mom is in the correct state and capable of breeding.
    fn check_mom_can_breed(mom: &KittyData) -> Result<(), Self::Error>;
    /// Checks if dad is in the correct state and capable of breeding.
    fn check_dad_can_breed(dad: &KittyData) -> Result<(), Self::Error>;
    /// Makes sure each parent has a non-zero number of free breedings.
    fn check_free_breedings(mom: &KittyData, dad: &KittyData) -> Result<(), Self::Error>;
    /// Checks outputs which consists of (Mom, Dad, Child) is correctly formulated.
    fn check_new_family(
        old_mom: &KittyData,
        old_dad: &KittyData,
        new_family: &[DynamicallyTypedData],
    ) -> Result<(), Self::Error>;
    /// Checks if new mom matches the old ones DNA and changes state correctly.
    fn check_new_mom(old_mom: &KittyData, new_mom: &KittyData) -> Result<(), Self::Error>;
    /// Checks if new dad matches the old ones DNA and changes state correctly.
    fn check_new_dad(old_dad: &KittyData, new_dad: &KittyData) -> Result<(), Self::Error>;
    /// Checks if new child DNA is formulated correctly and is initialized to the proper state.
    fn check_child(
        new_mom: &KittyData,
        new_dad: &KittyData,
        child: &KittyData,
    ) -> Result<(), Self::Error>;
}

pub struct KittyHelpers;
impl Breed for KittyHelpers {
    const COST: u128 = 5u128;
    const NUM_FREE_BREEDINGS: u64 = 2u64;
    type Error = ConstraintCheckerError;
    /// Checks:
    ///     - Mom can breed
    ///     - Dad can breed
    ///
    fn can_breed(mom: &KittyData, dad: &KittyData) -> Result<(), Self::Error> {
        Self::check_mom_can_breed(mom)?;
        Self::check_dad_can_breed(dad)?;
        Self::check_free_breedings(mom, dad)?;
        Ok(())
    }

    /// Checks:
    ///     - Mom is in `RearinToGo` state
    ///     - Mom number of breedings is not maxed out
    ///
    fn check_mom_can_breed(mom: &KittyData) -> Result<(), Self::Error> {
        match &mom.parent {
            Parent::Mom(status) => {
                if let MomKittyStatus::HadBirthRecently = status {
                    return Err(Self::Error::MomNotReadyYet);
                }
            }
            Parent::Dad(_) => return Err(Self::Error::TwoDadsNotValid),
        }
        mom.num_breedings
            .checked_add(1)
            .ok_or(Self::Error::TooManyBreedingsForKitty)?;
        Ok(())
    }

    /// Checks:
    ///     - Dad is in `RearinToGo` state
    ///     - Dad number of breedings is not maxed out
    ///
    fn check_dad_can_breed(dad: &KittyData) -> Result<(), Self::Error> {
        match &dad.parent {
            Parent::Dad(status) => {
                if let DadKittyStatus::Tired = status {
                    return Err(Self::Error::DadTooTired);
                }
            }
            Parent::Mom(_) => return Err(Self::Error::TwoMomsNotValid),
        }
        dad.num_breedings
            .checked_add(1)
            .ok_or(Self::Error::TooManyBreedingsForKitty)?;
        Ok(())
    }

    /// Checks:
    ///     - Both parents free breedings is non-zero
    ///
    fn check_free_breedings(mom: &KittyData, dad: &KittyData) -> Result<(), Self::Error> {
        let mom_breedings = mom.free_breedings;
        let dad_breedings = dad.free_breedings;
        if (mom_breedings == 0) || (dad_breedings == 0) {
            return Err(Self::Error::NotEnoughFreeBreedings);
        }
        Ok(())
    }

    fn check_new_family(
        old_mom: &KittyData,
        old_dad: &KittyData,
        new_family: &[DynamicallyTypedData],
    ) -> Result<(), Self::Error> {
        // Output Side
        ensure!(new_family.len() == 3, Self::Error::NotEnoughFamilyMembers);
        let new_mom = KittyData::try_from(&new_family[0])?;
        let new_dad = KittyData::try_from(&new_family[1])?;
        let child = KittyData::try_from(&new_family[2])?;
        Self::check_new_mom(old_mom, &new_mom)?;
        Self::check_new_dad(old_dad, &new_dad)?;
        Self::check_child(&new_mom, &new_dad, &child)?;
        Ok(())
    }

    /// Checks:
    ///     - Mom is now in `HadBirthRecently`
    ///     - Mom has 1 less `free_breedings`
    ///     - Mom's DNA matches old Mom
    ///     - Mom's num breedings is incremented
    ///
    fn check_new_mom(old_mom: &KittyData, new_mom: &KittyData) -> Result<(), Self::Error> {
        match &new_mom.parent {
            Parent::Mom(status) => {
                if let MomKittyStatus::RearinToGo = status {
                    return Err(Self::Error::NewMomIsStillRearinToGo);
                }
            }
            Parent::Dad(_) => return Err(Self::Error::TwoDadsNotValid),
        }

        ensure!(
            new_mom.free_breedings == old_mom.free_breedings - 1,
            Self::Error::NewParentFreeBreedingsIncorrect
        );
        ensure!(
            new_mom.num_breedings == old_mom.num_breedings + 1,
            Self::Error::NewParentNumberBreedingsIncorrect
        );
        ensure!(
            new_mom.dna == old_mom.dna,
            Self::Error::NewParentDnaDoesntMatchOld
        );

        Ok(())
    }

    /// Checks:
    ///     - Dad is now `Tired`
    ///     - Dad has 1 less `free_breedings`
    ///     - Dad's DNA matches old Dad
    ///     - Dad's num breedings is incremented
    ///
    fn check_new_dad(old_dad: &KittyData, new_dad: &KittyData) -> Result<(), Self::Error> {
        match &new_dad.parent {
            Parent::Dad(status) => {
                if let DadKittyStatus::RearinToGo = status {
                    return Err(Self::Error::NewDadIsStillRearinToGo);
                }
            }
            Parent::Mom(_) => return Err(Self::Error::TwoMomsNotValid),
        }

        ensure!(
            new_dad.free_breedings == old_dad.free_breedings - 1,
            Self::Error::NewParentFreeBreedingsIncorrect
        );
        ensure!(
            new_dad.num_breedings == old_dad.num_breedings + 1,
            Self::Error::NewParentNumberBreedingsIncorrect
        );
        ensure!(
            new_dad.dna == old_dad.dna,
            Self::Error::NewParentDnaDoesntMatchOld
        );

        Ok(())
    }

    /// Checks:
    ///     - DNA formation correct -> `hash_of(mom_dna + dad_dna + mom_num_breedings + dad_num_breedings)
    ///     - Free breedings is correct given the trait implementation in this case 2
    ///     - has non-zero bredings
    ///     - If Mom is in RearinToGo
    ///     - If Dad is in RearinToGo
    ///
    fn check_child(
        new_mom: &KittyData,
        new_dad: &KittyData,
        child: &KittyData,
    ) -> Result<(), Self::Error> {
        let new_dna = BlakeTwo256::hash_of(&(
            &new_mom.dna,
            &new_dad.dna,
            &new_mom.num_breedings,
            &new_dad.num_breedings,
        ));

        ensure!(
            child.dna == KittyDNA(new_dna),
            Self::Error::NewChildDnaIncorrect,
        );
        ensure!(
            child.free_breedings == Self::NUM_FREE_BREEDINGS,
            Self::Error::NewChildFreeBreedingsIncorrect
        );
        ensure!(
            child.num_breedings == 0,
            Self::Error::NewChildHasNonZeroBreedings,
        );

        match &child.parent {
            Parent::Mom(status) => {
                if let MomKittyStatus::HadBirthRecently = status {
                    return Err(Self::Error::NewChildIncorrectParentInfo);
                }
            }
            Parent::Dad(status) => {
                if let DadKittyStatus::Tired = status {
                    return Err(Self::Error::NewChildIncorrectParentInfo);
                }
            }
        }
        Ok(())
    }
}

impl TryFrom<&DynamicallyTypedData> for KittyData {
    type Error = ConstraintCheckerError;
    fn try_from(a: &DynamicallyTypedData) -> Result<Self, Self::Error> {
        a.extract::<KittyData>()
            .map_err(|_| ConstraintCheckerError::BadlyTyped)
    }
}

impl SimpleConstraintChecker for FreeKittyConstraintChecker {
    type Error = ConstraintCheckerError;
    /// Checks:
    ///     - `input_data` is of length 2
    ///     - `output_data` is of length 3
    ///
    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Input must be a Mom and a Dad
        ensure!(input_data.len() == 2, Self::Error::TwoParentsDoNotExist);

        let mom = KittyData::try_from(&input_data[0])?;
        let dad = KittyData::try_from(&input_data[1])?;
        KittyHelpers::can_breed(&mom, &dad)?;

        // Output must be Mom, Dad, Child
        ensure!(output_data.len() == 3, Self::Error::NotEnoughFamilyMembers);

        KittyHelpers::check_new_family(&mom, &dad, output_data)?;

        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    /// A bogus data type used in tests for type validation
    #[derive(Encode, Decode)]
    struct Bogus;

    impl UtxoData for Bogus {
        const TYPE_ID: [u8; 4] = *b"bogs";
    }

    struct TestKittyMaker;

    impl TestKittyMaker {
        pub fn get_bogus_type() -> DynamicallyTypedData {
            DynamicallyTypedData {
                data: Bogus.encode(),
                type_id: Bogus::TYPE_ID,
            }
        }

        pub fn get_default_mom() -> DynamicallyTypedData {
            DynamicallyTypedData {
                data: KittyData::default().encode(),
                type_id: KittyData::TYPE_ID,
            }
        }

        pub fn get_default_dad() -> DynamicallyTypedData {
            DynamicallyTypedData {
                data: KittyData {
                    parent: Parent::Dad(DadKittyStatus::RearinToGo),
                    ..Default::default()
                }
                .encode(),
                type_id: KittyData::TYPE_ID,
            }
        }

        pub fn get_default_child() -> DynamicallyTypedData {
            let mom: KittyData = KittyData::try_from(&Self::get_default_mom())
                .expect("Can get mom KittyData in test");
            let dad = KittyData::try_from(&Self::get_default_dad())
                .expect("Can get dad KittyData in test");
            DynamicallyTypedData {
                data: KittyData {
                    parent: Parent::Mom(MomKittyStatus::RearinToGo),
                    free_breedings: 2,
                    dna: KittyDNA(BlakeTwo256::hash_of(&(
                        mom.dna,
                        dad.dna,
                        mom.num_breedings + 1,
                        dad.num_breedings + 1,
                    ))),
                    num_breedings: 0,
                }
                .encode(),
                type_id: KittyData::TYPE_ID,
            }
        }
    }

    #[test]
    fn breed_happy_path_works() {
        let mut new_mom: KittyData = KittyData::try_from(&TestKittyMaker::get_default_mom())
            .expect("Can get mom KittyData in test");
        new_mom.parent = Parent::Mom(MomKittyStatus::HadBirthRecently);
        new_mom.num_breedings += 1;
        new_mom.free_breedings -= 1;
        let mut new_dad = KittyData::try_from(&TestKittyMaker::get_default_dad())
            .expect("Can get dad KittyData in test");
        new_dad.parent = Parent::Dad(DadKittyStatus::Tired);
        new_dad.num_breedings += 1;
        new_dad.free_breedings -= 1;
        let result = FreeKittyConstraintChecker::check(
            &FreeKittyConstraintChecker,
            &vec![
                TestKittyMaker::get_default_mom(),
                TestKittyMaker::get_default_dad(),
            ],
            &vec![
                new_mom.try_into().unwrap(),
                new_dad.try_into().unwrap(),
                TestKittyMaker::get_default_child(),
            ],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn breed_wrong_input_type_fails() {
        let result = FreeKittyConstraintChecker::check(
            &FreeKittyConstraintChecker,
            &vec![
                TestKittyMaker::get_bogus_type(),
                TestKittyMaker::get_bogus_type(),
            ],
            &vec![],
        );
        assert_eq!(result, Err(ConstraintCheckerError::BadlyTyped));
    }

    #[test]
    fn breed_wrong_output_type_fails() {
        let result = FreeKittyConstraintChecker::check(
            &FreeKittyConstraintChecker,
            &vec![
                TestKittyMaker::get_default_mom(),
                TestKittyMaker::get_default_dad(),
            ],
            &vec![
                TestKittyMaker::get_bogus_type(),
                TestKittyMaker::get_bogus_type(),
                TestKittyMaker::get_bogus_type(),
            ],
        );
        assert_eq!(result, Err(ConstraintCheckerError::BadlyTyped));
    }

    #[test]
    fn inputs_dont_contain_two_parents_fails() {
        let result = FreeKittyConstraintChecker::check(
            &FreeKittyConstraintChecker,
            &vec![TestKittyMaker::get_default_mom()],
            &vec![],
        );
        assert_eq!(result, Err(ConstraintCheckerError::TwoParentsDoNotExist));
    }

    #[test]
    fn outputs_dont_contain_all_family_members_fails() {
        let result = FreeKittyConstraintChecker::check(
            &FreeKittyConstraintChecker,
            &vec![
                TestKittyMaker::get_default_mom(),
                TestKittyMaker::get_default_dad(),
            ],
            &vec![TestKittyMaker::get_default_mom()],
        );
        assert_eq!(result, Err(ConstraintCheckerError::NotEnoughFamilyMembers));
    }

    #[test]
    fn breed_two_dads_fails() {
        let result = FreeKittyConstraintChecker::check(
            &FreeKittyConstraintChecker,
            &vec![
                TestKittyMaker::get_default_dad(),
                TestKittyMaker::get_default_dad(),
            ],
            &vec![TestKittyMaker::get_default_mom()],
        );
        assert_eq!(result, Err(ConstraintCheckerError::TwoDadsNotValid));
    }

    #[test]
    fn breed_two_moms_fails() {
        let result = FreeKittyConstraintChecker::check(
            &FreeKittyConstraintChecker,
            &vec![
                TestKittyMaker::get_default_mom(),
                TestKittyMaker::get_default_mom(),
            ],
            &vec![TestKittyMaker::get_default_mom()],
        );
        assert_eq!(result, Err(ConstraintCheckerError::TwoMomsNotValid));
    }

    #[test]
    fn first_input_not_mom_fails() {
        let result = FreeKittyConstraintChecker::check(
            &FreeKittyConstraintChecker,
            &vec![
                TestKittyMaker::get_default_dad(),
                TestKittyMaker::get_default_mom(),
            ],
            &vec![],
        );
        assert_eq!(result, Err(ConstraintCheckerError::TwoDadsNotValid))
    }

    #[test]
    fn first_output_not_mom_fails() {
        let result = FreeKittyConstraintChecker::check(
            &FreeKittyConstraintChecker,
            &vec![
                TestKittyMaker::get_default_mom(),
                TestKittyMaker::get_default_dad(),
            ],
            &vec![
                TestKittyMaker::get_default_dad(),
                TestKittyMaker::get_default_mom(),
                TestKittyMaker::get_default_child(),
            ],
        );
        assert_eq!(result, Err(ConstraintCheckerError::TwoDadsNotValid));
    }

    #[test]
    fn second_output_not_dad_fails() {
        // TODO
    }

    #[test]
    fn third_output_not_child_fails() {
        // TODO
    }

    #[test]
    fn breed_mom_when_she_gave_birth_recently_fails() {
        // TODO
    }

    #[test]
    fn breed_dad_when_he_is_tired_fails() {
        // TODO
    }

    #[test]
    fn check_mom_breedings_overflow_fails() {
        // TODO
    }

    #[test]
    fn check_dad_breedings_overflow_fails() {
        // TODO
    }

    #[test]
    fn check_mom_free_breedings_zero_fails() {
        // TODO
    }

    #[test]
    fn check_dad_free_breedings_zero_fails() {
        // TODO
    }

    #[test]
    fn check_new_mom_free_breedings_incorrect_fails() {
        // TODO
    }

    #[test]
    fn check_new_dad_free_breedings_incorrect_fails() {
        // TODO
    }

    #[test]
    fn check_new_mom_num_breedings_incorrect_fails() {
        // TODO
    }

    #[test]
    fn check_new_dad_num_breedings_incorrect_fails() {
        // TODO
    }

    #[test]
    fn check_new_mom_dna_doesnt_match_old_fails() {
        // TODO
    }

    #[test]
    fn check_new_dad_dna_doesnt_match_old_fails() {
        // TODO
    }

    #[test]
    fn check_child_dna_incorrect_fails() {
        // TODO
    }

    #[test]
    fn check_child_dad_parent_tired_fails() {
        // TODO
    }

    #[test]
    fn check_child_mom_parent_recently_gave_birth_fails() {
        // TODO
    }

    #[test]
    fn check_child_free_breedings_incorrect_fails() {
        // TODO
    }

    #[test]
    fn check_child_num_breedings_non_zero_fails() {
        // TODO
    }
}
