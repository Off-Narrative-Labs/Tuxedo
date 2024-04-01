//! Wallet features related to on-chain timestamps.

use anyhow::anyhow;
use parity_scale_codec::{Decode, Encode};
use runtime::{timestamp::Timestamp, OuterVerifier};
use sled::Db;
use tuxedo_core::types::Output;

/// The identifier for the current timestamp in the db.
const TIMESTAMP: &str = "timestamp";

pub(crate) fn apply_transaction(db: &Db, output: &Output<OuterVerifier>) -> anyhow::Result<()> {
    let timestamp = output.payload.extract::<Timestamp>()?.time;
    let timestamp_tree = db.open_tree(TIMESTAMP)?;
    timestamp_tree.insert([0], timestamp.encode())?;
    Ok(())
}

/// Apply a transaction to the local database, storing the new timestamp.
pub(crate) fn get_timestamp(db: &Db) -> anyhow::Result<u64> {
    let timestamp_tree = db.open_tree(TIMESTAMP)?;
    let timestamp = timestamp_tree
        .get([0])?
        .ok_or_else(|| anyhow!("Could not find timestamp in database."))?;
    u64::decode(&mut &timestamp[..])
        .map_err(|_| anyhow!("Could not decode timestamp from database."))
}
