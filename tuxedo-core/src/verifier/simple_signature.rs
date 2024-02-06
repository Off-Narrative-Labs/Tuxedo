//! This module contains `Verifier` implementations for simple signature checking.
//! This is the most common way to implement private ownership in a UTXO chain and will
//! likely be used by most chains.
//! 
//! Currently there is only an implementation for SR25519 signatures that makes use of
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
use sp_core::{H256, sr25519::{Public, Signature}};

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
    fn verify(&self, simplified_tx: &[u8], _: u32, redeemer: &[u8]) -> bool {
        let sig = match Signature::try_from(redeemer) {
            Ok(s) => s,
            Err(_) => return false,
        };

        sp_io::crypto::sr25519_verify(&sig, simplified_tx, &Public::from_h256(self.owner_pubkey))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sp_core::{crypto::Pair as _, sr25519::Pair};

    #[test]
    fn sr25519_signature_with_good_sig() {
        let pair = Pair::from_seed(&[0u8; 32]);
        let simplified_tx = b"hello world".as_slice();
        let sig = pair.sign(simplified_tx);
        let redeemer: &[u8] = sig.as_ref();

        let sr25519_signature = Sr25519Signature {
            owner_pubkey: pair.public().into(),
        };

        assert!(sr25519_signature.verify(simplified_tx, 0, redeemer));
    }


    #[test]
    fn sr25519_signature_with_bad_sig() {
        let simplified_tx = b"hello world".as_slice();
        let redeemer = b"bogus_signature".as_slice();

        let sr25519_signature = Sr25519Signature {
            owner_pubkey: H256::zero(),
        };

        assert!(!sr25519_signature.verify(simplified_tx, 0, redeemer));
    }

}