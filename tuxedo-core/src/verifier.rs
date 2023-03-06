//! A verifier is a piece of logic that determines whether a transaction as a whole is valid
//! and should be committed. Most tuxedo pieces will provide one or more verifiers. Verifiers
//!  do not typically calculate the correct final state, but rather determine whether the
//! proposed final state (as specified by the output set) meets the necessary constraints.

use sp_std::fmt::Debug;

use crate::dynamic_typing::DynamicallyTypedData;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::transaction_validity::TransactionPriority;

/// A single verifier that a transaction can choose to call. Verifies whether the input
/// and output data from a transaction meets the codified constraints.
///
/// Additional transient information may be passed to the verifier by including it in the fields
/// of the verifier struct itself. Information passed in this way does not come from state, nor
/// is it stored in state.
pub trait Verifier: Debug + Encode + Decode + Clone {
    /// The error type that this verifier may return
    type Error: Debug;

    /// The actual verification logic
    fn verify(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error>;
}


/// Simple verifiers for use in unit tests. Not for use in production runtimes.
pub mod testing {

    use super::*;

    /// A testing verifier that verifies everything.
    #[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
    pub struct AlwaysVerifies;

    impl Verifier for AlwaysVerifies {
        type Error = ();

        fn verify(
            &self,
            _input_data: &[DynamicallyTypedData],
            _output_data: &[DynamicallyTypedData],
        ) -> Result<TransactionPriority, ()> {
            Ok(0)
        }
    }

    /// A testing verifier that verifies nothing.
    #[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
    pub struct NeverVerifies;

    impl Verifier for NeverVerifies {
        type Error = ();

        fn verify(
            &self,
            _input_data: &[DynamicallyTypedData],
            _output_data: &[DynamicallyTypedData],
        ) -> Result<TransactionPriority, ()> {
            Err(())
        }
    }
}
