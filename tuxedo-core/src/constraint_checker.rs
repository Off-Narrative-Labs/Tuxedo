//! A constraint checker is a piece of logic that determines whether a transaction as a whole is valid
//! and should be committed. Most tuxedo pieces will provide one or more constraint checkers.
//! Constraint Checkers do not typically calculate the correct final state, but rather determine whether the
//! proposed final state (as specified by the output set) meets the necessary constraints.

use sp_std::{fmt::Debug, vec::Vec};

use crate::{dynamic_typing::DynamicallyTypedData, types::Output, Verifier};
use parity_scale_codec::{Decode, Encode};
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

// TODO Damn it, is there really no way to make this work?
// impl<T, U: From<T>> Into<ConstraintCheckingSuccess<T>> for ConstraintCheckingSuccess<U> {
//     fn into(old: Self) -> ConstraintCheckingSuccess<T> {
//         ConstraintCheckingSuccess::<T> {
//             priority: old.priority,
//             accumulator_value: old.accumulator_value.into(),
//         }
//     }
// }

/// An accumulator allows a Tuxedo piece to do some internal bookkeeping during the course
/// of a single block. The Bookkeeping must be done through this accumulator-like interface
/// where each transaction yields some intermediate value that is them folded into the accumulator
/// via som logic that is encapsulated in the impelementation.
///
/// Many Tuxedo pieces will not need accumulators. In such a case, the constraint checker should
/// simply use () as the accumulator type.
///
/// Some typical usecases for an accumulator would be making sure that the block author reward
/// does not exceed the block's transaction fees. Or making sure that a timestamp transaction occurs
/// at most once in a single block.
///
/// The accumulator is reset automatically at the end of every block. It is not possible to store
/// data here for use in subsequent blocks.
pub trait Accumulator {
    /// The type that is given and also the type of the accumulation result.
    /// I realize that the most general accumulator swill use two different types for those,
    /// but let's do that iff we ever need it. I probably will need to so I can do a simple counter.
    type ValueType;

    /// The accumulator value that should be used to start a fresh accumulation
    /// at the beginning of each new block.
    ///
    /// This is a function and takes a value, as opposed to being a constant for an important reason.
    /// Aggregate runtimes made from multiple pieces will need to give a different initial value depending
    /// which of the constituent constraint checkers is being called.
    fn initial_value(_: Self::ValueType) -> Self::ValueType;

    // TODO This needs to be improved because keeping them unique is kind of a footgun right now.
    // It is a function and not a const because in aggregate runtimes, we will need to determine,
    // at the executive level, what the key is, which means we need to match in the aggregate implementation.
    /// A unique key for this accumulator in the runtime. Like with storage types,
    /// Runtime authors must take care that this key is not used anywhere else in the runtime.
    ///
    /// This is a function and takes a value, as opposed to being a constant for an important reason.
    /// Aggregate runtimes made from multiple pieces will need to give a different initial value depending
    /// which of the constituent constraint checkers is being called.
    fn key_path(_: Self::ValueType) -> &'static str;

    /// This function is responsible for combining or "folding" the intermediate value
    /// from the current transaction into the accumulatoed value so far in this block.
    fn accumulate(a: Self::ValueType, b: Self::ValueType) -> Result<Self::ValueType, ()>;
}

impl Accumulator for () {
    type ValueType = ();

    fn initial_value(_: ()) -> () {
        ()
    }

    fn key_path(_: Self::ValueType) -> &'static str {
        "stub_acc"
    }

    fn accumulate(_: (), _: ()) -> Result<(), ()> {
        Ok(())
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
    use scale_info::TypeInfo;

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
