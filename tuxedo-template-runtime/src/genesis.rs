//! Helper module to build a genesis configuration for the template runtime.

#[cfg(feature = "std")]
pub use super::WASM_BINARY;
use super::{
    kitties::{KittyData, Parent},
    money::Coin,
    OuterConstraintChecker, Transaction,
};
use hex_literal::hex;
use sp_std::{vec, vec::Vec};
use tuxedo_core::{
    verifier::{Sr25519Signature, ThresholdMultiSignature, UpForGrabs},
    ConstraintChecker,
};

const SHAWN_PUB_KEY_BYTES: [u8; 32] =
    hex!("d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67");
const ANDREW_PUB_KEY_BYTES: [u8; 32] =
    hex!("baa81e58b1b4d053c2e86d93045765036f9d265c7dfe8b9693bbc2c0f048d93a");

/// This function returns a list of valid transactions to be included in the genesis block.
/// It is called by the `ChainSpec::build` method, via the `development_genesis_config` function.
/// The resulting transactions must be ordered: inherent first, then extrinsics.
pub fn development_genesis_transactions() -> Vec<Transaction> {
    let signatories = vec![SHAWN_PUB_KEY_BYTES.into(), ANDREW_PUB_KEY_BYTES.into()];

    // The inherents are computed using the appropriate method, and placed before the extrinsics.
    let mut genesis_transactions = OuterConstraintChecker::genesis_transactions();

    genesis_transactions.extend([
        // Money Transactions
        Coin::<0>::mint(100, Sr25519Signature::new(SHAWN_PUB_KEY_BYTES)),
        Coin::<0>::mint(100, ThresholdMultiSignature::new(1, signatories)),
        // Kitty Transactions
        KittyData::mint(Parent::mom(), b"mother", UpForGrabs),
        KittyData::mint(Parent::dad(), b"father", UpForGrabs),
        // TODO: Initial Transactions for Existence
    ]);

    genesis_transactions
}

pub fn development_genesis_config() -> serde_json::Value {
    serde_json::json!(development_genesis_transactions())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::OuterVerifier;
    use parity_scale_codec::{Decode, Encode};
    use sp_api::HashT;
    use sp_core::testing::SR25519;
    use sp_keystore::{testing::MemoryKeystore, Keystore, KeystoreExt};
    use sp_runtime::traits::BlakeTwo256;
    use std::sync::Arc;
    use tuxedo_core::{
        dynamic_typing::{DynamicallyTypedData, UtxoData},
        genesis::TuxedoGenesisConfigBuilder,
        types::{Output, OutputRef},
    };

    // other random account generated with subkey
    const SHAWN_PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    const ANDREW_PHRASE: &str =
        "monkey happy total rib lumber scrap guide photo country online rose diet";

    fn default_runtime_genesis_config() -> RuntimeGenesisConfig {
        let keystore = MemoryKeystore::new();

        let shawn_pub_key_bytes = keystore
            .sr25519_generate_new(SR25519, Some(SHAWN_PHRASE))
            .unwrap()
            .0;

        let andrew_pub_key_bytes = keystore
            .sr25519_generate_new(SR25519, Some(ANDREW_PHRASE))
            .unwrap()
            .0;

        let signatories = vec![shawn_pub_key_bytes.into(), andrew_pub_key_bytes.into()];

        let mut genesis_transactions = OuterConstraintChecker::genesis_transactions();
        genesis_transactions.extend([
            // Money Transactions
            Coin::<0>::mint(100, Sr25519Signature::new(shawn_pub_key_bytes)),
            Coin::<0>::mint(100, ThresholdMultiSignature::new(1, signatories)),
        ]);

        RuntimeGenesisConfig::new(
            WASM_BINARY
                .expect("Runtime WASM binary must exist.")
                .to_vec(),
            genesis_transactions,
        )
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let keystore = MemoryKeystore::new();
        let mut ext = sp_io::TestExternalities::new_empty();
        ext.register_extension(KeystoreExt(Arc::new(keystore)));

        ext.execute_with(|| {
            TuxedoGenesisConfigBuilder::build(development_genesis_transactions())
                .expect("Genesis Config Build Failed")
        });
        ext
    }

    #[test]
    fn genesis_utxo_money() {
        new_test_ext().execute_with(|| {
            let keystore = MemoryKeystore::new();
            let shawn_pub_key = keystore
                .sr25519_generate_new(SR25519, Some(SHAWN_PHRASE))
                .unwrap();

            // Grab genesis value from storage and assert it is correct
            let genesis_utxo = Output {
                verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
                    owner_pubkey: shawn_pub_key.into(),
                }),
                payload: DynamicallyTypedData {
                    data: 100u128.encode(),
                    type_id: <money::Coin<0> as UtxoData>::TYPE_ID,
                },
            };

            let inherents_len =
                OuterConstraintChecker::genesis_transactions::<OuterVerifier>().len();

            let tx = development_genesis_transactions()
                .get(inherents_len)
                .unwrap()
                .clone();

            assert_eq!(tx.outputs.first(), Some(&genesis_utxo));

            let tx_hash = BlakeTwo256::hash_of(&tx.encode());
            let output_ref = OutputRef {
                tx_hash,
                index: 0_u32,
            };

            let encoded_utxo =
                sp_io::storage::get(&output_ref.encode()).expect("Retrieve Genesis UTXO");
            let utxo = Output::decode(&mut &encoded_utxo[..]).expect("Can Decode UTXO correctly");
            assert_eq!(utxo, genesis_utxo);
        })
    }

    #[test]
    fn genesis_utxo_money_multi_sig() {
        new_test_ext().execute_with(|| {
            let keystore = MemoryKeystore::new();
            let shawn_pub_key = keystore
                .sr25519_generate_new(SR25519, Some(SHAWN_PHRASE))
                .unwrap();
            let andrew_pub_key = keystore
                .sr25519_generate_new(SR25519, Some(ANDREW_PHRASE))
                .unwrap();

            let genesis_multi_sig_utxo = Output {
                verifier: OuterVerifier::ThresholdMultiSignature(ThresholdMultiSignature {
                    threshold: 1,
                    signatories: vec![shawn_pub_key.into(), andrew_pub_key.into()],
                }),
                payload: DynamicallyTypedData {
                    data: 100u128.encode(),
                    type_id: <money::Coin<0> as UtxoData>::TYPE_ID,
                },
            };

            let inherents_len =
                OuterConstraintChecker::genesis_transactions::<OuterVerifier>().len();

            let tx = development_genesis_transactions()
                .get(1 + inherents_len)
                .unwrap()
                .clone();

            assert_eq!(tx.outputs.first(), Some(&genesis_multi_sig_utxo));

            let tx_hash = BlakeTwo256::hash_of(&tx.encode());
            let output_ref = OutputRef {
                tx_hash,
                index: 0_u32,
            };

            let encoded_utxo =
                sp_io::storage::get(&output_ref.encode()).expect("Retrieve Genesis MultiSig UTXO");
            let utxo = Output::decode(&mut &encoded_utxo[..]).expect("Can Decode UTXO correctly");
            assert_eq!(utxo, genesis_multi_sig_utxo);
        })
    }
}
