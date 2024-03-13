//! # TradableKitty Module
//!
//! This Tuxedo piece codifies additional features that work with the Kitties piece.
//! This piece should not and cannot be used without the Kitties and Money pieces.
//! The introduced features are:
//! This piece should not and cannot be used without the Kitties and Money pieces.
//! The introduced features are:
//!
//! - **ListKittiesForSale:** Convert basic kitties into tradable kitties, adding a `price` field.
//! - **DelistKittiesFromSale:** Transform tradable kitties back into regular kitties when owners decide not to sell.
//! - **UpdateKittiesPrice:** Allow owners to modify the `price` of TradableKitties.
//! - **UpdateKittiesName:** Permit owners to update the `name` of TradableKitties.
//!
//! - **Buy:** Enable users to securely purchase TradableKitty from others, ensuring fair exchanges.
//!   Make sure to place the kitty first and then coins in the inputs and outputs.
//!
//!   *Note: Only one kitty can be bought at a time.*
//!

#![cfg_attr(not(feature = "std"), no_std)]

use kitties::KittyData;
use money::{Coin, ConstraintCheckerError as MoneyError, MoneyConstraintChecker};
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

/// The default price of a kitten is 10.
const DEFAULT_KITTY_PRICE: u128 = 10;

/// A `TradableKittyData` is required for trading the Kitty.
/// It includes the `price` of the Kitty, in addition to the basic `KittyData` provided by the Kitties piece.
/// A `TradableKittyData` is required for trading the Kitty.
/// It includes the `price` of the Kitty, in addition to the basic `KittyData` provided by the Kitties piece.
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
    /// Basic `KittyData` composed from the `kitties` piece.
    pub kitty_basic_data: KittyData,
    /// Price of the `TradableKitty`
    pub price: u128,
}

impl Default for TradableKittyData {
    fn default() -> Self {
        Self {
            kitty_basic_data: KittyData::default(),
            price: DEFAULT_KITTY_PRICE,
        }
    }
}

impl TryFrom<&DynamicallyTypedData> for TradableKittyData {
    type Error = TradeableKittyError;
    fn try_from(a: &DynamicallyTypedData) -> Result<Self, Self::Error> {
        a.extract::<TradableKittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)
    }
}

impl UtxoData for TradableKittyData {
    const TYPE_ID: [u8; 4] = *b"tdkt";
}

/// Reasons that tradable kitty opertaion may go wrong.
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
pub enum TradeableKittyError {
    /// Error in the underlying `money` piece.
    MoneyError(money::ConstraintCheckerError),
    /// Error in the underlying `kitties` piece.
    KittyError(kitties::ConstraintCheckerError),
    /// Dynamic typing issue.
    /// This error doesn't discriminate between badly typed inputs and outputs.
    BadlyTyped,
    /// Input missing for the transaction.
    InputMissingError,
    /// Output missing for the transaction.
    OutputMissingError,
    /// Not enough amount to buy a `kitty`.
    InsufficientCollateralToBuyKitty,
    /// The number of input vs number of output doesn't match for a transaction.
    NumberOfInputOutputMismatch,
    /// Kitty basic properties such as `DNA`, `free breeding`, and the `number of breedings`, are altered error.
    KittyBasicPropertiesAltered,
    /// Kitty `price` can't be zero when it is available for sale.
    KittyPriceCantBeZero,
    /// Kitty `price` is unaltered and is not allowed for kitty price update transactions.
    KittyPriceUnaltered,
}

impl From<money::ConstraintCheckerError> for TradeableKittyError {
    fn from(error: money::ConstraintCheckerError) -> Self {
        TradeableKittyError::MoneyError(error)
    }
}

impl From<kitties::ConstraintCheckerError> for TradeableKittyError {
    fn from(error: kitties::ConstraintCheckerError) -> Self {
        TradeableKittyError::KittyError(error)
    }
}

/// The main constraint checker for the tradable kitty piece. Allows the following:
/// Listing kitty for sale: Multiple kitties are allowed, provided input and output are in the same order.
/// Delisting kitty from sale: Multiple tradable kitties are allowed, provided input and output are in the same order.
/// Update kitty price: Multiple tradable kitties are allowed, provided input and output are in the same order.
/// Update kitty name: Multiple tradable kitties are allowed, provided input and output are in the same order.
/// Buy tradable kitty: Multiple tradable kitties are not allowed. Only a single kitty operation is allowed.
/// For buying a kitty, you need to send the kitty first, and then coins in both input and output of the transaction.

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
    /// List the kitties for sale. This means the kitties will be converted to a tradable kitties once the transaction is executed.
    ListKittiesForSale,
    /// Delist the kitties from sale, This means tradable kitties will converted back to kitties.
    DelistKittiesFromSale,
    /// Update the `price` of tradable kitties.
    UpdateKittiesPrice,
    // Update the name of the kitties.
    UpdateKittiesName,
    /// For buying a new kitty from other owners.
    Buy,
}

/// Checks if buying the kitty is possible or not. It depends on the Money variable to validate the spending of coins.
/// Make sure to place the kitty first and then the coins in the transaction.
fn check_can_buy<const ID: u8>(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    ensure!(!input_data.is_empty(), {
        TradeableKittyError::InputMissingError
    });

    ensure!(!output_data.is_empty(), {
        TradeableKittyError::OutputMissingError
    });

    let mut input_coin_data: Vec<DynamicallyTypedData> = Vec::new();
    let mut output_coin_data: Vec<DynamicallyTypedData> = Vec::new();
    let input_kitty_to_be_traded: Option<TradableKittyData>;

    let mut total_input_amount: u128 = 0;
    let total_price_of_kitty: u128;

    if let Ok(td_input_kitty) = input_data[0].extract::<TradableKittyData>() {
        ensure!(
            td_input_kitty.price != 0,
            TradeableKittyError::KittyPriceCantBeZero
        );
        input_kitty_to_be_traded = Some(td_input_kitty.clone());
        total_price_of_kitty = td_input_kitty.price;
    } else {
        return Err(TradeableKittyError::BadlyTyped);
    }

    if let Ok(td_output_kitty) = output_data[0].extract::<TradableKittyData>() {
        ensure!(
            input_kitty_to_be_traded.clone().unwrap().kitty_basic_data
                == td_output_kitty.kitty_basic_data,
            TradeableKittyError::KittyBasicPropertiesAltered
        );
    } else {
        return Err(TradeableKittyError::BadlyTyped);
    }

    for coin_data in input_data.iter().skip(1) {
        let coin = coin_data
            .clone()
            .extract::<Coin<ID>>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        let utxo_value = coin.0;
        ensure!(
            utxo_value > 0,
            TradeableKittyError::MoneyError(MoneyError::ZeroValueCoin)
        );
        input_coin_data.push(coin_data.clone());
        total_input_amount = total_input_amount
            .checked_add(utxo_value)
            .ok_or(TradeableKittyError::MoneyError(MoneyError::ValueOverflow))?;
    }

    for coin_data in output_data.iter().skip(1) {
        let coin = coin_data
            .clone()
            .extract::<Coin<ID>>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        let utxo_value = coin.0;
        ensure!(
            utxo_value > 0,
            TradeableKittyError::MoneyError(MoneyError::ZeroValueCoin)
        );
        ensure!(
            utxo_value > 0,
            TradeableKittyError::MoneyError(MoneyError::ZeroValueCoin)
        );
        output_coin_data.push(coin_data.clone());
    }
    ensure!(
        total_price_of_kitty <= total_input_amount,
        TradeableKittyError::InsufficientCollateralToBuyKitty
    );

    // Filtered coins are sent to MoneyConstraintChecker for money validation.
    Ok(MoneyConstraintChecker::<0>::Spend.check(&input_coin_data, &[], &output_coin_data)?)
}

/// Checks if updates to the prices of tradable kitties are possible or not.
/// Prices of multiple tradable kitties can be updated in the same transaction.
fn check_kitties_price_update(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    ensure!(input_data.len() == output_data.len(), {
        TradeableKittyError::NumberOfInputOutputMismatch
    });
    for i in 0..input_data.len() {
        let utxo_input_tradable_kitty = input_data[i]
            .extract::<TradableKittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        let utxo_output_tradable_kitty = output_data[i]
            .extract::<TradableKittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        ensure!(
            utxo_input_tradable_kitty.kitty_basic_data
                == utxo_output_tradable_kitty.kitty_basic_data,
            TradeableKittyError::KittyBasicPropertiesAltered
        );
        ensure!(
            utxo_output_tradable_kitty.price != 0,
            TradeableKittyError::KittyPriceCantBeZero
        );
        ensure!(
            utxo_input_tradable_kitty.price != utxo_output_tradable_kitty.price,
            TradeableKittyError::KittyPriceUnaltered
        );
    }

    Ok(0)
}

/// Wrapper function for verifying the conversion from basic kitties to tradable kitties.
/// Multiple kitties can be converted in a single transaction.
fn check_can_list_kitties_for_sale(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    check_kitties_tdkitties_interconversion(input_data, output_data)?;
    Ok(0)
}

/// Wrapper function for verifying the conversion from tradable kitties to basic kitties.
/// Multiple tradable kitties can be converted in a single transaction.
fn check_can_delist_kitties_from_sale(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    // Below is the conversion from tradable kitty to regular kitty, the reverse of the ListKittiesForSale.
    // Hence, input parameters are reversed.
    check_kitties_tdkitties_interconversion(output_data, input_data)?;
    Ok(0)
}

/// Validates inter-conversion between both regular kitties and tradable kitties, as used by the `listForSale` and `delistFromSale` functions.
fn check_kitties_tdkitties_interconversion(
    kitty_data: &[DynamicallyTypedData],
    tradable_kitty_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    ensure!(kitty_data.len() == tradable_kitty_data.len(), {
        TradeableKittyError::NumberOfInputOutputMismatch
    });

    for i in 0..kitty_data.len() {
        let utxo_kitty = kitty_data[i]
            .extract::<KittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        let utxo_tradable_kitty = tradable_kitty_data[i]
            .extract::<TradableKittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        ensure!(
            utxo_kitty == utxo_tradable_kitty.kitty_basic_data,
            TradeableKittyError::KittyBasicPropertiesAltered
        );
        ensure!(
            utxo_tradable_kitty.price != 0,
            TradeableKittyError::KittyPriceCantBeZero
        );
    }

    Ok(0)
}

impl<const ID: u8> SimpleConstraintChecker for TradableKittyConstraintChecker<ID> {
    type Error = TradeableKittyError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        match &self {
            Self::ListKittiesForSale => {
                check_can_list_kitties_for_sale(input_data, output_data)?;
            }
            Self::DelistKittiesFromSale => {
                check_can_delist_kitties_from_sale(input_data, output_data)?;
            }
            Self::UpdateKittiesPrice => {
                check_kitties_price_update(input_data, output_data)?;
            }
            Self::UpdateKittiesName => {
                let result: Result<Vec<DynamicallyTypedData>, _> = input_data
                    .iter()
                    .map(|utxo| utxo.extract::<TradableKittyData>())
                    .collect::<Result<Vec<_>, _>>()
                    .map(|vec| {
                        vec.into_iter()
                            .map(|tdkitty| tdkitty.kitty_basic_data.clone().into())
                            .collect()
                    });

                let input_basic_kitty_data = result.map_err(|_| TradeableKittyError::BadlyTyped)?;

                let result: Result<Vec<DynamicallyTypedData>, _> = output_data
                    .iter()
                    .map(|utxo| utxo.extract::<TradableKittyData>())
                    .collect::<Result<Vec<_>, _>>()
                    .map(|vec| {
                        vec.into_iter()
                            .map(|tdkitty| tdkitty.kitty_basic_data.clone().into())
                            .collect()
                    });

                let output_basic_kitty_data =
                    result.map_err(|_| TradeableKittyError::BadlyTyped)?;

                kitties::can_kitties_name_be_updated(
                    &input_basic_kitty_data,
                    &output_basic_kitty_data,
                )?;
            }
            Self::Buy => {
                let priority = check_can_buy::<ID>(input_data, output_data)?;
                return Ok(priority);
            }
        }
        Ok(0)
    }
}
