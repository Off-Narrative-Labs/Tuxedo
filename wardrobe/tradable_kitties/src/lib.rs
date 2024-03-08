//! # TradableKitty Module
//!
//! This Tuxedo piece codifies additional features that work with the Kitties piece.
//! This piece should not and cannot be used without that piece.
//! The `TradableKitty` module defines a specialized type of kitty tailored for trading with unique features.
//!
//! TradableKitties enrich the user experience by introducing trading/selling capabilities.
//! The following are features supported:
//!
//! - **ListKittyForSale:** Convert basic kitties into tradable kitties, adding a `price` field.
//! - **DelistKittyFromSale:** Transform tradable kitties back into regular kitties when owners decide not to sell.
//! - **UpdateKittyPrice:** Allow owners to modify the `price` of TradableKitties.
//! - **UpdateKittyName:** Permit owners to update the `name` of TradableKitties.
//!
//!   *Note: Only one kitty can be traded at a time.*
//!
//! - **Buy:** Enable users to securely purchase TradableKitty from others, ensuring fair exchanges.
//!   Make sure to place the kitty first and then coins in the inputs and outputs.
//!
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

/// A `TradableKittyData` is required for trading the kitty.
/// It includes the `price` of the kitty, in addition to the basic `KittyData`.
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
    /// List the kitty for sale. This means the kitty will be converted to a tradable kitty once the transaction is executed..
    ListKittyForSale,
    /// Delist the kitty from sale, This means tradable kitty will converted back to kitty.
    DelistKittyFromSale,
    /// Update the `price` of tradable kitty.
    UpdateKittyPrice,
    // Update the name of the kitty.
    UpdateKittyName,
    /// For buying a new kitty from other owners.
    Buy,
}

/// Extract basic kitty data from a list of dynamically typed `TradableKitty` data, populating a list with basic kitty data.
fn extract_basic_kitty_list(
    tradable_kitty_data: &[DynamicallyTypedData],
    kitty_data_list: &mut Vec<DynamicallyTypedData>,
) -> Result<(), TradeableKittyError> {
    for utxo in tradable_kitty_data {
        if let Ok(tradable_kitty) = utxo.extract::<TradableKittyData>() {
            kitty_data_list.push(tradable_kitty.kitty_basic_data.clone().into());
        } else {
            return Err(TradeableKittyError::BadlyTyped);
        }
    }
    Ok(())
}

/// Checks if buying the kitty is possible or not. It depends on the Money variable to validate the spending of coins.
/// Make sure to place the kitty first and then the coins in the transaction.
fn check_can_buy<const ID: u8>(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
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

    for i in 1..input_data.len() {
        let coin = input_data[i]
            .clone()
            .extract::<Coin<ID>>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        let utxo_value = coin.0;
        ensure!(
            utxo_value > 0,
            TradeableKittyError::MoneyError(MoneyError::ZeroValueCoin)
        );
        input_coin_data.push(input_data[i].clone());
        total_input_amount = total_input_amount
            .checked_add(utxo_value)
            .ok_or(TradeableKittyError::MoneyError(MoneyError::ValueOverflow))?;
    }

    for i in 1..output_data.len() {
        let coin = output_data[i]
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
        output_coin_data.push(output_data[i].clone());
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
fn check_kitty_price_update(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    ensure!(
        input_data.len() == output_data.len() && !input_data.is_empty(),
        { TradeableKittyError::NumberOfInputOutputMismatch }
    );
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
fn check_can_list_kitty_for_sale(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    check_kitty_tdkitty_interconversion(&input_data, &output_data)?;
    Ok(0)
}

/// Wrapper function for verifying the conversion from tradable kitties to basic kitties.
/// Multiple tradable kitties can be converted in a single transaction.
fn check_can_delist_kitty_from_sale(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    // Below is the conversion from tradable kitty to regular kitty, the reverse of the ListKittyForSale.
    // Hence, input parameters are reversed.
    check_kitty_tdkitty_interconversion(&output_data, &input_data)?;
    Ok(0)
}

/// Validates inter-conversion between both regular kitties and tradable kitties, as used by the `listForSale` and `delistFromSale` functions.
fn check_kitty_tdkitty_interconversion(
    kitty_data: &[DynamicallyTypedData],
    tradable_kitty_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    ensure!(
        kitty_data.len() == tradable_kitty_data.len() && !kitty_data.is_empty(),
        { TradeableKittyError::NumberOfInputOutputMismatch }
    );

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
            Self::ListKittyForSale => {
                check_can_list_kitty_for_sale(&input_data, &output_data)?;
            }
            Self::DelistKittyFromSale => {
                check_can_delist_kitty_from_sale(&input_data, &output_data)?;
            }
            Self::UpdateKittyPrice => {
                check_kitty_price_update(input_data, output_data)?;
            }
            Self::UpdateKittyName => {
                let mut input_basic_kitty_data: Vec<DynamicallyTypedData> = Vec::new();
                let mut output_basic_kitty_data: Vec<DynamicallyTypedData> = Vec::new();
                let _ = extract_basic_kitty_list(&input_data, &mut input_basic_kitty_data)?;
                let _ = extract_basic_kitty_list(&output_data, &mut output_basic_kitty_data)?;
                kitties::can_kitty_name_be_updated(
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
