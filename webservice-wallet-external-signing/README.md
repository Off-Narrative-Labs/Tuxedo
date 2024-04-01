# Tuxedo web service functionality

A REST API for communicating with Tuxedo node template.

## Overview

This is a service built on Axum to support the decentralized application (DApp) built on the Tuxedo Blockchain that allows users to create, trade, breed, and manage virtual cats known as "Kitties". This README provides an overview of the available operations and REST APIs for interacting with the Cryptokitties platform.

Like many UTXO wallets, this web service synchronizes a local-to-the-wallet database of UTXOs that exist on the current best chain.Let's call this as Indexer from now on.
The Indexer does not sync the entire blockchain state.
Rather, it syncs a subset of the state that it considers "relevant".
Currently, the Indexer syncs all relevant UTXOs i.e. Coins, KittyData, TradableKittyData, Timestamps. 
However, the Indexer is designed so that this notion of "relevance" is generalizable.
This design allows developers building chains with Tuxedo to extend the Indexer for their own needs.
However, because this is a rest API-based web service, it is likely to be used by DApps which will leverage the REST API to achieve results.

The overall idea behind the web service architecture: https://github.com/mlabs-haskell/TuxedoDapp/issues/35

Links :
**Sequence dig for API flow:** https://github.com/mlabs-haskell/TuxedoDapp/issues/35#issuecomment-2020211287

**Algorithm to create the redeemer:** https://github.com/mlabs-haskell/TuxedoDapp/issues/35#issuecomment-2015171702

**The overall procedure required from DApp**: https://github.com/mlabs-haskell/TuxedoDapp/issues/35#issuecomment-2011277263

**Difference between signed transaction and unsigned transaction example:** https://github.com/mlabs-haskell/TuxedoDapp/issues/35#issuecomment-2020399526

## REST Documentation

Webservice can be run by using 

```sh
$ cargo run
```

## Guided tour for REST APIS usage 

This guided tour shows REST apis usage and curl command used to hit the endpoints :

### Minting coins 

Rest apis for minting coins

**end point:**: mint-coins

**Amount to mint:** 6000

**Public_key of owner:** d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67

```sh
$ curl -X POST -H "Content-Type: application/json" -d '{"amount": 6000,"owner_public_key":"d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}' http://localhost:3000/mint-coins

```

### Get all coins 

Rest apis for getting all the coins stored in the web service. Basically web service stores all the coin UTXO which are synced from the genesis block to the current height.

**end point:**: get-all-coins

```sh
$ curl -X GET -H "Content-Type: application/json" http://localhost:3000/get-all-coins

```

### Get all owned coins 

Rest API for getting all the coins owned by a particular user or public key  in the web service. Web service stores all the coin utxos which are synced from the genesis block to the current height. Webservice will filter the coin UTXO filtered by the supplied public jey.

**end point:**:get-owned-coins

**Public_key of owner:** Public key of owner: Note it should start without 0X. Example: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67

```sh
$ curl -X GET -H "Content-Type: application/json" -H "owner_public_key: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67" http://localhost:3000/get-owned-coins

```

### Create kitty:

Rest API for creating the kitty 

**end point:**:create-kitty

**Name of kitty to be created:**:amit

**Public_key of owner of kitty:** Public key of owner: Note it should start without 0X. Example: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67

**Returns:** Created kitty.


```sh
$ curl -X POST -H "Content-Type: application/json" -d '{"name": "amit","owner_public_key":"d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}' http://localhost:3000/create-kitty

```

### Get all kitties:

Rest API forgetting all the kitties stored in the local db. It returns all the kitties irrespective of onwer.

**end point:**:get-all-kitty-list

**Returns:** All basic kitties irrespective of owner.

```sh
$ curl -X GET -H "Content-Type: application/json"  http://localhost:3000/get-all-kitty-list

```

### Get owned kitties:

Rest API forgetting all the owned kitties by any particular owner i.e. public key stored in the local db.

**end point:**:get-owned-kitty-list

**Public_key of owner of kitty:**  Public key of owner: Note it should start without 0X. Example : d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67

**Returns:** All the kitties owned by the user i.e public key.

```sh
$ curl -X GET -H "Content-Type: application/json" -H "owner_public_key: 563b6da067f38dc194cbe41ce0b840a985dcbef92b1e5b0a6e04f35544ddfd16" http://localhost:3000/get-owned-kitty-list

```
### Get kitty details by DNA :

Rest API for getting all the details of the kitty by DNA.

**end point:**:get-kitty-by-dna

**DNA of kitty:**  Input the DNA of kitty. Note it should start without 0X. Example 95b951b609a4434b19eb4435dc4fe3eb6f0102ff3448922d933e6edf6b14f6de

**Returns:** The kitty whose DNA matches, else None.

```sh
$ curl -X GET -H "Content-Type: application/json" -H "kitty-dna: 95b951b609a4434b19eb4435dc4fe3eb6f0102ff3448922d933e6edf6b14f6de" http://localhost:3000/get-kitty-by-dna

```
## From now on all the below APIS will have two API Calls in Sequence for one operation: 

**1. Get Transaction and Input UTXO List:**
 Retrieves the transaction and input list required for generating the Redeemer by the web DApp. This call is not routed to the blockchain but is handled entirely by the web service.

 **2. Perform Actual Operation i.e send the signed transaction to the blockchain via web service :**
 Sends the signed transaction to the blockchain via web service for verification and validation using the verifier and constraint checker, respectively.


### List kitty for sale :
Rest API used for listing a Kitty for sale, converting it into a TradableKitty with an associated price.

**1. Get Transaction and Input UTXO List for list kitty for sale:**

**end point:**:get-txn-and-inpututxolist-for-listkitty-forsale

**DNA of kitty:**  Input the DNA of kitty. Note it should start without 0X. Example 95b951b609a4434b19eb4435dc4fe3eb6f0102ff3448922d933e6edf6b14f6de

**Public_key of owner of kitty:**  Public key of owner: Note it should start without 0X. Example : d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67

**kitty-price:**  Price of the kitty

**Returns:** Transaction for listing a kitty for sale without redeemer.

```sh
$ curl -X GET -H "Content-Type: application/json" -H "kitty-dna: 394bd079207af3e0b1a9b1eb1dc40d5d5694bd1fd904d56b96d6fad0039b1f7c" -H "kitty-price: 100" -H "owner_public_key: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67" http://localhost:3000/get-txn-and-inpututxolist-for-listkitty-forsale

```
 **2. Perform Actual Operation i.e send the signed transaction to the blockchain via web service:**

 **end point:**:listkitty-for-sale

**signed_transaction:**: Send the signed transaction. i.e all inputs should have redeemer to prove the ownership of spending or usage.

**Returns:** Tradable kitty .

 ```sh
$ curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "signed_transaction": {"inputs":[{"output_ref":{"tx_hash":"0x0367d974927186bdeb3f1f1c111352711d9e1106a68bde6e4cfd0e64722e4f3a","index":0},"redeemer":[198, 69, 78, 148, 249, 1, 63, 2, 217, 105, 106, 87, 179, 252, 24, 66, 129, 190, 253, 17, 31, 87, 71, 231, 100, 31, 9, 81, 93, 141, 7, 81, 155, 0, 27, 38, 87, 16, 30, 55, 164, 220, 174, 37, 207, 163, 82, 216, 155, 195, 166, 253, 67, 95, 47, 240, 74, 20, 108, 160, 185, 71, 199, 129]}],"peeks":[],"outputs":[{"payload":{"data":[1,0,2,0,0,0,0,0,0,0,57,75,208,121,32,122,243,224,177,169,177,235,29,196,13,93,86,148,189,31,217,4,213,107,150,214,250,208,3,155,31,124,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,97,109,105,116,100,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"type_id":[116,100,107,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}}],"checker":{"TradableKitty":"ListKittiesForSale"}},"input_utxo_list":[{"payload":{"data":[1,0,2,0,0,0,0,0,0,0,57,75,208,121,32,122,243,224,177,169,177,235,29,196,13,93,86,148,189,31,217,4,213,107,150,214,250,208,3,155,31,124,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,97,109,105,116],"type_id":[75,105,116,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}}]}' \
  http://localhost:3000/listkitty-for-sale

```

### Tradable kitty name update :
Rest API is used for updating the name of tradable kitty.

**1. Get Transaction and Input UTXO List for name update of tradable kitty:**

**end point:**:get-txn-and-inpututxolist-for-td-kitty-name-update

**DNA of tradable kitty:**  Input the DNA of kitty. Note it should start without 0X. Example 95b951b609a4434b19eb4435dc4fe3eb6f0102ff3448922d933e6edf6b14f6de

**Public_key of owner of tradable kitty:**  Public key of owner: Note it should start without 0X. Example: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67

**kitty-new-name:**  New name of the kitty

**Returns:** Transaction with tradable kitty name update without redeemer.

```sh
$ curl -X GET -H "Content-Type: application/json" -H "kitty-dna: 394bd079207af3e0b1a9b1eb1dc40d5d5694bd1fd904d56b96d6fad0039b1f7c" -H "kitty-new-name: jbbl" -H "owner_public_key: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67" http://localhost:3000/get-txn-and-inpututxolist-for-td-kitty-name-update

```
 **2. Perform Actual Operation i.e send the signed transaction to the blockchain via web service:**

 **end point:**:update-td-kitty-name

**signed_transaction:**: Send the signed transaction. i.e all inputs should have a redeemer to prove the ownership of spending or usage.

 **Returns:** Tradable kitty with updated name.

 ```sh
$ curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "signed_transaction": {"inputs":[{"output_ref":{"tx_hash":"0xb696b071fdbdca1adcec9149d21a167a04d851693e97b70900ac7547e23c0d0e","index":0},"redeemer":[232, 135, 109, 225, 49, 100, 3, 154, 233, 14, 37, 46, 219, 87, 87, 126, 194, 46, 21, 194, 58, 138, 235, 176, 121, 59, 164, 20, 98, 31, 165, 109, 121, 81, 63, 97, 243, 214, 105, 123, 163, 143, 8, 179, 52, 18, 168, 140, 193, 238, 120, 215, 59, 174, 231, 168, 22, 92, 124, 114, 78, 51, 15, 129]}],"peeks":[],"outputs":[{"payload":{"data":[1,0,2,0,0,0,0,0,0,0,57,75,208,121,32,122,243,224,177,169,177,235,29,196,13,93,86,148,189,31,217,4,213,107,150,214,250,208,3,155,31,124,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,106,98,98,108,231,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"type_id":[116,100,107,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}}],"checker":{"TradableKitty":"UpdateKittiesName"}},"input_utxo_list":[{"payload":{"data":[1,0,2,0,0,0,0,0,0,0,57,75,208,121,32,122,243,224,177,169,177,235,29,196,13,93,86,148,189,31,217,4,213,107,150,214,250,208,3,155,31,124,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,97,109,105,116,231,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"type_id":[116,100,107,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}}]}' \
  http://localhost:3000/update-td-kitty-name

```
### Tradable kitty price update :
Rest API is used for updating the price of tradable kitty.

**1. Get Transaction and Input UTXO List for price update of tradable kitty:**

**end point:**:get-txn-and-inpututxolist-for-td-kitty-name-update

**DNA of kitty:**  Input the DNA of kitty. Note it should start without 0X. Example 95b951b609a4434b19eb4435dc4fe3eb6f0102ff3448922d933e6edf6b14f6de

**Public_key of owner of kitty:**  Public key of owner: Note it should start without 0X. Example: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67

**kitty-new-name:**  New name of the kitty

**Returns:** Transaction with tradable kitty price update without redeemer.

```sh
$ curl -X GET -H "Content-Type: application/json" -H "kitty-dna: 394bd079207af3e0b1a9b1eb1dc40d5d5694bd1fd904d56b96d6fad0039b1f7c" -H "kitty-new-name: jbbl" -H "owner_public_key: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67" http://localhost:3000/get-txn-and-inpututxolist-for-td-kitty-name-update

```
 **2. Perform Actual Operation i.e send the signed transaction to the blockchain via web service:**

 **end point:**:update-td-kitty-name

**signed_transaction:**: Send the signed transaction. i.e all inputs should have a redeemer to prove the ownership of spending or usage.

 **Returns:** Tradable kitty with updated price.

 ```sh
$ curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "signed_transaction": {"inputs":[{"output_ref":{"tx_hash":"0xb696b071fdbdca1adcec9149d21a167a04d851693e97b70900ac7547e23c0d0e","index":0},"redeemer":[232, 135, 109, 225, 49, 100, 3, 154, 233, 14, 37, 46, 219, 87, 87, 126, 194, 46, 21, 194, 58, 138, 235, 176, 121, 59, 164, 20, 98, 31, 165, 109, 121, 81, 63, 97, 243, 214, 105, 123, 163, 143, 8, 179, 52, 18, 168, 140, 193, 238, 120, 215, 59, 174, 231, 168, 22, 92, 124, 114, 78, 51, 15, 129]}],"peeks":[],"outputs":[{"payload":{"data":[1,0,2,0,0,0,0,0,0,0,57,75,208,121,32,122,243,224,177,169,177,235,29,196,13,93,86,148,189,31,217,4,213,107,150,214,250,208,3,155,31,124,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,106,98,98,108,231,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"type_id":[116,100,107,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}}],"checker":{"TradableKitty":"UpdateKittiesName"}},"input_utxo_list":[{"payload":{"data":[1,0,2,0,0,0,0,0,0,0,57,75,208,121,32,122,243,224,177,169,177,235,29,196,13,93,86,148,189,31,217,4,213,107,150,214,250,208,3,155,31,124,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,97,109,105,116,231,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"type_id":[116,100,107,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}}]}' \
  http://localhost:3000/update-td-kitty-name
```

### De-List kitty from sale :
Rest API is used for removing a tradable Kitty from the sale, converting it back to a Basic Kitty without an associated price.

**1. Get Transaction and Input UTXO List for delist-kitty-from-sale:**

**end point:**:get-txn-and-inpututxolist-for-delist-kitty-from-sale

**DNA of kitty:**  Input the DNA of kitty. Note it should start without 0X. Example 95b951b609a4434b19eb4435dc4fe3eb6f0102ff3448922d933e6edf6b14f6de

**Public_key of owner of kitty:**  Public key of owner: Note it should start without 0X. Example: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67

 **Returns:** Transaction with a delisted  kitty without redeemer..

```sh
$ curl -X GET -H "Content-Type: application/json" -H "kitty-dna:95b951b609a4434b19eb4435dc4fe3eb6f0102ff3448922d933e6edf6b14f6de" -H "owner_public_key: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67" http://localhost:3000/get-txn-and-inpututxolist-for-delist-kitty-from-sale

```
 **2. Perform Actual Operation i.e send the signed transaction to the blockchain via web service:**

 **end point:**:update-td-kitty-name

**signed_transaction:**: Send the signed transaction. i.e all inputs should have a redeemer to prove the ownership of spending or usage.

 **Returns:** Basic kitty.

 ```sh
$ curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "signed_transaction": {"inputs":[{"output_ref":{"tx_hash":"0xe680ce989ddaa35c7ed9f3ec1f48ff956457e00a9f4635bd97f2e682cf7e300a","index":0},"redeemer":[74, 200, 62, 251, 42, 74, 130, 155, 97, 200, 209, 13, 99, 178, 179, 5, 181, 124, 177, 221, 67, 131, 151, 81, 188, 224, 7, 56, 253, 244, 36, 76, 23, 177, 67, 218, 177, 229, 88, 178, 78, 42, 182, 143, 133, 172, 75, 96, 169, 132, 83, 203, 16, 210, 96, 190, 19, 118, 84, 78, 40, 56, 236, 128]}],"peeks":[],"outputs":[{"payload":{"data":[1,0,2,0,0,0,0,0,0,0,57,75,208,121,32,122,243,224,177,169,177,235,29,196,13,93,86,148,189,31,217,4,213,107,150,214,250,208,3,155,31,124,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,106,98,98,108],"type_id":[75,105,116,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}}],"checker":{"TradableKitty":"DelistKittiesFromSale"}},"input_utxo_list":[{"payload":{"data":[1,0,2,0,0,0,0,0,0,0,57,75,208,121,32,122,243,224,177,169,177,235,29,196,13,93,86,148,189,31,217,4,213,107,150,214,250,208,3,155,31,124,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,106,98,98,108,231,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"type_id":[116,100,107,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}}]}' \
  http://localhost:3000/delist-kitty-from-sale

```

### kitty name update :
Rest API is used for updating the name of basic kitty.

**1. Get Transaction and Input UTXO List for name update of kitty:**

**end point:**:get-txn-and-inpututxolist-for-kitty-name-update

**DNA of kitty:**  Input the DNA of kitty. Note it should start without 0X. Example 95b951b609a4434b19eb4435dc4fe3eb6f0102ff3448922d933e6edf6b14f6de

**Public_key of owner of kitty:**  Public key of owner: Note it should start without 0X. Example: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67

**kitty-new-name:**  New name of the kitty

**Returns:** Transaction with kitty name update without redeemer.

```sh
$ curl -X GET -H "Content-Type: application/json" -H "kitty-dna: 95b951b609a4434b19eb4435dc4fe3eb6f0102ff3448922d933e6edf6b14f6de" -H "kitty-new-name: jram" -H "owner_public_key: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67" http://localhost:3000/get-txn-and-inpututxolist-for-kitty-name-update

```
 **2. Perform Actual Operation i.e send the signed transaction to the blockchain via web service:**

 **end point:**:update-kitty-name

**signed_transaction:**: Send the signed transaction. i.e all inputs should have a redeemer to prove the ownership of spending or usage.

**Returns:** Kitty with an updated name.

 ```sh
$ curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "signed_transaction": {"inputs":[{"output_ref":{"tx_hash":"0x9492d8c80fb5a8cf2720c0072d00c91c821502894fa4482a9c99fc027bf22daf","index":0},"redeemer":[132, 84, 163, 3, 64, 12, 74, 150, 176, 70, 223, 124, 252, 222, 23, 187, 141, 55, 207, 97, 55, 172, 128, 201, 147, 148, 8, 228, 108, 113, 36, 24, 10, 118, 178, 195, 8, 124, 127, 238, 172, 23, 127, 249, 203, 109, 196, 101, 76, 64, 162, 102, 184, 93, 63, 187, 193, 247, 129, 94, 44, 84, 200, 141]}],"peeks":[],"outputs":[{"payload":{"data":[1,0,2,0,0,0,0,0,0,0,57,75,208,121,32,122,243,224,177,169,177,235,29,196,13,93,86,148,189,31,217,4,213,107,150,214,250,208,3,155,31,124,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,106,114,97,109],"type_id":[75,105,116,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}}],"checker":{"FreeKitty":"UpdateKittiesName"}}}' \
  http://localhost:3000/update-kitty-name

```

### Breed kitty :
Rest API is used for breeding a new Kitty from two parent Kitties, creating a child DNA based on both 

**1. Get Transaction and Input UTXO List for breed kitty:**

**end point:**:get-txn-and-inpututxolist-for-breed-kitty

**DNA of mom kitty:**  Input the DNA of kitty. Note it should start without 0X. Example  e9243fb13a45a51d221cfca21a1a197aa35a1f0723cae3497fda971c825cb1d6

**DNA of dad kitty:**  Input the DNA of kitty. Note it should start without 0X. Example 9741b6456f4b82bb243adfe5e887de9ce3a70e01d7ab39c0f9f565b24a2b059b

**Public_key of the owner of kitties:**  Public key of owner: Note it should start without 0X. Example: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67

**"child-kitty-name**  Name of child kitty

**Returns:** Transaction with breeding info such as mom, dad, child i.e. new family without a redeemer.

```sh
$ curl -X GET -H "Content-Type: application/json" -H "mom-dna: e9243fb13a45a51d221cfca21a1a197aa35a1f0723cae3497fda971c825cb1d6" -H "dad-dna: 9741b6456f4b82bb243adfe5e887de9ce3a70e01d7ab39c0f9f565b24a2b059b" -H "child-kitty-name: jram" -H "owner_public_key: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67" http://localhost:3000/get-txn-and-inpututxolist-for-breed-kitty

```
 **2. Perform Actual Operation i.e send the signed transaction to the blockchain via web service:**

 **end point:**:breed-kitty

**signed_transaction:**: Send the signed transaction. i.e all inputs should have a redeemer to prove the ownership of spending or usage.

**Returns:** New family. I.e Mom kitty, Dad kitty and Child kitty. The mom and dad will have breeding status updated EX: From raringToGo to Tired or hadRecentBirth.

 ```sh
$ curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "signed_transaction": {"inputs":[{"output_ref":{"tx_hash":"0x8f83929cfc36c5ea445787421278f0688a2e7b482e71bd75d5ac7f36028c575b","index":0},"redeemer":[238, 126, 35, 95, 5, 149, 96, 160, 143, 172, 139, 56, 130, 116, 141, 93, 52, 181, 62, 9, 81, 32, 56, 199, 30, 48, 28, 186, 247, 72, 180, 125, 163, 197, 198, 5, 254, 86, 113, 164, 20, 112, 49, 37, 217, 91, 175, 248, 183, 126, 250, 169, 118, 165, 213, 242, 27, 47, 249, 32, 158, 89, 232, 141]},{"output_ref":{"tx_hash":"0x6bb11e2df46081e9252787342116b0b32be9d3302ca1dac535df85642ba46242","index":0},"redeemer":[112, 18, 73, 37, 101, 45, 254, 161, 83, 84, 12, 135, 125, 65, 6, 235, 200, 84, 16, 109, 12, 247, 240, 52, 116, 11, 46, 109, 86, 241, 69, 26, 223, 154, 215, 190, 247, 110, 248, 75, 246, 71, 126, 223, 23, 180, 233, 209, 98, 9, 178, 82, 46, 52, 110, 251, 52, 223, 232, 182, 82, 226, 5, 143]}],"peeks":[],"outputs":[{"payload":{"data":[0,1,1,0,0,0,0,0,0,0,233,36,63,177,58,69,165,29,34,28,252,162,26,26,25,122,163,90,31,7,35,202,227,73,127,218,151,28,130,92,177,214,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,97,109,105,116],"type_id":[75,105,116,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}},{"payload":{"data":[1,1,1,0,0,0,0,0,0,0,151,65,182,69,111,75,130,187,36,58,223,229,232,135,222,156,227,167,14,1,215,171,57,192,249,245,101,178,74,43,5,155,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,97,109,116,105],"type_id":[75,105,116,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}},{"payload":{"data":[0,0,2,0,0,0,0,0,0,0,191,64,163,127,195,246,227,90,81,218,5,243,219,78,156,51,82,162,4,192,66,249,180,130,64,229,219,239,136,216,243,153,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,106,114,97,109],"type_id":[75,105,116,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}}],"checker":{"FreeKitty":"Breed"}}}' \
  http://localhost:3000/breed-kitty

```
**The output message of breed looks like the below :**
 ```sh
$O/P message :
{"message":"Kitty breeding done successfully","mom_kitty":{"parent":{"Mom":"HadBirthRecently"},"free_breedings":1,"dna":"0xe9243fb13a45a51d221cfca21a1a197aa35a1f0723cae3497fda971c825cb1d6","num_breedings":1,"name":[97,109,105,116]},"dad_kitty":{"parent":{"Dad":"Tired"},"free_breedings":1,"dna":"0x9741b6456f4b82bb243adfe5e887de9ce3a70e01d7ab39c0f9f565b24a2b059b","num_breedings":1,"name":[97,109,116,105]},"child_kitty":{"parent":{"Mom":"RearinToGo"},"free_breedings":2,"dna":"0xbf40a37fc3f6e35a51da05f3db4e9c3352a204c042f9b48240e5dbef88d8f399","num_breedings":0,"name":[106,114,97,109]}}

```

### Buy tradable kitty :
Rest API that allows buying a Tradable Kitty from a seller using cryptocurrency i.e money/coin

**1. Get Transaction and Input UTXO List for buying tradable kitty:**

**end point:**:get-txn-and-inpututxolist-for-buy-kitty

**DNA of kitty:**  Input the DNA of kitty. Note it should start without 0X. Example 95b951b609a4434b19eb4435dc4fe3eb6f0102ff3448922d933e6edf6b14f6de

**input-coins:**  Reference of input coins owned by the buyer to be used for buying. We can input multiple input coins. EX: 4d732d8b0d0995151617c5c3beb600dc07a9e1be9fc8e95d9c792be42d65911000000000

**output_amount:** The amount to be paid for transaction which should be >= price of kitty.

**buyer_public_key:**  Public key of buyer i.e owner of coins used for buying: Note it should start without 0X. Example: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67

**seller_public_key:**  Public key of seller i.e owner of kitty to be sold: Note it should start without 0X. Example: fab33c8c12f8df78fa515faa2fcc4bbf7829804a4d187984e13253660a9c1223

**Returns:** Transaction containing coins and kitty used in trading along with public keys of owner without redeemer.

```sh
$ curl -X GET -H "Content-Type: application/json" -H "kitty-dna: bc147303f7d0a361ac22a50bf2ca2ec513d926a327ed678827c90d6512feadd6" -H "input-coins: 4d732d8b0d0995151617c5c3beb600dc07a9e1be9fc8e95d9c792be42d65911000000000" -H "output_amount: 200" -H "buyer_public_key: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67" -H "seller_public_key: fab33c8c12f8df78fa515faa2fcc4bbf7829804a4d187984e13253660a9c1223"http://localhost:3000/get-txn-and-inpututxolist-for-buy-kitty

```
 **2. Perform Actual Operation i.e send the signed transaction to the blockchain via web service:**

 **end point:**:/buy_kitty

**signed_transaction:**: Send the signed transaction. i.e all inputs should have a redeemer to prove the ownership of spending or usage.

 **Returns:** Traded kitty with success or fail message.

 ```sh
$ curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "signed_transaction": {"inputs":[{"output_ref":{"tx_hash":"0x9bffe2abf274e0008f3f34af60cd083e909f884f2064e10f25ca46166306ae81","index":0},"redeemer":[134, 152, 55, 235, 162, 163, 255, 144, 247, 94, 237, 234, 127, 220, 149, 66, 226, 223, 43, 116, 16, 156, 165, 251, 221, 234, 13, 136, 132, 189, 187, 27, 206, 197, 48, 23, 188, 43, 41, 94, 103, 242, 174, 100, 249, 158, 206, 55, 88, 199, 103, 246, 227, 126, 138, 252, 205, 7, 132, 3, 112, 239, 52, 129]},{"output_ref":{"tx_hash":"0x4d732d8b0d0995151617c5c3beb600dc07a9e1be9fc8e95d9c792be42d659110","index":0},"redeemer":[166, 2, 32, 88, 200, 30, 54, 252, 155, 169, 122, 237, 29, 44, 33, 22, 102, 77, 71, 128, 35, 214, 84, 147, 193, 59, 45, 110, 69, 52, 25, 75, 5, 248, 227, 232, 110, 165, 177, 178, 218, 240, 235, 61, 25, 248, 242, 132, 106, 115, 62, 88, 57, 238, 39, 150, 202, 64, 237, 111, 147, 210, 215, 131]}],"peeks":[],"outputs":[{"payload":{"data":[0,0,2,0,0,0,0,0,0,0,188,20,115,3,247,208,163,97,172,34,165,11,242,202,46,197,19,217,38,163,39,237,103,136,39,201,13,101,18,254,173,214,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,97,109,105,116,200,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"type_id":[116,100,107,116]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xd2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67"}}},{"payload":{"data":[200,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"type_id":[99,111,105,0]},"verifier":{"Sr25519Signature":{"owner_pubkey":"0xfab33c8c12f8df78fa515faa2fcc4bbf7829804a4d187984e13253660a9c1223"}}}],"checker":{"TradableKitty":"Buy"}}}' \
  http://localhost:3000/buy_kitty
```

After the transaction is success.We can verify the kitty is transferred to buyer and coins are transferred to the seller using the below rest APIS :

 ```sh
$ curl -X GET -H "Content-Type: application/json" -H "owner_public_key: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67" http://localhost:3000/get-owned-kitty-list


curl -X GET -H "Content-Type: application/json" -H "owner_public_key: fab33c8c12f8df78fa515faa2fcc4bbf7829804a4d187984e13253660a9c1223" http://localhost:3000/get-owned-kitty-list

```

My test results of buy kitty: https://github.com/mlabs-haskell/TuxedoDapp/issues/27#issuecomment-2029302071

Please also see the below link for how to achieve the buy transaction which involves of signing from both buyer and seller in the same transaction :  https://github.com/Off-Narrative-Labs/Tuxedo/issues/169
