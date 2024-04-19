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
fn claim_works_with_zero_claims() {
    assert_eq!(
        PoeClaim::<AlwaysBlockTwo>::default().check(&[], &[], &[], &[]),
        Ok(0)
    )
}

#[test]
fn claim_works_with_one_claim() {
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
fn claim_works_with_two_claims() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };
    let claim2 = ClaimData {
        claim: H256::repeat_byte(2),
        effective_height: 10,
    };
    assert_eq!(
        PoeClaim::<AlwaysBlockTwo>::default().check(&[], &[], &[], &[claim.into(), claim2.into()]),
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

#[test]
fn claim_with_input_fails() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };

    assert_eq!(
        PoeClaim::<AlwaysBlockTwo>::default().check(&[Bogus.into()], &[], &[], &[claim.into()]),
        Err(ConstraintCheckerError::WrongNumberInputs)
    )
}

#[test]
fn claim_with_eviction_fails() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };

    assert_eq!(
        PoeClaim::<AlwaysBlockTwo>::default().check(&[], &[Bogus.into()], &[], &[claim.into()]),
        Err(ConstraintCheckerError::WrongNumberInputs)
    )
}

// Revoke

#[test]
fn revoke_works_with_zero_claims() {
    assert_eq!(PoeRevoke.check(&[], &[], &[], &[]), Ok(0))
}

#[test]
fn revoke_works_with_one_claim() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };

    assert_eq!(PoeRevoke.check(&[claim.into()], &[], &[], &[]), Ok(0))
}
#[test]
fn revoke_works_with_two_claims() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };
    let claim2 = ClaimData {
        claim: H256::repeat_byte(2),
        effective_height: 10,
    };

    assert_eq!(
        PoeRevoke.check(&[claim.into(), claim2.into()], &[], &[], &[]),
        Ok(0)
    )
}

#[test]
fn revoke_with_eviction_fails() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };

    assert_eq!(
        PoeRevoke.check(&[claim.into()], &[Bogus.into()], &[], &[]),
        Err(ConstraintCheckerError::WrongNumberInputs)
    )
}

#[test]
fn revoke_with_outputs_fails() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };

    assert_eq!(
        PoeRevoke.check(&[claim.into()], &[], &[], &[Bogus.into()]),
        Err(ConstraintCheckerError::WrongNumberOutputs)
    )
}

// Challenge
// missing winner
// no inputs
// no outputs
