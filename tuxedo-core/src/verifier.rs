//! A verifier is a piece of logic that determines whether an input can be consumed in a given context.
//! Because there are multiple reasonable ways to make this decision, we expose a trait to encapsulate
//! the various options. Each runtime will choose to make one or more verifiers available to its users
//! and they will be aggregated into an enum. The most common and useful verifiers are included here
//! with Tuxedo core, but downstream developers are expected to create their own as well.

use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::sr25519::{Public, Signature};
use sp_core::H256;
use sp_std::fmt::Debug;

/// A means of checking that an output can be verified (aka spent). This check is made on a
/// per-output basis and neither knows nor cares anything about the validation logic that will
/// be applied to the transaction as a whole. Nonetheless, in order to avoid malleability, we
/// we take the entire stripped and serialized transaction as a parameter.
pub trait Verifier: Debug + Encode + Decode + Clone {
    fn verify(&self, simplified_tx: &[u8], redeemer: &[u8]) -> bool;
}

/// A typical verifier that checks an sr25519 signature
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct SigCheck {
    pub owner_pubkey: H256,
}

impl Verifier for SigCheck {
    fn verify(&self, simplified_tx: &[u8], redeemer: &[u8]) -> bool {
        let sig = match Signature::try_from(redeemer) {
            Ok(s) => s,
            Err(_) => return false,
        };

        sp_io::crypto::sr25519_verify(&sig, simplified_tx, &Public::from_h256(self.owner_pubkey))
    }
}

/// A simple verifier that allows anyone to consume an output at any time
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct UpForGrabs;

impl Verifier for UpForGrabs {
    fn verify(&self, _simplified_tx: &[u8], _redeemer: &[u8]) -> bool {
        true
    }
}

/// A testing verifier that passes or depending on the enclosed
/// boolean value.
#[cfg(feature = "std")]
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct TestVerifier {
    /// Whether the verifier should pass
    pub verifies: bool,
}

#[cfg(feature = "std")]
impl Verifier for TestVerifier {
    fn verify(&self, _simplified_tx: &[u8], _witness: &[u8]) -> bool {
        self.verifies
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sp_core::{crypto::Pair as _, sr25519::Pair};

    #[test]
    fn up_for_grabs_always_verifies() {
        assert!(UpForGrabs.verify(&[], &[]))
    }

    #[test]
    fn sig_check_with_good_sig() {
        let pair = Pair::from_entropy(b"entropy_entropy_entropy_entropy!".as_slice(), None).0;
        let simplified_tx = b"hello world".as_slice();
        let sig = pair.sign(simplified_tx);
        let witness: &[u8] = sig.as_ref();

        let sig_check = SigCheck {
            owner_pubkey: pair.public().into(),
        };

        assert!(sig_check.verify(simplified_tx, witness));
    }

    #[test]
    fn sig_check_with_bad_sig() {
        let simplified_tx = b"hello world".as_slice();
        let witness = b"bogus_signature".as_slice();

        let sig_check = SigCheck {
            owner_pubkey: H256::zero(),
        };

        assert!(!sig_check.verify(simplified_tx, witness));
    }

    #[test]
    fn test_verifier_passes() {
        let result = TestVerifier { verifies: true }.verify(&[], &[]);
        assert_eq!(result, true);
    }

    #[test]
    fn test_verifier_fails() {
        let result = TestVerifier { verifies: false }.verify(&[], &[]);
        assert_eq!(result, false);
    }
}
