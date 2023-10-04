//! A constraint checker is a piece of logic that determines whether a transaction as a whole is valid
//! and should be committed. Most tuxedo pieces will provide one or more constraint checkers.
//! Constraint Checkers do not typically calculate the correct final state, but rather determine whether the
//! proposed final state (as specified by the output set) meets the necessary constraints.

use sp_std::{fmt::Debug, vec::Vec};

use crate::{
    dynamic_typing::DynamicallyTypedData, inherents::InherentInternal, types::Output, Verifier,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::transaction_validity::TransactionPriority;

/// A simplified constraint checker that a transaction can choose to call. Checks whether the input
/// and output data from a transaction meets the codified constraints.
///
/// Additional transient information may be passed to the constraint checker by including it in the fields
/// of the constraint checker struct itself. Information passed in this way does not come from state, nor
/// is it stored in state.
pub trait SimpleConstraintChecker<V: Verifier>: Debug + Encode + Decode + Clone + TypeInfo {
    /// The error type that this constraint checker may return
    type Error: Debug;

    /// Optional Associated Inherent processing logic. If this transaction type is not
    /// an inherent, use (). If it is an inherent, use Self, implement the TuxedoInherent
    /// trait, and override the `is_inherent` function to return true.
    type InherentHooks: InherentInternal<V, Self>;

    /// The actual check validation logic
    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        peek_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error>;

    /// Tells whether this extrinsic is an inherent or not.
    /// The default implementation returns false as most transaction types are not inherents.
    /// If you override this to return true, you must supply meaningful Inherent Hooks.
    fn is_inherent(&self) -> bool {
        false
    }
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
pub trait ConstraintChecker<V: Verifier>: Debug + Encode + Decode + Clone + TypeInfo {
    /// The error type that this constraint checker may return
    type Error: Debug;

    /// Optional Associated Inherent processing logic. If this transaction type is not
    /// an inherent, use (). If it is an inherent, use Self, and implement the TuxedoInherent trait
    type InherentHooks: InherentInternal<V, Self>;

    /// The actual check validation logic
    fn check(
        &self,
        inputs: &[Output<V>],
        peeks: &[Output<V>],
        outputs: &[Output<V>],
    ) -> Result<TransactionPriority, Self::Error>;

    /// Tells whether this extrinsic is an inherent or not.
    /// The default implementation returns false as most transaction types are not inherents.
    /// If you override this to return true, you must supply meaningful Inherent Hooks.
    fn is_inherent(&self) -> bool {
        false
    }
}

// This blanket implementation makes it so that any type that chooses to
// implement the Simple trait also implements the more Powerful trait. This way
// the executive can always just call the more Powerful trait.
impl<T: SimpleConstraintChecker<V>, V: Verifier> ConstraintChecker<V> for T {
    // Use the same error type used in the simple implementation.
    type Error = <T as SimpleConstraintChecker<V>>::Error;

    type InherentHooks = <T as SimpleConstraintChecker<V>>::InherentHooks;

    fn check(
        &self,
        inputs: &[Output<V>],
        peeks: &[Output<V>],
        outputs: &[Output<V>],
    ) -> Result<TransactionPriority, Self::Error> {
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

    fn is_inherent(&self) -> bool {
        <Self as SimpleConstraintChecker<V>>::is_inherent(self)
    }
}

/// Utilities for writing constraint-checker-related unit tests
#[cfg(test)]
pub mod testing {
    use crate::verifier::TestVerifier;
    use serde::{Deserialize, Serialize};

    use super::*;

    /// A testing checker that passes (with zero priority) or not depending on
    /// the boolean value enclosed.
    #[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
    pub struct TestConstraintChecker {
        /// Whether the checker should pass.
        pub checks: bool,
    }

    impl<V: Verifier> SimpleConstraintChecker<V> for TestConstraintChecker {
        type Error = ();
        type InherentHooks = ();

        fn check(
            &self,
            _input_data: &[DynamicallyTypedData],
            _peek_data: &[DynamicallyTypedData],
            _output_data: &[DynamicallyTypedData],
        ) -> Result<TransactionPriority, ()> {
            if self.checks {
                Ok(0)
            } else {
                Err(())
            }
        }
    }

    #[test]
    fn test_checker_passes() {
        let result = SimpleConstraintChecker::<TestVerifier>::check(
            &TestConstraintChecker { checks: true },
            &[],
            &[],
            &[],
        );
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn test_checker_fails() {
        let result = SimpleConstraintChecker::<TestVerifier>::check(
            &TestConstraintChecker { checks: false },
            &[],
            &[],
            &[],
        );
        assert_eq!(result, Err(()));
    }
}
