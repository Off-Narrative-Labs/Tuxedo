//! A verifier is a piece of logic that determines whether a transaction as a whole is valid
//! and should be committed. Most tuxedo pieces will provide one or more verifiers. Verifiers
//!  do not typically calculate the correct final state, but rather determine whether the
//! proposed final state (as specified by the output set) meets the necessary constraints.
//! They are loosely analogous to frame pallet calls.

use std::fmt::Debug;

use crate::types::TypedData;
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
    type Error;

    /// The actual verification logic
    fn verify(
        &self,
        input_data: &[TypedData],
        output_data: &[TypedData],
    ) -> Result<TransactionPriority, Self::Error>;
}

// A trivial verifier that verifies everything. Not practical. More for testing
// and for the sake of making things compile before I get around to writing the
// amoeba nd PoE verifiers
impl Verifier for () {
    type Error = ();

    fn verify(
        &self,
        _input_data: &[TypedData],
        _output_data: &[TypedData],
    ) -> Result<TransactionPriority, ()> {
        Ok(0)
    }
}
