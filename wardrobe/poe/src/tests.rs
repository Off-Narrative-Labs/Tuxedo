//! Tests for the Proof of Existence Piece

use super::*;
use tuxedo_core::dynamic_typing::testing::Bogus;

//TODO this is now duplicated with the timestamp piece.
// perhaps it should be in a core utility somewhere.
// And maybe with a generic constant.
/// The mock config always says the block number is two.
pub struct AlwaysBlockTwo;

impl PoeConfig for AlwaysBlockTwo {
    fn block_height() -> u32 {
        2
    }
}

// Claim

#[test]
fn claim_works() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };

    assert_eq!(
        PoeClaim::<AlwaysBlockTwo>::default().check(&[], &[], &[], &[claim.into()]),
        Ok(0)
    )
}

#[test]
fn claim_exact_current_block_height_works() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 2,
    };

    assert_eq!(
        PoeClaim::<AlwaysBlockTwo>::default().check(&[], &[], &[], &[claim.into()]),
        Ok(0)
    )
}

#[test]
fn claim_old_block_height_fails() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 1,
    };

    assert_eq!(
        PoeClaim::<AlwaysBlockTwo>::default().check(&[], &[], &[], &[claim.into()]),
        Err(ConstraintCheckerError::EffectiveHeightInPast)
    )
}

// wrong block height
// input fails
// eviction fails
// no output fails
// extra output fails (or check docs, maybe this is allowed)

// Revert

// happy path
// extra inputs (check if it is allowed)
// missing input
// no evictions
// no outputs

// Challenge
// missing winner
// no inputs
// no outputs
