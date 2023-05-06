# Generic Parameters and `DexConfig`

At this point, we have a working constraint checker that allows users to open orders.
But there is one significant limitation still.
So far our types are not generic over the token types that the orders are made in.
So far every order is offering `Coin<0>` and there is no way to specify what coin you want in exchange.
We have learned a lot with this toy example so far, but it is now time to make this dex much more realistic by giving it a configuration trait to contain all of the generic configuration information.
We need these token types to both represent fungible assets that can be stored in the Utxo set, so we bound them with the [`Cash` trait](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/traits/trait.Cash.html) and the [`UtxoData` trait](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/dynamic_typing/trait.UtxoData.html) which guarantee these properties.

```rust
/// A Configuration for a Decentralized Exchange.
pub trait DexConfig {
    /// The type of verifiers that can be used in dex payouts.
    /// Typically this should just be the outer verifier type of the runtime.
    type Verifier: Verifier + PartialEq;
    /// The first token in the Dex's pair
    type A: Cash + UtxoData;
    /// The second token in the Dex's pair
    type B: Cash + UtxoData;
}
```

Our configuration trait contains the Verifier which was previously a stand-alone generic param.
It also has two tokens added.
These two tokens represent the trading pair that this dex is trading.

We will need to make our order type generic over this entire config rather than just the verifier.
And because we won't be using the two token types for any actual fields, we will need to use Rust's [`PhantomData` marker](https://doc.rust-lang.org/std/marker/struct.PhantomData.html).

```rust
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
/// An order in the order book represents a binding collateralized
/// offer to make a trade.
///
/// The user who opens this order must put up a corresponding amount of
/// token A. This order can be matched with other orders so long as
/// the ask amount of token B may be paid to this user.
///
/// When a match is made, the payment token will be protected with the
/// verifier contained in this order.
pub struct Order<T: DexConfig> {
    /// The amount of token A in this order
    pub offer_amount: u128,
    /// The amount of token B in this order
    pub ask_amount: u128,
    /// The verifier that will protect the payout coin
    /// in the event of a successful match.
    pub payout_verifier: T::Verifier,
    pub _ph_data: PhantomData<T>,
}
```

The most complex part of this change, is the implementation of the `UtxoData` trait.
Remember that the four bytes must uniquely specify the type being stored.
Now that there is an entire class of `Order` types instead of a single one, we must make sure that the four bytes reflect both generics.
Luckily the `Cash` trait provides a one-byte ID that we can use.

```rust
impl<T: DexConfig> UtxoData for Order<T> {
    const TYPE_ID: [u8; 4] = [b'$', b'$', T::A::ID, T::B::ID];
}
```

> The Tuxedo team understands that this trait is error-prone.
> We are working to make this more seamless and less error-prone.
> Please bear with us while we pioneer UTXOs on Substrate.
> This will be better soon(tm)

With our `Order` type properly generalized, we are now ready to generalize our `MakeOrder` constraint checker.

```rust
pub struct MakeOrder<T: DexConfig>(pub PhantomData<T>);
```

Of course you will need to update the generics on your implementation of the `SimpleConstraintChecker` trait as well.
We will not give you the exact code, just know that there are three changes necessary:
* the `impl` line will need new generics and trait bounds.
* the line where you extract the order will need a new type annotation.
* the line where you extract the collateral will need a similar
 change.
If you get completely stuck, remember that the `dex-solutions` branch shows potential solutions.

Finally, now that we have loosely coupled to our tokens through the `Cash` trait, we can move the dependency on the money piece in our dex's `Cargo.toml` file to the `[dev-dependencies]` section.

When you believe you have completed this section, run `cargo test --test dex_config`.

> After completing this section, many of the previous sections' test suites will no longer pass.
> This is expected.
