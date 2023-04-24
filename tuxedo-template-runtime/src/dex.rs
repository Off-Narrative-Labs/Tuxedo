//! An Order-book based dex to swap between two tokens, both of which
//! are instances of the money piece.
//!
//! For simplicity, we don't allow partial fills right now.

use core::marker::PhantomData;

use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::fmt::Debug;
use sp_std::prelude::*;
use tuxedo_core::{
    dynamic_typing::{DynamicTypingError, DynamicallyTypedData, UtxoData},
    ensure,
    traits::Cash,
    types::Output,
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
pub enum DexError {
    /// Some dynamically typed data was not of the expected type
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
/// configuration. It is also fine to take a more FRAME-like approach of
/// writing a configuration trait and taking a single generic that implements
/// that trait. In cases where a lot of configuration is required, the FRAME-like
/// approach is even preferable.
pub struct MakeOrder<V: Verifier, A: Cash, B: Cash>(PhantomData<(V, A, B)>);

impl<V: Verifier, A: Cash, B: Cash> Default for MakeOrder<V, A, B> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// Constraint checking logic for matching existing open orders against one another
pub struct MatchOrders<A: Cash, B: Cash>(PhantomData<(A, B)>);

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
impl<V: Verifier, A, B> SimpleConstraintChecker for MakeOrder<V, A, B>
where
    A: Cash + UtxoData + Encode + Decode + Debug + PartialEq + Eq + Clone,
    B: Cash + UtxoData + Encode + Decode + Debug + PartialEq + Eq + Clone,
{
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
        let order: Order<V, A, B> = output_data[0].extract()?;

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

impl<A, B> ConstraintChecker for MatchOrders<A, B>
where
    A: Cash + UtxoData + Encode + Decode + Debug + PartialEq + Eq + Clone,
    B: Cash + UtxoData + Encode + Decode + Debug + PartialEq + Eq + Clone,
{
    type Error = DexError;

    fn check<V: Verifier + PartialEq>(
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
                    payout.value() >= order.ask_amount,
                    DexError::PayoutDoesNotSatisfyOrder
                );

                // ensure that the payout was given to the right owner
                ensure!(
                    order.payout_verifier == output.verifier,
                    DexError::VerifierMismatchForTrade
                )
            } else if let Ok(order) = input.payload.extract::<Order<V, B, A>>() {
                b_so_far += order.offer_amount;
                total_a_required += order.ask_amount;

                // Ensure the payout is the right amount
                let payout = output.payload.extract::<A>()?;
                ensure!(
                    payout.value() >= order.ask_amount,
                    DexError::PayoutDoesNotSatisfyOrder
                );

                // ensure that the payout was given to the right owner
                ensure!(
                    order.payout_verifier == output.verifier,
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
    use tuxedo_core::verifier::TestVerifier;

    type MakeTestOrder = MakeOrder<TestVerifier, Coin<0>, Coin<1>>;
    type MatchTestOrders = MatchOrders<Coin<0>, Coin<1>>;

    fn a_for_b_order(
        offer_amount: u128,
        ask_amount: u128,
    ) -> Order<TestVerifier, Coin<0>, Coin<1>> {
        Order {
            offer_amount,
            ask_amount,
            payout_verifier: TestVerifier { verifies: true },
            _ph_data: Default::default(),
        }
    }

    fn b_for_a_order(
        offer_amount: u128,
        ask_amount: u128,
    ) -> Order<TestVerifier, Coin<1>, Coin<0>> {
        Order {
            offer_amount,
            ask_amount,
            payout_verifier: TestVerifier { verifies: true },
            _ph_data: Default::default(),
        }
    }

    fn output_from<T: Into<DynamicallyTypedData>>(payload: T) -> Output<TestVerifier> {
        Output {
            payload: payload.into(),
            verifier: TestVerifier { verifies: true },
        }
    }

    #[test]
    fn opening_an_order_seeking_a_works() {
        let order = a_for_b_order(100, 150);
        let input = Coin::<0>(100);

        let result = <MakeTestOrder as SimpleConstraintChecker>::check(
            &Default::default(),
            &vec![input.into()],
            &vec![order.into()],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn opening_order_with_no_inputs_fails() {
        let order = a_for_b_order(100, 150);

        let result = <MakeTestOrder as SimpleConstraintChecker>::check(
            &Default::default(),
            &vec![],
            &vec![order.into()],
        );
        assert_eq!(result, Err(DexError::NotEnoughCollateralToOpenOrder));
    }

    #[test]
    fn opening_order_with_no_outputs_fails() {
        let input = Coin::<0>(100);

        let result = <MakeTestOrder as SimpleConstraintChecker>::check(
            &Default::default(),
            &vec![input.into()],
            &vec![],
        );
        assert_eq!(result, Err(DexError::OrderMissing));
    }

    #[test]
    fn opening_order_with_insufficient_collateral_fails() {
        // Collateral is only worth 50, but order is for 100
        let input = Coin::<0>(50);
        let order = a_for_b_order(100, 150);

        let result = <MakeTestOrder as SimpleConstraintChecker>::check(
            &Default::default(),
            &vec![input.into()],
            &vec![order.into()],
        );

        assert_eq!(result, Err(DexError::NotEnoughCollateralToOpenOrder));
    }

    #[test]
    fn opening_order_with_collateral_in_wrong_asset() {
        // Collateral is in Token B but order offered token A
        let input = Coin::<1>(100);
        let order = a_for_b_order(100, 150);

        let result = <MakeTestOrder as SimpleConstraintChecker>::check(
            &Default::default(),
            &vec![input.into()],
            &vec![order.into()],
        );

        assert_eq!(result, Err(DexError::TypeError));
    }

    #[test]
    fn matching_two_orders_together_works() {
        let order_a = a_for_b_order(100, 150);
        let order_b = b_for_a_order(150, 100);

        let payout_a = Coin::<1>(150);
        let payout_b = Coin::<0>(100);

        let result = <MatchTestOrders as ConstraintChecker>::check(
            &MatchOrders(PhantomData),
            &vec![output_from(order_a), output_from(order_b)],
            &vec![output_from(payout_a), output_from(payout_b)],
        );
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn bad_match_insufficient_payout() {
        let order_a = a_for_b_order(100, 150);
        let order_b = b_for_a_order(100, 100);

        // Order a was asking for 150B. But the payout is for only 100B.
        let payout_a = Coin::<1>(100);
        let payout_b = Coin::<0>(100);

        let result = <MatchTestOrders as ConstraintChecker>::check(
            &MatchOrders(PhantomData),
            &vec![output_from(order_a), output_from(order_b)],
            &vec![output_from(payout_a), output_from(payout_b)],
        );
        assert_eq!(result, Err(DexError::PayoutDoesNotSatisfyOrder));
    }

    #[test]
    fn bad_match_payout_in_wrong_asset() {
        let order_a = a_for_b_order(100, 150);
        let order_b = b_for_a_order(100, 100);

        // Order a was asking for 150B. But the payout is for 100A.
        let payout_a = Coin::<0>(100);
        let payout_b = Coin::<0>(100);

        let result = <MatchTestOrders as ConstraintChecker>::check(
            &MatchOrders(PhantomData),
            &vec![output_from(order_a), output_from(order_b)],
            &vec![output_from(payout_a), output_from(payout_b)],
        );
        assert_eq!(result, Err(DexError::TypeError));
    }

    #[test]
    fn bad_match_not_enough_a() {
        let order_a = a_for_b_order(90, 150);
        let order_b = b_for_a_order(150, 100);

        let payout_a = Coin::<1>(150);
        let payout_b = Coin::<0>(100);

        let result = <MatchTestOrders as ConstraintChecker>::check(
            &MatchOrders(PhantomData),
            &vec![output_from(order_a), output_from(order_b)],
            &vec![output_from(payout_a), output_from(payout_b)],
        );
        assert_eq!(result, Err(DexError::InsufficientTokenAForMatch));
    }

    #[test]
    fn bad_match_not_enough_b() {
        let order_a = a_for_b_order(100, 150);
        let order_b = b_for_a_order(100, 100);

        let payout_a = Coin::<1>(150);
        let payout_b = Coin::<0>(100);

        let result = <MatchTestOrders as ConstraintChecker>::check(
            &MatchOrders(PhantomData),
            &vec![output_from(order_a), output_from(order_b)],
            &vec![output_from(payout_a), output_from(payout_b)],
        );
        assert_eq!(result, Err(DexError::InsufficientTokenBForMatch));
    }

    #[test]
    fn match_with_different_numbers_of_payouts_and_orders() {
        let order_a = a_for_b_order(90, 150);
        let order_b = b_for_a_order(150, 100);

        let payout_a = Coin::<1>(150);

        let result = <MatchTestOrders as ConstraintChecker>::check(
            &MatchOrders(PhantomData),
            &vec![output_from(order_a), output_from(order_b)],
            &vec![output_from(payout_a)],
        );
        assert_eq!(result, Err(DexError::OrderAndPayoutCountDiffer));
    }

    #[test]
    fn wrong_verifier_on_match_payout() {
        let order_a = a_for_b_order(90, 150);
        let order_b = b_for_a_order(150, 100);

        let payout_a = Coin::<1>(150);
        let payout_b = Coin::<0>(100);

        // We don't use the helper function to construct the full output
        // because we want to make sure the verifier does NOT match
        let payout_b_output = Output {
            payload: payout_b.into(),
            verifier: TestVerifier { verifies: false },
        };

        let result = <MatchTestOrders as ConstraintChecker>::check(
            &MatchOrders(PhantomData),
            &vec![output_from(order_a), output_from(order_b)],
            &vec![output_from(payout_a), payout_b_output],
        );
        assert_eq!(result, Err(DexError::VerifierMismatchForTrade));
    }
}
