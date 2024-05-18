# Tuxedo

Write UTXO-based Substrate Runtimes

Browse this repository, or get hands on with the [Tuxedo Order Book Dex Tutorial](https://github.com/Off-Narrative-Labs/Tuxedo-Order-Book-Dex-Tutorial/).

## Table of Contents

- [Repository Layout](#repository-layout)
  - [Tuxedo Core](#tuxedo-core)
  - [Template Runtime](#template-runtime)
  - [Template Node](#template-node)
  - [Wallet](#wallet)
- [Building and Running Locally](#building-and-running-locally)
- [Docker](#docker)
- [Parachain](#parachain)
- [Testing and Code Quality](#testing-and-code-quality)
- [License](#license)

## Repository Layout

This repository contains the Tuxedo Core code as well as an example runtime built with Tuxedo, a simple node to execute the example runtime, and a proof-of-concept wallet to transfer tokens.
The next few sections describe each of these in a little more detail

### Tuxedo Core

The reusable core of the Tuxedo framework lives in the `tuxedo-core` directory. This crate will be used by every runtime built with Tuxedo. The best way to explore this crate is by browsing its [code](./tuxedo-core/) or its [hosted rustdocs](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/index.html). It contains:

- The core data types for the UTXO model such as `Input`, `Output`, `OutputRef`, `Transaction`, and others.
- A standard interface for developers to access the UTXO set.
- Common transaction validation logic that all UTXO transactions need to conduct.
- A dynamic typing system to allow developers to store bespoke data types in the UTXO set in a type-safe manner,
- Public interfaces for developers to implement while writing their own Tuxedo pieces.

### Template Runtime

There is an example runtime built with Tuxedo in the `tuxedo-template-runtime` directory. This runtime is analogous to the popular [Substrate node template runtime](https://github.com/substrate-developer-hub/substrate-node-template/tree/main/runtime), but it uses Tuxedo and the UTXO model rather than the accounts model. Developers wanting to build with Tuxedo should inspect this example runtime to get familiar with how to use Tuxedo, and then fork it to begin developing their own runtime.

The best way to explore this runtime is by browsing its [code](./tuxedo-template-runtime/) or its [hosted rustdocs](https://off-narrative-labs.github.io/Tuxedo/tuxedo_template_runtime/index.html)

### Template Node

There is an example node built with the Tuxedo template runtime. Because Tuxedo is primarily a runtime development framework, it is mostly a copy from the popular [Substrate node template](https://github.com/substrate-developer-hub/substrate-node-template/tree/main/node).

The main difference is that Tuxedo nodes use a custom `GenesisBlockBuilder`, introduced in [PR #127](https://github.com/Off-Narrative-Labs/Tuxedo/pull/127), to include transactions in the genesis block.

#### Database

PR [#136](https://github.com/Off-Narrative-Labs/Tuxedo/pull/136) set ParityDB as the default database instead of RocksDB.
This choice is unopinionated, it was made simply because RocksDB takes longer time to compile and adds unnecessary dependencies for our current use case.

Developers are free to use RocksDB instead by building the node with the feature flag `rocksdb`.

### Wallet

The repo contains a CLI cryptocurrency wallet that works with the template node in the `wallet` directory.
The wallet allows users to see their token balances and send transactions.
It also allows advanced interactions like seeing the exact UTXOs you own, choosing specific UTXOs for a transaction, and constructing transactions with UTXOs from diverse owners.
From a developer perspective, this wallet can serve as a starting point for building your own CLI dApp UI.

## Building and Running Locally

If you want to learn how to use Tuxedo in your runtime, we recommend starting with the [Tuxedo Order Book Dex Tutorial](https://github.com/Off-Narrative-Labs/Tuxedo-Order-Book-Dex-Tutorial/).

If you want to develop closer to Tuxedo core, you can build this repository.
First you'll need to have a working Rust and [Substrate development environment](https://docs.substrate.io/install/).
Then you can build Tuxedo like any other Rust project

```sh
# Clone to repository
git clone https://github.com/Off-Narrative-Labs/Tuxedo
cd tuxedo

# Build the node
cargo build --release -p node-template

# If you need the parachain node, build it
# This will take a while to compile
cargo build --release -p parachain-template-node

# Build the wallet
cargo build --release -p tuxedo-template-wallet
```

Once you have the node and wallet built, you can run a development node.
(These commands work for the parachain node as well.)

```sh
# Check out the CLI if you want to
# It supports all standard Substrate CLI options
./target/release/node-template --help

# Start a development node
./target/release/node-template --dev
```

Then, in a separate terminal, experiment with the PoC wallet.

```sh
# Check out the minimal PoC CLI
./target/release/tuxedo-template-wallet --help

# Check your balance
./target/release/tuxedo-template-wallet show-balance

Balance Summary
0xd2bf…df67: 100
--------------------
total      : 100

# Split the 100 genesis tokens into two of values 20 and 25, burning the remaining 55
./target/release/tuxedo-template-wallet spend-coins \
  --output-amount 20 \
  --output-amount 25

  Created "337395dec41937478bb55c4e8c75911cbec061511ddbc38163b94e4386f1228c00000000" worth 20. owned by 0xd2bf…df67
  Created "337395dec41937478bb55c4e8c75911cbec061511ddbc38163b94e4386f1228c01000000" worth 25. owned by 0xd2bf…df67


# Check your balance again to confirm the burn worked
./target/release/tuxedo-template-wallet show-balance

Balance Summary
0xd2bf…df67: 45
--------------------
total      : 45
```

There is a more detailed walkthrough of the wallet in the [Wallet README](wallet/README.md).

## Docker

Developers and curious individuals who want to quickly try out Tuxedo and its template runtime can save the setup and compile time by using docker. CI publishes Docker images for both the example node and the PoC wallet at https://github.com/orgs/Off-Narrative-Labs/packages.

Docker is a complex software and there are many ways to pull and run images and map host ports to container ports. For those not already familiar with Docker, you may benefit from referencing the [docker documentation](https://docs.docker.com/) or [building and running locally](#building-and-running-locally) instead.

The following commands are meant as a quickstart that will work on most platforms for users who already have Docker setup.

```sh
# Run a development node with Docker.
# Use the sovereign node when possible, or the parachain node when necessary, not both.
docker run --network host ghcr.io/off-narrative-labs/tuxedo:main --dev
docker run --network host ghcr.io/off-narrative-labs/tuxedo-parachain:main --dev

# In a separate terminal, explore the PoC wallet's CLI
docker run --network host ghcr.io/off-narrative-labs/tuxedo-wallet:main --help

# Use the PoC wallet to confirm that a 100 token genesis utxo is present
docker run --network host ghcr.io/off-narrative-labs/tuxedo-wallet:main --dev show-balance

Balance Summary
0xd2bf…df67: 100
--------------------
total      : 100
```

More example commands are listed above in the section on [running locally](#building-and-running-locally). They all work with docker as well.


## Parachain

### The Dev Service

The Tuxedo parachain node provides a convenient "dev service" that allows it to run without a relay chain. The commands in the previous section all use this dev service. This mode is good for most situations and is much easier to start than a full relay-para network.

### Zombienet

When you do need, or want, a full relay-para network, it is convenient to use [zombienet](https://github.com/paritytech/zombienet) to start it. Tuxedo ships with a [zombienet config file](./zombienet.toml) to make this process easy.

```sh
zombienet --provider podman spawn zombienet.toml
```

Once your network is started, you can point the cli wallet to a local collator and perform the same token transfers and balance checking as any other Tuxedo node.

Be advised that zombienet is changing quickly, and podman has its own platform-specific issues. If you struggle with zombienet, please open an issue, or consider using the local backend instead.

## Testing and Code Quality

Tuxedo strives for excellent code quality which is enforced through unit tests, and [clippy linting](https://doc.rust-lang.org/stable/clippy/). Both of these are enforced in the CI, which you are free to inspect. You may also run them locally.

```sh
# Run unit tests on all aspects of the project
cargo test

# Run clippy with nightly
cargo +nightly clippy
```

## License

Apache 2.0
