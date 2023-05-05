# Tuxedo DEX Tutorial

This hands-on tutorial guides developers in writing their first UTXO-based Substrate runtime with Tuxedo, using a Decentralized Exchange as an example topic.

This repository is also the canonical template from which we encourage new projects to begin their Tuxedo runtimes.

## Use as a Tutorial

We learn best by doing!
This this tutorial guides you through the process of writing your runtime.
So clone this repo and get your editor ready to go.
You will need a working [Substrate Development Environment](todo) setup.

The instructions for each step of the process are below in this readme.
Read the description of what you are supposed to do, and then look into the code to try to complete the task.
As you look through the code, you will find comments and `todo!()`s giving further guidance about how to complete the task.
In many places we link to the [Tuxedo Rust Docs]() to learn more details about the types and traits you are working with.
Reading these reference docs as you progress through the process will add valuable context to the tasks you are completing.

At any time you can check whether you have completed the task by running the unit tests for the section you are on.
In many cases, the tests will not eve compile when you begin a section.
Your job is to make the tests compile _and_ pass.
When all the tests pass for the section you are working on, move on to the next section.

If you are stuck and need a hint, canonical solutions to the tutorial are published on the [`dex-solutions` branch](https://github.com/Off-Narrative-Labs/Tuxedo-Order-Book-Dex-Tutorial/tree/dex-solutions)

## Use as a Template

In addition to acting as a tutorial, this repository is also the canonical Tuxedo Template.
If you are ready to build a new Tuxedo project, fork this repo and get started.
Using the same repository as both a tutorial and a template allows developers to continue their learning journey from the tutorial into their own project in a seamless and natural way.
It also lowers the maintenance cost of having separate template and tutorial products.

Because this is the canonical way to get started with Tuxedo, this repository also comes with some real-world niceties including CI to build and test your Rust code as well as a Dockerfile.
You are free to remove or modify any of these, but we hope they will serve as a useful starting point.

## Git Strategy

The contents of this repository are branches of the same git history as the [upstream Tuxedo repository]().
This makes it easy to merge, cherry-pick, or rebase your project as changes come into Tuxedo itself.

## Tutorial Contents

We will implement a runtime with two fungible tokens and an decentralized exchange between those tokens based on an order book.
To get familiar with the concepts of an order book see [TODO link from slides]().

### Take a Look Around

To begin with, let's get oriented in this repository.
You should kick off an initial compilation right now so that the code has time to compile while we are looking through it.
To compile the entire project run `cargo build`.

While, that compiles, let's have a look around.

#### The `node` Directory

In the `node` directory, there is a nearly-unmodified copy of the Substrate node template.
We will use this standard Substrate client to run our node when we are interested in doing so.

#### The Runtime Directory
Our work will begin in the `tuxedo-template-runtime` directory.
This folder has only a single file,`lib.rs`, so take a look.
If you have ever seen a FRAME runtime, a lot of the contents will be familiar.


Let's skim this entire file to get a sense of its structure.

##### Imports

We import items from Substrate, Tuxedo Core, and two Tuxedo pieces (Money and Runtime Upgrade).

##### The Opaque Module

The Opaque module houses definitions of types that will be used on the client side.
This includes a few consensus keys types as well as our `OpaqueBlock` type which is based on Substrates [`OpaqueExtrinsic`]() type.

The idea behind opaque types is that they are all basically `Vec<u8>` under the hood.
This allows the client side to just see them as bytes and have no understanding of their structure or meaning.
Only when these types enter the runtime will they be decoded into Rust types.

##### Runtime Version

All Runtimes must provide an implementation of Substrate's [RuntimeVersion](). This looks nearly identical to a FRAME runtime.

##### Genesis Config

Each runtime needs to provide an implementation of `BuildStorage` to allow a chain to begin with some initial, or "genesis" state.

This runtime begins with two coins in its storage.
One is owned by a private key, and the other by a multisignature.
We will be able to mint our own tokens when working with the chain, so we don't need to worry much about the genesis tokens.

##### Concrete Types

Next we encounter several type aliases including, `Transaction`, `Header`, `Block`, `Executive`, and `Output`.
Each of these aliases takes a generic type from either Substrate or Tuxedo core, and fills in the generics to provide fully-concrete runtime-specific types.

##### Outer Verifier

Next we see the `OuterVerifier` enum.
Every Tuxedo runtime will have one of these Verifiers and it is required to implement the [`Verifier` trait]().

This enum is an amalgamation of several other individual verifiers.
In this case, we use the three that come standard with Tuxedo:

* [`UpForGrabs`]() - Allows anyone to spend a UTXO.
* [`SigCheck`]() - Allows a UTXO to be spent with a signature from the proper key. This is the most common and represents simple private ownership.
* [`ThresholdMultiSignature`]() - Allows a UTXO to be spent when enough members have signed.

Tuxedo developers can extend this enum by writing their own implementations of the `Verifier` trait.
However, we will not need to extend this while writing our dex.

##### Piece Configs

The next section of the runtime is typically piece configurations.
Currently our runtime is sufficiently simple that neither of our two pieces need any configuration.
We will need to add a configuration for our dex later on, and you can see a comment showing where that will happen.

##### Outer Constraint Checker

The `OuterConstraintChecker` enum is also an amalgamation enum that implements the [`ConstraintChecker`]() trait.
That means that each of its variants represents a separate constraint checker.

This enum represents all of the different transaction types in the runtime.
This template runtime includes two pieces currently.
The first is a fungible token (token id 1) provided by the [Money Piece]().
Second is the ability to perform wasm runtime upgrades using Substrate's forkless upgrade mechanism.
These pieces are included because nearly all runtimes will need them.
As a runtime developer, you can remove either of these pieces, or add more from the [wardrobe] or that you have written yourself.

We will add to it this enum several times throughout the tutorial.

##### `Runtime` and Helper Functions

We declare the `Runtime` struct.
And on it we implement a few consensus-related helper functions.
If you find that you need a helper function in the runtime, you can consider implementing it on the `Runtime` struct.

##### Runtime API Implementations

Finally, we see Substrate's runtime APIs implemented.
Implementing these api traits is what makes this program useable as a Substrate runtime.
In this sense they are the most important part.

However, most of the functions implementations are short and call into the appropriate helper in Tuxedo core.
So in this sense, they are least interesting and will rarely need to be modified.

#### The `dex` Directory
This brings us to the last of the directories: the `dex` directory.
In this tutorial we will write a new Tuxedo piece that represents an order book dex.
It's code will live in this `dex` directory, and we will spend most of our time there.

### Add a Token

Before we get to the dex logic itself, we will need some tokens to exchange, and this runtime only has one token at the moment.
Your task for this part is to extend the runtime's `OuterConstraintChecker` to have a third variant called `SecondToken` that uses token id 1.

When you believe you have completed this section, run `cargo test --test add_token`.

### `Order` Type

Now we turn our attention to the skeleton of a Dex piece in the `dex` directory.

Our Order Book will need to store orders on chain in UTXOs until they are matched together.
That means we will need an `Order` type.
This type is responsible for storing the order's ask amount and offer amount.

An Order is also responsible for storing something called the `payout_verifier`.
When some orders are matched together, the corresponding payouts must be made.
The payouts will be protected by this `payout_verifier`.
So when a user makes an order, they provide the verifier that will be used to protect their eventual payout, should the order ever match.

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

Now that we have declared our type, we need to indicate that this type can be stored in a UTXO by implementing the [`UtxoData` trait]().
We just need to be sure we provide a unique four-byte type identifier.

```rust
impl<V: Verifier> UtxoData for Order<V> {
    const TYPE_ID: [u8; 4] = *b"ordr";
}
```

When you believe you have completed this section, run `cargo test --test order_type`.

### Make an Error Type

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

### `MakeOrder` Type

We're now ready to write our first constraint checker.
We do this by creating a type, and implementing the [`ConstraintChecker` trait]() for it.

```rust
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, PartialEq, Eq, TypeInfo)]
/// The Constraint checking logic for opening a new order.
///
/// It is generic over the verifier type which can be used to protect
/// matched outputs. Typically this should be set to the runtime's
/// outer verifier type. By the end of the tutorial, it will also be
/// generic over the two coins that will trade in this order book.
/// But to begin, we will keep it simple.
pub struct MakeOrder<V: Verifier>(PhantomData<V>);
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
impl<T: Verifier> SimpleConstraintChecker for MakeOrder<V> {
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
        let order: Order<T> = output_data[0].extract()?;

        // There may be many inputs and they should all be tokens whose combined value
        // equals or exceeds the amount of token they need to provide for this order
        let mut total_input_amount = 0;
        for input in input_data {
            // TODO here you need to extract each input to the expected type (`Coin<0>`)
            // Look at how we did this above for the order output for inspiration.
            let coin: money::Coin::<0> = todo!();
            total_input_amount += coin.value();
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

When you believe you have completed this section, run `cargo test --test make_order

### Add it to Runtime

todo

### `MatchOrders` Type

Make it and impl `ConstraintChecker`. Explain difference between two checking traits. Add error variants. Add it to runtime.

### Generic Coin Parameters

Make order generic over two coins that implement the `Cash` trait.

### Tests for Make Order

### Tests for Match Order


## Additional Ideas

You may further your learning by continuing to extend and adapt this Tuxedo piece.
At this point we will no longer guide you so directly.
Here are some ideas:

* CancelOrder
* PartialMatching
* Orders featuring arbitrarily many tokens in various amounts.
* Proper prioritization
* Give change when opening orders - One additional output to represent funds coming back to the order maker.

## License

Apache 2.0
