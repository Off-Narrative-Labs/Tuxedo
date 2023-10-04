//! A verifier is a piece of logic that determines whether an input can be consumed in a given context.
//! Because there are multiple reasonable ways to make this decision, we expose a trait to encapsulate
//! the various options. Each runtime will choose to make one or more verifiers available to its users
//! and they will be aggregated into an enum. The most common and useful verifiers are included here
//! with Tuxedo core, but downstream developers are expected to create their own as well.
//!

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::sr25519::{Public, Signature};
use sp_core::H256;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::fmt::Debug;
use sp_std::vec::Vec;

/// A means of checking that an output can be verified (aka spent). This check is made on a
/// per-output basis and neither knows nor cares anything about the validation logic that will
/// be applied to the transaction as a whole. Nonetheless, in order to avoid malleability, we
/// we take the entire stripped and serialized transaction as a parameter.
pub trait Verifier: Debug + Encode + Decode + Clone + TypeInfo {
    fn verify(&self, simplified_tx: &[u8], redeemer: &[u8]) -> bool;
}

/// A typical verifier that checks an sr25519 signature
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct UpForGrabs;

impl Verifier for UpForGrabs {
    fn verify(&self, _simplified_tx: &[u8], _redeemer: &[u8]) -> bool {
        true
    }
}

/// A Threshold multisignature. Some number of member signatories collectively own inputs
/// guarded by this verifier. A valid redeemer must supply valid signatures by at least
/// `threshold` of the signatories. If the threshold is greater than the number of signatories
/// the input can never be consumed.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct ThresholdMultiSignature {
    /// The minimum number of valid signatures needed to consume this input
    pub threshold: u8,
    /// All the member signatories, some (or all depending on the threshold) of whom must
    /// produce signatures over the transaction that will consume this input.
    /// This should include no duplicates
    pub signatories: Vec<H256>,
}

impl ThresholdMultiSignature {
    pub fn has_duplicate_signatories(&self) -> bool {
        let set: BTreeSet<_> = self.signatories.iter().collect();
        set.len() < self.signatories.len()
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// Combination of a signature plus and index so that the signer can specify which
/// index this signature pertains too of the available signatories for a `ThresholdMultiSignature`
pub struct SignatureAndIndex {
    /// The signature of the signer
    pub signature: Signature,
    /// The index of this signer in the signatory vector
    pub index: u8,
}

impl Verifier for ThresholdMultiSignature {
    fn verify(&self, simplified_tx: &[u8], redeemer: &[u8]) -> bool {
        if self.has_duplicate_signatories() {
            return false;
        }

        let sigs = match Vec::<SignatureAndIndex>::decode(&mut &redeemer[..]) {
            Ok(s) => s,
            Err(_) => return false,
        };

        if sigs.len() < self.threshold.into() {
            return false;
        }

        {
            // Check range of indicies
            let index_out_of_bounds = sigs.iter().any(|sig| sig.index as usize >= sigs.len());
            if index_out_of_bounds {
                return false;
            }
        }

        {
            let set: BTreeMap<u8, Signature> = sigs
                .iter()
                .map(|sig_and_index| (sig_and_index.index, sig_and_index.signature.clone()))
                .collect();

            if set.len() < sigs.len() {
                return false;
            }
        }

        let valid_sigs: Vec<_> = sigs
            .iter()
            .map(|sig| {
                sp_io::crypto::sr25519_verify(
                    &sig.signature,
                    simplified_tx,
                    &Public::from_h256(self.signatories[sig.index as usize]),
                );
            })
            .collect();

        valid_sigs.len() >= self.threshold.into()
    }
}

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
    fn verify(&self, _simplified_tx: &[u8], _redeemer: &[u8]) -> bool {
        self.verifies
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sp_core::{crypto::Pair as _, sr25519::Pair};

    /// Generate a bunch of test keypairs
    fn generate_n_pairs(n: u8) -> Vec<Pair> {
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
        assert!(UpForGrabs.verify(&[], &[]))
    }

    #[test]
    fn sig_check_with_good_sig() {
        let pair = Pair::from_seed(&[0u8; 32]);
        let simplified_tx = b"hello world".as_slice();
        let sig = pair.sign(simplified_tx);
        let redeemer: &[u8] = sig.as_ref();

        let sig_check = SigCheck {
            owner_pubkey: pair.public().into(),
        };

        assert!(sig_check.verify(simplified_tx, redeemer));
    }

    #[test]
    fn threshold_multisig_with_enough_sigs_passes() {
        let threshold = 2;
        let pairs = generate_n_pairs(threshold);

        let signatories: Vec<H256> = pairs.iter().map(|p| H256::from(p.public())).collect();

        let simplified_tx = b"hello_world".as_slice();
        let sigs: Vec<_> = pairs
            .iter()
            .enumerate()
            .map(|(i, p)| SignatureAndIndex {
                signature: p.sign(simplified_tx),
                index: i.try_into().unwrap(),
            })
            .collect();

        let redeemer: &[u8] = &sigs.encode()[..];
        let threshold_multisig = ThresholdMultiSignature {
            threshold,
            signatories,
        };

        assert!(threshold_multisig.verify(simplified_tx, redeemer));
    }

    #[test]
    fn threshold_multisig_not_enough_sigs_fails() {
        let threshold = 3;
        let pairs = generate_n_pairs(threshold);

        let signatories: Vec<H256> = pairs.iter().map(|p| H256::from(p.public())).collect();

        let simplified_tx = b"hello_world".as_slice();
        let sigs: Vec<_> = pairs
            .iter()
            .take(threshold as usize - 1)
            .enumerate()
            .map(|(i, p)| SignatureAndIndex {
                signature: p.sign(simplified_tx),
                index: i.try_into().unwrap(),
            })
            .collect();

        let redeemer: &[u8] = &sigs.encode()[..];
        let threshold_multisig = ThresholdMultiSignature {
            threshold,
            signatories,
        };

        assert!(!threshold_multisig.verify(simplified_tx, redeemer));
    }

    #[test]
    fn threshold_multisig_extra_sigs_still_passes() {
        let threshold = 2;
        let pairs = generate_n_pairs(threshold + 1);

        let signatories: Vec<H256> = pairs.iter().map(|p| H256::from(p.public())).collect();

        let simplified_tx = b"hello_world".as_slice();
        let sigs: Vec<_> = pairs
            .iter()
            .enumerate()
            .map(|(i, p)| SignatureAndIndex {
                signature: p.sign(simplified_tx),
                index: i.try_into().unwrap(),
            })
            .collect();

        let redeemer: &[u8] = &sigs.encode()[..];
        let threshold_multisig = ThresholdMultiSignature {
            threshold,
            signatories,
        };

        assert!(threshold_multisig.verify(simplified_tx, redeemer));
    }

    #[test]
    fn threshold_multisig_replay_sig_attack_fails() {
        let threshold = 2;
        let pairs = generate_n_pairs(threshold);

        let signatories: Vec<H256> = pairs.iter().map(|p| H256::from(p.public())).collect();

        let simplified_tx = b"hello_world".as_slice();

        let sigs: Vec<SignatureAndIndex> = vec![
            SignatureAndIndex {
                signature: pairs[0].sign(simplified_tx),
                index: 0.try_into().unwrap(),
            },
            SignatureAndIndex {
                signature: pairs[0].sign(simplified_tx),
                index: 0.try_into().unwrap(),
            },
        ];

        let redeemer: &[u8] = &sigs.encode()[..];
        let threshold_multisig = ThresholdMultiSignature {
            threshold,
            signatories,
        };

        assert!(!threshold_multisig.verify(simplified_tx, redeemer));
    }

    #[test]
    fn threshold_multisig_has_duplicate_signatories_fails() {
        let threshold = 2;
        let pairs = generate_n_pairs(threshold);

        let signatories: Vec<H256> =
            vec![H256::from(pairs[0].public()), H256::from(pairs[0].public())];

        let simplified_tx = b"hello_world".as_slice();

        let sigs: Vec<_> = pairs
            .iter()
            .enumerate()
            .map(|(i, p)| SignatureAndIndex {
                signature: p.sign(simplified_tx),
                index: i.try_into().unwrap(),
            })
            .collect();
        let redeemer: &[u8] = &sigs.encode()[..];

        let threshold_multisig = ThresholdMultiSignature {
            threshold,
            signatories,
        };

        assert!(!threshold_multisig.verify(simplified_tx, redeemer));
    }

    #[test]
    fn threshold_multisig_bogus_redeemer_encoding_fails() {
        use crate::dynamic_typing::testing::Bogus;

        let bogus = Bogus;

        let threshold_multisig = ThresholdMultiSignature {
            threshold: 3,
            signatories: vec![],
        };

        assert!(!threshold_multisig.verify(b"bogus_message".as_slice(), bogus.encode().as_slice()))
    }

    #[test]
    fn sig_check_with_bad_sig() {
        let simplified_tx = b"hello world".as_slice();
        let redeemer = b"bogus_signature".as_slice();

        let sig_check = SigCheck {
            owner_pubkey: H256::zero(),
        };

        assert!(!sig_check.verify(simplified_tx, redeemer));
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
