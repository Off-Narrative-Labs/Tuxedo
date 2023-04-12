//! An Order-book based dex to swap between two hard-coded tokens A and B.
//!
//! For simplicity, we don't allow partial fills right now.

use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use tuxedo_core::{
    dynamic_typing::{DynamicTypingError, DynamicallyTypedData, UtxoData},
    ensure,
    types::Output,
    ConstraintChecker, SimpleConstraintChecker, Verifier,
};

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
    //TODO another field to hold the verifier that will protect the
    // output coin in the event of a successful match.
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
    /// This transaction has a different number of input orders than output payouts.
    /// When matching orders, the number of inputs and outputs must be equal.
    OrderAndPayoutCountDiffer,
    /// This transaction tries to match an order but provides an incorrect payout.
    PayoutDoesNotSatisfyOrder,
    /// The amount of token A supplied by the orders is not enough to match with the demand.
    InsufficientTokenAForMatch,
    /// The amount of token B supplied by the orders is not enough to match with the demand.
    InsufficientTokenBForMatch,
}

impl From<DynamicTypingError> for DexError {
    fn from(_value: DynamicTypingError) -> Self {
        Self::TypeError
    }
}

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// The Constraint checking logic for opening a new order.
struct MakeOrder;

#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// Constraint checking logic for matching existing open orders against one another
struct MatchOrders;

// The following lines brainstorm some other constraint checkers that could be added
// but currently are not implemented.
// /// Fullfil existing orders in the order book with the supplied funds.
// /// This is an atomic combination of making and order and matching it with
// /// an existing order.
// struct TakeOrders;
// /// Cancel an existing open order
// /// This is similar to taking your own order except for maybe things like fees.
// struct CancelOrder;
// /// A secondary constraint checker that allows minting tokens A and B
// /// This one is only useful for test networks. Of course it kills scarcity.
// struct DexTokenMinter;

impl SimpleConstraintChecker for MakeOrder {
    type Error = DexError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // There should be a single order as the output
        ensure!(!output_data.is_empty(), DexError::OrderMissing);
        ensure!(
            output_data.len() == 1,
            DexError::TooManyOutputsWhenMakingOrder
        );
        let DexItem::Order(order) = output_data[1].extract()? else {
            Err(DexError::TypeError)?
        };

        // There may be many inputs and they should all be tokens whose combined value
        // equals or exceeds the amount of token they need to provide for this order
        let mut total_input_amount = 0;
        for input in input_data {
            match input.extract::<DexItem>()? {
                DexItem::TokenA(amount) if order.side == SeekingTokenB => {
                    total_input_amount += amount;
                }
                DexItem::TokenB(amount) if order.side == SeekingTokenA => {
                    total_input_amount += amount;
                }
                _ => Err(DexError::WrongCollateralToOpenOrder)?,
            }
        }

        let required_input_amount = match order.side {
            SeekingTokenA => order.token_b,
            SeekingTokenB => order.token_a,
        };
        if total_input_amount < required_input_amount {
            Err(DexError::NotEnoughCollateralToOpenOrder)?
        }

        Ok(0)
    }
}

impl ConstraintChecker for MatchOrders {
    type Error = DexError;

    fn check<V: Verifier>(
        &self,
        inputs: &[Output<V>],
        outputs: &[Output<V>],
    ) -> Result<TransactionPriority, Self::Error> {
        // The input and output slices can be arbitrarily long. We
        // assume there is a 1:1 correspondence in the sorting such that
        // the first output is the coin associated with the first order etc.
        ensure!(
            inputs.len() == outputs.len(),
            DexError::OrderAndPayoutCountDiffer
        );

        let mut total_a_required = 0;
        let mut total_b_required = 0;
        let mut a_so_far = 0;
        let mut b_so_far = 0;

        for (input, output) in inputs.iter().zip(outputs) {
            let DexItem::Order(order) = input.payload.extract()? else {
                Err(DexError::TypeError)?
            };

            match order.side {
                SeekingTokenA => {
                    total_a_required += order.token_a;
                    b_so_far += order.token_b;

                    let DexItem::TokenA(payout_amount) = output.payload.extract()? else {
                        Err(DexError::TypeError)?
                    };

                    ensure!(
                        payout_amount == order.token_a,
                        DexError::PayoutDoesNotSatisfyOrder
                    );
                    // TODO ensure that the payout was given to the right owner
                }
                SeekingTokenB => {
                    a_so_far += order.token_a;
                    total_b_required += order.token_b;

                    let DexItem::TokenB(payout_amount) = output.payload.extract()? else {
                        Err(DexError::TypeError)?
                    };

                    ensure!(
                        payout_amount == order.token_b,
                        DexError::PayoutDoesNotSatisfyOrder
                    );
                    // TODO ensure that the payout was given to the right owner
                }
            }

            // TODO Allow the match maker to claim the spread as a reward.
        }

        // Make sure the amounts in the orders actually match and satisfy each other.
        ensure!(
            a_so_far >= total_a_required,
            DexError::InsufficientTokenAForMatch
        );
        ensure!(
            b_so_far >= total_b_required,
            DexError::InsufficientTokenBForMatch
        );

        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tuxedo_core::dynamic_typing::testing::Bogus;

    #[test]
    fn opening_an_order_seeking_a_works() {

    }

    #[test]
    fn opening_order_with_no_inputs_fails() {

    }

    #[test]
    fn opening_order_with_no_outputs_fails() {

    }

    #[test]
    fn opening_order_with_insufficient_collateral_fails() {

    }

    #[test]
    fn matching_two_orders_together_works() {

    }

    #[test]
    fn bad_match_not_enough_a() {

    }

    #[test]
    fn bad_match_not_enough_b() {

    }

    #[test]
    fn match_with_bad_payout() {

    }

    #[test]
    fn match_with_different_numbers_of_payouts_and_orders() {
        
    }

}