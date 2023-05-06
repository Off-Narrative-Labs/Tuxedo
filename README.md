# Tuxedo DEX Tutorial

This hands-on tutorial guides developers in writing their first UTXO-based Substrate runtime with Tuxedo, using a Decentralized Exchange as an example topic.

This repository is also the canonical template from which we encourage new projects to begin their Tuxedo runtimes.

## Use as a Tutorial

We learn best by doing!
This this tutorial guides you through the process of writing your runtime.
So clone this repo and get your editor ready to go.
You will need a working [Substrate Development Environment](https://docs.substrate.io/install/) setup.

The instructions for each step of the process are linked below.
Read the description of what you are supposed to do, and then look into the code to try to complete the task.
As you look through the code, you will find comments and `todo!()`s giving further guidance about how to complete the task.
In many places we link to the [Tuxedo Rust Docs](https://github.com/Off-Narrative-Labs/Tuxedo/) to learn more details about the types and traits you are working with.
Reading these reference docs as you progress through the process will add valuable context to the tasks you are completing.

At any time you can check whether you have completed the task by running the unit tests for the section you are on.
In many cases, the tests will not eve compile when you begin a section.
Your job is to make the tests compile _and_ pass.
When all the tests pass for the section you are working on, move on to the next section.

If you are stuck and need a hint, canonical solutions to the tutorial are published on the [`dex-solutions` branch](https://github.com/Off-Narrative-Labs/Tuxedo-Order-Book-Dex-Tutorial/tree/dex-solutions)

In this tutorial, we will implement a Tuxedo runtime with two fungible tokens and an decentralized exchange between those tokens based on an order book.
Make sure you are familiar with the [fundamentals of an order book](https://blog.atani.com/dex-orderbook-vs-liquidity-pool/) before diving in.

* [Take a Look Around](tutorial/01-look-around.md)
* [Add a Token](tutorial/02-add-a-token.md)
* [`Order` Type](tutorial/03-order-type.md)
* [Error Type](tutorial/04-error-type.md)
* [`MakeOrder` Constraint Checker](tutorial/05-make-order.md)
* [Generic Parameters and `DexConfig`](tutorial/06-dex-config.md)
* [Install `MakeOrder` in Runtime](tutorial/07-runtime-orders.md)
* [Unit Testing the Dex](tutorial/08-unit-tests.md)
* [`MatchOrders` Constraint Checker](tutorial/09-match-orders.md)
* [Additional Ideas](tutorial/10-additional-ideas.md)
## Use as a Template

In addition to acting as a tutorial, this repository is also the canonical Tuxedo Template.
If you are ready to build a new Tuxedo project, fork this repo and get started.
Using the same repository as both a tutorial and a template allows developers to continue their learning journey from the tutorial into their own project in a seamless and natural way.
It also lowers the maintenance cost of having separate template and tutorial products.

Because this is the canonical way to get started with Tuxedo, this repository also comes with some real-world niceties including CI to build and test your Rust code as well as a Dockerfile.
You are free to remove or modify any of these, but we hope they will serve as a useful starting point.

## Git Strategy

The contents of this repository are branches of the same git history as the upstream [Tuxedo repository](https://github.com/Off-Narrative-Labs/Tuxedo).
This makes it easy to merge, cherry-pick, or rebase your project as changes come into Tuxedo itself.

## License

Apache 2.0
