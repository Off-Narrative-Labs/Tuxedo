//! The common types that will be used across a Tuxedo runtime, and not specific to any one piece

// My IDE added this at some point. I'll leave it here as a reminder that maybe I don't need to
// re-invent the type-id wheel;
// use core::any::TypeId;

use sp_std::{vec::Vec, collections::btree_set::BTreeSet};
use parity_scale_codec::{Encode, Decode};
use sp_core::H256;
use crate::redeemer::{SigCheck, UpForGrabs};

/// A reference to a output that is expected to exist in the state.
struct OutputRef {
    /// A hash of the transaction that created this output
    tx_hash: H256,
    /// The index of this output among all outputs created by the same transaction
    index: u32,
}

/// A UTXO Transaction
struct Transaction {
    inputs: BTreeSet<Input>,
    //Todo peeks: BTreeMap<Input>,
    outputs: Vec<Output>,
    verifier: OuterVerifier,
}

struct Input {
    /// a reference to the output being consumed
    output_ref: OutputRef,
    // Eg the signature
    witness: Vec<u8>,
}

/// An opaque piece of Transaction output data. This is how the data appears at the Runtime level. After
/// the redeemer is checked, strongly typed data will be extracted and passed to the verifier.
/// In a cryptocurrency, the data represents a single coin. In Tuxedo, the type of
/// the contained data is generic.
struct Output {
    data: Vec<u8>,
    type_id: [u8; 4],
    redeemer: AvailableRedeemers,
}

trait UtxoData: Encode + Decode {
    //TODO this is ugly. But at least I'm not stuck anymore.
    /// A unique identifier for this type. For now choosing this value and making sure it
    /// really is unique is the problem of the developer. Ideally this would be better.
    /// Maybe macros... Doesn't frame somehow pass info about the string in construct runtime to the pallet-level storage items?
    const TYPE_ID: [u8; 4];
}

impl Output {
    /// Extracts strongly typed data from an Output, iff the output contains the type of data
    /// specified. If the contained data is not the specified type, or decoding fails, this errors.
    fn extract_typed_data<T: UtxoData>(&self) -> Result<T, ()> {
        
        // The first four bytes represent the type id that that was encoded. If they match the type
        // we are trying to decode into, we continue, otherwise we error out.
        if self.type_id == <T as UtxoData>::TYPE_ID {
            T::decode(&mut &self.data[..]).map_err(|_| ())
        } else {
            Err(())
        }
    }
}

//Idea: Maybe we don't need either AmoebaDeath or PoeRevoke? Should there be a single verifier that
// comes with Tuxedo that allows simply deleting UTXOs from the state.
// Thinking more about it, I guess not, because for some applications, it may be invalid to simply delete
// a UTXO without any further processing.

// This will have to be re-written for each runtime, thus it should probably move to the main lib.rs file.
// Perhaps we should consider a macro to aggregate this type for us.
/// A verifier is a piece of logic that can be used to check a transaction.
/// For any given Tuxedo runtime there is a finite set of such verifiers.
/// For example, this may check that input token values exceed output token values.
enum OuterVerifier {
    /// Verifies that an amoeba can split into two new amoebas
    AmoebaMitosis,
    /// Verifies that a single amoeba is simply removed from the state
    AmoebaDeath,
    /// Verifies that a new valid proof of existence claim is made
    PoeClaim,
    /// Verifies that a single PoE is revoked.
    PoeRevoke,
}

trait Verifier {

    //TODO see if this is even necessary. I keep having moments where I think it will be, but
    // don't yet have a very clear usecase.
    /// Additional transient information that is passed to the verifier from the transaction.
    /// This information does not come from existing UTXOs, nor is it stored in new UTXOs.
    type AdditionalInformation;

    fn verify(&self, inputs: BTreeSet<OutputRef>, outputs: Vec<OutputRef>);
}

// Like above, this will probably be aggregates separately for each runtime and maybe should
// move into the main runtime lib.rs file
/// A redeemer checks that an individual input can be consumed. For example that it is signed properly
/// To begin playing, we will have two kinds. A simple signature check, and an anyone-can-consume check.
enum AvailableRedeemers {
    SigCheck(SigCheck),
    UpForGrabs(UpForGrabs),
}

// I think with the TypeId thing I created, this won't be necessary
/// All the possible types that can be stored in a UTXO in this runtime
/// It is recommended to Piece developers that they use the new type pattern
/// often rather than just storing plain primitive types to ensure that data
/// that was stored as one type is not decoded to another type when the UTXO is consumed
// enum StorableData {
//     Amoeba(AmoebaDetails),
//     PoeClaim(H256),
// }

/// An amoeba tracked by our simple Amoeba APP
struct AmoebaDetails {
    /// How many generations after the original Eve Amoeba this one is.
    /// When going through mitosis, this number must increase by 1 each time.
    generation: u32,
    /// Four totally arbitrary bytes that each amoeba has. There is literally no
    /// validation on this field whatsoever. I just had an instinct to include a second field.
    four_bytes: [u8; 4],
}