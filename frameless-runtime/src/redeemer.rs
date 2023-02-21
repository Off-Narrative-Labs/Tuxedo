//! A redeemer is a piece of logic that determines whether an input can be consumed in a given context.
//! Because there are multiple reasonable ways to make this decision, we expose a trait to encapsulate
//! the various options. Each runtime will choose to make one or more redeemers available to its users
//! and they will be aggregated into an enum. The most common and useful redeemers are included here
//! with Tuxedo core, but downstream developers are expected to create their own as well.

use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_application_crypto::sr25519::{Public, Signature};
use sp_core::H256;

/// A means of checking that an output can be redeemed (aka spent). This check is made on a
/// per-output basis and neither knows nor cares anything about the verification logic that will
/// be applied to the transaction as a whole. Nonetheless, in order to avoid malleability, we
/// we take the entire stripped and serialized transaction as a parameter.
pub trait Redeemer {
    fn redeem(&self, simplified_tx: &[u8], witness: &[u8]) -> bool;
}

/// A typical redeemer that checks an sr25519 signature
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct SigCheck {
    owner_pubkey: H256,
}

impl Redeemer for SigCheck {
    fn redeem(&self, simplified_tx: &[u8], witness: &[u8]) -> bool {
        let sig = match Signature::try_from(&witness[..]) {
            Ok(s) => s,
            Err(_) => return false,
        };

        sp_io::crypto::sr25519_verify(&sig, &simplified_tx, &Public::from_h256(self.owner_pubkey))
    }
}

/// A simple redeemer that allows anyone to consume an output at any time
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct UpForGrabs;

impl Redeemer for UpForGrabs {
    fn redeem(&self, _simplified_tx: &[u8], _witness: &[u8]) -> bool {
        true
    }
}
