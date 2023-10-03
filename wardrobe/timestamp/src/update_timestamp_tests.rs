//! Unit tests for the Timestamp piece.
//! This module tests the primary flow of updating the timestamp via an inherent after it has been initialized.

use super::*;
use tuxedo_core::dynamic_typing::testing::Bogus;
use TimestampError::*;

/// The mock config always says the block number is two.
/// We only need this to work around the first block hack.
pub struct AlwaysBlockTwo;

impl TimestampConfig for AlwaysBlockTwo {
    fn block_height() -> u32 {
        2
    }
}

#[test]
fn update_timestamp_happy_path() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best = BestTimestamp(100);
    let new_best = BestTimestamp(400);
    let new_noted = NotedTimestamp(400);
    let input_data = vec![old_best.into()];
    let output_data = vec![new_best.into(), new_noted.into()];

    assert_eq!(checker.check(&input_data, &[], &output_data), Ok(0),);
}

#[test]
fn update_timestamp_bogus_input() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best = Bogus;
    let new_best = BestTimestamp(400);
    let new_noted = NotedTimestamp(400);
    let input_data = vec![old_best.into()];
    let output_data = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(BadlyTyped)
    );
}

#[test]
fn update_timestamp_input_noted_not_best() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best = NotedTimestamp(100);
    let new_best = BestTimestamp(400);
    let new_noted = NotedTimestamp(400);
    let input_data = vec![old_best.into()];
    let output_data = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(BadlyTyped)
    );
}

#[test]
fn update_timestamp_no_input() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let new_best = BestTimestamp(400);
    let new_noted = NotedTimestamp(400);
    let input_data = vec![];
    let output_data = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(MissingPreviousBestTimestamp),
    );
}

#[test]
fn update_timestamp_output_earlier_than_input() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best = BestTimestamp(500);
    let new_best = BestTimestamp(400);
    let new_noted = NotedTimestamp(400);
    let input_data = vec![old_best.into()];
    let output_data = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(TimestampTooOld)
    );
}

#[test]
fn update_timestamp_output_newer_than_previous_best_nut_not_enough_to_meet_threshold() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best = BestTimestamp(100);
    let new_best = BestTimestamp(200);
    let new_noted = NotedTimestamp(200);
    let input_data = vec![old_best.into()];
    let output_data = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(TimestampTooOld)
    );
}

#[test]
fn update_timestamp_too_many_inputs() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best = BestTimestamp(100);
    let new_best = BestTimestamp(400);
    let new_noted = NotedTimestamp(400);
    let input_data = vec![old_best.clone().into(), old_best.into()];
    let output_data = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(TooManyInputsWhileSettingTimestamp)
    );
}

#[test]
fn update_timestamp_new_best_and_new_noted_inconsistent() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best = BestTimestamp(100);
    let new_best = BestTimestamp(400);
    let new_noted = NotedTimestamp(401);
    let input_data = vec![old_best.into()];
    let output_data = vec![new_best.into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(InconsistentBestAndNotedTimestamps)
    );
}

#[test]
fn update_timestamp_no_outputs() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best = BestTimestamp(100);
    let input_data = vec![old_best.into()];
    let output_data = vec![];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(MissingNewBestTimestamp)
    );
}

#[test]
fn update_timestamp_no_new_best() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best = BestTimestamp(100);
    let new_noted = NotedTimestamp(400);
    let input_data = vec![old_best.into()];
    let output_data = vec![new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(BadlyTyped)
    );
}

#[test]
fn update_timestamp_no_new_noted() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best = BestTimestamp(100);
    let new_best = BestTimestamp(400);
    let input_data = vec![old_best.into()];
    let output_data = vec![new_best.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(MissingNewNotedTimestamp)
    );
}

#[test]
fn update_timestamp_too_many_outputs() {
    let checker = SetTimestamp::<AlwaysBlockTwo>(Default::default());

    let old_best = BestTimestamp(100);
    let new_best = BestTimestamp(400);
    let new_noted = NotedTimestamp(400);
    let input_data = vec![old_best.into()];
    let output_data = vec![new_best.into(), new_noted.clone().into(), new_noted.into()];

    assert_eq!(
        checker.check(&input_data, &[], &output_data),
        Err(TooManyOutputsWhileSettingTimestamp)
    );
}

// #[test]
// fn creation_invalid_generation_fails() {
//     let to_spawn = AmoebaDetails {
//         generation: 100,
//         four_bytes: *b"test",
//     };
//     let input_data = Vec::new();
//     let output_data = vec![to_spawn.into()];

//     assert_eq!(
//         AmoebaCreation.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::WrongGeneration),
//     );
// }

// #[test]
// fn creation_with_inputs_fails() {
//     let example = AmoebaDetails {
//         generation: 0,
//         four_bytes: *b"test",
//     };
//     let input_data = vec![example.clone().into()];
//     let output_data = vec![example.into()];

//     assert_eq!(
//         AmoebaCreation.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::CreationMayNotConsume),
//     );
// }

// #[test]
// fn creation_with_badly_typed_output_fails() {
//     let input_data = Vec::new();
//     let output_data = vec![Bogus.into()];

//     assert_eq!(
//         AmoebaCreation.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::BadlyTypedOutput),
//     );
// }

// #[test]
// fn creation_multiple_fails() {
//     let to_spawn = AmoebaDetails {
//         generation: 0,
//         four_bytes: *b"test",
//     };
//     let input_data = Vec::new();
//     let output_data = vec![to_spawn.clone().into(), to_spawn.into()];

//     assert_eq!(
//         AmoebaCreation.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::CreatedTooMany),
//     );
// }

// #[test]
// fn creation_with_no_output_fails() {
//     let input_data = Vec::new();
//     let output_data = Vec::new();

//     assert_eq!(
//         AmoebaCreation.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::CreatedNothing),
//     );
// }

// #[test]
// fn mitosis_valid_transaction_works() {
//     let mother = AmoebaDetails {
//         generation: 1,
//         four_bytes: *b"test",
//     };
//     let d1 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     let d2 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     let input_data = vec![mother.into()];
//     let output_data = vec![d1.into(), d2.into()];

//     assert_eq!(AmoebaMitosis.check(&input_data, &[], &output_data), Ok(0),);
// }

// #[test]
// fn mitosis_wrong_generation() {
//     let mother = AmoebaDetails {
//         generation: 1,
//         four_bytes: *b"test",
//     };
//     let d1 = AmoebaDetails {
//         generation: 3, // This daughter has the wrong generation
//         four_bytes: *b"test",
//     };
//     let d2 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     let input_data = vec![mother.into()];
//     let output_data = vec![d1.into(), d2.into()];

//     assert_eq!(
//         AmoebaMitosis.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::WrongGeneration),
//     );
// }

// #[test]
// fn mitosis_badly_typed_input() {
//     let mother = Bogus;
//     let d1 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     let d2 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     let input_data = vec![mother.into()];
//     let output_data = vec![d1.into(), d2.into()];

//     assert_eq!(
//         AmoebaMitosis.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::BadlyTypedInput),
//     );
// }

// #[test]
// fn mitosis_no_input() {
//     let d1 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     let d2 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     let input_data = Vec::new();
//     let output_data = vec![d1.into(), d2.into()];

//     assert_eq!(
//         AmoebaMitosis.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::WrongNumberOfMothers),
//     );
// }

// #[test]
// fn mitosis_badly_typed_output() {
//     let mother = AmoebaDetails {
//         generation: 1,
//         four_bytes: *b"test",
//     };
//     let d1 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     let d2 = Bogus;
//     let input_data = vec![mother.into()];
//     let output_data = vec![d1.into(), d2.into()];

//     assert_eq!(
//         AmoebaMitosis.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::BadlyTypedOutput),
//     );
// }

// #[test]
// fn mitosis_only_one_output() {
//     let mother = AmoebaDetails {
//         generation: 1,
//         four_bytes: *b"test",
//     };
//     let d1 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     let input_data = vec![mother.into()];
//     // There is only one daughter when there should be two
//     let output_data = vec![d1.into()];

//     assert_eq!(
//         AmoebaMitosis.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::WrongNumberOfDaughters),
//     );
// }

// #[test]
// fn mitosis_too_many_outputs() {
//     let mother = AmoebaDetails {
//         generation: 1,
//         four_bytes: *b"test",
//     };
//     let d1 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     let d2 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     // Mitosis requires exactly two daughters. There should not be a third one.
//     let d3 = AmoebaDetails {
//         generation: 2,
//         four_bytes: *b"test",
//     };
//     let input_data = vec![mother.into()];
//     let output_data = vec![d1.into(), d2.into(), d3.into()];

//     assert_eq!(
//         AmoebaMitosis.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::WrongNumberOfDaughters),
//     );
// }

// #[test]
// fn death_valid_transaction_works() {
//     let example = AmoebaDetails {
//         generation: 1,
//         four_bytes: *b"test",
//     };
//     let input_data = vec![example.into()];
//     let output_data = vec![];

//     assert_eq!(AmoebaDeath.check(&input_data, &[], &output_data), Ok(0),);
// }

// #[test]
// fn death_no_input() {
//     let input_data = vec![];
//     let output_data = vec![];

//     assert_eq!(
//         AmoebaDeath.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::NoVictim),
//     );
// }

// #[test]
// fn death_multiple_inputs() {
//     let a1 = AmoebaDetails {
//         generation: 1,
//         four_bytes: *b"test",
//     };
//     let a2 = AmoebaDetails {
//         generation: 4,
//         four_bytes: *b"test",
//     };
//     let input_data = vec![a1.into(), a2.into()];
//     let output_data = vec![];

//     assert_eq!(
//         AmoebaDeath.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::TooManyVictims),
//     );
// }

// #[test]
// fn death_with_output() {
//     let example = AmoebaDetails {
//         generation: 1,
//         four_bytes: *b"test",
//     };
//     let input_data = vec![example.clone().into()];
//     let output_data = vec![example.into()];

//     assert_eq!(
//         AmoebaDeath.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::DeathMayNotCreate),
//     );
// }

// #[test]
// fn death_badly_typed_input() {
//     let example = Bogus;
//     let input_data = vec![example.into()];
//     let output_data = vec![];

//     assert_eq!(
//         AmoebaDeath.check(&input_data, &[], &output_data),
//         Err(ConstraintCheckerError::BadlyTypedInput),
//     );
// }
