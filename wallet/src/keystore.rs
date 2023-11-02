//! Wallet's local keystore.
//!
//! This is a thin wrapper around sc-cli for use in tuxedo wallets

use anyhow::anyhow;
use parity_scale_codec::Encode;
use sc_keystore::LocalKeystore;
use sp_core::{
    crypto::Pair as PairT,
    sr25519::{Pair, Public},
    H256,
};
use sp_keystore::Keystore;
use sp_runtime::KeyTypeId;
use std::path::Path;

/// A KeyTypeId to use in the keystore for Tuxedo transactions. We'll use this everywhere
/// until it becomes clear that there is a reason to use multiple of them
const KEY_TYPE: KeyTypeId = KeyTypeId(*b"_tux");

/// A default seed phrase for signing inputs when none is provided
/// Corresponds to the default pubkey.
pub const SHAWN_PHRASE: &str =
    "news slush supreme milk chapter athlete soap sausage put clutch what kitten";

/// The public key corresponding to the default seed above.
pub const SHAWN_PUB_KEY: &str = "d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67";

/// Insert the example "Shawn" key into the keystore for the current session only.
pub fn insert_development_key_for_this_session(keystore: &LocalKeystore) -> anyhow::Result<()> {
    keystore.sr25519_generate_new(KEY_TYPE, Some(SHAWN_PHRASE))?;
    Ok(())
}

/// Sign a given message with the private key that corresponds to the given public key.
///
/// Returns an error if the keystore itself errors, or does not contain the requested key.
pub fn sign_with(
    keystore: &LocalKeystore,
    public: &Public,
    message: &[u8],
) -> anyhow::Result<Vec<u8>> {
    let sig = keystore
        .sr25519_sign(KEY_TYPE, public, message)?
        .ok_or(anyhow!("Key doesn't exist in keystore"))?;

    Ok(sig.encode())
}

/// Insert the private key associated with the given seed into the keystore for later use.
pub fn insert_key(keystore: &LocalKeystore, seed: &str) -> anyhow::Result<()> {
    // We need to provide a public key to the keystore manually, so let's calculate it.
    let public_key = Pair::from_phrase(seed, None)?.0.public();
    println!("The generated public key is {:?}", public_key);
    keystore
        .insert(KEY_TYPE, seed, public_key.as_ref())
        .map_err(|()| anyhow!("Error inserting key"))?;
    Ok(())
}

/// Generate a new key from system entropy and insert it into the keystore, optionally
/// protected by a password.
///
/// TODO there is no password support when using keys later when signing.
pub fn generate_key(keystore: &LocalKeystore, password: Option<String>) -> anyhow::Result<()> {
    let (pair, phrase, _) = Pair::generate_with_phrase(password.as_deref());
    println!("Generated public key is {:?}", pair.public());
    println!("Generated Phrase is {}", phrase);
    keystore
        .insert(KEY_TYPE, phrase.as_ref(), pair.public().as_ref())
        .map_err(|()| anyhow!("Error inserting key"))?;
    Ok(())
}

/// Check whether a specific key is in the keystore
pub fn has_key(keystore: &LocalKeystore, pubkey: &H256) -> bool {
    keystore.has_keys(&[(pubkey.encode(), KEY_TYPE)])
}

pub fn get_keys(keystore: &LocalKeystore) -> anyhow::Result<impl Iterator<Item = Vec<u8>>> {
    Ok(keystore.keys(KEY_TYPE)?.into_iter())
}

/// Caution. Removes key from keystore. Call with care.
pub fn remove_key(keystore_path: &Path, pub_key: &H256) -> anyhow::Result<()> {
    // The keystore doesn't provide an API for removing keys, so we
    // remove them from the filesystem directly
    let filename = format!("{}{}", hex::encode(KEY_TYPE.0), hex::encode(pub_key.0));
    let path = keystore_path.join(filename);

    std::fs::remove_file(path)?;

    Ok(())
}
