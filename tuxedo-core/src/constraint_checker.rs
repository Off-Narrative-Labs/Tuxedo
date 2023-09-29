//! A constraint checker is a piece of logic that determines whether a transaction as a whole is valid
//! and should be committed. Most tuxedo pieces will provide one or more constraint checkers.
//! Constraint Checkers do not typically calculate the correct final state, but rather determine whether the
//! proposed final state (as specified by the output set) meets the necessary constraints.

use sp_std::{fmt::Debug, vec::Vec};

use crate::{dynamic_typing::DynamicallyTypedData, types::Output, Verifier};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;

/// A type representing a successful result of checking a transaction's constraints.
#[derive(Debug, Clone, Default)]
pub struct ConstraintCheckingSuccess<ValueType> {
    /// The priority of this transaction that should be reported to the transaction pool.
    pub priority: TransactionPriority,
    /// An intermediate value that should be passed to an accumulator that track transient intra-block data.
    pub accumulator_value: ValueType,
}

pub trait Accumulator {
    /// The type that is given and also the type of the accumulation result.
    /// I realize that the most general accumulator swill use two different types for those,
    /// but let's do that iff we ever need it. I probably will need to so I can do a simple counter.
    type ValueType;

    const ID: [u8; 8];

    const INITIAL_VALUE: Self::ValueType;

    fn accumulate(a: Self::ValueType, b: Self::ValueType) -> Self::ValueType;
}

impl Accumulator for () {
    type ValueType = ();

    const ID: [u8; 8] = *b"stub_acc";

    const INITIAL_VALUE: () = ();

    fn accumulate(_: (), _: ()) -> () {
        ()
    }
}

/// A simplified constraint checker that a transaction can choose to call. Checks whether the input
/// and output data from a transaction meets the codified constraints.
///
/// Additional transient information may be passed to the constraint checker by including it in the fields
/// of the constraint checker struct itself. Information passed in this way does not come from state, nor
/// is it stored in state.
pub trait SimpleConstraintChecker: Debug + Encode + Decode + Clone {
    /// The error type that this constraint checker may return
    type Error: Debug;

    /// A transient accumulator that can be used to track intermediate data during the course of a block's execution.
    type Accumulator: Accumulator;

    /// The actual check validation logic
    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        peek_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<ConstraintCheckingSuccess<<Self::Accumulator as Accumulator>::ValueType>, Self::Error>;
}

/// A single constraint checker that a transaction can choose to call. Checks whether the input
/// and output data from a transaction meets the codified constraints.
///
/// This full ConstraintChecker should only be used if there is more that a piece wants to do such
/// as check the verifier information in some unique way.
///
/// Additional transient information may be passed to the constraint checker by including it in the fields
/// of the constraint checker struct itself. Information passed in this way does not come from state, nor
/// is it stored in state.
pub trait ConstraintChecker<V: Verifier>: Debug + Encode + Decode + Clone {
    /// the error type that this constraint checker may return
    type Error: Debug;

    /// A transient accumulator that can be used to track intermediate data during the course of a block's execution.
    type Accumulator: Accumulator;

    /// The actual check validation logic
    fn check(
        &self,
        inputs: &[Output<V>],
        peeks: &[Output<V>],
        outputs: &[Output<V>],
    ) -> Result<ConstraintCheckingSuccess<<Self::Accumulator as Accumulator>::ValueType>, Self::Error>;
}

// This blanket implementation makes it so that any type that chooses to
// implement the Simple trait also implements the more Powerful trait. This way
// the executive can always just call the more Powerful trait.
impl<T: SimpleConstraintChecker, V: Verifier> ConstraintChecker<V> for T {
    // Use the same error type used in the simple implementation.
    type Error = <T as SimpleConstraintChecker>::Error;

    // Use the same accumulator type used in the simple implementation.
    type Accumulator = <T as SimpleConstraintChecker>::Accumulator;

    fn check(
        &self,
        inputs: &[Output<V>],
        peeks: &[Output<V>],
        outputs: &[Output<V>],
    ) -> Result<ConstraintCheckingSuccess<<Self::Accumulator as Accumulator>::ValueType>, Self::Error>
    {
        // Extract the input data
        let input_data: Vec<DynamicallyTypedData> =
            inputs.iter().map(|o| o.payload.clone()).collect();

        // Extract the peek data
        let peek_data: Vec<DynamicallyTypedData> =
            peeks.iter().map(|o| o.payload.clone()).collect();

        // Extract the output data
        let output_data: Vec<DynamicallyTypedData> =
            outputs.iter().map(|o| o.payload.clone()).collect();

        // Call the simple constraint checker
        SimpleConstraintChecker::check(self, &input_data, &peek_data, &output_data)
    }
}

/// Utilities for writing constraint-checker-related unit tests
#[cfg(feature = "std")]
pub mod testing {
    use super::*;

    /// A testing checker that passes (with zero priority) or not depending on
    /// the boolean value enclosed.
    #[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
    pub struct TestConstraintChecker {
        /// Whether the checker should pass.
        pub checks: bool,
    }

    impl SimpleConstraintChecker for TestConstraintChecker {
        type Error = ();

        type Accumulator = ();

        fn check(
            &self,
            _input_data: &[DynamicallyTypedData],
            _peek_data: &[DynamicallyTypedData],
            _output_data: &[DynamicallyTypedData],
        ) -> Result<ConstraintCheckingSuccess<()>, ()> {
            if self.checks {
                Ok(ConstraintCheckingSuccess {
                    priority: 0,
                    accumulator_value: (),
                })
            } else {
                Err(())
            }
        }
    }

    #[test]
    fn test_checker_passes() {
        let result =
            SimpleConstraintChecker::check(&TestConstraintChecker { checks: true }, &[], &[], &[]);
        assert_eq!(
            result,
            Ok(ConstraintCheckingSuccess {
                priority: 0,
                accumulator_value: (),
            })
        );
    }

    #[test]
    fn test_checker_fails() {
        let result =
            SimpleConstraintChecker::check(&TestConstraintChecker { checks: false }, &[], &[], &[]);
        assert_eq!(result, Err(()));
    }

    // TODO add tests for accumulator stuff.
}
