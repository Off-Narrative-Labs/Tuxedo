//! A constraint checker is a piece of logic that determines whether a transaction as a whole is valid
//! and should be committed. Most tuxedo pieces will provide one or more constraint checkers.
//! Constraint Checkers do not calculate the correct final state, but rather determine whether the
//! proposed final state (as specified by the output set) meets the necessary constraints.
//! 
//! Constraint Checkers can be used to codify the laws of a monetary system, a chemistry or physics simulation,
//! NFT kitties, public elections and much more.
//! 
//! By far the most common and most important way to write a constraint checker is with the `SimpleConstraintChecker`
//! trait. It provides a single method called `check` which determines whether the relationship between the inputs
//! and outputs (and peeks) is valid. For example making sure no extra money was created, or making sure the chemical
//! reaction balances.
//! 
//! ## Inherents
//! 
//! If you need to tap in to [Substrate's inherent system](https://docs.substrate.io/learn/transaction-types/#inherent-transactions)
//! you may choose to implement the `ConstraintCheckerWithInherent` trait instead of the simple one. This trait is more complex
//! but if you really need an inherent, it is required. Make sure you know what you are doing before
//! you start writing an inherent.
//! 
//! ## Constraint Checker Internals
//! 
//! One of Tuxedo's killer features is its ability to aggregating pieces recursively.
//! To achieve this we have to consider that many intermediate layers in the aggregation tree
//! will have multiple inherent types. For this reason, we provide a much more flexible interface
//! that the aggregation macro can use.
//! 
//! The current design is based on a chain of blanket implementations. Each trait has a blanket
//! impl for the next more complex one.
//! 
//! `SimpleConstraintChecker` -> `ConstraintCheckerWithInherent` -> ConstraintChecker
//! https://github.com/rust-lang/rust/issues/42721

use sp_std::fmt::Debug;

use crate::{dynamic_typing::DynamicallyTypedData, inherents::InherentInternal};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::transaction_validity::TransactionPriority;

/// A particular constraint checker that a transaction can choose to be checked by.
/// Checks whether the input and output data from a transaction meets the codified constraints.
///
/// Additional transient information may be passed to the constraint checker by including it in the fields
/// of the constraint checker struct itself. Information passed in this way does not come from state, nor
/// is it stored in state.
pub trait ConstraintChecker: Debug + Encode + Decode + Clone {
    /// The error type that this constraint checker may return
    type Error: Debug;

    /// Optional Associated Inherent processing logic. If this transaction type is not an inherent, use ().
    /// If it is an inherent, use Self, and implement the TuxedoInherent trait.
    type InherentHooks: InherentInternal<Self>;

    /// The actual check validation logic
    fn check(
        &self,
        inputs: &[DynamicallyTypedData],
        peeks: &[DynamicallyTypedData],
        outputs: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error>;

    //TODO move this to the inherent hooks
    /// Tells whether this extrinsic is an inherent or not.
    /// If you return true here, you must provide the correct inherent hooks above.
    fn is_inherent(&self) -> bool;
}

/// Utilities for writing constraint-checker-related unit tests
#[cfg(test)]
pub mod testing {
    use scale_info::TypeInfo;
    use serde::{Deserialize, Serialize};

    use super::*;

    /// A testing checker that passes (with zero priority) or not depending on
    /// the boolean value enclosed.
    #[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
    pub struct TestConstraintChecker {
        /// Whether the checker should pass.
        pub checks: bool,
        /// Whether this constraint checker is an inherent.
        pub inherent: bool,
    }

    impl ConstraintChecker for TestConstraintChecker {
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

        fn is_inherent(&self) -> bool {
            self.inherent
        }
    }

    #[test]
    fn test_checker_passes() {
        let result = TestConstraintChecker {
            checks: true,
            inherent: false,
        }
        .check(&[], &[], &[]);
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn test_checker_fails() {
        let result = TestConstraintChecker {
            checks: false,
            inherent: false,
        }
        .check(&[], &[], &[]);
        assert_eq!(result, Err(()));
    }
}
