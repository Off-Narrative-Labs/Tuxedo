//! The common types that will be used across a Tuxedo runtime, and not specific to any one piece

use crate::dynamic_typing::DynamicallyTypedData;
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::traits::Extrinsic;
use sp_std::vec::Vec;

/// A reference to a output that is expected to exist in the state.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct OutputRef {
    /// A hash of the transaction that created this output
    pub tx_hash: H256,
    /// The index of this output among all outputs created by the same transaction
    pub index: u32,
}

/// A UTXO Transaction
///
/// Each transaction consumes some UTXOs (the inputs) and creates some new ones (the outputs).
///
/// The Transaction type is generic over two orthogonal pieces of validation logic:
/// 1. Verifier - A verifier checks that an individual input may be consumed. A typical example
///    of a verifier is checking that there is a signature by the proper owner. Other examples
///    may be that anyone can consume the input or no one can, or that a proof of work is required.
/// 2. ConstraintCheckers - A constraint checker checks that the transaction as a whole meets a set of requirements.
///    For example, that the total output value of a cryptocurrency transaction does not exceed its
///    input value. Or that a cryptokitty was created with the correct genetic material from its parents.
///
/// In the future, there may be additional notions of peeks (inputs that are not consumed)
/// and evictions (inputs that are forcefully consumed.)
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Transaction<V, C> {
    pub inputs: Vec<Input>,
    //Todo peeks: Vec<Input>,
    pub outputs: Vec<Output<V>>,
    pub checker: C,
}

// We must implement this Extrinsic trait to use our Transaction type as the Block's Transaction type
// See https://paritytech.github.io/substrate/master/sp_runtime/traits/trait.Block.html#associatedtype.Extrinsic
//
// This trait's design has a preference for transactions that will have a single signature over the
// entire block, so it is not very useful for us. We still need to implement it to satisfy the bound,
// so we do a minimal implementation.
impl<V, C> Extrinsic for Transaction<V, C> {
    type Call = Self;
    type SignaturePayload = ();

    fn new(data: Self, _: Option<Self::SignaturePayload>) -> Option<Self> {
        Some(data)
    }

    // This function has a default implementation that returns None.
    // TODO what are the consequences of returning Some(false) vs None?
    fn is_signed(&self) -> Option<bool> {
        Some(false)
    }
}

/// A reference the a utxo that will be consumed along with proof that it may be consumed
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Input {
    /// a reference to the output being consumed
    pub output_ref: OutputRef,
    // Eg the signature
    pub redeemer: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UtxoError<ConstraintCheckerError> {
    /// This transaction defines the same input multiple times
    DuplicateInput,
    /// This transaction defines an output that already existed in the UTXO set
    PreExistingOutput,
    /// The contraint checker errored.
    ConstraintCheckerError(ConstraintCheckerError),
    /// The Verifier errored.
    /// TODO determine whether it is useful to relay an inner error from the verifier.
    /// So far, I haven't seen a case, although it seems reasonable to think there might be one.
    VerifierError,
    /// One or more of the inputs required by this transaction is not present in the UTXO set
    MissingInput,
}

/// The Result of dispatching a UTXO transaction.
pub type DispatchResult<VerifierError> = Result<(), UtxoError<VerifierError>>;

/// An opaque piece of Transaction output data. This is how the data appears at the Runtime level. After
/// the verifier is checked, strongly typed data will be extracted and passed to the constraint checker.
/// In a cryptocurrency, the data represents a single coin. In Tuxedo, the type of
/// the contained data is generic.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Output<V> {
    pub payload: DynamicallyTypedData,
    pub verifier: V,
}

#[cfg(test)]
pub mod tests {

    use crate::{constraint_checker::testing::TestConstraintChecker, verifier::UpForGrabs};

    use super::*;

    #[test]
    fn extrinsic_no_signed_payload() {
        let checker = TestConstraintChecker { checks: true };
        let tx: Transaction<UpForGrabs, TestConstraintChecker> = Transaction {
            inputs: Vec::new(),
            outputs: Vec::new(),
            checker,
        };
        let e = Transaction::new(tx.clone(), None).unwrap();

        assert_eq!(e, tx);
        assert_eq!(e.is_signed(), Some(false));
    }

    #[test]
    fn extrinsic_is_signed_works() {
        let checker = TestConstraintChecker { checks: true };
        let tx: Transaction<UpForGrabs, TestConstraintChecker> = Transaction {
            inputs: Vec::new(),
            outputs: Vec::new(),
            checker,
        };
        let e = Transaction::new(tx.clone(), Some(())).unwrap();

        assert_eq!(e, tx);
        assert_eq!(e.is_signed(), Some(false));
    }
}
