# Tuxedo DEX Tutorial

This hands-on tutorial will guide Runtime developers through writing their first UTXO-based runtime with Tuxedo

We will implement a runtime with two fungible tokens and an decentralized exchange between those tokens based on an order book.
To get familiar with the concepts of an order book se [TODO link from slides]().

## How to Use this Tutorial

We learn best by doing!
This this tutorial guides you through the process of writing your runtime.
So clone this repo and get your editor ready to go.
You will need a working [Substrate Development Environment](todo) setup.

The instructions for each step of the process are below in this readme.
Read the description of what you are supposed to do, and then look into the code to try to complete the task.
As you look through the code, you will find comments and `todo!()`s giving further guidance about how to complete the task.
In many places we link to the [Tuxedo Rust Docs]() to learn more details about the types and traits you are working with.
Reading these reference docs as you progress through the process will add valuable context to the tasks you are completing.

At any time you can check whether you have completed the task by running the unit tests.
This entire tutorial is tested using the [trybuild]() crate.
Thanks to this crate, when a test case would fail to compile against your code, this is treated as a test failure.
When all the tests pass for the section you are working on, move on to the next section

## Tutorial Contents

### Add a Token

To begin with, let's get oriented in this repository.
There are two main directories.
In the `node` directory, there is a nearly-unmodified copy of the Substrate node template.
We will use this standard Substrate client to run our node when we are interested in doing so.
The main focus of our work will be the wasm runtime in the `tuxedo-template-runtime` directory.

This file has only a single file, so take a look.
If you have ever seen a FRAME runtime, a lot of the contents will look familiar.


TODO brief description of each here.

* RuntimeVersion
* Opaque
* Concrete Types
* OuterVerifier
* Piece Configs
* OuterConstraintChecker
* Helpers
* Runtime API Impls

Our work in this tutorial will mainly touch the Piece Configs and the OuterConstraintChecker.
This template runtime includes two pieces currently.
The first is a fungible token (token id 1) provided by the [Money Piece]().
Second is the ability to perform wasm runtime upgrades using Substrate's forkless upgrade mechanism.
These pieces are included because nearly all runtimes will need them.
As a runtime developer, you can remove either of these pieces, or add more from the [wardrobe] or that you have written yourself.
In this tutorial we will write a new piece that represents our order book dex, but before we get to all that, we will need a second token so that we can trade between the two on the dex.

Your task for this part is to extend the `OuterConstraintChecker` to have a third variant called `SecondToken` that uses token id 1.

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
