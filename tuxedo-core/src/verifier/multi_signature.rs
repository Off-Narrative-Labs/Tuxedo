//! This module contains a `Verifier` implementation that acts as an N of M multisig.
//! It also contains the necessary auxiliary types.

/// A Threshold multisignature. Some number of member signatories collectively own inputs
/// guarded by this verifier. A valid redeemer must supply valid signatures by at least
/// `threshold` of the signatories. If the threshold is greater than the number of signatories
/// the input can never be consumed.
use super::Verifier;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{
    sr25519::{Public, Signature},
    H256,
};
use sp_std::{
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
    vec::Vec,
};

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct ThresholdMultiSignature {
    /// The minimum number of valid signatures needed to consume this input
    pub threshold: u8,
    /// All the member signatories, some (or all depending on the threshold) of whom must
    /// produce signatures over the transaction that will consume this input.
    /// This should include no duplicates
    pub signatories: Vec<H256>,
}

impl ThresholdMultiSignature {
    pub fn new(threshold: u8, signatories: Vec<H256>) -> Self {
        ThresholdMultiSignature {
            threshold,
            signatories,
        }
    }

    pub fn has_duplicate_signatories(&self) -> bool {
        let set: BTreeSet<_> = self.signatories.iter().collect();
        set.len() < self.signatories.len()
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// Combination of a signature plus and index so that the signer can specify which
/// index this signature pertains too of the available signatories for a `ThresholdMultiSignature`
pub struct SignatureAndIndex {
    /// The signature of the signer
    pub signature: Signature,
    /// The index of this signer in the signatory vector
    pub index: u8,
}

impl Verifier for ThresholdMultiSignature {
    type Redeemer = Vec<SignatureAndIndex>;

    fn verify(&self, simplified_tx: &[u8], _: u32, sigs: &Vec<SignatureAndIndex>) -> bool {
        if self.has_duplicate_signatories() {
            return false;
        }

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
                .map(|sig_and_index| (sig_and_index.index, sig_and_index.signature))
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

    fn new_unspendable() -> Option<Self> {
        Some(Self {
            threshold: 1,
            signatories: Vec::new(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::verifier::test::generate_n_pairs;
    use sp_core::crypto::Pair as _;

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

        let threshold_multisig = ThresholdMultiSignature {
            threshold,
            signatories,
        };

        assert!(threshold_multisig.verify(simplified_tx, 0, &sigs));
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

        let threshold_multisig = ThresholdMultiSignature {
            threshold,
            signatories,
        };

        assert!(!threshold_multisig.verify(simplified_tx, 0, &sigs));
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

        let threshold_multisig = ThresholdMultiSignature {
            threshold,
            signatories,
        };

        assert!(threshold_multisig.verify(simplified_tx, 0, &sigs));
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

        let threshold_multisig = ThresholdMultiSignature {
            threshold,
            signatories,
        };

        assert!(!threshold_multisig.verify(simplified_tx, 0, &sigs));
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

        let threshold_multisig = ThresholdMultiSignature {
            threshold,
            signatories,
        };

        assert!(!threshold_multisig.verify(simplified_tx, 0, &sigs));
    }
}
