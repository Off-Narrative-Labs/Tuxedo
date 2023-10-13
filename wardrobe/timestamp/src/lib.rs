//! Allow block authors to include a timestamp via an inherent transaction.
//!
//! This is roughly analogous to FRAME's pallet timestamp. It relies on the same client-side inherent data provider,
//! as well as Tuxedo's own previous block inehrent data provider.
//!
//! In each block, the block author must include a single `SetTimestamp` transaction that peeks at the
//! Timestamp UTXO that was created in the previous block, and creates a new one with an updated timestamp.
//!
//! This piece currently features two prominent hacks which will need to be cleaned up in due course.
//! 1. It abuses the UpForGrabs verifier. This should be replaced with an Unspendable verifier and an eviction workflow.
//! 2. In block #1 it allows creating a new best timestamp without comsuming a previous one.
//!    This should be removed once we are able to include a timestamp in the genesis block.

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_inherents::{CheckInherentsResult, InherentData};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::{vec, vec::Vec};
use sp_timestamp::InherentError::TooFarInFuture;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure,
    inherents::{TuxedoInherent, TuxedoInherentAdapter},
    support_macros::{CloneNoBound, DebugNoBound, DefaultNoBound},
    types::{Output, OutputRef, Transaction},
    verifier::UpForGrabs,
    ConstraintChecker, SimpleConstraintChecker, Verifier,
};

#[cfg(test)]
mod cleanup_tests;
#[cfg(test)]
mod first_block_special_case_tests;
#[cfg(test)]
mod update_timestamp_tests;

/// A piece-wide target for logging
const LOG_TARGET: &str = "timestamp-piece";

/// A timestamp, since the unix epoch, noted at some point in the history of the chain.
/// It also records the block height in which it was included.
#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord)]
pub struct Timestamp {
    /// The time, in milliseconds, since the unix epoch.
    pub time: u64,
    /// The block number in which this timestamp was noted.
    pub block: u32,
}

impl UtxoData for Timestamp {
    const TYPE_ID: [u8; 4] = *b"time";
}

impl Timestamp {
    pub fn new(time: u64, block: u32) -> Self {
        Self { time, block }
    }
}

/// Options to configure the timestamp piece in your runtime.
/// Currently we only need access to a block number.
pub trait TimestampConfig {
    /// A means of getting the current block height.
    /// Probably this will be the Tuxedo Executive
    fn block_height() -> u32;

    /// The minimum amount of time by which the timestamp may be updated.
    ///
    /// The default is 2 seconds which should be slightly lower than most chains' block times.
    const MINIMUM_TIME_INTERVAL: u64 = 2_000;

    /// The maximum amount by which a valid block's timestamp may be ahead of an importing
    /// node's current local time.
    ///
    /// Default is 1 minute.
    const MAX_DRIFT: u64 = 60_000;

    /// The minimum amount of time that must have passed before an old timestamp
    /// may be cleaned up.
    ///
    /// Default is 1 day.
    const MIN_TIME_BEFORE_CLEANUP: u64 = 1000 * 60 * 60 * 24;

    /// The minimum number of blocks that must have passed before an old timestamp
    /// may be cleaned up.
    ///
    /// Default is 15 thousand which is roughly equivalent to 1 day with 6 second
    /// block times which is a common default in Substrate chains because of Polkadot.
    const MIN_BLOCKS_BEFORE_CLEANUP: u32 = 15_000;
}

/// Reasons that setting or cleaning up the timestamp may go wrong.
#[derive(Debug, Eq, PartialEq)]
pub enum TimestampError {
    /// UTXO data has an unexpected type
    BadlyTyped,
    /// When attempting to set a new best timestamp, you have not included a new timestamp output.
    MissingNewTimestamp,
    /// The block height reported in the new timestamp does not match the block into which it was inserted.
    NewTimestampWrongHeight,
    /// Multiple outputs were specified while setting the timestamp, but exactly one is required.
    TooManyOutputsWhileSettingTimestamp,
    /// The previous timestamp that is peeked at must be from the immediate ancestor block, but this one is not.
    PreviousTimestampWrongHeight,
    /// No previous timestamp was peeked at in this transaction, but at least one peek is required.
    MissingPreviousTimestamp,
    /// Inputs were specified while setting the timestamp, but none are allowed.
    InputsWhileSettingTimestamp,
    /// The new timestamp is not sufficiently far after the previous (or may even be before it).
    TimestampTooOld,

    /// When cleaning up old timestamps, you must supply exactly one peek input which is the "new time reference"
    /// All the timestamps that will be cleaned up must be at least the CLEANUP_AGE older than this reference.
    CleanupRequiresOneReference,
    /// When cleaning up old timestamps, you may not create any new state at all.
    /// However, you have supplied some new outputs in this transaction.
    CleanupCannotCreateState,
    /// You may not clean up old timestamps until they are at least the CLEANUP_AGE older than another
    /// noted timestamp on-chain.
    DontBeSoHasty,
}

/// A constraint checker for the simple act of setting a new best timetamp.
///
/// This is expected to be performed through an inherent, and to happen exactly once per block.
///
/// This transaction comsumes a single input which is the previous best timestamp,
/// And it creates two new outputs. A best timestamp, and a noted timestamp, both of which
/// include the same timestamp. The purpose of the best timestamp is to be consumed immediately
/// in the next block and guarantees that the timestamp is always increasing by at least the minimum.
/// On the other hand, the noted timestamps stick around in storage for a while so that other
/// transactions that need to peek at them are not immediately invalidated. Noted timestamps
/// can be voluntarily cleand up later by another transaction.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, DebugNoBound, DefaultNoBound, PartialEq, Eq, CloneNoBound, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct SetTimestamp<T>(PhantomData<T>);

impl<T: TimestampConfig + 'static, V: Verifier + From<UpForGrabs>> ConstraintChecker<V>
    for SetTimestamp<T>
{
    type Error = TimestampError;
    type InherentHooks = TuxedoInherentAdapter<Self>;

    fn check(
        &self,
        input_data: &[tuxedo_core::types::Output<V>],
        peek_data: &[tuxedo_core::types::Output<V>],
        output_data: &[tuxedo_core::types::Output<V>],
    ) -> Result<TransactionPriority, Self::Error> {
        log::debug!(
            target: LOG_TARGET,
            "🕰️🖴 Checking constraints for SetTimestamp."
        );

        // Make sure there are no inputs. Setting a new timestamp does not consume anything.
        ensure!(
            input_data.is_empty(),
            Self::Error::InputsWhileSettingTimestamp
        );

        // Make sure the only output is a new best timestamp
        ensure!(!output_data.is_empty(), Self::Error::MissingNewTimestamp);
        let new_timestamp = output_data[0]
            .payload
            .extract::<Timestamp>()
            .map_err(|_| Self::Error::BadlyTyped)?;
        ensure!(
            output_data.len() == 1,
            Self::Error::TooManyOutputsWhileSettingTimestamp
        );

        // Make sure the block height from this timestamp matches the current block height.
        ensure!(
            new_timestamp.block == T::block_height(),
            Self::Error::NewTimestampWrongHeight,
        );

        // Next we need to check peeks, but there is a special case for block 1.
        // We need to initialize the timestamp in block 1, so there are no requirements on
        // the peeks at that height.
        if T::block_height() == 1 {
            // If this special case remains for a while, we should do some checks here like
            // making sure there are no inputs at all. For now, We'll just leave it as is.
            log::debug!(
                target: LOG_TARGET,
                "🕰️🖴 Executing timestamp inherent. Triggering first-block special case."
            );
            return Ok(0);
        }

        // Make sure there at least one peek that is the previous block's timestamp.
        // We don't expect any additional peeks typically, but they are harmless.
        ensure!(!peek_data.is_empty(), Self::Error::MissingPreviousTimestamp);
        let old_timestamp = peek_data[0]
            .payload
            .extract::<Timestamp>()
            .map_err(|_| Self::Error::BadlyTyped)?;

        // Compare the new timestamp to the previous timestamp
        ensure!(
            new_timestamp.time >= old_timestamp.time + T::MINIMUM_TIME_INTERVAL,
            Self::Error::TimestampTooOld
        );

        // Make sure the block height from the previous timestamp matches the previous block height.
        ensure!(
            new_timestamp.block == old_timestamp.block + 1,
            Self::Error::PreviousTimestampWrongHeight,
        );

        Ok(0)
    }

    fn is_inherent(&self) -> bool {
        true
    }
}

impl<V: Verifier + From<UpForGrabs>, T: TimestampConfig + 'static> TuxedoInherent<V, Self>
    for SetTimestamp<T>
{
    type Error = sp_timestamp::InherentError;
    const INHERENT_IDENTIFIER: sp_inherents::InherentIdentifier = sp_timestamp::INHERENT_IDENTIFIER;

    fn create_inherent(
        authoring_inherent_data: &InherentData,
        previous_inherent: Option<(Transaction<V, Self>, H256)>,
    ) -> tuxedo_core::types::Transaction<V, Self> {
        let current_timestamp: u64 = authoring_inherent_data
            .get_data(&sp_timestamp::INHERENT_IDENTIFIER)
            .expect("Inherent data should decode properly")
            .expect("Timestamp inherent data should be present.");
        let new_timestamp = Timestamp {
            time: current_timestamp,
            block: T::block_height(),
        };

        log::debug!(
            target: LOG_TARGET,
            "🕰️🖴 Local timestamp while creating inherent i:: {current_timestamp}"
        );

        let mut peeks = Vec::new();
        match (previous_inherent, T::block_height()) {
            (None, 1) => {
                // This is the first block hack case.
                // We don't need any inputs, so just do nothing.
            }
            (None, _) => panic!("Attemping to construct timestamp inherent with no previous inherent (and not block 1)."),
            (Some((_previous_inherent, previous_id)), _) => {
                // This is the the normal case. We create a full previous to peek at.

                // We are given the entire previous inherent in case we need data from it or need to scrape the outputs.
                // But out transactions are simple enough that we know we just need the one and only output.
                peeks.push(OutputRef {
                    tx_hash: previous_id,
                    // There is always 1 output, so we know right where to find it.
                    index: 0,
                });
            }
        }

        let new_output = Output {
            payload: new_timestamp.into(),
            verifier: UpForGrabs.into(),
        };

        Transaction {
            inputs: Vec::new(),
            peeks,
            outputs: vec![new_output],
            checker: Self::default(),
        }
    }

    fn check_inherent(
        importing_inherent_data: &InherentData,
        inherent: Transaction<V, Self>,
        result: &mut CheckInherentsResult,
    ) {
        let local_time: u64 = importing_inherent_data
            .get_data(&sp_timestamp::INHERENT_IDENTIFIER)
            .expect("Inherent data should decode properly")
            .expect("Timestamp inherent data should be present.");

        log::debug!(
            target: LOG_TARGET,
            "🕰️🖴 Local timestamp while checking inherent is: {:#?}", local_time
        );

        let on_chain_timestamp = inherent.outputs[0].payload.extract::<Timestamp>().expect(
            "SetTimestamp extrinsic should have an output that decodes as a StorableTimestamp.",
        );

        log::debug!(
            target: LOG_TARGET,
            "🕰️🖴 In-block timestamp is: {:#?}", on_chain_timestamp
        );

        // Although FRAME makes the check for the minimum interval here, we don't.
        // We make that check in the on-chain constraint checker.
        // That is a deterministic check that all nodes should agree upon and thus it belongs onchain.
        // FRAME's checks: github.com/paritytech/polkadot-sdk/blob/945ebbbc/substrate/frame/timestamp/src/lib.rs#L299-L306

        // Make the comparison for too far in future
        if on_chain_timestamp.time > local_time + T::MAX_DRIFT {
            log::debug!(
                target: LOG_TARGET,
                "🕰️🖴 Block timestamp is too far in future. About to push an error"
            );

            result
                .put_error(sp_timestamp::INHERENT_IDENTIFIER, &TooFarInFuture)
                .expect("Should be able to push some error");
        }
    }
}

/// Allows users to voluntarily clean up old timestamps by showing that there
/// exists another timestamp that is at least the CLEANUP_AGE newer.
///
/// You can clean up multiple timestamps at once, but you only peek at a single
/// new reference. Although it is useless to do so, it is valid for a transaction
/// to clean up zero timestamps.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, DebugNoBound, DefaultNoBound, PartialEq, Eq, CloneNoBound, TypeInfo)]
pub struct CleanUpTimestamp<T>(PhantomData<T>);

impl<T: TimestampConfig> SimpleConstraintChecker for CleanUpTimestamp<T> {
    type Error = TimestampError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        peek_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there at least one peek that is the new reference time.
        // We don't expect any additional peeks typically, but as above, they are harmless.
        ensure!(
            !peek_data.is_empty(),
            Self::Error::CleanupRequiresOneReference
        );
        let new_reference_timestamp = peek_data[0]
            .extract::<Timestamp>()
            .map_err(|_| Self::Error::BadlyTyped)?;

        // Make sure there are no outputs
        ensure!(
            output_data.is_empty(),
            Self::Error::CleanupCannotCreateState
        );

        // Make sure each input is old enough to be cleaned up
        // in terms of both time and block height.
        for input_datum in input_data {
            let old_timestamp = input_datum
                .extract::<Timestamp>()
                .map_err(|_| Self::Error::BadlyTyped)?;

            ensure!(
                old_timestamp.time + T::MIN_TIME_BEFORE_CLEANUP < new_reference_timestamp.time,
                Self::Error::DontBeSoHasty
            );
            ensure!(
                old_timestamp.block + T::MIN_BLOCKS_BEFORE_CLEANUP < T::block_height(),
                Self::Error::DontBeSoHasty
            );
        }

        Ok(0)
    }
}
