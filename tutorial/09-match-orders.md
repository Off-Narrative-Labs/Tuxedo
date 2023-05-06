# `MatchOrders` Type

As a final step that reviews most of what we have already covered, we will add another constraint checker along with its error variants.

This time we will be working on the `MatchOrders` type which will be responsible for matching multiple users orders together and fulfilling the trades.

```rust
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, PartialEq, Eq, CloneNoBound, DebugNoBound, DefaultNoBound, TypeInfo)]
/// Constraint checking logic for matching existing open orders against one another
pub struct MatchOrders<T: DexConfig>(pub PhantomData<T>);
```

The basic checking logic will be as follows:
* Ensure there is a 1:1 correspondence between input orders and output payouts
* Iterate through all the inputs tracking how much of each token is offered and how much is asked
* For both tokens ensure the total offer is greater than the total ask
* Ensure each payout goes to the corresponding `payout_verifier` and is for the right amount

Thinking through this logic, we can see that there will be several new error variants to add:
* `OrderAndPayoutCountDiffer` - This transaction has a different number of input orders than output payouts.
  When matching orders, the number of inputs and outputs must be equal.
* `PayoutDoesNotSatisfyOrder` - This transaction tries to match an order but provides an incorrect payout.
* `InsufficientTokenAForMatch` - The amount of token A supplied by the orders is not enough to match with the demand.
* `InsufficientTokenBForMatch` - The amount of token B supplied by the orders is not enough to match with the demand.
* `VerifierMismatchForTrade` - The verifier who is receiving the tokens is not correct one that was specified in the original order.

With our `DexError` enum properly extended, we are now ready to implement the [`ConstraintChecker` trait](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/constraint_checker/trait.ConstraintChecker.html).
We are using the full `ConstraintChecker` this time as opposed to the [`SimpleConstraintChecker`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/constraint_checker/trait.SimpleConstraintChecker.html) that we used when making orders because we need to ensure that outputs are guarded by specific `Verifiers`.
Specifically we need to make sure that payouts are guarded by the proper `payout_verifiers` that were specified in the order.
This is the one and only difference between the `ConstraintChecker` and `SimpleConstraintChecker`: The simple one does not give you access to information about the verifiers on the inputs or outputs.
As the name implies, any type that implements `SimpleConstraintChecker` automatically implements `ConstraintChecker`.

Here is a sketch of the implementation with several `todo!()`s left for the learner to implement.
If you get stuck, use the tests to guide you.
And if you are still stuck, consider peeking at the `dex-solutions` branch only until you are unstuck and then continue on your own.

```rust
impl<T: DexConfig> ConstraintChecker<T::Verifier> for MatchOrders<T> {
    type Error = DexError;

    fn check(
        &self,
        inputs: &[Output<T::Verifier>],
        outputs: &[Output<T::Verifier>],
    ) -> Result<TransactionPriority, Self::Error> {
        // The input and output slices can be arbitrarily long. We
        // assume there is a 1:1 correspondence in the sorting such that
        // the first output is the coin associated with the first order etc.
        ensure!(todo!(), DexError::OrderAndPayoutCountDiffer);

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
            if let Ok(order) = input.payload.extract::<Order<T>>() {
                a_so_far += todo!();
                total_b_required += todo!();

                // Ensure the payout is the right amount
                let payout = output.payload.extract::<T::B>()?;
                ensure!(
                    todo!(),
                    DexError::PayoutDoesNotSatisfyOrder
                );

                // ensure that the payout was given to the right owner
                ensure!(
                    todo!(),
                    DexError::VerifierMismatchForTrade
                )
            } else if let Ok(order) = input.payload.extract::<Order<OppositeSide<T>>>() {
                todo!("repeat similar but reversed logic for the opposite side of the order");
            } else {
                // If the order doesn't decode to either side of this pair, then it is not the
                // right type and we return the general type error.
                Err(DexError::TypeError)?
            };
        }

        // Make sure the amounts in the orders actually match and satisfy each other.
        ensure!(
            todo!(),
            DexError::InsufficientTokenAForMatch
        );
        ensure!(
            todo!(),
            DexError::InsufficientTokenBForMatch
        );

        Ok(0)
    }
}
```

Now that our `MatchOrders` constraint checker is written, we can install it in the runtime.
Unlike `MakeOrder`, we do not need to install this constraint checker twice.
Remember that matching necessarily takes orders on both sides of the trade.
So installing `MatchOrders` configured with either side of the trade is enough.
Of course installing it twice is harmless, just redundant.

When you believe you have completed this section, run `cargo test --test match_orders`.
