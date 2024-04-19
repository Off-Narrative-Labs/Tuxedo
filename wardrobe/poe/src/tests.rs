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

// Dispute

#[test]
fn dispute_with_zero_losers_works() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };

    assert_eq!(PoeDispute.check(&[], &[], &[claim.into()], &[]), Ok(0))
}

#[test]
fn dispute_with_one_loser_works() {
    let win_claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };
    let lose_claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 11,
    };

    assert_eq!(
        PoeDispute.check(&[], &[lose_claim.into()], &[win_claim.into()], &[]),
        Ok(0)
    )
}

#[test]
fn dispute_with_two_losers_works() {
    let win_claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };
    let lose_claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 11,
    };
    let lose_claim2 = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 12,
    };

    assert_eq!(
        PoeDispute.check(
            &[],
            &[lose_claim.into(), lose_claim2.into()],
            &[win_claim.into()],
            &[]
        ),
        Ok(0)
    )
}

#[test]
fn dispute_with_loser_older_than_winner_fails() {
    let win_claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };
    let lose_claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 9,
    };

    assert_eq!(
        PoeDispute.check(&[], &[lose_claim.into()], &[win_claim.into()], &[]),
        Err(ConstraintCheckerError::IncorrectDisputeWinner)
    )
}

#[test]
fn dispute_with_loser_same_age_as_winner_fails() {
    let win_claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };
    let lose_claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };

    assert_eq!(
        PoeDispute.check(&[], &[lose_claim.into()], &[win_claim.into()], &[]),
        Err(ConstraintCheckerError::IncorrectDisputeWinner)
    )
}

#[test]
fn dispute_with_mismatched_claims_fails() {
    let win_claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };
    let lose_claim = ClaimData {
        claim: H256::repeat_byte(2),
        effective_height: 11,
    };

    assert_eq!(
        PoeDispute.check(&[], &[lose_claim.into()], &[win_claim.into()], &[]),
        Err(ConstraintCheckerError::DisputingMismatchedClaims)
    )
}

#[test]
fn dispute_with_input_fails() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };

    assert_eq!(
        PoeDispute.check(&[Bogus.into()], &[claim.into()], &[], &[]),
        Err(ConstraintCheckerError::WrongNumberInputs)
    )
}

#[test]
fn dispute_with_no_winner_fails() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };

    assert_eq!(
        PoeDispute.check(&[], &[claim.into()], &[], &[]),
        Err(ConstraintCheckerError::WrongNumberInputs)
    )
}

#[test]
fn dispute_with_multiple_winners_fails() {
    let claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };
    let claim2 = ClaimData {
        claim: H256::repeat_byte(2),
        effective_height: 10,
    };

    assert_eq!(
        PoeDispute.check(&[], &[claim.into(), claim2.into()], &[], &[]),
        Err(ConstraintCheckerError::WrongNumberInputs)
    )
}

#[test]
fn dispute_with_output_fails() {
    let win_claim = ClaimData {
        claim: H256::repeat_byte(1),
        effective_height: 10,
    };
    let lose_claim = ClaimData {
        claim: H256::repeat_byte(2),
        effective_height: 11,
    };

    assert_eq!(
        PoeDispute.check(
            &[],
            &[lose_claim.into()],
            &[win_claim.into()],
            &[Bogus.into()]
        ),
        Err(ConstraintCheckerError::WrongNumberOutputs)
    )
}
