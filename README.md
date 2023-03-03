# Tuxedo

Write UTXO-based Substrate Runtimes

## Table of Contents

* [Architecture](#architecture)
* [Repository Contents](#repository-contents)
  * [Wallet](#wallet)
  * 
* [Funding and Roadmap](#funding-and-roadmap)
* [Building and Running  Locally](#building-and-running-locally)
* [Docker](#docker)
* [Testing and Code Quality](#testing-and-code-quality)
* [License](#license)

## Architecture

## Repository Contents

This mono-repo contains the core Tuxedo code as well as an example node built with Tuxedo and a proof-of-concept wallet to transfer tokens.

### Tuxedo Core

The reusable core of the Tuxedo framework lives in the `tuxedo-core` directory. This crate will be used by every runtime built with Tuxedo. The best way to explore this crate is by browsing its [code](./tuxedo-core/) or its [hosted rustdocs](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/index.html). It contains:

* The core datatypes for the UTXO model such as `Input`, `Output`, `OutputRef`, `Transaction`, and others.
* A standard interface for developers to access the UTXO set.
* Common transaction validation logic that all UTXO transactions need to conduct.
* A dynamic typing system to allow developers to store bespoke data types in the UTXO set in a type-safe manner, 
* Public interfaces for developers to implement while writing their own Tuxedo pieces.

### Template Runtime

There is an example runtime built with Tuxedo in the `frameless-runtime` directory. This runtime is analogous to the popular [Substrate node template runtime](https://github.com/substrate-developer-hub/substrate-node-template/tree/main/runtime), but it uses Tuxedo and the UTXO model rather than the accounts model. Developers wanting to build with Tuxedo should inspect this example runtime to get familiar with how to use Tuxedo, and then fork it to begin developing their own runtime.

### Template Node

There is an example node built with the Tuxedo template runtime. Because Tuxedo is primarily a runtime development framework, there is not much interesting or unique to Tuxedo to see in this crate. It is mostly a copy from the popular [Substrate node template](https://github.com/substrate-developer-hub/substrate-node-template/tree/main/node).

### Wallet

The repo contains a proof-of-concept wallet in the `wallet` directory. This wallet will be expanded to be a fully-featured usable cryptocurrency wallet over the next few weeks (see the [roadmap](#funding-and-roadmap) above). For now, the PoC is enough to demonstrate that transferring tokens works.

## Funding and Roadmap

Special thanks to the [Web 3 Foundation](https://web3.foundation/) for their support of Tuxedo through their [grants program](https://github.com/w3f/Grants-Program/blob/master/applications/tuxedo.md).

As part of this grant we will deliver three milestones. More details are available in the grant application.
* ✅ Core Tuxedo Functionality (complete)
* 🏗️ User wallet (in development)
* 🔜 Full Documentation and Tutorial (not yet started)

After the grant work is complete we intend to continue developing Tuxedo. The future is less clear, but our current ideas include:
* 🔮 Cumulus and Parachain support including cross-chain UTXOs
* 🔮 Zero-knowledge runtimes a-la [zero-cash](https://www.ieee-security.org/TC/SP2014/papers/Zerocash_c_DecentralizedAnonymousPaymentsfromBitcoin.pdf) and [zexe](https://ieeexplore.ieee.org/stampPDF/getPDF.jsp?tp=&arnumber=9152634&ref=)
* 🔮 UTXO-native Smart Contracts based on the pi-calculus

## Building and Running Locally

First you'll need to have a working Rust and [Substrate development environment](https://docs.substrate.io/install/). When you can build it like any other Rust project

```sh
# Clone to repository
git clone https://github.com/Off-Narrative-Labs/Tuxedo
cd tuxedo

# Build the node
cargo build --release -p node-template

# Build the wallet
cargo build --release -p tuxedo-template-wallet
```

Once you have the node and wallet built you can run a development node, and transfer some tokens.
```sh
# Check out the CLI if you want to.
# It supports all standard Substrate CLI options
./target/release/node-template --help

# Start a development node
./target/release/node-template --dev

# In a separate terminal, run the PoC wallet to transfer 10 tokens to the specified user
./target/release/tuxedo-template-wallet 10 "news slush supreme milk chapter athlete soap sausage put clutch what kitten"
```

## Docker

Developers and Curious individuals who want to quickly try out Tuxedo and its template runtime can save the setup and compile time by using docker. our CI publishes Docker images for both the example node and the PoC wallet at https://github.com/orgs/Off-Narrative-Labs/packages.

Docker is a complex software and there are many ways to pull and run images and map host ports to container ports. For those not already familiar with Docker, you may benefit from referencing the [docker documentation](https://docs.docker.com/) or [building and running locally](#building-and-running-locally) instead.

The following commands are meant as a quickstart that will work on most platforms for users who already have Docker setup.

```sh
# Run a development node with Docker
docker run --network host ghcr.io/off-narrative-labs/tuxedo:ci-build-publish-docker --dev

# In a separate terminal, run the PoC wallet to transfer 10 tokens to the specified user
docker run ghcr.io/off-narrative-labs/tuxedo-wallet:ci-build-publish-docker 10 "news slush supreme milk chapter athlete soap sausage put clutch what kitten"
```

## Testing and Code Quality

Tuxedo strives for excellent code quality which is enforced through unit tests, and [clippy linting](https://doc.rust-lang.org/stable/clippy/). Both of these are enforced in the CI and you are free to inspect. You may also run them locally.

```sh
# Run unit tests on all aspects of the project
cargo test

# Run clippy (which requires nightly)
cargo +nightly clippy
```

## License

Apache 2.0