# Install `MakeOrder` in Runtime

With a properly generalized `MakeOrder` constraint checker now written, we are ready to add the order-making logic to our runtime.
The runtime does not yet depend on the dex piece, so you will need to begin by adding the dex dependency in the runtime's `Cargo.toml` file.

With the dependency now added, we are ready to write a dex configuration.
In some cases, you may implement the `DexConfig` trait directly on your `Runtime` type.
This approach will be familiar if you are already familiar with FRAME.
However, we will opt for the more general approach of creating a dedicated config type.
Insert this code just before your `OuterConstraintChecker`:

```rust
#[derive(PartialEq, Eq, TypeInfo)]
/// A Dex Configuration for the Dex that trades tokens 0 and 1
pub struct DexConfig01;
impl dex::DexConfig for DexConfig01 {
    type Verifier = OuterVerifier;
    type A = money::Coin<0>;
    type B = money::Coin<1>;
}
```

And now you can add a line to the `OuterConstraintChecker` line similar to the one you added earlier when installing a second token.

At this point, out users can make orders offering to trade token 0 for token 1.
But they cannot open orders on the opposite side, offering to trade token 1 for token 0.
To solve this we will need to add another line to the `OuterConstraintChecker` with the opposite config.
We could choose to manually write a second config that is the opposite of the first, but we can see already that all runtimes that use our dex will face this issue.
So we will choose to write an adapter that lives with the dex and allows reversing the tokens in the config to represent the opposite side of a trade.

```rust
#[derive(PartialEq, Eq, TypeInfo)]
/// This type represents a configuration that has the tokens swapped from
/// some original configuration.
///
/// When opening orders, we want to allow orders for both sides of the trade.
/// Similarly, when matching orders we have to be sure that the matched orders are on
/// opposite sides of the same trading pair. This type allows us to conveniently
/// express "same pair, but opposite side".
pub struct OppositeSide<T: DexConfig>(PhantomData<T>);

impl<T: DexConfig> DexConfig for OppositeSide<T> {
    type Verifier = T::Verifier;
    type A = T::B;
    type B = T::A;
}
```

Now we can use this adapter to add the second `MakeOrder` constraint checker to our runtime.

When you believe you have completed this section, run `cargo test --test runtime_orders`.
