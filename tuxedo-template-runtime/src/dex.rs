//! An Order-book based dex to swap between two hard-coded tokens A and B.
//! 
//! For simplicity, we don't allow partial fills right now.

use tuxedo_core::{SimpleConstraintChecker, dynamic_typing::{DynamicallyTypedData, UtxoData, DynamicTypingError}, ensure};
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// All the data that this piece could care to store. Here I'm choosing to use a
/// single enum to experiment with some stronger typing.
enum DexItem {
    /// A coin of token A
    TokenA(u128),
    /// A coin of token B
    TokenB(u128),
    /// An order in the order book
    Order(Order),
}

impl UtxoData for DexItem {
    const TYPE_ID: [u8; 4] = *b"$dex";
}

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// Which side of a trade the order maker is on
enum Side {
    /// The order maker wants to obtain more or token A (by selling some of token B)
    SeekingTokenA,
    /// The order maker wants to obtain more of token B (by selling some of token A)
    SeekingTokenB,
}
use Side::*;

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// An order in the book consists of amounts of each token, A and B, as well as which side
/// of the trade the order maker is on.
struct Order {
    /// The amount of token A in this order
    token_a: u128,
    /// The amount of token B in this order
    token_b: u128,
    /// Which side of the trade this order maker is on
    side: Side,
}

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// All the things that can go wrong while checking constraints on dex transactions
enum DexError {
    /// Some dynamically typed data was not of the expected the expected type
    TypeError,
    /// No outputs were supplied when making an order.
    /// When making an order, exactly one output should be supplied, which is the order.
    OrderMissing,
    /// More than one output was supplied.
    /// When making an order, exactly one output should be supplied, which is the order.
    TooManyOutputsWhenMakingOrder,
    /// Transactions that open orders should only take inputs of the token needed to back
    /// the order.
    WrongCollateralToOpenOrder,
    /// The coins provided do not have enough combined value to back the order that you attempted to open.
    NotEnoughCollateralToOpenOrder,
}

impl From<DynamicTypingError> for DexError {
    fn from(value: DynamicTypingError) -> Self {
        Self::TypeError
    }
}

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// The Constraint checking logic for the Dex. We are taking the approach of a single
/// constraint checker with multiple variants.
enum DexConstraintChecker {
    /// Open a new order in the order book
    MakeOrder,
    /// Match existing open orders against one another
    MatchOrders,
    /// Fullfil existing orders in the order book with the supplied funds.
    /// This is an atomic combination of making and order and matching it with
    /// an existing order.
    TakeOrders,
    /// Cancel an existing open order
    /// This is similar to taking your own order except for maybe things like fees.
    CancelOrder,
}

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// A secondary constraint checker that allows minting tokens A and B
/// This one is only useful for test networks. Of course it kills scarcity.
struct DexTokenMinter;
//TODO impl methods on this guy

impl SimpleConstraintChecker for DexConstraintChecker {
    type Error = DexError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        match self {
            DexConstraintChecker::MakeOrder => {
                // There should be a single order as the output
                ensure!(!output_data.is_empty(), DexError::OrderMissing);
                ensure!(output_data.len() == 1, DexError::TooManyOutputsWhenMakingOrder);
                let DexItem::Order(order) = output_data[1].extract()? else {
                    DexError::TypeError?
                };

                // There may be many inputs and they should all be tokens whose combined value
                // equals or exceeds the amount of token they need to provide for this order
                let mut total_input_amount = 0;
                for input in input_data {
                    match input.extract::<DexItem>()? {
                        DexItem::TokenA(amount) if order.side == SeekingTokenB => {
                            total_input_amount += amount;
                        },
                        DexItem::TokenB(amount) if order.side == SeekingTokenA => {
                            total_input_amount += amount;
                        },
                        _ => DexError::WrongCollateralToOpenOrder?,
                    }
                }

                let required_input_amount = match order.side {
                    SeekingTokenA => order.token_b,
                    SeekingTokenB => order.token_a,
                };
                if total_input_amount < required_input_amount {
                    DexError::NotEnoughCollateralToOpenOrder?
                }

                Ok(0)
            },
            DexConstraintChecker::MatchOrders => todo!(),
            DexConstraintChecker::TakeOrders => todo!(),
            DexConstraintChecker::CancelOrder => todo!(),
        }
    }
}

// impl DexConstraintChecker {
//     fn make_order
// }
