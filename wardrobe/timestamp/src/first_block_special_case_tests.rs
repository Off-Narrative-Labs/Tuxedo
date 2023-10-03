//! Unit tests for the Timestamp piece.
//! This module tests the "hack / workaround" where we allow setting a timestamp in block #1
//! without consuming any previous one. I hope to remove this hack by including a timestamp extrinsic
//! in the genesis block. I've asked for some background about that in
//! https://substrate.stackexchange.com/questions/10105/extrinsics-in-genesis-block
//! And also sketched a path toward a timestamp in the genesis block in
//! https://github.com/Off-Narrative-Labs/Tuxedo/issues/107

use super::*;
use tuxedo_core::dynamic_typing::testing::Bogus;

use TimestampError::*;

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

    let new_best = BestTimestamp(400);
    let new_noted = NotedTimestamp(400);

    let input_data = vec![];
    let output_data = vec![new_best.into(), new_noted.into()];

    assert_eq!(checker.check(&input_data, &[], &output_data), Ok(0),);
}
