use cumulus_primitives_core::{ParachainBlockData, PersistedValidationData};
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use parity_scale_codec::{Decode, DecodeAll, Encode};
use sp_keyring::AccountKeyring::*;
use sp_runtime::traits::Header as HeaderT;
use std::{env, process::Command};

use crate::{validate_block::MemoryOptimizedValidationParams, types::Transaction};

use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::transaction_validity::ValidTransactionBuilder;

use crate::{
    constraint_checker::testing::TestConstraintChecker,
    dynamic_typing::{testing::Bogus, UtxoData},
    types::{Input, Output},
    verifier::TestVerifier,
};

use super::*;

type TestTransaction = Transaction<TestVerifier, TestConstraintChecker>;
pub type TestHeader = sp_runtime::generic::Header<u32, BlakeTwo256>;
pub type TestBlock = sp_runtime::generic::Block<TestHeader, TestTransaction>;
pub type TestExecutive = Executive<TestBlock, TestVerifier, TestConstraintChecker>;

//TODO right now this is no extrinsics at all.
// It currently doesn't actually check the presence of inherents yet
// Eventually I'll need to fix that and also fix this test accordingly.
#[test]
fn validate_block_no_extra_extrinsics() {
    
    let simple_genesis_header: TestHeader = Header {
        parent_hash: Default::default(),
        number: 0,
        state_root: Default::default(),
        extrinsics_root: Default::default(),
        digest: Default::default(),
    };

    let params = MemoryOptimizedValidationParams {
        parent_head: todo!(),
        block_data: todo!(),
        relay_parent_number: 0,
        relay_parent_storage_root: todo!(),
    };

    let res = super::implementation::validate_block::<TestBlock, TestVerifier, TestConstraintChecker>(params);

    println!(
        "Result:\nHead data: {:?}\nNew validation code: {:?}\n, upward messages:{:?}",
        res.head_data,
        res.new_validation_code,
        res.upward_messages,
    );

    panic!("Reached end of test");
}

// #[test]
// fn validate_block_with_extra_extrinsics() {
//     sp_tracing::try_init_simple();

//     let (client, parent_head) = create_test_client();
//     let extra_extrinsics = vec![
//         transfer(&client, Alice, Bob, 69),
//         transfer(&client, Bob, Charlie, 100),
//         transfer(&client, Charlie, Alice, 500),
//     ];

//     let TestBlockData {
//         block,
//         validation_data,
//     } = build_block_with_witness(
//         &client,
//         extra_extrinsics,
//         parent_head.clone(),
//         Default::default(),
//     );
//     let header = block.header().clone();

//     let res_header = call_validate_block(
//         parent_head,
//         block,
//         validation_data.relay_parent_storage_root,
//     )
//     .expect("Calls `validate_block`");
//     assert_eq!(header, res_header);
// }

// #[test]
// fn validate_block_returns_custom_head_data() {
//     sp_tracing::try_init_simple();

//     let expected_header = vec![1, 3, 3, 7, 4, 5, 6];

//     let (client, parent_head) = create_test_client();
//     let extra_extrinsics = vec![
//         transfer(&client, Alice, Bob, 69),
//         generate_extrinsic(
//             &client,
//             Charlie,
//             TestPalletCall::set_custom_validation_head_data {
//                 custom_header: expected_header.clone(),
//             },
//         ),
//         transfer(&client, Bob, Charlie, 100),
//     ];

//     let TestBlockData {
//         block,
//         validation_data,
//     } = build_block_with_witness(
//         &client,
//         extra_extrinsics,
//         parent_head.clone(),
//         Default::default(),
//     );
//     let header = block.header().clone();
//     assert_ne!(expected_header, header.encode());

//     let res_header = call_validate_block_encoded_header(
//         parent_head,
//         block,
//         validation_data.relay_parent_storage_root,
//     )
//     .expect("Calls `validate_block`");
//     assert_eq!(expected_header, res_header);
// }

// #[test]
// fn validate_block_invalid_parent_hash() {
//     sp_tracing::try_init_simple();

//     if env::var("RUN_TEST").is_ok() {
//         let (client, parent_head) = create_test_client();
//         let TestBlockData {
//             block,
//             validation_data,
//         } = build_block_with_witness(&client, Vec::new(), parent_head.clone(), Default::default());
//         let (mut header, extrinsics, witness) = block.deconstruct();
//         header.set_parent_hash(Hash::from_low_u64_be(1));

//         let block_data = ParachainBlockData::new(header, extrinsics, witness);
//         call_validate_block(
//             parent_head,
//             block_data,
//             validation_data.relay_parent_storage_root,
//         )
//         .unwrap_err();
//     } else {
//         let output = Command::new(env::current_exe().unwrap())
//             .args(["validate_block_invalid_parent_hash", "--", "--nocapture"])
//             .env("RUN_TEST", "1")
//             .output()
//             .expect("Runs the test");
//         assert!(output.status.success());

//         assert!(dbg!(String::from_utf8(output.stderr).unwrap()).contains("Invalid parent hash"));
//     }
// }

// #[test]
// fn validate_block_fails_on_invalid_validation_data() {
//     sp_tracing::try_init_simple();

//     if env::var("RUN_TEST").is_ok() {
//         let (client, parent_head) = create_test_client();
//         let TestBlockData { block, .. } =
//             build_block_with_witness(&client, Vec::new(), parent_head.clone(), Default::default());

//         call_validate_block(parent_head, block, Hash::random()).unwrap_err();
//     } else {
//         let output = Command::new(env::current_exe().unwrap())
//             .args([
//                 "validate_block_fails_on_invalid_validation_data",
//                 "--",
//                 "--nocapture",
//             ])
//             .env("RUN_TEST", "1")
//             .output()
//             .expect("Runs the test");
//         assert!(output.status.success());

//         assert!(dbg!(String::from_utf8(output.stderr).unwrap())
//             .contains("Relay parent storage root doesn't match"));
//     }
// }

// #[test]
// fn check_inherents_are_unsigned_and_before_all_other_extrinsics() {
//     sp_tracing::try_init_simple();

//     if env::var("RUN_TEST").is_ok() {
//         let (client, parent_head) = create_test_client();

//         let TestBlockData {
//             block,
//             validation_data,
//         } = build_block_with_witness(&client, Vec::new(), parent_head.clone(), Default::default());

//         let (header, mut extrinsics, proof) = block.deconstruct();

//         extrinsics.insert(0, transfer(&client, Alice, Bob, 69));

//         call_validate_block(
//             parent_head,
//             ParachainBlockData::new(header, extrinsics, proof),
//             validation_data.relay_parent_storage_root,
//         )
//         .unwrap_err();
//     } else {
//         let output = Command::new(env::current_exe().unwrap())
//             .args([
//                 "check_inherents_are_unsigned_and_before_all_other_extrinsics",
//                 "--",
//                 "--nocapture",
//             ])
//             .env("RUN_TEST", "1")
//             .output()
//             .expect("Runs the test");
//         assert!(output.status.success());

//         assert!(String::from_utf8(output.stderr)
//             .unwrap()
//             .contains("Could not find `set_validation_data` inherent"));
//     }
// }

// /// Test that ensures that `ValidationParams` and `MemoryOptimizedValidationParams`
// /// are encoding/decoding.
// #[test]
// fn validation_params_and_memory_optimized_validation_params_encode_and_decode() {
//     const BLOCK_DATA: &[u8] = &[1, 2, 3, 4, 5];
//     const PARENT_HEAD: &[u8] = &[1, 3, 4, 5, 6, 7, 9];

//     let validation_params = ValidationParams {
//         block_data: BlockData(BLOCK_DATA.encode()),
//         parent_head: HeadData(PARENT_HEAD.encode()),
//         relay_parent_number: 1,
//         relay_parent_storage_root: Hash::random(),
//     };

//     let encoded = validation_params.encode();

//     let decoded = MemoryOptimizedValidationParams::decode_all(&mut &encoded[..]).unwrap();
//     assert_eq!(
//         decoded.relay_parent_number,
//         validation_params.relay_parent_number
//     );
//     assert_eq!(
//         decoded.relay_parent_storage_root,
//         validation_params.relay_parent_storage_root
//     );
//     assert_eq!(decoded.block_data, validation_params.block_data.0);
//     assert_eq!(decoded.parent_head, validation_params.parent_head.0);

//     let encoded = decoded.encode();

//     let decoded = ValidationParams::decode_all(&mut &encoded[..]).unwrap();
//     assert_eq!(decoded, validation_params);
// }
