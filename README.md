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
When all the tests pass for the section you are working on, move on to the next section

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
There are three main directories.

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

### `Order` Type

Create an Order type and imple `UtxoData` for it.

### Make an Error Type

Create the Error enum, and brainstorm some variants. Specifically the ones related to making orders.

### `MakeOrder` Type

Create a MakeORder type impl `SimpleConstraintChecker` for it.

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

## License

Apache 2.0
