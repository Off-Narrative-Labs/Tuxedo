//! Allow block authors to include a timestamp via an inherent transaction.
//!
//! This is roughly analogous to FRAME's pallet timestamp. It relies on the same client-side inherent data provider,
//! as well as Tuxedo's own previous block inehrent data provider.
//!
//! In each block ,the block author must include a single `SetTimestamp` transaction.
//! 1. Comsumes the existing best timestamp UTXO (which was created in the previous block).
//! 2. Creates a new best timestamp UTXO (which will be cleaned up in the next block).
//! 3. Creates a new noted timestamp UTXO which will stick around for a minimum amount of time
//!    and, perhaps, eventually be cleaned up by a volunteer.
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
    types::{Input, Output, OutputRef, Transaction},
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

// It might make sense to have a minimum number of blocks before cleanup in addition
// that if the chain stalls, all the transactions in the pool can still be used when
// it comes back alive.

/// A timestamp, since the unix epoch, that is the latest time ever seen in the history
/// of this chain.
#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord)]
pub struct BestTimestamp(pub u64);

impl UtxoData for BestTimestamp {
    const TYPE_ID: [u8; 4] = *b"best";
}

/// A timestamp, since the unix epoch, that was noted at some point in the history of
/// this chain.
#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord)]
pub struct NotedTimestamp(pub u64);

impl UtxoData for NotedTimestamp {
    const TYPE_ID: [u8; 4] = *b"note";
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

    /// When attempting to set a new best timestamp, you have not included a best timestamp utxo.
    MissingNewBestTimestamp,
    /// When attempting to set a new best timestamp, you have not included a noted timestamp utxo.
    MissingNewNotedTimestamp,
    /// Multiple outputs were specified while setting the timestamp, but exactly one is required.
    TooManyOutputsWhileSettingTimestamp,
    /// No inputs were specified when setting the timestamp, but exactly one is required.
    MissingPreviousBestTimestamp,
    /// Multiple inputs were specified while setting the timestamp, but exactly one is required.
    TooManyInputsWhileSettingTimestamp,
    /// The new timestamp is not sufficiently far after the previous (or may even be before it).
    TimestampTooOld,
    /// The best timestamp and noted timestamp outputs are not for the same time.
    /// The two outputs must be for the exact same time in order for the timestamp to be successfully updated.
    InconsistentBestAndNotedTimestamps,

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
        _peek_data: &[tuxedo_core::types::Output<V>],
        output_data: &[tuxedo_core::types::Output<V>],
    ) -> Result<TransactionPriority, Self::Error> {
        log::debug!(
            target: LOG_TARGET,
            "🕰️🖴 Checking constraints for SetTimestamp."
        );

        // FRAME pallet authors are required to ensure_none! to make sure this is an inherent.
        // In Tuxedo, inherent transaction types are identified explicitly, and the tuxedo
        // core can make this check automatically.

        // Make sure the first output is a new best timestamp
        ensure!(
            !output_data.is_empty(),
            Self::Error::MissingNewBestTimestamp
        );
        let new_best = output_data[0]
            .payload
            .extract::<BestTimestamp>()
            .map_err(|_| Self::Error::BadlyTyped)?
            .0;

        // Make sure the second output is a new noted timestamp
        ensure!(
            output_data.len() >= 2,
            Self::Error::MissingNewNotedTimestamp
        );
        let new_noted = output_data[1]
            .payload
            .extract::<NotedTimestamp>()
            .map_err(|_| Self::Error::BadlyTyped)?
            .0;

        // Make sure there are no extra outputs
        ensure!(
            output_data.len() == 2,
            Self::Error::TooManyOutputsWhileSettingTimestamp
        );

        // Make sure that the new best and new noted timestamps are actually for the same time.
        ensure!(
            new_best == new_noted,
            Self::Error::InconsistentBestAndNotedTimestamps
        );

        // Although we expect there to typically be no peeks, there is no harm in allowing them.

        // Next we need to check inputs, but there is a special case for block 1.
        // We need to initialize the timestamp in block 1, so there are no requirements on
        // the inputs at that height.
        if T::block_height() == 1 {
            // If this special case remains for a while, we should do some checks here like
            // making sure there are no inputs at all. For now, We'll just leave it as is.
            log::debug!(
                target: LOG_TARGET,
                "🕰️🖴 Executing timestamp inherent. Triggering first-block special case."
            );
            return Ok(0);
        }

        // Make sure there is exactly one input which is the previous best timestamp
        ensure!(
            !input_data.is_empty(),
            Self::Error::MissingPreviousBestTimestamp
        );
        ensure!(
            input_data.len() == 1,
            Self::Error::TooManyInputsWhileSettingTimestamp
        );

        // Compare the new timestamp to the previous timestamp
        let old_best = input_data[0]
            .payload
            .extract::<BestTimestamp>()
            .map_err(|_| Self::Error::BadlyTyped)?
            .0;
        ensure!(
            new_best >= old_best + T::MINIMUM_TIME_INTERVAL,
            Self::Error::TimestampTooOld
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
        // Extract the current timestamp from the inherent data
        let timestamp_millis: u64 = authoring_inherent_data
            .get_data(&sp_timestamp::INHERENT_IDENTIFIER)
            .expect("Inherent data should decode properly")
            .expect("Timestamp inherent data should be present.");
        let new_best_timestamp = BestTimestamp(timestamp_millis);
        let new_noted_timestamp = NotedTimestamp(timestamp_millis);

        log::debug!(
            target: LOG_TARGET,
            "🕰️🖴 Local timestamp while creating inherent i:: {timestamp_millis}"
        );

        let mut inputs = Vec::new();
        match (previous_inherent, T::block_height()) {
            (None, 1) => {
                // This is the first block hack case.
                // We don't need any inputs, so just do nothing.
            }
            (None, _) => panic!("Attemping to construct timestamp inherent with no previous inherent (and not block 1)."),
            (Some((previous_inherent, previous_id)), _) => {
                // This is the the normal case. We create a full previous input to consume.
                let prev_best_index = previous_inherent
                    .outputs
                    .iter()
                    .position(|output| {
                        output.payload.extract::<BestTimestamp>().is_ok()
                    })
                    .expect("SetTimestamp extrinsic should have an output that decodes as a StorableTimestamp.")
                    .try_into()
                    .expect("There should not be more than u32::max_value outputs in a transaction.");

                let output_ref = OutputRef {
                    tx_hash: previous_id,
                    index: prev_best_index,
                };

                let input = Input {
                    output_ref,
                    // The best time needs to be easily taken. For now I'll assume it is up for grabs.
                    // We can make this an eviction once that is implemented.
                    // Once this is fixed more properly (like by using evictions)
                    // I should be able to not mention UpForGrabs here at all.
                    redeemer: Vec::new(),
                };

                inputs.push(input);
            }
        }

        let best_output = Output {
            payload: new_best_timestamp.into(),
            verifier: UpForGrabs.into(),
        };
        let noted_output = Output {
            payload: new_noted_timestamp.into(),
            verifier: UpForGrabs.into(),
        };

        Transaction {
            inputs,
            peeks: Vec::new(),
            outputs: vec![best_output, noted_output],
            checker: Self::default(),
        }
    }

    fn check_inherent(
        importing_inherent_data: &InherentData,
        inherent: Transaction<V, Self>,
        result: &mut CheckInherentsResult,
    ) {
        // Extract the local view of time from the inherent data
        let local_timestamp: u64 = importing_inherent_data
            .get_data(&sp_timestamp::INHERENT_IDENTIFIER)
            .expect("Inherent data should decode properly")
            .expect("Timestamp inherent data should be present.");

        log::debug!(
            target: LOG_TARGET,
            "🕰️🖴 Local timestamp while checking inherent is: {:#?}", local_timestamp
        );

        let on_chain_timestamp = inherent
            .outputs
            .iter()
            .find_map(|output| output.payload.extract::<BestTimestamp>().ok().map(|o| o.0))
            .expect(
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
        if on_chain_timestamp > local_timestamp + T::MAX_DRIFT {
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
        let new_reference_time = peek_data[0]
            .extract::<NotedTimestamp>()
            .map_err(|_| Self::Error::BadlyTyped)?
            .0;

        // Make sure there are no outputs
        ensure!(
            output_data.is_empty(),
            Self::Error::CleanupCannotCreateState
        );

        // Make sure each input is old enough to be cleaned up
        for input_datum in input_data {
            let old_time = input_datum
                .extract::<NotedTimestamp>()
                .map_err(|_| Self::Error::BadlyTyped)?
                .0;

            ensure!(
                old_time + T::MIN_TIME_BEFORE_CLEANUP < new_reference_time,
                Self::Error::DontBeSoHasty
            );
            //TODO ensure height too
            // And also need to check that the previous block height and current block height
            // are right when setting the time
            // ensure!(
            //     old_height + T::MIN_BLOCKS_BEFORE_CLEANUP < T::block_height(),
            //     Self::Error::DontBeSoHasty
            // );
        }

        Ok(0)
    }
}
