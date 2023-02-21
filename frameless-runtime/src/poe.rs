//! This file represents a simple Proof of Existence application, identical in behavior
//! to the FRAME tutorial https://docs.substrate.io/tutorials/work-with-pallets/use-macros-in-a-custom-pallet/
//! Of course, this implementation is based on UTXOs and works with Tuxedo rather than FRAME
//!
//! The application allows users to claim the existence of a preimage for a particular hash with a
//! transaction. Thus, the blockchain network acts as a decentralized notary service. Claims are
//! stored in the state, and can be "revoked" from the state later, although the witness to the original
//! claim will always remain in the history of the blockchain.
//! 
//! The main design deviation from the FRAME PoE pallet is the means by which redundant claims are settled.
//! In FRAME, the exact storage location of each claim is known globally, whereas in the UTXO model, all state
//! is local. This means that when a new claim is registered, it is not possible to efficiently check that the
//! same claim has not already been registered. Instead there is a verifier to boot subsequent redundant claims
//! when they are discovered. This difference is analogous to the difference between recorded and registered
//! land https://cannerlaw.com/blog/the-difference-of-recorded-and-registered-land/

use crate::Verifier;
use crate::{ensure, fail};
use crate::tuxedo_types::{TypedData, UtxoData};
use parity_scale_codec::{Encode, Decode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_runtime::transaction_validity::TransactionPriority;

// Notice this type doesn't have to be public. Cool.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
struct ClaimData {
    /// The hash of the data whose existence is being proven.
    claim: H256,
    /// the time (in block height) at which the claim becomes valid.
    height: u64, //TODO get the generic block height type
}

impl UtxoData for PoeClaim {
    const TYPE_ID: [u8; 4] = *b"poe_";
}

/// Errors that can occur when verifying PoE Transactions
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum PoeError {

}

/// A verifier to create claims.
/// 
/// This verifier allows the creation of many claims in a single operation
/// It also allows the creation of zero claims, although such a transaction is useless and is simply a
/// waste of caller fees.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PoeClaim;

impl Verifier for PoeClaim {
    type Error = PoeError;

    fn verify(&self, input_data: &[TypedData], output_data: &[TypedData]) -> Result<TransactionPriority, Self::Error> {
        todo!()
        // Make sure there are no inputs

        // For each output, make sure the claimed block height is >= the current block height.

        // If we required exact equality, this would mean that transactors needed to get their transactions
        // in exactly the next block which is challenging in times of network congestion. Relaxing the
        // requirement allows the caller to make a somewhat weaker claim with the advantage that they have a longer
        // period of time during which their transaction is valid.
    }
}

/// A verifier to revoke claims.
/// 
/// Like the creation verifier, this allows batch revocation.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PoeRevoke;

impl Verifier for PoeRevoke {
    type Error = PoeError;

    fn verify(&self, input_data: &[TypedData], output_data: &[TypedData]) -> Result<TransactionPriority, Self::Error> {
        todo!()

        // Make sure there are no outputs

        // Make sure the inputs are properly typed
    }
}

/// A verifier that resolves claim disputes by keeping whichever claim came first.
/// 
/// TODO this will work much more elegantly once peek is implemented. We only need to peek at the
/// older winning claim because it will remain in state afterwards.
/// 
/// TODO what shall we do about the redeemer? Each claimer may have given their claim a redeemer
/// such that their own private signature. Perhaps there should be a way for a verifier to override
/// the redeemer logic? This is a concrete case where the verifier redeemer separation is not ideal.
/// Another, weaker example, is when trying o implement something like sudo. Where we want a signature,
/// but we want to authorized signer to come from the a different part of state.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PoeDispute;

impl Verifier for PoeDispute {
    type Error = PoeError;

    fn verify(&self, input_data: &[TypedData], output_data: &[TypedData]) -> Result<TransactionPriority, Self::Error> {
        todo!()

        // Make sure there is at least one input (once peek is ready, it will become a peek)
        // This first input (or only peek) is the claim that will be retained.

        // Make sure that all other inputs, claim the same hash as the winner.

        // Make sure that all other claims have block heights strictly greater than the winner.

        //TODO what to do about the redeemers on those losing claims.
    }
}

/// One workable solution to the problem above is modifying the core transaction structure to something like this
struct Transaction {
    /// A classic input that is consumed from the utxo set. Its redeemer must be satisfied for the tx to be valid
    redemptions: Vec<InputRef>,
    /// Similar to a redemption, this is an input that is consumed from the utxo set, but its redeemer need not be satisfied
    /// In the Poe case above, the losing claims that came later would be evictions.
    evictions: Vec<InputRef>,
    /// Similar to an input, but it is not consumed. This is a way to read pre-existing state without removing it from the utxo set
    /// this also indicates when transaction are not competing for state despite reading the same state, and thus commute.
    /// TBD whether it makes sense to have a redeemer check. My gut instinct is no redeemer check, but it needs more careful thought.
    peeks: Vec<InputRef>,
    /// Newly created pieces of state to be added to the utxo set.
    outputs: Vec<Output>,
}

// Aliases so my sketch above compiles
type InputRef = ();
type Output = ();
use sp_std::vec::Vec;