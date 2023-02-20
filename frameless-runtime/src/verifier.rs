//! A verifier is a piece of logic that determines whether a transaction as a whole is valid
//! and should be committed. Most tuxedo pieces will provide one or more verifiers. Verifiers
//!  do not typically calculate the correct final state, but rather determine whether the
//! proposed final state (as specified by the output set) meets the necessary constraints.
//! They are loosely analogous to frame pallet calls.

use crate::TypedData;

/// A single verifier that a transaction can choose to call. Verifies whether the input
/// and output data from a transaction meets the codified constraints.
/// 
/// Additional transient information may be passed to the verifier by including it in the fields
/// of the verifier struct itself. Information passed in this way does not come from state, nor
/// is it stored in state.
pub trait Verifier {

    /// The actual verification logic
    /// TODO This should return Result<Priority, Error> rather than a simple bool
    fn verify(&self, input_data: &[TypedData], output_data: &[TypedData]) -> bool;
}

// A trivial verifier that verifies everything. Not practical. More for testing
// and for the sake of making things compile before I get around to writing the
// amoeba nd PoE verifiers
impl Verifier for () {

    fn verify(&self, _input_data: &[TypedData], _output_data: &[TypedData]) -> bool {
        true
    }
}