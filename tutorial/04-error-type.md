# Make an Error Type

There are many things that can go wrong when trying to validate dex transactions.
Errors can happen both when opening new orders, and when matching orders together.

Our Piece needs a type (usually an enum) that captures the different error cases.
Let's create this type and brainstorm some error variants.
For now I'll focus on things that can go wrong at the stage of making orders.
But if ideas about matching come to your mind, feel free to write them down too.

* `TypeError` - Some data that came out of storage was not the expected type.
  For example, this would happen if a user were supposed to put up collateral of 10 coins, but instead supplied an input containing a cryptokitty.
  Almost every Tuxedo piece will have an error variant like this.
* `OrderMissing` - No outputs were supplied when making an order.
  When making an order, exactly one output should be supplied, which is the order.
* `TooManyOutputsWhenMakingOrder` - This is the opposite of the previous error.
  Now the order maker is trying to open an order, but supplied two or more outputs.
  It would also be reasonable to combine this output with the previous one, but I prefer to give a more specific error when no output at all exists.
* `NotEnoughCollateralToOpenOrder` - The coins provided do not have enough combined value to back the order that you attempted to open.

Your task for this part is to create an enum with variants for each of these errors.
For the purpose of this tutorial, you have to use the same names I did to get the tests to pass.
In your own code, you should name the variants as you see best fit.

```rust
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// All the things that can go wrong while checking constraints on dex transactions
pub enum DexError {
    /// Some dynamically typed data was not of the expected type
    TypeError,

    /// Your task is to fill in all the variants we brainstormed for the order opening
    /// portion of the dex logic. The first one, `TypeError` is done for you. You need
    /// to fill in the rest. And remember to have clear doc comments.
    TodoAddMoreVariants,
}
```

Our `DexError` has a variant for anytime an input or output contained the wrong type of UTXO.
We will convert all [`DynamicTypingError`]()s we encounter to this same variant.
For that it will be convenient to have a conversion implemented.

```rust
impl From<DynamicTypingError> for DexError {
    fn from(_value: DynamicTypingError) -> Self {
        todo!()
    }
}
```

When you believe you have completed this section, run `cargo test --test error_enum`.