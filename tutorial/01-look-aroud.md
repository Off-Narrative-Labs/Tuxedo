# Take a Look Around

Kick off an initial compilation right now so that the code has time to compile while we get oriented.
To compile the entire project run `cargo build`.

While, that compiles, let's have a look around.

## The `node` Directory

In the `node` directory, there is a nearly-unmodified copy of the Substrate node template.
We will use this standard Substrate client to run our node when we are interested in doing so.

## The Runtime
Our work will begin in the `tuxedo-template-runtime` directory.
This folder has only a single file,`lib.rs`, so take a look.
If you have ever seen a FRAME runtime, a lot of the contents will be familiar.

Let's skim this entire file to get a sense of its structure.

### Imports

We import items from Substrate, Tuxedo Core, and two Tuxedo pieces (Money and Runtime Upgrade).

### The Opaque Module

The Opaque module houses definitions of types that will be used on the client side.
This includes a few consensus keys types as well as our `OpaqueBlock` type which is based on Substrates [`OpaqueExtrinsic`](https://paritytech.github.io/substrate/master/sp_runtime/struct.OpaqueExtrinsic.html) type.

The idea behind opaque types is that they are all basically `Vec<u8>` under the hood.
This allows the client side to just see them as bytes and have no understanding of their structure or meaning.
Only when these types enter the runtime will they be decoded into Rust types.

### Runtime Version

All Runtimes must provide an implementation of Substrate's [RuntimeVersion](https://paritytech.github.io/substrate/master/sp_version/struct.RuntimeVersion.html). This looks nearly identical to a FRAME runtime.

### Genesis Config

Each runtime needs to provide an implementation of [`BuildStorage`](https://paritytech.github.io/substrate/master/sp_runtime/trait.BuildStorage.html) to allow a chain to begin with some initial, or "genesis" state.

This runtime begins with two coins in its storage.
One is owned by a private key, and the other by a multisignature.
We will be able to mint our own tokens when working with the chain, so we don't need to worry much about the genesis tokens.

### Concrete Types

Next we encounter several type aliases including, [`Transaction`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_template_runtime/type.Transaction.html), [`Header`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_template_runtime/type.Header.html), [`Block`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_template_runtime/type.Block.html), [`Executive`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_template_runtime/type.Executive.html), and [`Output`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_template_runtime/type.Output.html).
Each of these aliases takes a generic type from either Substrate or Tuxedo core, and fills in the generics to provide fully-concrete runtime-specific types.

### Outer Verifier

Next we see the [`OuterVerifier`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_template_runtime/enum.OuterVerifier.html) enum.
Every Tuxedo runtime will have one of these Verifiers and it is required to implement the [`Verifier` trait](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/verifier/trait.Verifier.html).

This enum is an amalgamation of several other individual verifiers.
In this case, we use the three that come standard with Tuxedo:

* [`UpForGrabs`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/verifier/struct.UpForGrabs.html) - Allows anyone to spend a UTXO.
* [`SigCheck`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/verifier/struct.SigCheck.html) - Allows a UTXO to be spent with a signature from the proper key. This is the most common and represents simple private ownership.
* [`ThresholdMultiSignature`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/verifier/struct.ThresholdMultiSignature.html) - Allows a UTXO to be spent when enough members have signed.

Tuxedo developers can extend this enum by writing their own implementations of the `Verifier` trait.
However, we will not need to extend this while writing our dex.

### Piece Configs

The next section of the runtime is typically piece configurations.
Currently our runtime is sufficiently simple that neither of our two pieces need any configuration.
We will need to add a configuration for our dex later on, and you can see a comment showing where that will happen.

### Outer Constraint Checker

The [`OuterConstraintChecker`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_template_runtime/enum.OuterConstraintChecker.html) enum is also an amalgamation enum that implements the [`ConstraintChecker`](https://off-narrative-labs.github.io/Tuxedo/tuxedo_core/constraint_checker/trait.ConstraintChecker.html) trait.
That means that each of its variants represents a separate constraint checker.

This enum represents all of the different transaction types in the runtime.
This template runtime includes two pieces currently.
The first is a fungible token (token id 1) provided by the [Money Piece](https://off-narrative-labs.github.io/Tuxedo/money/index.html).
Second is the ability to perform wasm runtime upgrades using Substrate's forkless upgrade mechanism.
These pieces are included because nearly all runtimes will need them.
As a runtime developer, you can remove either of these pieces, or add more from the [wardrobe](https://github.com/off-Narrative-Labs/tuxedo/tree/main/wardrobe) or that you have written yourself.

We will add to it this enum several times throughout the tutorial.

### `Runtime` and Helper Functions

Next we declare the [`Runtime` struct](https://off-narrative-labs.github.io/Tuxedo/tuxedo_template_runtime/struct.Runtime.html).
And on it we implement a few consensus-related helper functions.
If you find that you need a helper function in the runtime, you can consider implementing it on the `Runtime` struct.

### Runtime API Implementations

Finally, we see Substrate's runtime APIs implemented.
Implementing these api traits is what makes this program useable as a Substrate runtime.
In this sense they are the most important part.

However, most of the functions implementations are short and call into the appropriate helper in Tuxedo core.
So in this sense, they are least interesting and will rarely need to be modified.

## The `dex` Directory
This brings us to the last of the directories: the `dex` directory.
In this tutorial we will write a new Tuxedo piece that represents an order book dex.
It's code will live in this `dex` directory, and we will spend most of our time there.
