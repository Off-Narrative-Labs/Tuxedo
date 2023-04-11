# Tuxedo Template Wallet

A cli wallet for the Tuxedo Node Template

## Overview

This wallet works with the Tuxedo Node Template and Tuxedo Template Runtime which is also contained in this repository.

Like many UTXO wallets, this one synchronizes a local-to-the-wallet database of UTXOs that exist on the current best chain.
The wallet does not sync the entire blockchain state.
Rather, it syncs a subset of the state that it considers "relevant".
Currently, the wallet syncs any UTXOs that contain tokens owned by a key in the wallet's keystore.
However, the wallet is designed so that this notion of "relevance" is generalizeable.
This design allows developers building chains with Tuxedo to extend the wallet for their own needs.
However, because this is a text- based wallet, it is likely not well-suited for end users of popular dapps.

## CLI Documentation

The node application has a thorough help page that you can access on the CLI. It also has help pages for all subcommands. Please explore and read these docs thoroughly.

```sh
# Show the wallet's main help page
$ tuxedo-template-wallet --help

A simple example / template wallet built for the tuxedo template runtime

Usage: tuxedo-template-wallet [OPTIONS] <COMMAND>

Commands:

...

# Show the help for a subcommand
$ tuxedo-template-wallet verify-coin --help
Verify that a particular coin exists.

Show its value and owner from both chain storage and the local database.

Usage: tuxedo-template-wallet verify-coin <OUTPUT_REF>

Arguments:
  <OUTPUT_REF>
          A hex-encoded output reference

Options:
  -h, --help
          Print help (see a summary with '-h')
```

## Guided Tour

This guided tour shows off some of the most common and important wallet features. It can serve as a quickstart, but is not a substitute for reading the help pages mentioned above. (Seriously, please rtfm).

To follow this walkthrough, you should already have a fresh tuxedo template dev node running as described in the [main readme](../README.md). For example, `node-template --dev`.

### Syncing up an Initial Wallet

The wallet is not a long-running process.
The wallet starts up, syncs with the latest chain state, performs the action invoked, and exits.

Let's begin by just starting a new wallet and letting it sync.

```sh
$ tuxedo-template-wallet

[2023-04-11T17:44:40Z INFO  tuxedo_template_wallet::sync] Initializing fresh sync from genesis 0x12aba3510dc0918aec178a32927f145d22d62afe63392713cb65b85570206327
[2023-04-11T17:44:40Z INFO  tuxedo_template_wallet] Number of blocks in the db: 0
[2023-04-11T17:44:40Z INFO  tuxedo_template_wallet] Wallet database synchronized with node to height 20
[2023-04-11T17:44:40Z INFO  tuxedo_template_wallet] No Wallet Command invoked. Exiting.
```

The logs indicate that a fresh database was created and had no blocks in it. Then, by communicating with the node, the wallet was able to sync 20 blocks. Finally it tells us that we didn't ask the wallet to tell us any specific information or send any transactions, so it just exits.

Let's run the same command again and see that the wallet persists state.

```sh
$ tuxedo-template-wallet

[2023-04-11T17:46:17Z INFO  tuxedo_template_wallet] Number of blocks in the db: 20
[2023-04-11T17:46:17Z INFO  tuxedo_template_wallet] Wallet database synchronized with node to height 52
[2023-04-11T17:46:17Z INFO  tuxedo_template_wallet] No Wallet Command invoked. Exiting.
```

This time, it is not a fresh database. In fact it starts from block 20, where it left off previously, and syncs up to block 52. Again, we didn't tell the wallet any specific action to take, so it just exits.

We can also tell the wallet to skip the initial sync if we want to for any reason.
```sh
$ tuxedo-template-wallet --no-sync

[2023-04-11T17:47:48Z INFO  tuxedo_template_wallet] Number of blocks in the db: 52
[2023-04-11T17:47:48Z WARN  tuxedo_template_wallet] Skipping sync with node. Using previously synced information.
[2023-04-11T17:47:48Z INFO  tuxedo_template_wallet] No Wallet Command invoked. Exiting.
```

Now that we understand that the wallet syncs up with the node each time it starts, let's explore our first wallet command. Like most wallets, it will tell you how many tokens you own.

```sh
$ tuxedo-template-wallet show-balance

[2023-04-11T18:07:52Z INFO  tuxedo_template_wallet] Number of blocks in the db: 52
[2023-04-11T18:07:52Z INFO  tuxedo_template_wallet] Wallet database synchronized with node to height 55
Balance Summary
0xd2bf…df67: 100
--------------------
total      : 100
```
The wallet begins by syncing the blockchain state, as usual.
Then it shows us that it knows about this `0xd2bf...` account.
This is the test account, or the "SHAWN" account.
The wallet already contains these keys so you can start learning quickly.
And it seems this account has some money.
Let's look further.

### Exploring the Genesis Coin

The chain begins with a single coin in storage.
We can confirm that the node and the wallet are familiar with the genesis coin using the `verify-coin` subcommand.

```sh
$ tuxedo-template-wallet verify-coin 000000000000000000000000000000000000000000000000000000000000000000000000

[2023-04-11T17:50:04Z INFO  tuxedo_template_wallet] Number of blocks in the db: 52
[2023-04-11T17:50:04Z INFO  tuxedo_template_wallet] Wallet database synchronized with node to height 128
Details of coin 000000000000000000000000000000000000000000000000000000000000000000000000:
Found in storage.  Value: 100, owned by 0xd2bf…df67
Found in local db. Value: 100, owned by 0xd2bf…df67
```

After syncing, it tells us the status of the coin that we are asking about.
That number with all the `0`s is called an `OutputRef` and it is a unique way to refer to a utxo.
The wallet tells us that the coin is found in the chain's storage and in the wallet's own local db.
Both sources agree that the coin exists, is worth 100, and is owned by Shawn.

Let's "split" this coin by creating a transaction that spends it and creates two new coins worth 40 and 50, burning the remaining 10.

```sh
$ tuxedo-template-wallet spend-coins \
  --output-amount 40 \
  --output-amount 50

[2023-04-11T17:58:00Z INFO  tuxedo_template_wallet] Number of blocks in the db: 128
[2023-04-11T17:58:00Z INFO  tuxedo_template_wallet] Wallet database synchronized with node to height 287
[2023-04-11T17:58:00Z INFO  tuxedo_template_wallet] Node's response to spend transaction: Ok("0x1261bc4cf1d8ba65653948292f30c8c260d2a5d15047d4bedb9e9dd37e6a24d2")

Created "7ff2d54aab484363676ba2380800b95dc646a594c55c35524505fe45de70d33d00000000" worth 40. owned by 0xd2bf…df67
Created "7ff2d54aab484363676ba2380800b95dc646a594c55c35524505fe45de70d33d01000000" worth 50. owned by 0xd2bf…df67
```

Our command told the wallet to create a transaction that spends some coins (in this case the genesis coin) and creates two new coins with the given amounts, burning the remaining 10.
It also tells us the `OutputRef`s of the new coins created.

A balance check reveals that our balance has decreased by the 10 burnt tokes as expected.

```sh
TODO
```

In this case we didn't specify a recipient of the new outputs, so the same default address was used. Next let's explore using some other keys.

### Using Your Own Keys

Of course we can use other keys than the example Shawn key.
The wallet supports generating our own keys, or inserting pre-existing keys.
To follow this guide as closely as possible, you should insert the same key we generated.

```sh
# Generate a new key
$ tuxedo-template-wallet generate-key

  Generated public key is f41a866782d45a4d2d8a623a097c62aee6955a9e580985e3910ba49eded9e06b (5HamRMAa...)
  Generated Phrase is decide city tattoo arrest jeans split main sad slam blame crack farm

# Or, to continue on with demo, insert the same generated key
$ tuxedo-template-wallet insert-key "decide city tattoo arrest jeans split main sad slam blame crack farm"

  The generated public key is f41a866782d45a4d2d8a623a097c62aee6955a9e580985e3910ba49eded9e06b (5HamRMAa...)
```

With our new keys in the keystore, let's send some coins from Shawn to our own key.

```sh
TODO spend-coins \
 --recipient f41a866782d45a4d2d8a623a097c62aee6955a9e580985e3910ba49eded9e06b \
 --output-amount 20 \
 --output-amount 10
```

This command will consume one of the existing coins, and create two new ones owned by our key.
Our new coins will be worth 20 and 10 tokens.
Let's check the balance summary to confirm.

```sh
TODO show-balance
```

### Manually Selecting Inputs

So far, we have let the wallet select which inputs to spend on our behalf.
This is typically fine, but some users like to select specific inputs for their transactions.
The wallet supports this.
But before we can spend specific inputs, let's learn how to print the complete list of unspent outputs.

```sh
TODO show-all-outputs
```

Now that we know precisely which outputs exist in our chain, we can choose to spend a specific one.
Let's consume out 20 token input and send 15 of its coins to Shawn, burning the remaining 5.
Because we are sending to Shawn, and Shawn is the default recipient, we could leave off the `--recipient` flag, but I'll choose to include it anyway.

```sh
spend-coins \
  --input TODO \
  --recipient TODO \
  --output-amount 15


```

You should confirm for yourself that both the balance summary and the complete list of UTXOs look as you expect.

### Multi Owner

The final wallet feature that we will demonstrate is its ability to construct transactions with inputs coming from multiple different owners.

Here we will create a transaction with a single output worth 70 units owned by some address that we'll call Jose, and we'll let the wallet select the inputs itself.
This will require inputs from both Shawn and us, and the wallet is able to handle this.

```sh
TODO
```

Now we check the balance summary and find it is empty.
That is because Jose's keys are not in the keystore, so the wallet does not track his tokens.
