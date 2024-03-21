//! Unit tests for the Amoeba piece

use super::*;
use tuxedo_core::dynamic_typing::testing::Bogus;

#[test]
fn creation_valid_transaction_works() {
    let to_spawn = AmoebaDetails {
        generation: 0,
        four_bytes: *b"test",
    };
    let input_data = Vec::new();
    let output_data = vec![to_spawn.into()];

    assert_eq!(
        AmoebaCreation.check(&input_data, &[], &[], &output_data),
        Ok(0)
    );
}

#[test]
fn creation_invalid_generation_fails() {
    let to_spawn = AmoebaDetails {
        generation: 100,
        four_bytes: *b"test",
    };
    let input_data = Vec::new();
    let output_data = vec![to_spawn.into()];

    assert_eq!(
        AmoebaCreation.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::WrongGeneration),
    );
}

#[test]
fn creation_with_inputs_fails() {
    let example = AmoebaDetails {
        generation: 0,
        four_bytes: *b"test",
    };
    let input_data = vec![example.clone().into()];
    let output_data = vec![example.into()];

    assert_eq!(
        AmoebaCreation.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::CreationMayNotConsume),
    );
}

#[test]
fn creation_with_badly_typed_output_fails() {
    let input_data = Vec::new();
    let output_data = vec![Bogus.into()];

    assert_eq!(
        AmoebaCreation.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::BadlyTypedOutput),
    );
}

#[test]
fn creation_multiple_fails() {
    let to_spawn = AmoebaDetails {
        generation: 0,
        four_bytes: *b"test",
    };
    let input_data = Vec::new();
    let output_data = vec![to_spawn.clone().into(), to_spawn.into()];

    assert_eq!(
        AmoebaCreation.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::CreatedTooMany),
    );
}

#[test]
fn creation_with_no_output_fails() {
    let input_data = Vec::new();
    let output_data = Vec::new();

    assert_eq!(
        AmoebaCreation.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::CreatedNothing),
    );
}

#[test]
fn mitosis_valid_transaction_works() {
    let mother = AmoebaDetails {
        generation: 1,
        four_bytes: *b"test",
    };
    let d1 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    let d2 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    let input_data = vec![mother.into()];
    let output_data = vec![d1.into(), d2.into()];

    assert_eq!(
        AmoebaMitosis.check(&input_data, &[], &[], &output_data),
        Ok(0)
    );
}

#[test]
fn mitosis_wrong_generation() {
    let mother = AmoebaDetails {
        generation: 1,
        four_bytes: *b"test",
    };
    let d1 = AmoebaDetails {
        generation: 3, // This daughter has the wrong generation
        four_bytes: *b"test",
    };
    let d2 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    let input_data = vec![mother.into()];
    let output_data = vec![d1.into(), d2.into()];

    assert_eq!(
        AmoebaMitosis.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::WrongGeneration),
    );
}

#[test]
fn mitosis_badly_typed_input() {
    let mother = Bogus;
    let d1 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    let d2 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    let input_data = vec![mother.into()];
    let output_data = vec![d1.into(), d2.into()];

    assert_eq!(
        AmoebaMitosis.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::BadlyTypedInput),
    );
}

#[test]
fn mitosis_no_input() {
    let d1 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    let d2 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    let input_data = Vec::new();
    let output_data = vec![d1.into(), d2.into()];

    assert_eq!(
        AmoebaMitosis.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::WrongNumberOfMothers),
    );
}

#[test]
fn mitosis_badly_typed_output() {
    let mother = AmoebaDetails {
        generation: 1,
        four_bytes: *b"test",
    };
    let d1 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    let d2 = Bogus;
    let input_data = vec![mother.into()];
    let output_data = vec![d1.into(), d2.into()];

    assert_eq!(
        AmoebaMitosis.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::BadlyTypedOutput),
    );
}

#[test]
fn mitosis_only_one_output() {
    let mother = AmoebaDetails {
        generation: 1,
        four_bytes: *b"test",
    };
    let d1 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    let input_data = vec![mother.into()];
    // There is only one daughter when there should be two
    let output_data = vec![d1.into()];

    assert_eq!(
        AmoebaMitosis.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::WrongNumberOfDaughters),
    );
}

#[test]
fn mitosis_too_many_outputs() {
    let mother = AmoebaDetails {
        generation: 1,
        four_bytes: *b"test",
    };
    let d1 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    let d2 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    // Mitosis requires exactly two daughters. There should not be a third one.
    let d3 = AmoebaDetails {
        generation: 2,
        four_bytes: *b"test",
    };
    let input_data = vec![mother.into()];
    let output_data = vec![d1.into(), d2.into(), d3.into()];

    assert_eq!(
        AmoebaMitosis.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::WrongNumberOfDaughters),
    );
}

#[test]
fn death_valid_transaction_works() {
    let example = AmoebaDetails {
        generation: 1,
        four_bytes: *b"test",
    };
    let input_data = vec![example.into()];
    let output_data = vec![];

    assert_eq!(
        AmoebaDeath.check(&input_data, &[], &[], &output_data),
        Ok(0)
    );
}

#[test]
fn death_no_input() {
    let input_data = vec![];
    let output_data = vec![];

    assert_eq!(
        AmoebaDeath.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::NoVictim),
    );
}

#[test]
fn death_multiple_inputs() {
    let a1 = AmoebaDetails {
        generation: 1,
        four_bytes: *b"test",
    };
    let a2 = AmoebaDetails {
        generation: 4,
        four_bytes: *b"test",
    };
    let input_data = vec![a1.into(), a2.into()];
    let output_data = vec![];

    assert_eq!(
        AmoebaDeath.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::TooManyVictims),
    );
}

#[test]
fn death_with_output() {
    let example = AmoebaDetails {
        generation: 1,
        four_bytes: *b"test",
    };
    let input_data = vec![example.clone().into()];
    let output_data = vec![example.into()];

    assert_eq!(
        AmoebaDeath.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::DeathMayNotCreate),
    );
}

#[test]
fn death_badly_typed_input() {
    let example = Bogus;
    let input_data = vec![example.into()];
    let output_data = vec![];

    assert_eq!(
        AmoebaDeath.check(&input_data, &[], &[], &output_data),
        Err(ConstraintCheckerError::BadlyTypedInput),
    );
}
