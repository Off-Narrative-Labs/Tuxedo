//! Allow block authors to include a timestamp via an inherent transaction
//!
//! This strives to be roughly analogous to FRAME's pallet timestamp and uses the same client-side inherent data provider logic.
//!
//! In this first iteration, block authors set timestamp once per block by adding a new utxo. The timestamps are never cleaned up. There are no incentives.
//! In the future, it may make sense to have them consume the previous timestamp or a timestamp from n blocks ago, or just provide incentives for users to clean up old ones.
//!
//! Some things that are still a little unclear. How do we make sure that this function is only called via inherent? Maybe forbid it in the pool.
//! If forbidding them in the pool is the answer, then we need to  consider a way to make it easy / nice for piece developers to declare that their transactions are inherents.
//! One way to make it work for users would be to adapt the macro so that you can add an `#[inherent]` to some call variants, then the macro creates a function called is_inherent that checks if it is an inherent or not.

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
    ConstraintChecker, Verifier, SimpleConstraintChecker,
};

#[cfg(test)]
mod cleanup_tests;
#[cfg(test)]
mod first_block_special_case_tests;
#[cfg(test)]
mod update_timestamp_tests;

/// A piece-wide target for logging
const LOG_TARGET: &str = "timestamp-piece";

/// The minimum amount by which the timestamp may be updated. This should probably
/// be a configuration traint value, but for now I'm hard-coding it.
const MINIMUM_TIME_INTERVAL: u64 = 200;

// It might make sense to have a minimum number of blocks in addition or instead so
// that if the chai nstalls, all the transactions in the pool can still be used when
// it comes back alive.

/// The minimum amount of time that a timestamp utxo must have been stored before it
/// can be cleaned up.
///
/// Currently set to 1 day
const CLEANUP_AGE: u64 = 1000 * 60 * 60 * 24;

/// The maximum amount by which a valid block's timestamp may be ahead of an importing
/// node's current local time. 1 minute.
/// TODO make it part of the config trait.
const MAX_DRIFT: u64 = 60_000;

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
/// In the future maybe the minimum interval will be configurable too.
pub trait TimestampConfig {
    /// A means of getting the current block height.
    /// Probably this will be the Tuxedo Executive
    fn block_height() -> u32;
}

/// Reasons that setting or reading the timestamp may go wrong.
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
/// The earlier it happens in the block the better, and concretely we expect authoring nodes to
/// insert this information first via an inehrent extrinsic.
///
/// This transaction comsumes a single input which is the previous best timestamp,
/// And it creates two new outputs. A best timestamp, and a noted timestamp, both of which
/// include the same timestamp. The puspose of the best timestamp is to be consumed immediately
/// in the next block and guarantees that the timestamp is always increasing by enough.
/// On the other hand the noted timestamps stick around in storage for a while so that other
/// transactions that need to peek at them are not immediately invsalidated. Noted timestamps
/// can be voluntarily cleand up later by another transaction.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, DebugNoBound, DefaultNoBound, PartialEq, Eq, CloneNoBound, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct SetTimestamp<T>(pub PhantomData<T>);

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
        log::info!(
            target: LOG_TARGET,
            "🕰️🖴 Checking constraints for SetTimestamp."
        );

        // In FRAME we use ensure_none! to make sure this is an inherent (no origin means inherent).
        // The implementation is easy in FRAME where nearly every transaction is signed.
        // However in the UTXO model, no transactions are signed in their entirety, so there is no simple way to
        // tell that this is an inherent at this point. My plan to address this (at the moment) is to match on
        // the call type in the pool, and not propagate ones that were marked as inherents.

        // Although we expect there to typically be no peeks, there is no harm in allowing them.

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

        // Next we need to check inputs, but there is a special case for block 1.
        // We need to initialize the timestamp in block 1, so there are no requirements on
        // the inputs at that height.
        // Ideally this will go away soon. And if it makes it to production, we should add some checks for empty inputs here.
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
            new_best >= old_best + MINIMUM_TIME_INTERVAL,
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

        let timestamp_tx = Transaction {
            inputs,
            peeks: Vec::new(),
            outputs: vec![best_output, noted_output],
            checker: Self::default(),
        };

        timestamp_tx
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
        // Plus, That's where we have easy access to the timestamp of the previous block
        // FRAME's checks: github.com/paritytech/polkadot-sdk/blob/945ebbbc/substrate/frame/timestamp/src/lib.rs#L299-L306

        // Make the comparison for too far in future
        if on_chain_timestamp > local_timestamp + MAX_DRIFT {
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
/// to clean up zero timestampe
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct CleanUpTimestamp;

impl SimpleConstraintChecker for CleanUpTimestamp {
    type Error = TimestampError;
    
    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        peek_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure there at least one peek that is the new reference time.
        // We don;t expect any additional peeks typically, but as above, they are harmless.
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
                old_time + CLEANUP_AGE < new_reference_time,
                Self::Error::DontBeSoHasty
            );
        }

        Ok(0)
    }
}
