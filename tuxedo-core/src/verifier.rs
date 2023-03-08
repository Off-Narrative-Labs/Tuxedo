//! A verifier is a piece of logic that determines whether a transaction as a whole is valid
//! and should be committed. Most tuxedo pieces will provide one or more verifiers. Verifiers
//!  do not typically calculate the correct final state, but rather determine whether the
//! proposed final state (as specified by the output set) meets the necessary constraints.

use sp_std::{fmt::Debug, vec::Vec};

use crate::{
    Redeemer,
    types::{Input, Output},
    dynamic_typing::DynamicallyTypedData,
};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::transaction_validity::TransactionPriority;

/// A single verifier that a transaction can choose to call. Verifies whether the input
/// and output data from a transaction meets the codified constraints.
///
/// Additional transient information may be passed to the verifier by including it in the fields
/// of the verifier struct itself. Information passed in this way does not come from state, nor
/// is it stored in state.
pub trait SimpleVerifier: Debug + Encode + Decode + Clone {
    /// The error type that this verifier may return
    type Error: Debug;

    /// The actual verification logic
    fn verify(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error>;
}

pub trait Verifier: Debug + Encode + Decode + Clone {
    /// the error type that this verifier may return
    type Error: Debug;

    /// The actual verification logic
    fn verify<R: Redeemer>(
        &self,
        inputs: &[Output<R>],
        outputs: &[Output<R>],
    ) -> Result<TransactionPriority, Self::Error>;
}

// This blanket implementation makes it so that any type that chooses to
// implement the Simple trait also implements the Powerful trait. This way
// the executive can always just call the Powerful trait.
impl<T: SimpleVerifier> Verifier for T {

    // When using the blanket implementation, we directly return a UtxoError here
    // So that we have access to the `RedeemerError` variant
    type Error = <T as SimpleVerifier>::Error;

    fn verify<R: Redeemer>(
        &self,
        inputs: &[Output<R>],
        outputs: &[Output<R>],
    ) -> Result<TransactionPriority, Self::Error> {

        // Extract the input data
        let input_data: Vec<DynamicallyTypedData> = inputs
            .iter()
            .map(|o| o.payload.clone())
            .collect();

        // Extract the output data
        let output_data: Vec<DynamicallyTypedData> = outputs
            .iter()
            .map(|o| o.payload.clone())
            .collect();

        // Call the simple verifier
        SimpleVerifier::verify(self, &input_data, &output_data)
    }
}

// impl PowerfulVerifier for () {
//     type Error = ();

//     fn verify(
//         &self,
//         _inputs: &[Input],
//         _outputs: &[Output<R>],
//     ) -> Result<TransactionPriority, Self::Error> {
//         Ok(0)
//     }
// }

// A trivial verifier that verifies everything. Not practical. More for testing
// and for the sake of making things compile before I get around to writing the
// amoeba nd PoE verifiers
impl SimpleVerifier for () {
    type Error = ();

    fn verify(
        &self,
        _input_data: &[DynamicallyTypedData],
        _output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, ()> {
        Ok(0)
    }
}
