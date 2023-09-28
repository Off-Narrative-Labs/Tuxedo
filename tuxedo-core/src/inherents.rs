//! Utilities for working with inherents in Tuxedo based chains
//! 
//! # Background on inherents
//! 
//! Inherents are a Substrate feature that allows block authors to insert some data from the environment
//! into the body of the block. Inherents are similar to pre-runtime digests which are persisted in the block header,
//! but they differ because inherents go into the block body.
//! 
//! Some classic usecases are a block timestamp, information about the relay chain (if the current chain is a parachain),
//! or information about who should receive the block reward.
//! 
//! # Complexities in UTXO chains
//! 
//! In account based systems, the classic way to use an inherent is that the block author calls the runtime providing the current timestamp.
//! The runtime returns an extrinsic that will set the timestamp on-chain. The block author includes that inherent extrinsic in the block author.
//! The purpose for the extra round-trip to the runtime is to facilitate runtime upgrades, I believe. Then when the extrinsic executes it, for example,
//! overwrites the timestamp in a dedicated storage item.
//! 
//! In UTXO chains, there are no storage items, and all state is local to a UTXO. This is the case with, for example, the timestamp as well.
//! This means that when the author calls into the runtime with a timestamp, the transaction that is returned must include the correct reference
//! to the UTXO that contained the previous best timestamp. This is the crux of the problem. There is no easy way to know the location of
//! the previous timestamp in the utxo-space from inside the runtime.
//! 
//! # Scraping the Parent Block
//! 
//! The solution is to provide the entirety of the previous block to the runtime when asking it to construct inherents.
//! This module provides an inherent data provider that does just this. Any Tuxedo runtime that uses inherents (At least ones,
//! like I've described above), they need to first include this foundational inherent data provider that provvides the previous
//! block so that the Tuxedo executive can scrape it to find the output references of the previous inherent transactions.
//! 
//! # Inherent-Per-Block Cadence Assumption
//! 
//! This entire process assumes that the previous state that needs to be consumed exists in the immediate parent block.
//! This is only guaranteed if the inherent is included in every single block. Currently all the usecases I mentioned above
//! meet this assumption, and this seems to be the way that inehrents are used in general.
//! 
//! If we find that there are compelling usecases for a more flexible cadence, some options to explore include:
//! 1. Use a custom authoring trait which we are already considering for end-of-block inherents anyway.
//! 2. Use the auxiliary storage to keep track of the utxo refs on a block-by-block basis
//!
//! I'm starting to think that inherents are always going to be tied to block cadence. If this is not the case,
//! I would argue that the task of inserting data should not be tied to block authorship, and instead left to some kind of on-chain
//! game or dao that anyone can participate in.

//TODO create an inherent data provider that includes the previous block.
// We may have to actually look it up in the database.