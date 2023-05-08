# `Order` Type

Now we turn our attention to the skeleton of a Dex piece in the `dex` directory.

Our Order Book will need to store orders on chain in UTXOs until they are matched together.
That means we will need an `Order` type.
This type is responsible for storing the order's ask amount and offer amount.

An Order is also responsible for storing something called the `payout_verifier`.
A [`Verifier`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/verifier/trait.Verifier.html) is a bit of logic that determines whether a UTXO can be spent or consumed.
When some orders are matched together, the corresponding payouts must be made.
The payouts will be protected by this `payout_verifier`.
So when a user makes an order, they provide the verifier that will be used to protect their eventual payout, should the order ever match.
Typically users will just make sure that their new tokens are protected by a signature from their public key.

For now, we will not worry about making our `Order` type generic over token types.
We will add that later after we have understood the fundamentals of piece design.

Paste this `Order` type into the dex module's source.

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
pub struct Order<V: Verifier> {
    /// The amount of token A in this order
    pub offer_amount: u128,
    /// The amount of token B in this order
    pub ask_amount: u128,
    /// The verifier that will protect the payout coin
    /// in the event of a successful match.
    pub payout_verifier: V,
}
```

Now that we have declared our type, we need to indicate that this type can be stored in a UTXO by implementing the [`UtxoData` trait](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/dynamic_typing/trait.UtxoData.html).
We just need to be sure we provide a unique four-byte type identifier.

```rust
impl<V: Verifier> UtxoData for Order<V> {
    const TYPE_ID: [u8; 4] = *b"ordr";
}
```

When you believe you have completed this section, run `cargo test --test order_type`.