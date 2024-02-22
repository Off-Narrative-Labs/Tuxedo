//! This module defines `TradableKitty`, a specialized type of kitty designed for trading with unique features.
//!
//! ## Features
//!
//! - **ListKittyForSale:** Convert basic kitties into tradable kitties, adding a `Price` field.
//! - **DelistKittyFromSale:** Transform tradable kitties back into regular kitties when owners decide not to sell.
//! - **UpdateKittyPrice:** Allow owners to modify the price of TradableKitties.
//! - **UpdateKittyName:** Permit owners to update the name of TradableKitties.
//! - **Buy:** Enable users to securely purchase TradableKitties from others, ensuring fair exchanges.
//!
//!
//! TradableKitties enrich the user experience by introducing advanced trading capabilities.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::prelude::*;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, SimpleConstraintChecker,
};

use kitties::{KittyDNA, KittyData};
use money::ConstraintCheckerError as MoneyError;
use money::{Coin, MoneyConstraintChecker};

#[cfg(test)]
mod tests;

/// A tradableKittyData, required for trading the kitty..
/// It contains optional price of tradable-kitty in addition to the basic kitty data.
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
    /// Basic kitty data composed from kitties piece
    pub kitty_basic_data: KittyData,
    /// Price of the tradable kitty
    pub price: Option<u128>,
}

impl Default for TradableKittyData {
    fn default() -> Self {
        Self {
            kitty_basic_data: KittyData::default(),
            price: None,
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
    /// Error in the underlying money piece.
    MoneyError(money::ConstraintCheckerError),
    /// Error in the underlying kitty piece.
    KittyError(kitties::ConstraintCheckerError),
    /// Dynamic typing issue.
    /// This error doesn't discriminate between badly typed inputs and outputs.
    BadlyTyped,
    /// output missing updating nothing.
    OutputUtxoMissingError,
    /// Input missing for the transaction.
    InputMissingError,
    /// Not enough amount to buy a kitty.
    InsufficientCollateralToBuyKitty,
    /// The number of input vs number of output doesn't match for a transaction.
    NumberOfInputOutputMismatch,
    /// Kitty basic properties such as DNA, free breeding, and a number of breedings, are altered error.
    KittyBasicPropertiesAltered,
    /// Kitty not available for sale.Occur when price is None.
    KittyNotForSale,
    /// Kitty price cant be none when it is available for sale.
    KittyPriceCantBeNone,
    /// Kitty price is unaltered for kitty price update transactions.
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

/// The main constraint checker for the trdable kitty piece. Allows below :
/// Listing kitty for sale
/// Delisting kitty from sale
/// Update kitty price
/// Update kitty name
/// Buy tradable kitty
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
    /// List kitty for sale, means kitty will converted to tradable kitty once transaction is executed
    ListKittyForSale,
    /// Delist kitty from sale, means tradable kitty will converted back to kitty  
    DelistKittyFromSale,
    /// Update price of tradable kitty.
    UpdateKittyPrice,
    // Update name of kitty
    UpdateKittyName,
    /// For buying a new kitty from others
    Buy,
}

/// Extracts basic kitty data from a list of dynamically typed TradableKitty data, populating basic kitty data list.
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

/// checks if buying the kitty is possible of not. It depends on Money piece to validat spending of coins.
fn check_can_buy<const ID: u8>(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<(), TradeableKittyError> {
    let mut input_coin_data: Vec<DynamicallyTypedData> = Vec::new();
    let mut output_coin_data: Vec<DynamicallyTypedData> = Vec::new();
    let mut input_kitty_data: Vec<DynamicallyTypedData> = Vec::new();
    let mut output_kitty_data: Vec<DynamicallyTypedData> = Vec::new();

    let mut total_input_amount: u128 = 0;
    let mut total_price_of_kitty: u128 = 0;

    // Map to verify that output_kitty is same as input_kitty based on the dna after buy operation
    let mut dna_to_tdkitty_map: BTreeMap<KittyDNA, TradableKittyData> = BTreeMap::new();

    // Seperate the coin and tdkitty in to seperate vecs from the input_data .
    for utxo in input_data {
        if let Ok(coin) = utxo.extract::<Coin<ID>>() {
            let utxo_value = coin.0;

            ensure!(
                utxo_value > 0,
                TradeableKittyError::MoneyError(MoneyError::ZeroValueCoin)
            );
            input_coin_data.push(utxo.clone());
            total_input_amount = total_input_amount
                .checked_add(utxo_value)
                .ok_or(TradeableKittyError::MoneyError(MoneyError::ValueOverflow))?;

            // Process Kitty
        } else if let Ok(td_input_kitty) = utxo.extract::<TradableKittyData>() {
            // Trying to buy kitty which is not listed for sale.
            let price = match td_input_kitty.price {
                None => return Err(TradeableKittyError::KittyNotForSale),
                Some(p) => p,
            };

            input_kitty_data.push(utxo.clone());
            dna_to_tdkitty_map.insert(td_input_kitty.clone().kitty_basic_data.dna, td_input_kitty);
            total_price_of_kitty = total_price_of_kitty
                .checked_add(price)
                .ok_or(TradeableKittyError::MoneyError(MoneyError::ValueOverflow))?;
        } else {
            return Err(TradeableKittyError::BadlyTyped);
        }
    }

    // Seperate the coin and tdkitty in to seperate vecs from the output_data .
    for utxo in output_data {
        if let Ok(coin) = utxo.extract::<Coin<ID>>() {
            let utxo_value = coin.0;
            ensure!(
                utxo_value > 0,
                TradeableKittyError::MoneyError(MoneyError::ZeroValueCoin)
            );
            output_coin_data.push(utxo.clone());
            // Process Coin
        } else if let Ok(td_output_kitty) = utxo.extract::<TradableKittyData>() {
            match dna_to_tdkitty_map.remove(&td_output_kitty.kitty_basic_data.dna) {
                Some(found_kitty) => {
                    // During buy opertaion, basic kitty properties cant be updated in the same transaction.
                    ensure!(
                        found_kitty.kitty_basic_data == td_output_kitty.kitty_basic_data, // basic kitty data is unaltered
                        TradeableKittyError::KittyBasicPropertiesAltered // this need to be chan
                    );
                }
                None => {
                    return Err(TradeableKittyError::OutputUtxoMissingError);
                }
            };
            output_kitty_data.push(utxo.clone());
        } else {
            return Err(TradeableKittyError::BadlyTyped);
        }
    }

    ensure!(
        input_kitty_data.len() == output_kitty_data.len() && !input_kitty_data.is_empty(),
        { TradeableKittyError::NumberOfInputOutputMismatch }
    );

    ensure!(
        total_price_of_kitty <= total_input_amount,
        TradeableKittyError::InsufficientCollateralToBuyKitty
    );

    // Filterd coins sent to MoneyConstraintChecker for money validation.
    MoneyConstraintChecker::<0>::Spend.check(&input_coin_data, &[], &output_coin_data)?;
    Ok(())
}

/// checks if kitty price updates is possible of not.
/// Price of multiple kitties can be updated in the same txn.
fn check_kitty_price_update(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    ensure!(
        input_data.len() == output_data.len() && !input_data.is_empty(),
        { TradeableKittyError::NumberOfInputOutputMismatch }
    );

    let mut dna_to_tdkitty_map: BTreeMap<KittyDNA, TradableKittyData> = BTreeMap::new();

    for utxo in input_data {
        let td_input_kitty = utxo
            .extract::<TradableKittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;
        dna_to_tdkitty_map.insert(td_input_kitty.clone().kitty_basic_data.dna, td_input_kitty);
    }

    for utxo in output_data {
        let td_output_kitty = utxo
            .extract::<TradableKittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        if let Some(found_kitty) = dna_to_tdkitty_map.remove(&td_output_kitty.kitty_basic_data.dna)
        {
            // Element found, access the value
            ensure!(
                found_kitty.kitty_basic_data == td_output_kitty.kitty_basic_data, // basic kitty data is unaltered
                TradeableKittyError::KittyBasicPropertiesAltered // this need to be chan
            );
            match td_output_kitty.price {
                Some(_) => {
                    ensure!(
                        found_kitty.price != td_output_kitty.price, // kitty ptice is unaltered
                        TradeableKittyError::KittyPriceUnaltered    // this need to be chan
                    );
                }
                None => return Err(TradeableKittyError::KittyPriceCantBeNone),
            };
        } else {
            return Err(TradeableKittyError::OutputUtxoMissingError);
        }
    }
    return Ok(0);
}

/// Wrapper function for checking conversion from basic kitty to tradable kitty.
/// Multiple kitties can be converted in the same txn.
fn check_can_list_kitty_for_sale(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    check_kitty_tdkitty_interconversion(&input_data, &output_data)?;
    return Ok(0);
}

/// Wrapper function for checking conversion from  tradable kitty to basic kitty.
/// Multiple kitties can be converted from tradable to non-tradable in the same txn.
fn check_can_delist_kitty_from_sale(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    // Below is conversion from tradable kitty to kitty, reverse of the ListKittyForSale,
    // hence input params are rebversed
    check_kitty_tdkitty_interconversion(&output_data, &input_data)?;
    return Ok(0);
}

/// Validaes inter-conversion b/w both kitty & tradable kitty.Used by listForSale & delistFromSale functions.
fn check_kitty_tdkitty_interconversion(
    kitty_data: &[DynamicallyTypedData],
    tradable_kitty_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    ensure!(
        kitty_data.len() == tradable_kitty_data.len() && !kitty_data.is_empty(),
        { TradeableKittyError::NumberOfInputOutputMismatch }
    );

    let mut map: BTreeMap<KittyDNA, KittyData> = BTreeMap::new();

    for utxo in kitty_data {
        let utxo_kitty = utxo
            .extract::<KittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;
        map.insert(utxo_kitty.clone().dna, utxo_kitty);
    }

    for utxo in tradable_kitty_data {
        let utxo_tradable_kitty = utxo
            .extract::<TradableKittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        match map.remove(&utxo_tradable_kitty.kitty_basic_data.dna) {
            Some(kitty) => {
                ensure!(
                    kitty == utxo_tradable_kitty.kitty_basic_data, // basic kitty data is unaltered
                    TradeableKittyError::KittyBasicPropertiesAltered  // this need to be chan
                );
                let _ = match utxo_tradable_kitty.price {
                    Some(_) => {}
                    None => return Err(TradeableKittyError::KittyPriceCantBeNone),
                };
            }
            None => {
                return Err(TradeableKittyError::InputMissingError);
            }
        };
    }
    return Ok(0);
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
                return Ok(0);
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
                check_can_buy::<ID>(input_data, output_data)?;
                return Ok(0);
            }
        }
        Ok(0)
    }
}
