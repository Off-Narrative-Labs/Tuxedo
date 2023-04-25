//! An order book decentralized exchange between two specific tokens.

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

use sp_std::prelude::*;

use parity_scale_codec::{Decode, Encode};

use tuxedo_core::{
    dynamic_typing::DynamicallyTypedData, dynamic_typing::UtxoData, ensure, traits::Cash,
    types::Output, SimpleConstraintChecker, Verifier,
};
use sp_runtime::transaction_validity::TransactionPriority;

/// An order in the book. Represents a binding collateralized
/// offer to make a trade.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub struct Order<V: Verifier /* TODO more generics */> {
    /// The amount of token A I'm willing to trade away
    offer_amount: u128,
    /// The amount of token B I demand to receive
    ask_amount: u128,
    /// The verifier (maybe a signature check) that will protect the payout coin
    payout_verifier: V,
}

impl<V: Verifier> UtxoData for Order<V> {
    // TODO fix when we have generic coins
    const TYPE_ID: [u8; 4] = *b"ordr";
}

/// Anything that can go wrong (make a transaction invalid) when using a dex
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub enum DexError {
    /// Some output had a type that wasn't the expected type
    TypeError,
    /// User is trying to make an order, but has not put up enough collateral
    /// to back the offer amount
    InsufficientCollateral,
}

/// Place a new order in the book
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub struct MakeOrder;

impl SimpleConstraintChecker for MakeOrder {
    type Error = DexError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!()
    }
}

/// Match two or more orders together
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub struct MatchOrders;

impl SimpleConstraintChecker for MatchOrders {
    type Error = DexError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!()
    }
}
