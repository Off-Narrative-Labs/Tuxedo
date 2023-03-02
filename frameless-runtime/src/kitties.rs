use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
    transaction_validity::TransactionPriority,
    traits::{BlakeTwo256, Hash as HashT},
};
use sp_std::{
    prelude::*,
    marker::PhantomData,
    fmt::Debug,
};
use sp_core::H256;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, Verifier,
};
use crate::money::{Coin, MoneyVerifier};

use log::info;

// 1.) First need a Verifier types
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct FreeKittyVerifier;

// 2.) Then need UtxoData type
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum DadKittyStatus {
    #[default]
    RearinToGo,
    Tired,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum MomKittyStatus {
    #[default]
    RearinToGo,
    HadBirthRecently,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
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

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct KittyDNA(H256);

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct KittyData {
    parent: Parent,
    free_breedings: u64, // Ignore in breed for money case
    dna: KittyDNA,
    num_breedings: u128,
}

impl UtxoData for KittyData {
    const TYPE_ID: [u8; 4] = *b"Kitt";
}

// 3.) Need Verifier Error
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub enum VerifierError {
    // TODO: Add documentation for each of these Error types
    /// Dynamic typing issue.
    /// This error doesn't discriminate between badly typed inputs and outputs.
    BadlyTyped,
    /// Needed when spending for breeding
    MinimumSpendAndBreedNotMet,
    /// Need two parents to breed
    TwoParentsDoNotExist,
    /// Incorrect number of outputs when it comes to breeding
    NotEnoughFamilyMembers,
    /// Mom has recently given birth and isnt ready to breed
    MomNotReadyYet,
    /// Dad cannot breed because he is still too tired
    DadTooTired,
    /// Cannot have two moms when breeding
    TwoMomsNotValid,
    /// Cannot have two dads when breeding
    TwoDadsNotValid,
    /// New Mom after breeding should be in HadBirthRecently state
    NewMomIsStillRearinToGo,
    /// New Dad after breeding should be in Tired state
    NewDadIsStillRearinToGo,
    /// Number of free breedings of new parent is not correct
    NewParentFreeBreedingsIncorrect,
    /// New parents DNA does not match the old one parent has to still be the same kitty
    NewParentDnaDoesntMatchOld,
    /// New parent Breedings has not incremented or is incorrect
    NewParentNumberBreedingsIncorrect,
    /// New child DNA is not correct given the protocol
    NewChildDnaIncorrect,
    /// New child doesnt have the correct number of free breedings
    NewChildFreeBreedingsIncorrect,
    /// New child has non zero breedings which is impossible because it was just born
    NewChildHasNonZeroBreedings,
    /// New child parent info is either in Tired state or HadBirthRecently state which is not possible
    NewChildIncorrectParentInfo,
    /// Too many breedings for this kitty can no longer breed
    TooManyBreedingsForKitty,
    /// Not enough free breedings available for these parents
    NotEnoughFreeBreedings,
}

// TODO: Add documentation for each of these trait items
trait Breed {
    const COST: u128;
    const NUM_FREE_BREEDINGS: u64;
    type Error: Into<VerifierError>;
    fn can_breed(mom: &KittyData, dad: &KittyData) -> Result<(), Self::Error>;
    fn check_mom_can_breed(mom: &KittyData) -> Result<(), Self::Error>;
    fn check_dad_can_breed(dad: &KittyData) -> Result<(), Self::Error>;
    fn check_free_breedings(mom: &KittyData, dad: &KittyData) -> Result<(), Self::Error>;
    fn check_new_family(
        old_mom: &KittyData,
        old_dad: &KittyData,
        new_family: &[DynamicallyTypedData]
    ) -> Result<(), Self::Error>;
    fn check_new_mom(old_mom: &KittyData, new_mom: &KittyData) -> Result<(), Self::Error>;
    fn check_new_dad(old_dad: &KittyData, new_dad: &KittyData) -> Result<(), Self::Error>;
    fn check_child(new_mom: &KittyData, new_dad: &KittyData, child: &KittyData) -> Result<(), Self::Error>;
}

pub struct KittyHelpers;
impl Breed for KittyHelpers
{
    const COST: u128 = 5u128;
    const NUM_FREE_BREEDINGS: u64 = 2u64;
    type Error = VerifierError;
    fn can_breed(mom: &KittyData, dad: &KittyData) -> Result<(), Self::Error> {
        Self::check_mom_can_breed(mom)?;
        Self::check_dad_can_breed(dad)?;
        Self::check_free_breedings(mom, dad)?;
        Ok(())
    }

    fn check_mom_can_breed(mom: &KittyData) -> Result<(), Self::Error> {
        match &mom.parent {
            Parent::Mom(status) => {
                if let MomKittyStatus::HadBirthRecently = status {
                    return Err(Self::Error::MomNotReadyYet)
                }
            },
            Parent::Dad(_) => {
                return Err(Self::Error::TwoDadsNotValid)
            }
        }
        mom.num_breedings.checked_add(1).ok_or(Self::Error::TooManyBreedingsForKitty)?;
        Ok(())
    }

    fn check_dad_can_breed(dad: &KittyData) -> Result<(), Self::Error> {
        match &dad.parent {
            Parent::Dad(status) => {
                if let DadKittyStatus::Tired = status {
                    return Err(Self::Error::DadTooTired)
                }
            },
            Parent::Mom(_) => {
                return Err(Self::Error::TwoMomsNotValid)
            }
        }
        dad.num_breedings.checked_add(1).ok_or(Self::Error::TooManyBreedingsForKitty)?;
        Ok(())
    }

    fn check_free_breedings(mom: &KittyData, dad: &KittyData) -> Result<(), Self::Error> {
        let mom_breedings = mom.free_breedings;
        let dad_breedings = dad.free_breedings;
        if (mom_breedings == 0) && (dad_breedings == 0) {
            return Err(Self::Error::NotEnoughFreeBreedings)
        }
        Ok(())
    }

    fn check_new_family(
        old_mom: &KittyData,
        old_dad: &KittyData,
        new_family: &[DynamicallyTypedData]
    ) -> Result<(), Self::Error> {
        // Output Side
        ensure!(
            new_family.len() == 3,
            Self::Error::NotEnoughFamilyMembers
        );
        let new_mom = KittyData::try_from(&new_family[0])?;
        let new_dad = KittyData::try_from(&new_family[1])?;
        let child = KittyData::try_from(&new_family[2])?;
        Self::check_new_mom(old_mom, &new_mom)?;
        Self::check_new_dad(old_dad, &new_dad)?;
        Self::check_child(&new_mom, &new_dad, &child)?;
        Ok(())
    }

    fn check_new_mom(old_mom: &KittyData, new_mom: &KittyData) -> Result<(), Self::Error> {
        match &new_mom.parent {
            Parent::Mom(status) => {
                if let MomKittyStatus::RearinToGo = status {
                    return Err(Self::Error::NewMomIsStillRearinToGo)
                }
            },
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

    fn check_new_dad(old_dad: &KittyData, new_dad: &KittyData) -> Result<(), Self::Error> {
        match &new_dad.parent {
            Parent::Dad(status) => {
                if let DadKittyStatus::RearinToGo = status {
                    return Err(Self::Error::NewDadIsStillRearinToGo)
                }
            },
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

    fn check_child(new_mom: &KittyData, new_dad: &KittyData, child: &KittyData) -> Result<(), Self::Error> {
        let new_dna =
            BlakeTwo256::hash_of(&(&new_mom.dna, &new_dad.dna, &new_mom.num_breedings, &new_dad.num_breedings));

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
                    return Err(Self::Error::NewChildIncorrectParentInfo)
                }
            },
            Parent::Dad(status) => {
                if let DadKittyStatus::Tired = status {
                    return Err(Self::Error::NewChildIncorrectParentInfo)
                }
            }
        }
        Ok(())
    }
}

impl TryFrom<&DynamicallyTypedData> for KittyData {
    type Error = VerifierError;
    fn try_from(a: &DynamicallyTypedData) -> Result<Self, Self::Error> {
        a.extract::<KittyData>().map_err(|_| VerifierError::BadlyTyped)
    }
}

// 4.) Implement Verifier Trait on my new Verifier
impl Verifier for FreeKittyVerifier {
    type Error = VerifierError;
    fn verify(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData]
    ) -> Result<TransactionPriority, Self::Error> {
        // Input must be a Mom and a Dad
        ensure!(
            input_data.len() == 2,
            Self::Error::TwoParentsDoNotExist
        );

        let mom = KittyData::try_from(&input_data[0])?;
        let dad = KittyData::try_from(&input_data[0])?;
        KittyHelpers::can_breed(&mom, &dad)?;

        // Output must be Mom, Dad, Child
        ensure!(
            output_data.len() == 3,
            Self::Error::NotEnoughFamilyMembers
        );

        KittyHelpers::check_new_family(&mom, &dad, output_data)?;

        Ok(0)
    }
}

pub trait KittyConfigTrait {
    type Money;
    type MoneyVerifier;
}

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct KittyConfig;
impl KittyConfigTrait for KittyConfig {
    type Money = Coin;
    type MoneyVerifier = MoneyVerifier;
}
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
pub struct MoneyKittyVerifier<Config>(PhantomData<Config>);
impl<Config> Verifier for MoneyKittyVerifier<Config>
where
    Config: KittyConfigTrait + Clone + Debug
{
    type Error = VerifierError;
    fn verify(
        &self,
        input_data: &[DynamicallyTypedData],
        _output_data: &[DynamicallyTypedData]
    ) -> Result<TransactionPriority, Self::Error> {
        // TODO: Verify that there is a spend that happens which covers the cost to breed.
        // First input and output must be money
        ensure!(
            input_data.len() >= 2,
            Self::Error::MinimumSpendAndBreedNotMet,
        );
        // MoneyVerifier::verify(&MoneyVerifier::Spend, &input_data[0], &output_money[0])?;

        // <Config as MoneyVerifier>::verify(&Config::MoneyVerifier::Spend)
        // TODO: now check if the money is enough and maybe address is correct?
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

    #[test]
    fn breed_wrong_input_type_fails() {
        todo!();
    }

    #[test]
    fn breed_wrong_output_type_fails() {
        todo!();
    }

    #[test]
    fn inputs_dont_contain_two_parents_fails() {
        todo!();
    }

    #[test]
    fn outputs_dont_contain_all_family_members_fails() {
        todo!();
    }

    #[test]
    fn breed_two_dads_fails() {
        todo!();
    }

    #[test]
    fn breed_two_moms_fails() {
        todo!();
    }

    #[test]
    fn first_input_not_mom_fails() {
        todo!();
    }

    #[test]
    fn second_input_not_dad_fails() {
        todo!();
    }

    #[test]
    fn first_output_not_mom_fails() {
        todo!();
    }

    #[test]
    fn second_output_not_dad_fails() {
        todo!();
    }

    #[test]
    fn third_output_not_child_fails() {
        todo!();
    }

    #[test]
    fn breed_mom_when_she_gave_birth_recently_fails() {
        todo!();
    }

    #[test]
    fn breed_dad_when_he_is_tired_fails() {
        todo!();
    }

    #[test]
    fn check_mom_breedings_overflow_fails() {
        todo!();
    }

    #[test]
    fn check_dad_breedings_overflow_fails() {
        todo!();
    }

    #[test]
    fn check_mom_free_breedings_zero_fails() {
        todo!();
    }

    #[test]
    fn check_dad_free_breedings_zero_fails() {
        todo!();
    }

    #[test]
    fn check_new_mom_free_breedings_incorrect_fails() {
        todo!();
    }

    #[test]
    fn check_new_dad_free_breedings_incorrect_fails() {
        todo!();
    }

    #[test]
    fn check_new_mom_num_breedings_incorrect_fails() {
        todo!();
    }

    #[test]
    fn check_new_dad_num_breedings_incorrect_fails() {
        todo!();
    }

    #[test]
    fn check_new_mom_dna_doesnt_match_old_fails() {
        todo!();
    }

    #[test]
    fn check_new_dad_dna_doesnt_match_old_fails() {
        todo!();
    }

    #[test]
    fn check_child_dna_incorrect_fails() {
        todo!();
    }

    #[test]
    fn check_child_dad_parent_tired_fails() {
        todo!();
    }

    #[test]
    fn check_child_mom_parent_recently_gave_birth_fails() {
        todo!();
    }

    #[test]
    fn check_child_free_breedings_incorrect_fails() {
        todo!();
    }

    #[test]
    fn check_child_num_breedings_non_zero_fails() {
        todo!();
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     /// A bogus data type used in tests for type validation
//     #[derive(Encode, Decode)]
//     struct Bogus;

//     impl UtxoData for Bogus {
//         const TYPE_ID: [u8; 4] = *b"bogs";
//     }

//     #[test]
//     fn spend_valid_transaction_work() {
//         let input_data = vec![Coin(5).into(), Coin(7).into()]; // total 12
//         let output_data = vec![Coin(10).into(), Coin(1).into()]; // total 11
//         let expected_priority = 1u64;

//         assert_eq!(
//             MoneyVerifier::Spend.verify(&input_data, &output_data),
//             Ok(expected_priority),
//         );
//     }

//     #[test]
//     fn spend_with_zero_value_output_fails() {
//         let input_data = vec![Coin(5).into(), Coin(7).into()]; // total 12
//         let output_data = vec![Coin(10).into(), Coin(1).into(), Coin(0).into()]; // total 1164;

//         assert_eq!(
//             MoneyVerifier::Spend.verify(&input_data, &output_data),
//             Err(VerifierError::ZeroValueCoin),
//         );
//     }

//     #[test]
//     fn spend_no_outputs_is_a_burn() {
//         let input_data = vec![Coin(5).into(), Coin(7).into()]; // total 12
//         let output_data = vec![];
//         let expected_priority = 12u64;

//         assert_eq!(
//             MoneyVerifier::Spend.verify(&input_data, &output_data),
//             Ok(expected_priority),
//         );
//     }

//     #[test]
//     fn spend_no_inputs_fails() {
//         let input_data = vec![];
//         let output_data = vec![Coin(10).into(), Coin(1).into()];

//         assert_eq!(
//             MoneyVerifier::Spend.verify(&input_data, &output_data),
//             Err(VerifierError::SpendingNothing),
//         );
//     }

//     #[test]
//     fn spend_wrong_input_type_fails() {
//         let input_data = vec![Bogus.into()];
//         let output_data = vec![Coin(10).into(), Coin(1).into()];

//         assert_eq!(
//             MoneyVerifier::Spend.verify(&input_data, &output_data),
//             Err(VerifierError::BadlyTyped),
//         );
//     }

//     #[test]
//     fn spend_wrong_output_type_fails() {
//         let input_data = vec![Coin(5).into(), Coin(7).into()]; // total 12
//         let output_data = vec![Bogus.into()];

//         assert_eq!(
//             MoneyVerifier::Spend.verify(&input_data, &output_data),
//             Err(VerifierError::BadlyTyped),
//         );
//     }

//     #[test]
//     fn spend_output_value_exceeds_input_value_fails() {
//         let input_data = vec![Coin(10).into(), Coin(1).into()]; // total 11
//         let output_data = vec![Coin(5).into(), Coin(7).into()]; // total 12

//         assert_eq!(
//             MoneyVerifier::Spend.verify(&input_data, &output_data),
//             Err(VerifierError::OutputsExceedInputs),
//         );
//     }

//     #[test]
//     fn mint_valid_transaction_works() {
//         let input_data = vec![];
//         let output_data = vec![Coin(10).into(), Coin(1).into()];

//         assert_eq!(MoneyVerifier::Mint.verify(&input_data, &output_data), Ok(0),);
//     }

//     #[test]
//     fn mint_with_zero_value_output_fails() {
//         let input_data = vec![];
//         let output_data = vec![Coin(0).into()];

//         assert_eq!(
//             MoneyVerifier::Mint.verify(&input_data, &output_data),
//             Err(VerifierError::ZeroValueCoin),
//         );
//     }

//     #[test]
//     fn mint_with_inputs_fails() {
//         let input_data = vec![Coin(5).into()];
//         let output_data = vec![Coin(10).into(), Coin(1).into()];

//         assert_eq!(
//             MoneyVerifier::Mint.verify(&input_data, &output_data),
//             Err(VerifierError::MintingWithInputs),
//         );
//     }

//     #[test]
//     fn mint_with_no_outputs_fails() {
//         let input_data = vec![];
//         let output_data = vec![];

//         assert_eq!(
//             MoneyVerifier::Mint.verify(&input_data, &output_data),
//             Err(VerifierError::MintingNothing),
//         );
//     }

//     #[test]
//     fn mint_wrong_output_type_fails() {
//         let input_data = vec![];
//         let output_data = vec![Coin(10).into(), Bogus.into()];

//         assert_eq!(
//             MoneyVerifier::Mint.verify(&input_data, &output_data),
//             Err(VerifierError::BadlyTyped),
//         );
//     }
// }
