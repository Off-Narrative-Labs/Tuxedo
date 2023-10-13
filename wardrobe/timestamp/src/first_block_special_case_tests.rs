//! Unit tests for the Timestamp piece.
//! This module tests the "hack / workaround" where we allow setting a timestamp in block #1
//! without consuming any previous one. I hope to remove this hack by including a timestamp extrinsic
//! in the genesis block. I've asked for some background about that in
//! https://substrate.stackexchange.com/questions/10105/extrinsics-in-genesis-block
//! And also sketched a path toward a timestamp in the genesis block in
//! https://github.com/Off-Narrative-Labs/Tuxedo/issues/107

use super::*;

/// The mock config always says the block number is one.
pub struct AlwaysBlockOne;

impl TimestampConfig for AlwaysBlockOne {
    fn block_height() -> u32 {
        1
    }
}

#[test]
fn set_timestamp_first_block_happy_path() {
    let checker = SetTimestamp::<AlwaysBlockOne>(Default::default());

    let new: DynamicallyTypedData = Timestamp::new(1_000, 1).into();
    let out: Vec<Output<UpForGrabs>> = vec![new.into()];

    assert_eq!(checker.check(&[], &[], &out), Ok(0));
}
