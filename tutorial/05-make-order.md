# `MakeOrder` Constraint Checker

We're now ready to write our first constraint checker.
We do this by creating a type, and implementing the [`SimpleConstraintChecker` trait](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/constraint_checker/trait.SimpleConstraintChecker.html) for it.

```rust
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, PartialEq, Eq, Clone, Default, Debug, TypeInfo)]
/// The Constraint checking logic for opening a new order.
///
/// It is generic over the verifier type which can be used to protect
/// matched outputs. Typically this should be set to the runtime's
/// outer verifier type. By the end of the tutorial, it will also be
/// generic over the two coins that will trade in this order book.
/// But to begin, we will keep it simple.
pub struct MakeOrder<V: Verifier>(pub PhantomData<V>);
```

Let us recall how a UTXO transaction works.
The user supplies a set of input UTXOs from the existing state that will be consumed, as well as a set of output UTXOs that will go into the new state.
The job of the constraint checker is to make sure that the supplied output state is valid given the supplied input state.
Read more about this in the [`Transaction` type](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/types/struct.Transaction.html)'s documentation.

In the case of opening new dex orders, we need to make sure that all of the following are true:
* The user has specified a single output that is an `Order`.
* All the inputs are coins of type `Coin<1>`.
* The user has provided exactly enough input collateral to cover the `offer_amount`.

```rust
impl<V: Verifier> SimpleConstraintChecker for MakeOrder<V> {
    type Error = compile_error!("TODO use our dex error enum here");

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // There should be a single order as the output.
        // Make sure this is the case. If it is not, give an
        // appropriate error.
        ensure!(todo!(), DexError::OrderMissing);
        ensure!(todo!(), DexError::TooManyOutputsWhenMakingOrder);

        // Now that we know there is a single output, we can
        // try to extract it to the proper type. If the input
        // is not an `Order` the extraction will fail.
        let order: Order<V> = output_data[0].extract()?;

        // There may be many inputs and they should all be tokens whose combined value
        // equals or exceeds the amount of token they need to provide for this order
        let mut total_collateral = 0;
        for input in input_data {
            // TODO here you need to extract each input to the expected type (`Coin<0>`)
            // Look at how we did this above for the order output for inspiration.
            let coin: money::Coin::<0> = todo!();
            total_collateral += coin.value();
        }

        // Now that we know the total amount of input collateral, we
        // need to make sure it is enough to cover the `offer_amount`
        ensure!(todo!(), todo!());

        // All constraints have passed their checks, so this transaction is valid.
        // We return a priority of 0 for now. There is an exercise afterwards where
        // you can change this to make it more realistic.
        Ok(0)
    }
}
```

When you believe you have completed this section, run `cargo test --test make_order`.
