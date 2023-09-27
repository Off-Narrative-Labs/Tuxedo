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

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_timestamp::Timestamp;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, SimpleConstraintChecker,
};

#[cfg(test)]
mod tests;

/// A piece-wide target for logging
const LOG_TARGET: &str = "timestamp-piece";

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

/// Reasons that setting or reading the timestamp may go wrong.
#[derive(Debug, Eq, PartialEq)]
pub enum TimestampError {
    /// The timetamp may not eb set more than once in a block, but this block attempts to do so.
    TimestampAlreadySet,

    // TODO I'm getting tired of checking the right number of inputs and outputs in every single piece, maybe we should ahve some helpers for this common task.
    // Like it expects exactly N inputs (or outputs) and you give it an error for when there are too many and another for when there are too few.
    /// UTXO data has an unexpected type
    BadlyTypedOutput,
    /// No outputs were specified when setting the timestamp, but exactly one is required.
    MissingTimestamp,
    /// Multiple outputs were specified while setting the timestamp, but exactly one is required.
    TooManyOutputsWhileSettingTimestamp,
    /// No inputs are expected when setting the timestamp. But this transaction specified one.
    UnexpectedInputWhileSettingTimestamp,
}

/// A constraint checker for the simple act of setting the timetamp.
///
/// This is expected to be performed through an inherent, and to happen exactly once per block.
/// The earlier it happens in the block the better, and concretely we expect authoring nodes to
/// insert this information first via an inehrent extrinsic.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct SetTimestamp;

impl SimpleConstraintChecker for SetTimestamp {
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
        ensure!(!output_data.is_empty(), Self::Error::MissingTimestamp);
        ensure!(
            output_data.len() == 1,
            Self::Error::TooManyOutputsWhileSettingTimestamp
        );
        let millis_since_epoch = output_data[0]
            .extract::<StorableTimestamp>()
            .map_err(|_| Self::Error::BadlyTypedOutput)?
            .0;

        // TODO We really need to make sure that the new timestamp is greater than the previous high water mark.
        // Im not handling this right now. It will require more inputs and outputs. I just want to get something basic working right meow.

        // Make sure there are no inputs.
        // We may require cleaning up an old timestamp in the future, or may provide incentives for anyone to do that.
        // For now they stick around as storage bloat.
        ensure!(
            input_data.is_empty(),
            Self::Error::UnexpectedInputWhileSettingTimestamp
        );

        Ok(0)
    }
}
