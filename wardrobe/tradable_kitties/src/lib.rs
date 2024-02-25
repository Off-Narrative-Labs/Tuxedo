//! # TradableKitty Module
//!
//! The `TradableKitty` module defines a specialized type of kitty tailored for trading with unique features.
//!
//! ## Features
//!
//! The module supports multiple tradable kitties in the same transactions. Note that the input and output kitties must follow the same order.
//!
//! - **ListKittyForSale:** Convert basic kitties into tradable kitties, adding a `Price` field.
//! - **DelistKittyFromSale:** Transform tradable kitties back into regular kitties when owners decide not to sell.
//! - **UpdateKittyPrice:** Allow owners to modify the price of TradableKitties.
//! - **UpdateKittyName:** Permit owners to update the name of TradableKitties.
//!
//!   *Note: Only one kitty can be traded at a time.*
//!
//! - **Buy:** Enable users to securely purchase TradableKitty from others, ensuring fair exchanges.
//!
//! TradableKitties enrich the user experience by introducing advanced trading capabilities.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use kitties::{KittyDNA, KittyData};
use money::ConstraintCheckerError as MoneyError;
use money::{Coin, MoneyConstraintChecker};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::collections::btree_set::BTreeSet; // For checking the uniqueness of input and output based on dna.
use sp_std::prelude::*;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, SimpleConstraintChecker,
};

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
    /// Input missing for the transaction.
    InputMissingError,
    /// Not enough amount to buy a kitty.
    InsufficientCollateralToBuyKitty,
    /// The number of input vs number of output doesn't match for a transaction.
    NumberOfInputOutputMismatch,
    /// Can't buy more than one kitty at a time.
    CannotBuyMoreThanOneKittyAtTime,
    /// Kitty basic properties such as DNA, free breeding, and a number of breedings, are altered error.
    KittyBasicPropertiesAltered,
    /// Kitty not available for sale.Occur when price is None.
    KittyNotForSale,
    /// Kitty price cant be none when it is available for sale.
    KittyPriceCantBeNone,
    /// Duplicate kitty foundi.e based on the DNA.
    DuplicateKittyFound,
    /// Duplicate tradable kitty foundi.e based on the DNA.
    DuplicateTradableKittyFound,
    /// Kitty price is unaltered is not allowed for kitty price update transactions.
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
/// Listing kitty for sale : multiple kiities are allowed, provided input and output in same order.
/// Delisting kitty from sale : multiple tradable kiities are allowed, provided input and output in same order.
/// Update kitty price : multiple tradable kiities are allowed, provided input and output in same order.
/// Update kitty name : multiple tradable kiities are allowed, provided input and output in same order.
/// Buy tradable kitty :  multiple tradable kiities are not allowed, Only single kiity operation is allowed.
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

/// Checks if buying the kitty is possible of not. It depends on Money piece to validate spending of coins.
fn check_can_buy<const ID: u8>(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<(), TradeableKittyError> {
    let mut input_coin_data: Vec<DynamicallyTypedData> = Vec::new();
    let mut output_coin_data: Vec<DynamicallyTypedData> = Vec::new();
    let mut input_ktty_to_be_traded: Option<TradableKittyData> = None;
    let mut output_ktty_to_be_traded: Option<TradableKittyData> = None;

    let mut total_input_amount: u128 = 0;
    let mut total_price_of_kitty: u128 = 0;

    // Seperate the coin and tradable kitty in to seperate vecs from the input_data .
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
        } else if let Ok(td_input_kitty) = utxo.extract::<TradableKittyData>() {
            // Process tradable kitty
            // Checking if more than 1 kitty is sent for trading.
            match input_ktty_to_be_traded {
                None => {
                    // 1st tradable kitty is received in the input.
                    input_ktty_to_be_traded = Some(td_input_kitty.clone());
                    let price = match td_input_kitty.price {
                        None => return Err(TradeableKittyError::KittyNotForSale),
                        Some(p) => p,
                    };
                    total_price_of_kitty = total_price_of_kitty
                        .checked_add(price)
                        .ok_or(TradeableKittyError::MoneyError(MoneyError::ValueOverflow))?;
                }
                Some(_) => {
                    // More than 1 tradable kitty are sent for trading.
                    return Err(TradeableKittyError::CannotBuyMoreThanOneKittyAtTime);
                }
            };
        } else {
            return Err(TradeableKittyError::BadlyTyped);
        }
    }

    // Ensuring we found kitty in the input
    ensure!(input_ktty_to_be_traded != None, {
        TradeableKittyError::InputMissingError
    });

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
            // Checking if more than 1 kitty in output is sent for trading.
            match output_ktty_to_be_traded {
                None => {
                    // 1st tradable kitty is received in the output.
                    output_ktty_to_be_traded = Some(td_output_kitty.clone());
                    ensure!(
                        input_ktty_to_be_traded.clone().unwrap().kitty_basic_data
                            == td_output_kitty.kitty_basic_data, // basic kitty data is unaltered
                        TradeableKittyError::KittyBasicPropertiesAltered // this need to be chan
                    );
                }
                Some(_) => {
                    // More than 1 tradable kitty are sent in output for trading.
                    return Err(TradeableKittyError::CannotBuyMoreThanOneKittyAtTime);
                }
            };
        } else {
            return Err(TradeableKittyError::BadlyTyped);
        }
    }

    // Ensuring we found kitty in the output
    ensure!(output_ktty_to_be_traded != None, {
        TradeableKittyError::InputMissingError
    });

    // Ensuring total money sent is enough to buy the kitty
    ensure!(
        total_price_of_kitty <= total_input_amount,
        TradeableKittyError::InsufficientCollateralToBuyKitty
    );

    // Filterd coins sent to MoneyConstraintChecker for money validation.
    MoneyConstraintChecker::<0>::Spend.check(&input_coin_data, &[], &output_coin_data)?;
    Ok(())
}

/// checks if tradable kitty price updates is possible of not.
/// Price of multiple tradable kitties can be updated in the same txn.
fn check_kitty_price_update(
    input_data: &[DynamicallyTypedData],
    output_data: &[DynamicallyTypedData],
) -> Result<TransactionPriority, TradeableKittyError> {
    ensure!(
        input_data.len() == output_data.len() && !input_data.is_empty(),
        { TradeableKittyError::NumberOfInputOutputMismatch }
    );

    let mut dna_to_tdkitty_set: BTreeSet<KittyDNA> = BTreeSet::new(); // to check the uniqueness in input
                                                                      //input td-kitty and output td kitties need to be in same order.
    for i in 0..input_data.len() {
        let utxo_input_tradable_kitty = input_data[i]
            .extract::<TradableKittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        if dna_to_tdkitty_set.contains(&utxo_input_tradable_kitty.kitty_basic_data.dna) {
            return Err(TradeableKittyError::DuplicateTradableKittyFound);
        } else {
            dna_to_tdkitty_set.insert(utxo_input_tradable_kitty.clone().kitty_basic_data.dna);
        }

        let utxo_output_tradable_kitty = output_data[i]
            .extract::<TradableKittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        ensure!(
            utxo_input_tradable_kitty.kitty_basic_data
                == utxo_output_tradable_kitty.kitty_basic_data,
            TradeableKittyError::KittyBasicPropertiesAltered
        );
        match utxo_output_tradable_kitty.price {
            Some(_) => {
                ensure!(
                    utxo_input_tradable_kitty.price != utxo_output_tradable_kitty.price, // kitty ptice is unaltered
                    TradeableKittyError::KittyPriceUnaltered
                );
            }
            None => return Err(TradeableKittyError::KittyPriceCantBeNone),
        };
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

    let mut dna_to_tdkitty_set: BTreeSet<KittyDNA> = BTreeSet::new();
    let mut dna_to_kitty_set: BTreeSet<KittyDNA> = BTreeSet::new();

    for i in 0..kitty_data.len() {
        let utxo_kitty = kitty_data[i]
            .extract::<KittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        if dna_to_kitty_set.contains(&utxo_kitty.dna) {
            return Err(TradeableKittyError::DuplicateKittyFound);
        } else {
            dna_to_kitty_set.insert(utxo_kitty.clone().dna);
        }

        let utxo_tradable_kitty = tradable_kitty_data[i]
            .extract::<TradableKittyData>()
            .map_err(|_| TradeableKittyError::BadlyTyped)?;

        if dna_to_tdkitty_set.contains(&utxo_tradable_kitty.kitty_basic_data.dna) {
            return Err(TradeableKittyError::DuplicateTradableKittyFound);
        } else {
            dna_to_tdkitty_set.insert(utxo_tradable_kitty.clone().kitty_basic_data.dna);
        }

        ensure!(
            utxo_kitty == utxo_tradable_kitty.kitty_basic_data,
            TradeableKittyError::KittyBasicPropertiesAltered
        );
        ensure!(
            utxo_tradable_kitty.price != None, // basic kitty data is unaltered
            TradeableKittyError::KittyPriceCantBeNone  // this need to be chan
        );
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
