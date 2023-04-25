//! An order book decentralized exchange between two specific tokens.

use core::marker::PhantomData;

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

use sp_std::prelude::*;
use sp_std::fmt::Debug;

use parity_scale_codec::{Decode, Encode};

use tuxedo_core::{
    dynamic_typing::DynamicallyTypedData, dynamic_typing::{UtxoData, DynamicTypingError}, ensure, traits::Cash,
    types::Output, SimpleConstraintChecker, Verifier,
};
use sp_runtime::transaction_validity::TransactionPriority;

use crate::money::Coin;

/// An order in the book. Represents a binding collateralized
/// offer to make a trade.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub struct Order<V: Verifier, A: Cash + UtxoData, B: Cash + UtxoData> {
    /// The amount of token A I'm willing to trade away
    offer_amount: u128,
    /// The amount of token B I demand to receive
    ask_amount: u128,
    /// The verifier (maybe a signature check) that will protect the payout coin
    payout_verifier: V,
    _ph_data: PhantomData<(A, B)>,
}

// NOTE: We know this is error prone
// It's gonna get better. TODO @ viewers make a PR :)
impl<V: Verifier, A: Cash, B: Cash> UtxoData for Order<V, A, B> {
    const TYPE_ID: [u8; 4] = [b'o', b'r', A::ID, B::ID];
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
    /// User is trying to make an order but didnt provide an order
    NoOrderProvided,
    /// Uer provided too many outputs when making an order (expected exactly one)
    TooManyOutputs,
}

impl From<DynamicTypingError> for DexError {
    fn from(value: DynamicTypingError) -> Self {
        Self::TypeError
    }
}

/// Place a new order in the book
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct MakeOrder<V: Verifier, A: Cash, B: Cash>(PhantomData<(V, A, B)>);

impl<V: Verifier, A: Cash, B: Cash> Clone for MakeOrder<V, A, B> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V: Verifier, A: Cash, B: Cash> Debug for MakeOrder<V, A, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("MakeOrder").field(&self.0).finish()
    }
}

impl<V: Verifier, A: Cash, B: Cash> SimpleConstraintChecker for MakeOrder<V, A, B> {
    type Error = DexError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // There should be one output which is the new order
        ensure!(!input_data.is_empty(), DexError::NoOrderProvided);
        ensure!(input_data.len() == 1, DexError::TooManyOutputs);

        // TODO fix the generic
        let order = input_data[0].extract::<Order<V, A, B>>()?;

        // There could be many inputs whose value sums to the offer amount
        let mut total_collateral = 0u128;
        for input in input_data {
            let coin = input.extract::<A>()?;
            total_collateral += coin.value();
        }

        ensure!(total_collateral == order.offer_amount, DexError::InsufficientCollateral);

        Ok(0)
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
