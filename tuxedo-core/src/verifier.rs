//! A verifier is a piece of logic that determines whether an input can be consumed in a given context.
//! Because there are multiple reasonable ways to make this decision, we expose a trait to encapsulate
//! the various options. Each runtime will choose to make one or more verifiers available to its users
//! and they will be aggregated into an enum. The most common and useful verifiers are included here
//! with Tuxedo core, but downstream developers are expected to create their own as well.
//!

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::fmt::Debug;

mod multi_signature;
mod simple_signature;

pub use multi_signature::ThresholdMultiSignature;
pub use simple_signature::{Sr25519SignatureCheck, P2PKH};

/// A means of checking that an output can be spent. This check is made on a
/// per-output basis and neither knows nor cares anything about the validation logic that will
/// be applied to the transaction as a whole. Nonetheless, in order to avoid malleability, we
/// we take the entire stripped and serialized transaction as a parameter.
///
/// Information available when verifying an input includes:
/// * The simplified transaction - a stripped encoded version of the transaction
/// * Some environmental information such as the block current block number
/// * An redeemer supplied by the user attempting to spend the input.
pub trait Verifier: Debug + Encode + Decode + Clone {
    /// The type that will be supplied to satisfy the verifier and redeem the UTXO.
    type Redeemer: Decode;

    /// Main function in the trait. Does the checks to make sure an output can be spent.
    fn verify(&self, simplified_tx: &[u8], block_height: u32, redeemer: &Self::Redeemer) -> bool;
}

/// A simple verifier that allows anyone to consume an output at any time
#[derive(
    Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo, Default,
)]
pub struct UpForGrabs;

impl Verifier for UpForGrabs {
    type Redeemer = ();

    fn verify(&self, _simplified_tx: &[u8], _: u32, _: &()) -> bool {
        true
    }
}

/// A simple verifier that allows no one to consume an output ever.
///
/// This is useful for UTXOs that are expected to only ever be consumed by evictions,
/// such as inherents for example.
#[derive(
    Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo, Default,
)]
pub struct Unspendable;

impl Verifier for Unspendable {
    type Redeemer = ();

    fn verify(&self, _simplified_tx: &[u8], _: u32, _: &()) -> bool {
        false
    }
}

// Idea: It could be useful to allow delay deciding whether the redemption should succeed
//       until spend-time. In that case you could pass it in as a verifier.
/// A testing verifier that passes or depending on the enclosed
/// boolean value.
#[cfg(feature = "std")]
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct TestVerifier {
    /// Whether the verifier should pass
    pub verifies: bool,
}

#[cfg(feature = "std")]
impl Verifier for TestVerifier {
    type Redeemer = ();

    fn verify(&self, _simplified_tx: &[u8], _: u32, _: &()) -> bool {
        self.verifies
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sp_core::{crypto::Pair as _, sr25519::Pair};

    /// Generate a bunch of test keypairs
    pub(crate) fn generate_n_pairs(n: u8) -> Vec<Pair> {
        let mut seed = [0u8; 32];
        let mut pairs = Vec::new();

        // We generate the pairs from sequential seeds. Just changing the last byte of the seed each time.
        for i in 0..n {
            seed[31] = i;

            let pair = Pair::from_seed(&seed);
            pairs.push(pair);
        }

        pairs
    }

    #[test]
    fn up_for_grabs_always_verifies() {
        assert!(UpForGrabs.verify(&[], 0, &()))
    }

    #[test]
    fn test_verifier_passes() {
        let result = TestVerifier { verifies: true }.verify(&[], 0, &());
        assert!(result);
    }

    #[test]
    fn test_verifier_fails() {
        let result = TestVerifier { verifies: false }.verify(&[], 0, &());
        assert!(!result);
    }
}
