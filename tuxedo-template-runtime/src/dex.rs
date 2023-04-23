//! An Order-book based dex to swap between two tokens, both of which
//! are instances of the money piece.
//!
//! For simplicity, we don't allow partial fills right now.

use core::marker::PhantomData;

use crate::money::Cash;
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::prelude::*;
use tuxedo_core::{
    dynamic_typing::{DynamicTypingError, DynamicallyTypedData, UtxoData},
    ensure,
    types::Output,
    verifier::SigCheck,
    ConstraintChecker, SimpleConstraintChecker, Verifier,
};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// An order in the order book represents a binding collateralized
/// offer to make a trade.
///
/// The user who opens this order must put up a corresponding amount of
/// token A. This order can be matched with other orders so long as
/// the ask amount of token B may be paid to this user.
///
/// When a match is made, the payment token will be protected with the
/// verifier contained in this order.
struct Order<V: Verifier, A: Cash, B: Cash> {
    /// The amount of token A in this order
    offer_amount: u128,
    /// The amount of token B in this order
    ask_amount: u128,
    /// The verifier that will protect the payout coin
    /// in the event of a successful match.
    payout_verifier: V,
    _ph_data: PhantomData<(A, B)>,
}

impl<V: Verifier, A: Cash, B: Cash> UtxoData for Order<V, A, B> {
    const TYPE_ID: [u8; 4] = [b'$', b'$', A::ID, B::ID];
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
    /// The verifier who is receiving the tokens is not correct
    VerifierMismatchForTrade,
}

impl From<DynamicTypingError> for DexError {
    fn from(_value: DynamicTypingError) -> Self {
        Self::TypeError
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// The Constraint checking logic for opening a new order.
///
/// It is generic over the verifier type which can be used to protect
/// matched outputs. Typically this should be set to the runtime's
/// outer verifier type. It is also generic over the two coins that will
/// trade in this order book.
///
/// This constraint checker demonstrates taking configuration information
/// from the broader runtime. Here we use separate generics for each piece of
/// configuration. It is also be fine to take a more FRAME-like approach of
/// writing a configuration trait and taking a single generic that implements
/// that trait. In cases where a lot of configuration is required, the FRAME-like
/// approach is even preferable.
struct MakeOrder<V: Verifier, A: Cash, B: Cash>(PhantomData<(V, A, B)>);

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// Constraint checking logic for matching existing open orders against one another
struct MatchOrders<V: Verifier, A: Cash, B: Cash>(PhantomData<(V, A, B)>);

// The following lines brainstorm some other constraint checkers that could be added
// but currently are not implemented.
// /// Fullfil existing orders in the order book with the supplied funds.
// /// This is an atomic combination of making and order and matching it with
// /// an existing order.
// #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
// struct TakeOrders;
// /// Cancel an existing open order
// /// This is similar to taking your own order except for maybe things like fees.
// #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
// struct CancelOrder;

// Here we see an example
impl<V: Verifier, A: Cash, B: Cash> SimpleConstraintChecker for MakeOrder<V, A, B> {
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
        let order: Order<V, A, B> = output_data[0].extract()? else {
            Err(DexError::TypeError)?
        };

        // There may be many inputs and they should all be tokens whose combined value
        // equals or exceeds the amount of token they need to provide for this order
        let mut total_input_amount = 0;
        for input in input_data {
            let coin = input.extract::<A>()?;
            total_input_amount += coin.value();
        }

        if total_input_amount < order.offer_amount {
            Err(DexError::NotEnoughCollateralToOpenOrder)?
        }

        Ok(0)
    }
}

impl<V: Verifier + PartialEq, A: Cash, B: Cash> ConstraintChecker for MatchOrders<V, A, B> {
    type Error = DexError;

    fn check(
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

        // Each order will add some tokens to the matching pot
        // and demand some tokens from the matching pot.
        // As we loop through the orders, we will keep track of these totals.
        // After all orders have been inspected, we will make sure the
        // amounts add up.
        let mut total_a_required = 0;
        let mut total_b_required = 0;
        let mut a_so_far = 0;
        let mut b_so_far = 0;

        // As we loop through all the orders, we:
        // 1. Make sure the output properly fills the order's ask
        // 2. Update the totals for checking at the end
        for (input, output) in inputs.iter().zip(outputs) {
            // It could be Order<V, A, B> or Order<V, B, A> so we will try both.
            if let Ok(order) = input.payload.extract::<Order<V, A, B>>() {
                a_so_far += order.offer_amount;
                total_b_required += order.ask_amount;

                // Ensure the payout is the right amount
                let payout = output.payload.extract::<B>()?;
                ensure!(
                    payout.value() == order.ask_amount,
                    DexError::PayoutDoesNotSatisfyOrder
                );

                // ensure that the payout was given to the right owner
                ensure!(
                    order.outcome_verifier == output.verifier,
                    DexError::VerifierMismatchForTrade
                )
            } else if let Ok(order) = input.payload.extract::<Order<V, B, A>>() {
                b_so_far += order.offer_amount;
                total_a_required += order.ask_amount;

                // Ensure the payout is the right amount
                let payout = output.payload.extract::<A>()?;
                ensure!(
                    payout.value() == order.ask_amount,
                    DexError::PayoutDoesNotSatisfyOrder
                );

                // ensure that the payout was given to the right owner
                ensure!(
                    order.outcome_verifier == output.verifier,
                    DexError::VerifierMismatchForTrade
                )
            } else {
                Err(DexError::TypeError)?
            };
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

        // Allow the match maker to claim the spread as a reward.
        let _a_for_matcher = a_so_far - total_a_required;
        let _b_for_matcher = b_so_far - total_b_required;
        //TODO actually pay these amounts out. This would mean there
        // are two additional outputs at the end

        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::money::Coin;
    use sp_core::H256;
    use tuxedo_core::{dynamic_typing::testing::Bogus, verifier::TestVerifier};

    type TestOrder = Order<TestVerifier, Coin<0>, Coin<1>>;

    impl TestOrder {
        pub fn default_test_order() -> Self {
            TestOrder {
                offer_amount: 100,
                ask_amount: 150,
                payout_verifier: Default::default(),
                _ph_data: Default::default(),
            }
        }
    }

    #[test]
    fn opening_an_order_seeking_a_works() {
        let order = Order::default_test_order();
        let input = DexItem::TokenA(100);
        let output = DexItem::Order(order);

        let result = <MakeOrder as SimpleConstraintChecker>::check(
            &MakeOrder,
            &vec![input.into()],
            &vec![output.into()],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn opening_order_with_no_inputs_fails() {
        let order = Order::default_test_order();
        let output = DexItem::Order(order);

        let result = <MakeOrder as SimpleConstraintChecker>::check(
            &MakeOrder,
            &vec![],
            &vec![output.into()],
        );
        assert_eq!(result, Err(DexError::NotEnoughCollateralToOpenOrder));
    }

    #[test]
    fn opening_order_with_no_outputs_fails() {
        let input = DexItem::TokenA(100);

        let result =
            <MakeOrder as SimpleConstraintChecker>::check(&MakeOrder, &vec![input.into()], &vec![]);
        assert_eq!(result, Err(DexError::OrderMissing));
    }

    #[test]
    fn opening_order_with_insufficient_collateral_fails() {}

    #[test]
    fn matching_two_orders_together_works() {
        let order_a = Order::default_test_order();
        let order_b = Order::<TestVerifier, Coin<1>, Coin<0>> {
            offer_amount: 100,
            ask_amount: 150,
            ..Default::default()
        };
        let input_a = DexItem::Order(order_a);
        let input_b = DexItem::Order(order_b);

        let input_a = Output {
            payload: input_a.into(),
            verifier: SigCheck {
                owner_pubkey: H256::zero(),
            },
        };
        let input_b = Output {
            payload: input_b.into(),
            verifier: SigCheck {
                owner_pubkey: H256::zero(),
            },
        };

        let output_a = DexItem::TokenB(150);
        let output_b = DexItem::TokenA(100);

        let output_a = Output {
            payload: output_a.into(),
            verifier: SigCheck {
                owner_pubkey: H256::zero(),
            },
        };
        let output_b = Output {
            payload: output_b.into(),
            verifier: SigCheck {
                owner_pubkey: H256::zero(),
            },
        };

        let result = <MatchOrders as ConstraintChecker>::check(
            &MatchOrders,
            &vec![input_a, input_b],
            &vec![output_a, output_b],
        );
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn bad_match_not_enough_a() {}

    #[test]
    fn bad_match_not_enough_b() {}

    #[test]
    fn match_with_bad_payout() {}

    #[test]
    fn match_with_different_numbers_of_payouts_and_orders() {}
}
