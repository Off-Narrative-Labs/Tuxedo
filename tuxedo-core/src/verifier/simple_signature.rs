//! This module contains `Verifier` implementations for simple signature checking.
//! This is the most common way to implement private ownership in a UTXO chain and will
//! likely be used by most chains.
//!
//! Directly locking a UTXO to a public key is supported as well as locking behind a
//! public key hash like bitcoin's P2PKH. For the merits of each approach see:
//! https://bitcoin.stackexchange.com/q/72184
//!
//! Currently there are only implementations for SR25519 signatures that makes use of
//! Substrate's host functions to do the actual cryptography. Other signature schemes or
//! pure wasm implementations are also welcome here.

/// A very commonly used verifier that checks an sr25519 signature.
///
/// This verifier relies on Substrate's host functions to perform the signature checking
/// natively and gain performance.
use super::Verifier;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{
    sr25519::{Public, Signature},
    H256,
};
use sp_runtime::traits::{BlakeTwo256, Hash};

/// Require a signature from the private key corresponding to the given public key.
/// This is the simplest way to require a signature. If you prefer not to expose the
/// public key until spend time, use P2PKH instead.
///
/// Uses the Sr25519 signature scheme and Substrate's host functions.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct Sr25519Signature {
    pub owner_pubkey: H256,
}

impl Sr25519Signature {
    /// Create a new instance that requires a signature from the given public key
    pub fn new<T: Into<H256>>(owner_pubkey: T) -> Self {
        Sr25519Signature {
            owner_pubkey: owner_pubkey.into(),
        }
    }
}

impl Verifier for Sr25519Signature {
    type Redeemer = Signature;

    fn verify(&self, simplified_tx: &[u8], _: u32, sig: &Signature) -> bool {
        sp_io::crypto::sr25519_verify(sig, simplified_tx, &Public::from_h256(self.owner_pubkey))
    }

    fn new_unspendable() -> Option<Self> {
        Some(Self::new(H256::zero()))
    }
}

/// Pay To Public Key Hash (P2PKH)
///
/// Require a signature from the private key corresponding to the public key whose _hash_ is given.
/// This is the most common way to represent private ownership in UTXO networks like Bitcoin.
/// It is more complex than providing the public key directly but does not reveal the public key until spend time.
///
/// Uses the Sr25519 signature scheme and BlakeTwo256 hashing algorithm via Substrate's host functions.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct P2PKH {
    pub owner_pubkey_hash: H256,
}

impl Verifier for P2PKH {
    type Redeemer = (Public, Signature);

    fn verify(&self, simplified_tx: &[u8], _: u32, (pubkey, signature): &Self::Redeemer) -> bool {
        BlakeTwo256::hash(pubkey) == self.owner_pubkey_hash
            && sp_io::crypto::sr25519_verify(signature, simplified_tx, pubkey)
    }

    fn new_unspendable() -> Option<Self> {
        Some(Self {
            owner_pubkey_hash: H256::zero(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sp_core::{crypto::Pair as _, sr25519::Pair, ByteArray};

    fn bad_sig() -> Signature {
        Signature::from_slice(
            b"bogus_signature_bogus_signature_bogus_signature_bogus_signature!".as_slice(),
        )
        .expect("Should be able to create a bogus signature.")
    }

    #[test]
    fn sr25519_signature_with_good_sig() {
        let pair = Pair::from_seed(&[0u8; 32]);
        let simplified_tx = b"hello world".as_slice();
        let sig = pair.sign(simplified_tx);

        let sr25519_signature = Sr25519Signature {
            owner_pubkey: pair.public().into(),
        };

        assert!(sr25519_signature.verify(simplified_tx, 0, &sig));
    }

    #[test]
    fn sr25519_signature_with_bad_sig() {
        let simplified_tx = b"hello world".as_slice();
        let sr25519_signature = Sr25519Signature {
            owner_pubkey: H256::zero(),
        };

        assert!(!sr25519_signature.verify(simplified_tx, 0, &bad_sig()));
    }

    #[test]
    fn p2pkh_success() {
        let pair = Pair::from_seed(&[0u8; 32]);
        let owner_pubkey_hash = BlakeTwo256::hash(&pair.public());
        let simplified_tx = b"hello world".as_slice();
        let sig = pair.sign(simplified_tx);

        let p2pkh = P2PKH { owner_pubkey_hash };

        assert!(p2pkh.verify(simplified_tx, 0, &(pair.public(), sig)));
    }

    #[test]
    fn p2pkh_correct_pubkey_bad_sig() {
        let pair = Pair::from_seed(&[0u8; 32]);
        let owner_pubkey_hash = BlakeTwo256::hash(&pair.public());
        let simplified_tx = b"hello world".as_slice();

        let p2pkh = P2PKH { owner_pubkey_hash };

        assert!(!p2pkh.verify(simplified_tx, 0, &(pair.public(), bad_sig())));
    }

    #[test]
    fn p2pkh_incorrect_pubkey_but_valid_sig_from_provided_pubkey() {
        let owner_pair = Pair::from_seed(&[0u8; 32]);
        let owner_pubkey_hash = BlakeTwo256::hash(&owner_pair.public());
        let simplified_tx = b"hello world".as_slice();

        let p2pkh = P2PKH { owner_pubkey_hash };

        let attacker_pair = Pair::from_seed(&[1u8; 32]);
        let attacker_sig = attacker_pair.sign(simplified_tx);

        assert!(!p2pkh.verify(simplified_tx, 0, &(attacker_pair.public(), attacker_sig)));
    }

    #[test]
    fn p2pkh_incorrect_pubkey_and_bogus_sig() {
        let owner_pair = Pair::from_seed(&[0u8; 32]);
        let owner_pubkey_hash = BlakeTwo256::hash(&owner_pair.public());
        let simplified_tx = b"hello world".as_slice();

        let p2pkh = P2PKH { owner_pubkey_hash };

        let attacker_pair = Pair::from_seed(&[1u8; 32]);

        assert!(!p2pkh.verify(simplified_tx, 0, &(attacker_pair.public(), bad_sig())));
    }
}
