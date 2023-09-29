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
use sp_runtime::transaction_validity::TransactionPriority;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure,
    support_macros::{CloneNoBound, DebugNoBound},
    SimpleConstraintChecker,
};

#[cfg(test)]
mod tests;

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

/// A wrapper around a u64 that holds the Unix epoch time in milliseconds.
/// Basically the same as sp_timestamp::Timestamp, but we need this type
/// to implement the UtxoData trait since they are both foreign.
#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord)]
pub struct StorableTimestamp(pub u64);

// impl From<Timestamp> for StorableTimestamp {
//     //todo
// }

// impl From<StorableTimestamp> for Timestamp {
//     //todo
// }

impl UtxoData for StorableTimestamp {
    const TYPE_ID: [u8; 4] = *b"time";
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
    /// The timetamp may not eb set more than once in a block, but this block attempts to do so.
    TimestampAlreadySet,

    // TODO I'm getting tired of checking the right number of inputs and outputs in every single piece, maybe we should ahve some helpers for this common task.
    // Like it expects exactly N inputs (or outputs) and you give it an error for when there are too many and another for when there are too few.
    /// UTXO data has an unexpected type
    BadlyTyped,
    /// No outputs were specified when setting the timestamp, but exactly one is required.
    MissingNewTimestamp,
    /// Multiple outputs were specified while setting the timestamp, but exactly one is required.
    TooManyOutputsWhileSettingTimestamp,
    /// No inputs were specified when setting the timestamp, but exactly one is required.
    MissingPreviousTimestamp,
    /// Multiple inputs were specified while setting the timestamp, but exactly one is required.
    TooManyInputsWhileSettingTimestamp,
    /// The new timestamp is either before the previous timestamp or not sufficiently far after it.
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

/// A constraint checker for the simple act of setting the timetamp.
///
/// This is expected to be performed through an inherent, and to happen exactly once per block.
/// The earlier it happens in the block the better, and concretely we expect authoring nodes to
/// insert this information first via an inehrent extrinsic.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, DebugNoBound, PartialEq, Eq, CloneNoBound, TypeInfo)]
pub struct SetTimestamp<T>(pub PhantomData<T>);

impl<T: TimestampConfig> SimpleConstraintChecker for SetTimestamp<T> {
    type Error = TimestampError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
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

        // Make sure there is a single output of the correct type
        ensure!(!output_data.is_empty(), Self::Error::MissingNewTimestamp);
        ensure!(
            output_data.len() == 1,
            Self::Error::TooManyOutputsWhileSettingTimestamp
        );

        // We lax the rules a lot for the first block so that we can initialize the timestamp.
        if T::block_height() == 1 {
            //TODO should probably make sure there are no inputs here?
            return Ok(0);
        }

        // Make sure there is exactly one input which is the previous timestamp
        ensure!(
            !output_data.is_empty(),
            Self::Error::MissingPreviousTimestamp
        );
        ensure!(
            input_data.len() == 1,
            Self::Error::TooManyInputsWhileSettingTimestamp
        );

        // Compare the new timestamp to the previous timestamp
        let old_timestamp = input_data[0]
            .extract::<StorableTimestamp>()
            .map_err(|_| Self::Error::BadlyTyped)?
            .0;
        let new_timestamp = output_data[0]
            .extract::<StorableTimestamp>()
            .map_err(|_| Self::Error::BadlyTyped)?
            .0;
        ensure!(
            new_timestamp >= old_timestamp + MINIMUM_TIME_INTERVAL,
            Self::Error::TimestampTooOld
        );

        Ok(0)
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
        // Make sure there is a single peek that is the new reference time
        ensure!(peek_data.len() == 1, Self::Error::CleanupRequiresOneReference);
        let new_reference_time = peek_data[0]
            .extract::<StorableTimestamp>()
            .map_err(|_| Self::Error::BadlyTyped)?
            .0;

        // Make sure there are no outputs
        ensure!(output_data.is_empty(), Self::Error::CleanupCannotCreateState);

        // Make sure each input is old enough to be cleaned up
        for input_datum in input_data {
            let old_time = input_datum
                .extract::<StorableTimestamp>()
                .map_err(|_| Self::Error::BadlyTyped)?
                .0;

            ensure!(old_time + CLEANUP_AGE < new_reference_time, Self::Error::DontBeSoHasty);
        }

        Ok(0)
    }
}