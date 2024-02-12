//! This module defines TradableKitty, a specialized type of kitty with additional features.
//!
//! TradableKitties are designed for trading, and they extend the functionality of the basic kitty.
//! Key features of TradableKitties include:
//! The TradableKitty module extends the basic functionality of kitties with additional features such as buying, updating properties, minting, and breeding. The provided validation functionality includes:
//!
//! - `Mint`: Create new TradableKitties, supporting the generation of kitties without parents.
//! - `Breed`: Consume kitties and create a new family, including parents (mom and dad) and a child.
//! - `UpdateProperties`: Update properties of a TradableKitty, including `is_available_for_sale`, `price`, and `name`.
//!    A single API, `updateKittyProperties()`, is provided for updating these properties for below reasons:
//!        1. Updating atomically in a single transaction, ensuring consistency.
//!        2. Benfit of less number of transaction is reduced weight or gas fees.
//! - `Buy`: Enable users to purchase TradableKitties from others, facilitating secure and fair exchanges.
//!
//!
//! TradableKitties provide an enhanced user experience by introducing trading capabilities
//! and additional customization for kitty properties.

#![cfg_attr(not(feature = "std"), no_std)]

use kitties::Breed as BasicKittyBreed;
use kitties::ConstraintCheckerError;
use kitties::KittyData;
use kitties::KittyHelpers;
use money::{Coin, MoneyConstraintChecker};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::prelude::*;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, SimpleConstraintChecker,
};

#[cfg(test)]
mod tests;

#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
    Hash,
    Debug,
    TypeInfo,
)]
pub struct TradableKittyData {
    pub kitty_basic_data: KittyData,
    pub price: Option<u128>,
    pub is_available_for_sale: bool,
}

impl Default for TradableKittyData {
    fn default() -> Self {
        Self {
            kitty_basic_data: KittyData::default(),
            price: None,
            is_available_for_sale: false,
        }
    }
}

impl TryFrom<&DynamicallyTypedData> for TradableKittyData {
    type Error = TradableKittyConstraintCheckerError;
    fn try_from(a: &DynamicallyTypedData) -> Result<Self, Self::Error> {
        a.extract::<TradableKittyData>()
            .map_err(|_| TradableKittyConstraintCheckerError::BadlyTyped)
    }
}

impl UtxoData for TradableKittyData {
    const TYPE_ID: [u8; 4] = *b"tdkt";
}

#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
    Hash,
    Debug,
    TypeInfo,
)]
pub enum TradableKittyConstraintCheckerError {
    /// Dynamic typing issue.
    /// This error doesn't discriminate between badly typed inputs and outputs.
    BadlyTyped,
    /// Needed when spending for breeding.
    MinimumSpendAndBreedNotMet,
    /// Need two parents to breed.
    TwoParentsDoNotExist,
    /// Incorrect number of outputs when it comes to breeding.
    NotEnoughFamilyMembers,
    /// Incorrect number of outputs when it comes to Minting.
    IncorrectNumberOfKittiesForMintOperation,
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
    /// The transaction attempts to mint no Kitty.
    MintingNothing,
    /// Inputs(Parents) not required for mint.
    MintingWithInputs,
    /// No input for kitty Update.
    InputMissingUpdatingNothing,
    /// Mismatch innumberof inputs and number of outputs .
    MismatchBetweenNumberOfInputAndUpdateUpadtingNothing,
    /// TradableKitty Update can't have more than one outputs.
    MultipleOutputsForKittyUpdateError,
    /// Kitty Update has more than one inputs.
    InValidNumberOfInputsForKittyUpdate,
    /// Basic kitty properties cannot be updated.
    KittyGenderCannotBeUpdated,
    /// Basic kitty properties cannot be updated.
    KittyDnaCannotBeUpdated,
    /// Kitty FreeBreeding cannot be updated.
    FreeBreedingCannotBeUpdated,
    /// Kitty NumOfBreeding cannot be updated.
    NumOfBreedingCannotBeUpdated,
    /// Updated price of is incorrect.
    UpdatedKittyIncorrectPrice,

    /// No input for kitty buy operation.
    InputMissingBuyingNothing,
    /// No output for kitty buy operation.
    OutputMissingBuyingNothing,
    /// Incorrect number of outputs buy operation.
    IncorrectNumberOfInputKittiesForBuyOperation,
    /// Incorrect number of outputs for buy operation.
    IncorrectNumberOfOutputKittiesForBuyOperation,
    /// Kitty not avilable for sale
    KittyNotForSale,
    /// Kitty price cant be none when it is avilable for sale
    KittyPriceCantBeNone,
    /// Kitty price cant be zero when it is avilable for sale
    KittyPriceCantBeZero,
    /// Not enough amount to buy kitty
    InsufficientCollateralToBuyKitty,

    // From below money constraintchecker errors are added
    /// The transaction attempts to spend without consuming any inputs.
    /// Either the output value will exceed the input value, or if there are no outputs,
    /// it is a waste of processing power, so it is not allowed.
    SpendingNothing,
    /// The value of the spent input coins is less than the value of the newly created
    /// output coins. This would lead to money creation and is not allowed.
    OutputsExceedInputs,
    /// The value consumed or created by this transaction overflows the value type.
    /// This could lead to problems like https://bitcointalk.org/index.php?topic=823.0
    ValueOverflow,
    /// The transaction attempted to create a coin with zero value. This is not allowed
    /// because it wastes state space.
    ZeroValueCoin,
}

impl From<money::ConstraintCheckerError> for TradableKittyConstraintCheckerError {
    fn from(error: money::ConstraintCheckerError) -> Self {
        match error {
            money::ConstraintCheckerError::BadlyTyped => {
                TradableKittyConstraintCheckerError::BadlyTyped
            }
            money::ConstraintCheckerError::MintingWithInputs => {
                TradableKittyConstraintCheckerError::MintingWithInputs
            }
            money::ConstraintCheckerError::MintingNothing => {
                TradableKittyConstraintCheckerError::MintingNothing
            }
            money::ConstraintCheckerError::SpendingNothing => {
                TradableKittyConstraintCheckerError::SpendingNothing
            }
            money::ConstraintCheckerError::OutputsExceedInputs => {
                TradableKittyConstraintCheckerError::OutputsExceedInputs
            }
            money::ConstraintCheckerError::ValueOverflow => {
                TradableKittyConstraintCheckerError::ValueOverflow
            }
            money::ConstraintCheckerError::ZeroValueCoin => {
                TradableKittyConstraintCheckerError::ZeroValueCoin
            }
        }
    }
}

// Implement From trait for mapping ConstraintCheckerError to TradableKittyConstraintCheckerError
impl From<ConstraintCheckerError> for TradableKittyConstraintCheckerError {
    fn from(error: ConstraintCheckerError) -> Self {
        match error {
            ConstraintCheckerError::BadlyTyped => TradableKittyConstraintCheckerError::BadlyTyped,
            ConstraintCheckerError::MinimumSpendAndBreedNotMet => {
                TradableKittyConstraintCheckerError::MinimumSpendAndBreedNotMet
            }
            ConstraintCheckerError::TwoParentsDoNotExist => {
                TradableKittyConstraintCheckerError::TwoParentsDoNotExist
            }
            ConstraintCheckerError::NotEnoughFamilyMembers => {
                TradableKittyConstraintCheckerError::NotEnoughFamilyMembers
            }
            ConstraintCheckerError::IncorrectNumberOfKittiesForMintOperation => {
                TradableKittyConstraintCheckerError::IncorrectNumberOfKittiesForMintOperation
            }
            ConstraintCheckerError::MomNotReadyYet => {
                TradableKittyConstraintCheckerError::MomNotReadyYet
            }
            ConstraintCheckerError::DadTooTired => TradableKittyConstraintCheckerError::DadTooTired,
            ConstraintCheckerError::TwoMomsNotValid => {
                TradableKittyConstraintCheckerError::TwoMomsNotValid
            }
            ConstraintCheckerError::TwoDadsNotValid => {
                TradableKittyConstraintCheckerError::TwoDadsNotValid
            }
            ConstraintCheckerError::NewMomIsStillRearinToGo => {
                TradableKittyConstraintCheckerError::NewMomIsStillRearinToGo
            }
            ConstraintCheckerError::NewDadIsStillRearinToGo => {
                TradableKittyConstraintCheckerError::NewDadIsStillRearinToGo
            }
            ConstraintCheckerError::NewParentFreeBreedingsIncorrect => {
                TradableKittyConstraintCheckerError::NewParentFreeBreedingsIncorrect
            }
            ConstraintCheckerError::NewParentDnaDoesntMatchOld => {
                TradableKittyConstraintCheckerError::NewParentDnaDoesntMatchOld
            }
            ConstraintCheckerError::NewParentNumberBreedingsIncorrect => {
                TradableKittyConstraintCheckerError::NewParentNumberBreedingsIncorrect
            }
            ConstraintCheckerError::NewChildDnaIncorrect => {
                TradableKittyConstraintCheckerError::NewChildDnaIncorrect
            }
            ConstraintCheckerError::NewChildFreeBreedingsIncorrect => {
                TradableKittyConstraintCheckerError::NewChildFreeBreedingsIncorrect
            }
            ConstraintCheckerError::NewChildHasNonZeroBreedings => {
                TradableKittyConstraintCheckerError::NewChildHasNonZeroBreedings
            }
            ConstraintCheckerError::NewChildIncorrectParentInfo => {
                TradableKittyConstraintCheckerError::NewChildIncorrectParentInfo
            }
            ConstraintCheckerError::TooManyBreedingsForKitty => {
                TradableKittyConstraintCheckerError::TooManyBreedingsForKitty
            }
            ConstraintCheckerError::NotEnoughFreeBreedings => {
                TradableKittyConstraintCheckerError::NotEnoughFreeBreedings
            }
            ConstraintCheckerError::MintingNothing => {
                TradableKittyConstraintCheckerError::MintingNothing
            }
            ConstraintCheckerError::MintingWithInputs => {
                TradableKittyConstraintCheckerError::MintingWithInputs
            }
        }
    }
}

#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
    Hash,
    Debug,
    TypeInfo,
)]
pub enum TradableKittyConstraintChecker<const ID: u8> {
    /// A mint transaction that creates Tradable Kitties
    Mint,
    /// A typical Breed transaction where kitties are consumed and new family(Parents(mom,dad) and child) is created.
    Breed,
    ///Update various properties of kitty.
    UpdateProperties,
    ///Can buy a new kitty from others
    Buy,
}

pub trait Breed {
    /// The Cost to breed a kitty.
    const COST: u128;
    type Error: Into<TradableKittyConstraintCheckerError>;
    fn can_breed(mom: &TradableKittyData, dad: &TradableKittyData) -> Result<(), Self::Error>;
    fn check_new_family(
        mom: &TradableKittyData,
        dad: &TradableKittyData,
        new_family: &[DynamicallyTypedData],
    ) -> Result<(), Self::Error>;
}

trait UpdateKittyProperty {
    /// The Cost to update a kitty property if it is not free.
    const COST: u128;
    /// Error type for all Kitty errors.
    type Error: Into<TradableKittyConstraintCheckerError>;

    fn check_updated_kitty(
        original_kitty: &TradableKittyData,
        updated_kitty: &TradableKittyData,
    ) -> Result<(), Self::Error>;
}

trait Buy {
    /// Error type for all Kitty errors.
    type Error: Into<TradableKittyConstraintCheckerError>;

    fn can_buy(
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<(), Self::Error>;
}

pub struct TradableKittyHelpers<const ID: u8>;

impl<const ID: u8> UpdateKittyProperty for TradableKittyHelpers<ID> {
    const COST: u128 = 5u128;
    type Error = TradableKittyConstraintCheckerError;

    fn check_updated_kitty(
        original_kitty: &TradableKittyData,
        updated_kitty: &TradableKittyData,
    ) -> Result<(), Self::Error> {
        ensure!(
            original_kitty.kitty_basic_data.parent == updated_kitty.kitty_basic_data.parent,
            Self::Error::KittyGenderCannotBeUpdated,
        );
        ensure!(
            original_kitty.kitty_basic_data.free_breedings
                == updated_kitty.kitty_basic_data.free_breedings,
            Self::Error::FreeBreedingCannotBeUpdated,
        );
        ensure!(
            original_kitty.kitty_basic_data.dna == updated_kitty.kitty_basic_data.dna,
            Self::Error::KittyDnaCannotBeUpdated,
        );
        ensure!(
            original_kitty.kitty_basic_data.num_breedings
                == updated_kitty.kitty_basic_data.num_breedings,
            Self::Error::NumOfBreedingCannotBeUpdated,
        );

        if !updated_kitty.is_available_for_sale && updated_kitty.price != None {
            return Err(Self::Error::UpdatedKittyIncorrectPrice);
        }

        if updated_kitty.is_available_for_sale
            && (updated_kitty.price == None || updated_kitty.price.unwrap() == 0)
        {
            return Err(Self::Error::UpdatedKittyIncorrectPrice);
        }
        Ok(())
    }
}

impl<const ID: u8> Breed for TradableKittyHelpers<ID> {
    const COST: u128 = 5u128;
    type Error = TradableKittyConstraintCheckerError;
    fn can_breed(mom: &TradableKittyData, dad: &TradableKittyData) -> Result<(), Self::Error> {
        KittyHelpers::can_breed(&mom.kitty_basic_data, &dad.kitty_basic_data)?;
        Ok(())
    }

    fn check_new_family(
        mom: &TradableKittyData,
        dad: &TradableKittyData,
        new_tradable_kitty_family: &[DynamicallyTypedData],
    ) -> Result<(), Self::Error> {
        let new_tradable_kitty_mom = TradableKittyData::try_from(&new_tradable_kitty_family[0])?;
        let new_tradable_kitty_dad = TradableKittyData::try_from(&new_tradable_kitty_family[1])?;
        let new_tradable_kitty_child = TradableKittyData::try_from(&new_tradable_kitty_family[2])?;

        let new_basic_kitty_mom: DynamicallyTypedData =
            new_tradable_kitty_mom.kitty_basic_data.into();
        let new_basic_kitty_dad: DynamicallyTypedData =
            new_tradable_kitty_dad.kitty_basic_data.into();
        let new_basic_kitty_child: DynamicallyTypedData =
            new_tradable_kitty_child.kitty_basic_data.into();

        let mut new_family: Vec<DynamicallyTypedData> = Vec::new();
        new_family.push(new_basic_kitty_mom);
        new_family.push(new_basic_kitty_dad);
        new_family.push(new_basic_kitty_child);

        KittyHelpers::check_new_family(&mom.kitty_basic_data, &dad.kitty_basic_data, &new_family)?;
        Ok(())
    }
}

impl<const ID: u8> Buy for TradableKittyHelpers<ID> {
    type Error = TradableKittyConstraintCheckerError;
    fn can_buy(
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<(), Self::Error> {
        let mut input_coin_data: Vec<DynamicallyTypedData> = Vec::new();
        let mut output_coin_data: Vec<DynamicallyTypedData> = Vec::new();
        let mut input_kitty_data: Vec<DynamicallyTypedData> = Vec::new();
        let mut output_kitty_data: Vec<DynamicallyTypedData> = Vec::new();

        let mut total_input_amount: u128 = 0;
        let mut total_price_of_kitty: u128 = 0;

        for utxo in input_data {
            if let Ok(coin) = utxo.extract::<Coin<ID>>() {
                let utxo_value = coin.0;
                ensure!(
                    utxo_value > 0,
                    TradableKittyConstraintCheckerError::ZeroValueCoin
                );
                input_coin_data.push(utxo.clone());
                total_input_amount = total_input_amount
                    .checked_add(utxo_value)
                    .ok_or(TradableKittyConstraintCheckerError::ValueOverflow)?;

                // Process Kitty
            } else if let Ok(tradable_kitty) = utxo.extract::<TradableKittyData>() {
                if !tradable_kitty.is_available_for_sale {
                    return Err(Self::Error::KittyNotForSale);
                }
                let price = match tradable_kitty.price {
                    None => return Err(Self::Error::KittyPriceCantBeNone),
                    Some(p) => p,
                };

                input_kitty_data.push(utxo.clone());
                total_price_of_kitty = total_price_of_kitty
                    .checked_add(price)
                    .ok_or(TradableKittyConstraintCheckerError::ValueOverflow)?;
                // Process TradableKittyData
                // You can also use the `tradable_kitty` variable herex
            } else {
                return Err(Self::Error::BadlyTyped);
            }
        }

        // Need to filter only Coins and send to MoneyConstraintChecker
        for utxo in output_data {
            if let Ok(coin) = utxo.extract::<Coin<ID>>() {
                let utxo_value = coin.0;
                ensure!(
                    utxo_value > 0,
                    TradableKittyConstraintCheckerError::ZeroValueCoin
                );
                output_coin_data.push(utxo.clone());
                // Process Coin
            } else if let Ok(_tradable_kitty) = utxo.extract::<TradableKittyData>() {
                output_kitty_data.push(utxo.clone());
            } else {
                return Err(Self::Error::BadlyTyped);
            }
        }

        ensure!(
            !input_kitty_data.is_empty(),
            TradableKittyConstraintCheckerError::InputMissingBuyingNothing
        );

        // Make sure there is at least one output being minted
        ensure!(
            !output_kitty_data.is_empty(),
            TradableKittyConstraintCheckerError::OutputMissingBuyingNothing
        );
        ensure!(
            total_price_of_kitty <= total_input_amount,
            TradableKittyConstraintCheckerError::InsufficientCollateralToBuyKitty
        );

        // Need to filter only Coins and send to MoneyConstraintChecker
        MoneyConstraintChecker::<0>::Spend.check(&input_coin_data, &[], &output_coin_data)?;
        Ok(())
    }
}

impl<const ID: u8> SimpleConstraintChecker for TradableKittyConstraintChecker<ID> {
    type Error = TradableKittyConstraintCheckerError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        match &self {
            Self::Mint => {
                // Make sure there are no inputs being consumed
                ensure!(
                    input_data.is_empty(),
                    TradableKittyConstraintCheckerError::MintingWithInputs
                );

                // Make sure there is at least one output being minted
                ensure!(
                    !output_data.is_empty(),
                    TradableKittyConstraintCheckerError::MintingNothing
                );

                // Make sure the outputs are the right type
                for utxo in output_data {
                    let _utxo_kitty = utxo
                        .extract::<TradableKittyData>()
                        .map_err(|_| TradableKittyConstraintCheckerError::BadlyTyped)?;
                }
                return Ok(0);
            }
            Self::Breed => {
                ensure!(input_data.len() == 2, Self::Error::TwoParentsDoNotExist);
                let mom = TradableKittyData::try_from(&input_data[0])?;
                let dad = TradableKittyData::try_from(&input_data[1])?;
                TradableKittyHelpers::<ID>::can_breed(&mom, &dad)?;
                ensure!(output_data.len() == 3, Self::Error::NotEnoughFamilyMembers);
                TradableKittyHelpers::<ID>::check_new_family(&mom, &dad, output_data)?;
                return Ok(0);
            }
            Self::UpdateProperties => {
                ensure!(
                    !input_data.is_empty(),
                    TradableKittyConstraintCheckerError::InputMissingUpdatingNothing
                );

                ensure!(
                    input_data.len() == output_data.len(),
                    TradableKittyConstraintCheckerError::MismatchBetweenNumberOfInputAndUpdateUpadtingNothing
                );

                let original_kitty = TradableKittyData::try_from(&input_data[0])?;
                let updated_kitty = TradableKittyData::try_from(&output_data[0])?;
                TradableKittyHelpers::<ID>::check_updated_kitty(&original_kitty, &updated_kitty)?;
            }
            Self::Buy => {
                TradableKittyHelpers::<ID>::can_buy(input_data, output_data)?;
                return Ok(0);
            }
        }
        Ok(0)
    }
}
