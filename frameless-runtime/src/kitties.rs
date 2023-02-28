
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
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
pub struct GenerateGender {
    is_mom: bool,
}

impl GenerateGender {
    /// For now this just flips a bool to see what the gender of the next child is and rotates
    pub fn random_gender(&self) -> Self {
        let mut is_mom = self.is_mom;
        if is_mom {
            is_mom = false;
        } else {
            is_mom = true
        }
        Self {
            is_mom: is_mom,
        }
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
    is_mom: bool,
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
    /// Dynamic typing issue.
    /// This error doesn't discriminate between badly typed inputs and outputs.
    BadlyTyped,
    BreedingFailed,
    MinimumSpendAndBreedNotMet,
    // TODO: Add others..
}

trait Breed<Kitty> {
    const COST: u128;
    type Error: Into<VerifierError>;
    fn breed(mom: Kitty, dad: Kitty) -> Result<(), VerifierError>;
}

pub struct KittyHelpers<Kitty>(PhantomData<Kitty>);
impl<Kitty> Breed<Kitty> for KittyHelpers<Kitty>
where
    Kitty: From<KittyData>,
{
    const COST: u128 = 5u128;
    type Error = VerifierError;
    fn breed(mom: Kitty, dad: Kitty) -> Result<(), Self::Error> {
        // TODO: Implment breeding algo

        // Input side
        // 1.) Check if mom is a mom
        // 2.) Check if dad is a dad
        // 3.) Check if mom has recently given birth
        // 4.) Check if dad is tired
        // 5.) Check if mom and dad have enough free breedings

        // Output Side
        // 1.) If both checks pass you can breed and swap the states
        // 2.) Check if Mom and Dad new status(Output) has been swapped and that it is still a mom and dad
        //      - Check Mom is still a mom, Dad is still a dad i.e. Cant have two dads now or two moms
        //      - Check Mom and Dad still has the same DNA
        //      - Check free breedings for both parents have been decreased by 1
        // 3.) Check if Child Kitty created (Output) is a Mom or Dad and is initialized correctly

        Ok(())
    }
}

impl TryFrom<DynamicallyTypedData> for KittyData {
    type Error = VerifierError;
    fn try_from(a: DynamicallyTypedData) -> Result<Self, Self::Error> {
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
        // TODO:
        // Implement normal breed scenario
        let mom: KittyData = input_data[0].clone().try_into()?;
        let dad: KittyData = input_data[1].clone().try_into()?;
        let _ = KittyHelpers::<KittyData>::breed(mom, dad)?;
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
        output_data: &[DynamicallyTypedData]
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


// #[cfg_attr(
//     feature = "std",
//     derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
// )]
// #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
// pub enum MoneyVerifier {
//     /// A typical spend transaction where some coins are consumed and others are created.
//     Spend,
//     /// A mint transaction that creates no coins out of the void. In a real-world chain,
//     /// this should be protected somehow, or not included at all. For now it is publicly
//     /// available. I'm adding it to explore multiple validation paths in a single piece.
//     Mint,
// }

// /// A single coin in the fungible money system.
// /// A new type wrapper around a u128 value.
// #[cfg_attr(
//     feature = "std",
//     derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
// )]
// #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
// pub struct Coin(u128);

// impl UtxoData for Coin {
//     const TYPE_ID: [u8; 4] = *b"coin";
// }

// #[cfg_attr(
//     feature = "std",
//     derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
// )]
// #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Hash, Debug, TypeInfo)]
// pub enum VerifierError {
//     /// Dynamic typing issue.
//     /// This error doesn't discriminate between badly typed inputs and outputs.
//     BadlyTyped,
//     /// The transaction attempts to consume inputs while minting. This is not allowed.
//     MintingWithInputs,
//     /// The transaction attempts to mint zero coins. This is not allowed.
//     MintingNothing,
//     /// The transaction attempts to spend without consuming any inputs.
//     /// Either the output value will exceed the input value, or if there are no outputs,
//     /// it is a waste of processing power, so it is not allowed.
//     SpendingNothing,
//     /// The value of the spent input coins is less than the value of the newly created
//     /// output coins. This would lead to money creation and is not allowed.
//     OutputsExceedInputs,
//     /// The value consumed or created by this transaction overflows the value type.
//     /// This could lead to problems like https://bitcointalk.org/index.php?topic=823.0
//     ValueOverflow,
//     /// The transaction attempted to create a coin with zero value. This is not allowed
//     /// because it wastes state space.
//     ZeroValueCoin,
// }

// impl Verifier for MoneyVerifier {
//     type Error = VerifierError;

//     fn verify(
//         &self,
//         input_data: &[DynamicallyTypedData],
//         output_data: &[DynamicallyTypedData],
//     ) -> Result<TransactionPriority, Self::Error> {
//         match &self {
//             Self::Spend => {
//                 // Check that we are consuming at least one input
//                 ensure!(!input_data.is_empty(), VerifierError::SpendingNothing);

//                 let mut total_input_value: u128 = 0;
//                 let mut total_output_value: u128 = 0;

//                 // Check that sum of input values < output values
//                 for input in input_data {
//                     let utxo_value = input
//                         .extract::<Coin>()
//                         .map_err(|_| VerifierError::BadlyTyped)?
//                         .0;
//                     total_input_value = total_input_value
//                         .checked_add(utxo_value)
//                         .ok_or(VerifierError::ValueOverflow)?;
//                 }

//                 for utxo in output_data {
//                     let utxo_value = utxo
//                         .extract::<Coin>()
//                         .map_err(|_| VerifierError::BadlyTyped)?
//                         .0;
//                     ensure!(utxo_value > 0, VerifierError::ZeroValueCoin);
//                     total_output_value = total_output_value
//                         .checked_add(utxo_value)
//                         .ok_or(VerifierError::ValueOverflow)?;
//                 }

//                 ensure!(
//                     total_output_value <= total_input_value,
//                     VerifierError::OutputsExceedInputs
//                 );

//                 // Priority is based on how many token are burned
//                 // Type stuff is kinda ugly. Maybe division would be better?
//                 let burned = total_input_value - total_output_value;
//                 Ok(if burned < u64::max_value() as u128 {
//                     burned as u64
//                 } else {
//                     u64::max_value()
//                 })
//             }
//             Self::Mint => {
//                 // Make sure there are no inputs being consumed
//                 ensure!(input_data.is_empty(), VerifierError::MintingWithInputs);

//                 // Make sure there is at least one output being minted
//                 ensure!(!output_data.is_empty(), VerifierError::MintingNothing);

//                 // Make sure the outputs are the right type
//                 for utxo in output_data {
//                     let utxo_value = utxo
//                         .extract::<Coin>()
//                         .map_err(|_| VerifierError::BadlyTyped)?
//                         .0;
//                     ensure!(utxo_value > 0, VerifierError::ZeroValueCoin);
//                 }

//                 // No priority for minting
//                 Ok(0)
//             }
//         }
//     }
// }

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
