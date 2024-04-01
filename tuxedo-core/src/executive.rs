//! # Executive Module
//!
//! The executive is the main orchestrator for the entire runtime.
//! It has functions that implement the Core, BlockBuilder, and TxPool runtime APIs.
//!
//! It does all the reusable verification of UTXO transactions such as checking that there
//! are no duplicate inputs, and that the verifiers are satisfied.

use crate::{
    constraint_checker::ConstraintChecker,
    ensure,
    inherents::{InherentInternal, PARENT_INHERENT_IDENTIFIER},
    types::{DispatchResult, OutputRef, Transaction, UtxoError},
    utxo_set::TransparentUtxoSet,
    verifier::Verifier,
    EXTRINSIC_KEY, HEADER_KEY, LOG_TARGET,
};
use log::debug;
use parity_scale_codec::{Decode, Encode};
use sp_api::{BlockT, HashT, HeaderT, TransactionValidity};
use sp_core::H256;
use sp_inherents::{CheckInherentsResult, InherentData};
use sp_runtime::{
    traits::BlakeTwo256,
    transaction_validity::{
        InvalidTransaction, TransactionLongevity, TransactionSource, TransactionValidityError,
        ValidTransaction,
    },
    ApplyExtrinsicResult, StateVersion,
};
use sp_std::marker::PhantomData;
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

/// The executive. Each runtime is encouraged to make a type alias called `Executive` that fills
/// in the proper generic types.
pub struct Executive<B, V, C>(PhantomData<(B, V, C)>);

impl<B: BlockT<Extrinsic = Transaction<V, C>>, V: Verifier, C: ConstraintChecker<V>>
    Executive<B, V, C>
{
    /// Does pool-style validation of a tuxedo transaction.
    /// Does not commit anything to storage.
    /// This returns Ok even if some inputs are still missing because the tagged transaction pool can handle that.
    /// We later check that there are no missing inputs in `apply_tuxedo_transaction`
    pub fn validate_tuxedo_transaction(
        transaction: &Transaction<V, C>,
    ) -> Result<ValidTransaction, UtxoError<C::Error>> {
        log::info!(
            target: LOG_TARGET,
            "validating tuxedo transaction",
        );

        // Make sure there are no duplicate inputs
        // Duplicate peeks are allowed, although they are inefficient and wallets should not create such transactions
        {
            let input_set: BTreeSet<_> = transaction.inputs.iter().map(|o| o.encode()).collect();
            log::info!(
                target: LOG_TARGET,
                "input_set.len() {} and transaction.inputs.len()  {}",input_set.len(),
                transaction.inputs.len()
            );
            ensure!(
                input_set.len() == transaction.inputs.len(),
                UtxoError::DuplicateInput
            );
        }

        // Build the stripped transaction (with the redeemers stripped) and encode it
        // This will be passed to the verifiers
        let mut stripped = transaction.clone();
        for input in stripped.inputs.iter_mut() {
            input.redeemer = Vec::new();
        }
        let stripped_encoded = stripped.encode();

        // Check that the verifiers of all inputs are satisfied
        // Keep a Vec of the input utxos for passing to the constraint checker
        // Keep track of any missing inputs for use in the tagged transaction pool
        let mut input_utxos = Vec::new();
        let mut missing_inputs = Vec::new();
        for input in transaction.inputs.iter() {
            if let Some(input_utxo) = TransparentUtxoSet::<V>::peek_utxo(&input.output_ref) {
                ensure!(
                    input_utxo
                        .verifier
                        .verify(&stripped_encoded, &input.redeemer),
                    UtxoError::VerifierError
                );
                input_utxos.push(input_utxo);
            } else {
                missing_inputs.push(input.output_ref.clone().encode());
            }
        }

        // Make a Vec of the peek utxos for passing to the constraint checker
        // Keep track of any missing peeks for use in the tagged transaction pool
        // Use the same vec as previously to keep track of missing peeks
        let mut peek_utxos = Vec::new();
        for output_ref in transaction.peeks.iter() {
            if let Some(peek_utxo) = TransparentUtxoSet::<V>::peek_utxo(output_ref) {
                peek_utxos.push(peek_utxo);
            } else {
                missing_inputs.push(output_ref.encode());
            }
        }

        // Make sure no outputs already exist in storage
        let tx_hash = BlakeTwo256::hash_of(&transaction.encode());
        for index in 0..transaction.outputs.len() {
            let output_ref = OutputRef {
                tx_hash,
                index: index as u32,
            };

            log::info!(
                target: LOG_TARGET,
                "Checking for pre-existing output {:?}", output_ref
            );

            ensure!(
                TransparentUtxoSet::<V>::peek_utxo(&output_ref).is_none(),
                UtxoError::PreExistingOutput
            );
        }

        // Calculate the tx-pool tags provided by this transaction, which
        // are just the encoded OutputRefs
        let provides = (0..transaction.outputs.len())
            .map(|i| {
                let output_ref = OutputRef {
                    tx_hash,
                    index: i as u32,
                };
                output_ref.encode()
            })
            .collect::<Vec<_>>();

        // If any of the inputs are missing, we cannot make any more progress
        // If they are all present, we may proceed to call the constraint checker
        if !missing_inputs.is_empty() {
            log::info!(
                target: LOG_TARGET,
                "Transaction is valid but still has missing inputs. Returning early.",
            );
            return Ok(ValidTransaction {
                requires: missing_inputs,
                provides,
                priority: 0,
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            });
        }

        // Call the constraint checker
        transaction
            .checker
            .check(&input_utxos, &peek_utxos, &transaction.outputs)
            .map_err(UtxoError::ConstraintCheckerError)?;

        // Return the valid transaction
        Ok(ValidTransaction {
            requires: Vec::new(),
            provides,
            priority: 0,
            longevity: TransactionLongevity::max_value(),
            propagate: true,
        })
    }

    /// Does full verification and application of tuxedo transactions.
    /// Most of the validation happens in the call to `validate_tuxedo_transaction`.
    /// Once those checks are done we make sure there are no missing inputs and then update storage.
    pub fn apply_tuxedo_transaction(transaction: Transaction<V, C>) -> DispatchResult<C::Error> {
        debug!(
            target: LOG_TARGET,
            "applying tuxedo transaction {:?}", transaction
        );

        // Re-do the pre-checks. These should have been done in the pool, but we can't
        // guarantee that foreign nodes to these checks faithfully, so we need to check on-chain.
        let valid_transaction = Self::validate_tuxedo_transaction(&transaction)?;

        // If there are still missing inputs, we cannot execute this,
        // although it would be valid in the pool
        ensure!(
            valid_transaction.requires.is_empty(),
            UtxoError::MissingInput
        );

        // At this point, all validation is complete, so we can commit the storage changes.
        Self::update_storage(transaction);

        Ok(())
    }

    /// Helper function to update the utxo set according to the given transaction.
    /// This function does absolutely no validation. It assumes that the transaction
    /// has already passed validation. Changes proposed by the transaction are written
    /// blindly to storage.
    fn update_storage(transaction: Transaction<V, C>) {
        // Remove verified UTXOs
        for input in &transaction.inputs {
            TransparentUtxoSet::<V>::consume_utxo(&input.output_ref);
        }

        debug!(
            target: LOG_TARGET,
            "Transaction before updating storage {:?}", transaction
        );
        // Write the newly created utxos
        for (index, output) in transaction.outputs.iter().enumerate() {
            let output_ref = OutputRef {
                tx_hash: BlakeTwo256::hash_of(&transaction.encode()),
                index: index as u32,
            };
            TransparentUtxoSet::<V>::store_utxo(output_ref, output);
        }
    }

    /// A helper function that allows tuxedo runtimes to read the current block height
    pub fn block_height() -> <<B as BlockT>::Header as HeaderT>::Number
    where
        B::Header: HeaderT,
    {
        *sp_io::storage::get(HEADER_KEY)
            .and_then(|d| B::Header::decode(&mut &*d).ok())
            .expect("A header is always stored at the beginning of the block")
            .number()
    }

    // These next three methods are for the block authoring workflow.
    // Open the block, apply zero or more extrinsics, close the block

    pub fn open_block(header: &<B as BlockT>::Header) {
        debug!(
            target: LOG_TARGET,
            "Entering initialize_block. header: {:?}", header
        );

        // Store the transient partial header for updating at the end of the block.
        // This will be removed from storage before the end of the block.
        sp_io::storage::set(HEADER_KEY, &header.encode());
    }

    pub fn apply_extrinsic(extrinsic: <B as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
        debug!(
            target: LOG_TARGET,
            "Entering apply_extrinsic: {:?}", extrinsic
        );

        // Append the current extrinsic to the transient list of extrinsics.
        // This will be used when we calculate the extrinsics root at the end of the block.
        let mut extrinsics = sp_io::storage::get(EXTRINSIC_KEY)
            .and_then(|d| <Vec<Vec<u8>>>::decode(&mut &*d).ok())
            .unwrap_or_default();
        extrinsics.push(extrinsic.encode());
        sp_io::storage::set(EXTRINSIC_KEY, &extrinsics.encode());

        // Now actually
        Self::apply_tuxedo_transaction(extrinsic)
            .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Custom(0)))?;

        Ok(Ok(()))
    }

    pub fn close_block() -> <B as BlockT>::Header {
        let mut header = sp_io::storage::get(HEADER_KEY)
            .and_then(|d| <B as BlockT>::Header::decode(&mut &*d).ok())
            .expect("We initialized with header, it never got mutated, qed");

        // the header itself contains the state root, so it cannot be inside the state (circular
        // dependency..). Make sure in execute block path we have the same rule.
        sp_io::storage::clear(HEADER_KEY);

        let extrinsics = sp_io::storage::get(EXTRINSIC_KEY)
            .and_then(|d| <Vec<Vec<u8>>>::decode(&mut &*d).ok())
            .unwrap_or_default();
        let extrinsics_root = <<B as BlockT>::Header as HeaderT>::Hashing::ordered_trie_root(
            extrinsics,
            StateVersion::V0,
        );
        sp_io::storage::clear(EXTRINSIC_KEY);
        header.set_extrinsics_root(extrinsics_root);

        let raw_state_root = &sp_io::storage::root(StateVersion::V1)[..];
        let state_root =
            <<B as BlockT>::Header as HeaderT>::Hash::decode(&mut &raw_state_root[..]).unwrap();
        header.set_state_root(state_root);

        debug!(target: LOG_TARGET, "finalizing block {:?}", header);
        header
    }

    // This one is for the Core api. It is used to import blocks authored by foreign nodes.

    pub fn execute_block(block: B) {
        debug!(
            target: LOG_TARGET,
            "Entering execute_block. block: {:?}", block
        );

        // Store the header. Although we don't need to mutate it, we do need to make
        // info, such as the block height, available to individual pieces. This will
        // be cleared before the end of the block
        sp_io::storage::set(HEADER_KEY, &block.header().encode());

        // Tuxedo requires that inherents are at the beginning (and soon end) of the
        // block and not scattered throughout. We use this flag to enforce that.
        let mut finished_with_opening_inherents = false;

        // Apply each extrinsic
        for extrinsic in block.extrinsics() {
            // Enforce that inherents are in the right place
            let current_tx_is_inherent = extrinsic.checker.is_inherent();
            if current_tx_is_inherent && finished_with_opening_inherents {
                panic!("Tried to execute opening inherent after switching to non-inherents.");
            }
            if !current_tx_is_inherent && !finished_with_opening_inherents {
                // This is the first non-inherent, so we update our flag and continue.
                finished_with_opening_inherents = true;
            }

            match Self::apply_tuxedo_transaction(extrinsic.clone()) {
                Ok(()) => debug!(
                    target: LOG_TARGET,
                    "Successfully executed extrinsic: {:?}", extrinsic
                ),
                Err(e) => panic!("{:?}", e),
            }
        }

        // Clear the transient header out of storage
        sp_io::storage::clear(HEADER_KEY);

        // Check state root
        let raw_state_root = &sp_io::storage::root(StateVersion::V1)[..];
        let state_root =
            <<B as BlockT>::Header as HeaderT>::Hash::decode(&mut &raw_state_root[..]).unwrap();
        assert_eq!(
            *block.header().state_root(),
            state_root,
            "state root mismatch"
        );

        // Print state for quick debugging
        // let mut key = vec![];
        // while let Some(next) = sp_io::storage::next_key(&key) {
        //     let val = sp_io::storage::get(&next).unwrap().to_vec();
        //     log::trace!(
        //         target: LOG_TARGET,
        //         "{} <=> {}",
        //         HexDisplay::from(&next),
        //         HexDisplay::from(&val)
        //     );
        //     key = next;
        // }

        // Check extrinsics root.
        let extrinsics = block
            .extrinsics()
            .iter()
            .map(|x| x.encode())
            .collect::<Vec<_>>();
        let extrinsics_root = <<B as BlockT>::Header as HeaderT>::Hashing::ordered_trie_root(
            extrinsics,
            StateVersion::V0,
        );
        assert_eq!(
            *block.header().extrinsics_root(),
            extrinsics_root,
            "extrinsics root mismatch"
        );
    }

    // This one is the pool api. It is used to make preliminary checks in the transaction pool

    pub fn validate_transaction(
        source: TransactionSource,
        tx: <B as BlockT>::Extrinsic,
        block_hash: <B as BlockT>::Hash,
    ) -> TransactionValidity {
        debug!(
            target: LOG_TARGET,
            "Entering validate_transaction. source: {:?}, tx: {:?}, block hash: {:?}",
            source,
            tx,
            block_hash
        );

        // Inherents are not permitted in the pool. They only come from the block author.
        // We perform this check here rather than in the `validate_tuxedo_transaction` helper,
        // because that helper is called again during on-chain execution. Inherents are valid
        // during execution, so we do not want this check repeated.
        let r = if tx.checker.is_inherent() {
            Err(TransactionValidityError::Invalid(InvalidTransaction::Call))
        } else {
            // TODO, we need a good way to map our UtxoError into the supposedly generic InvalidTransaction
            // https://paritytech.github.io/substrate/master/sp_runtime/transaction_validity/enum.InvalidTransaction.html
            // For now, I just make them all custom zero, and log the error variant
            Self::validate_tuxedo_transaction(&tx).map_err(|e| {
                log::warn!(
                    target: LOG_TARGET,
                    "Tuxedo Transaction did not validate (in the pool): {:?}",
                    e,
                );
                TransactionValidityError::Invalid(InvalidTransaction::Custom(0))
            })
        };

        debug!(target: LOG_TARGET, "Validation result: {:?}", r);

        r
    }

    // The next two are for the standard beginning-of-block inherent extrinsics.
    pub fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<B as BlockT>::Extrinsic> {
        debug!(
            target: LOG_TARGET,
            "Entering `inherent_extrinsics`."
        );

        // Extract the complete parent block from the inheret data
        let parent: B = data
            .get_data(&PARENT_INHERENT_IDENTIFIER)
            .expect("Parent block inherent data should be able to decode.")
            .expect("Parent block should be present among authoring inherent data.");

        // Extract the inherents from the previous block, which can be found at the beginning of the extrinsics list.
        // The parent is already imported, so we know it is valid and we know its inherents came first.
        // We also annotate each transaction with its original hash for purposes of constructing output refs later.
        // This is necessary because the transaction hash changes as we unwrap layers of aggregation,
        // and we need an original universal transaction id.
        let previous_blocks_inherents: Vec<(<B as BlockT>::Extrinsic, H256)> = parent
            .extrinsics()
            .iter()
            .cloned()
            .take_while(|tx| tx.checker.is_inherent())
            .map(|tx| {
                let id = BlakeTwo256::hash_of(&tx.encode());
                (tx, id)
            })
            .collect();

        debug!(
            target: LOG_TARGET,
            "The previous block had {} extrinsics ({} inherents).", parent.extrinsics().len(), previous_blocks_inherents.len()
        );

        // Call into constraint checker's own inherent hooks to create the actual transactions
        C::InherentHooks::create_inherents(&data, previous_blocks_inherents)
    }

    pub fn check_inherents(block: B, data: InherentData) -> sp_inherents::CheckInherentsResult {
        debug!(
            target: LOG_TARGET,
            "Entering `check_inherents`"
        );

        let mut result = CheckInherentsResult::new();

        // Tuxedo requires that all inherents come at the beginning of the block.
        // (Soon we will also allow them at the end, but never throughout the body.)
        // (TODO revise this logic once that is implemented.)
        // At this off-chain pre-check stage, we assume that requirement is upheld.
        // It will be verified later once we are executing on-chain.
        let inherents: Vec<Transaction<V, C>> = block
            .extrinsics()
            .iter()
            .cloned()
            .take_while(|tx| tx.checker.is_inherent())
            .collect();

        C::InherentHooks::check_inherents(&data, inherents, &mut result);

        result
    }
}

#[cfg(test)]
mod tests {
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

    /// Construct a mock OutputRef from a transaction number and index in that transaction.
    ///
    /// When setting up tests, it is often useful to have some Utxos in the storage
    /// before the test begins. There are no real transactions before the test, so there
    /// are also no real OutputRefs. This function constructs an OutputRef that can be
    /// used in the test from a "transaction number" (a simple u32) and an output index in
    /// that transaction (also a u32).
    fn mock_output_ref(tx_num: u32, index: u32) -> OutputRef {
        OutputRef {
            tx_hash: H256::from_low_u64_le(tx_num as u64),
            index,
        }
    }

    /// Builder pattern for test transactions.
    #[derive(Default)]
    struct TestTransactionBuilder {
        inputs: Vec<Input>,
        peeks: Vec<OutputRef>,
        outputs: Vec<Output<TestVerifier>>,
    }

    impl TestTransactionBuilder {
        fn with_input(mut self, input: Input) -> Self {
            self.inputs.push(input);
            self
        }

        fn with_peek(mut self, peek: OutputRef) -> Self {
            self.peeks.push(peek);
            self
        }

        fn with_output(mut self, output: Output<TestVerifier>) -> Self {
            self.outputs.push(output);
            self
        }

        fn build(self, checks: bool, inherent: bool) -> TestTransaction {
            TestTransaction {
                inputs: self.inputs,
                peeks: self.peeks,
                outputs: self.outputs,
                checker: TestConstraintChecker { checks, inherent },
            }
        }
    }

    /// Builds test externalities using a minimal builder pattern.
    #[derive(Default)]
    struct ExternalityBuilder {
        utxos: Vec<(OutputRef, Output<TestVerifier>)>,
        pre_header: Option<TestHeader>,
        noted_extrinsics: Vec<Vec<u8>>,
    }

    impl ExternalityBuilder {
        /// Add the given Utxo to the storage.
        ///
        /// There are no real transactions to calculate OutputRefs so instead we
        /// provide an output ref as a parameter. See the function `mock_output_ref`
        /// for a convenient way to construct testing output refs.
        ///
        /// For the Outputs themselves, this function accepts payloads of any type that
        /// can be represented as DynamicallyTypedData, and a boolean about whether the
        /// verifier should succeed or not.
        fn with_utxo<T: UtxoData>(
            mut self,
            output_ref: OutputRef,
            payload: T,
            verifies: bool,
        ) -> Self {
            let output = Output {
                payload: payload.into(),
                verifier: TestVerifier { verifies },
            };
            self.utxos.push((output_ref, output));
            self
        }

        /// Add a preheader to the storage.
        ///
        /// In normal execution `open_block` stores a header in storage
        /// before any extrinsics are applied. This function allows setting up
        /// a test case with a stored pre-header.
        ///
        /// Rather than passing in a header, we pass in parts of it. This ensures
        /// that a realistic pre-header (without extrinsics root or state root)
        /// is stored.
        ///
        /// Although a partial digest would be part of the pre-header, we have no
        /// use case for setting one, so it is also omitted here.
        fn with_pre_header(mut self, parent_hash: H256, number: u32) -> Self {
            let h = TestHeader {
                parent_hash,
                number,
                state_root: H256::zero(),
                extrinsics_root: H256::zero(),
                digest: Default::default(),
            };

            self.pre_header = Some(h);
            self
        }

        /// Add a noted extrinsic to the state.
        ///
        /// In normal block authoring, extrinsics are noted in state as they are
        /// applied so that an extrinsics root can be calculated at the end of the
        /// block. This function allows setting up a test case with som extrinsics
        /// already noted.
        ///
        /// The extrinsic is already encoded so that it doesn't have to be a proper
        /// extrinsic, but can just be some example bytes.
        fn with_noted_extrinsic(mut self, ext: Vec<u8>) -> Self {
            self.noted_extrinsics.push(ext);
            self
        }

        /// Build the test externalities with all the utxos already stored
        fn build(self) -> TestExternalities {
            let mut ext = TestExternalities::default();

            // Write all the utxos
            for (output_ref, output) in self.utxos {
                ext.insert(output_ref.encode(), output.encode());
            }

            // Write the pre-header
            if let Some(pre_header) = self.pre_header {
                ext.insert(HEADER_KEY.to_vec(), pre_header.encode());
            }

            // Write the noted extrinsics
            ext.insert(EXTRINSIC_KEY.to_vec(), self.noted_extrinsics.encode());

            ext
        }
    }

    #[test]
    fn validate_empty_works() {
        let tx = TestTransactionBuilder::default().build(true, false);

        let vt = TestExecutive::validate_tuxedo_transaction(&tx).unwrap();

        let expected_result = ValidTransactionBuilder::default().into();

        assert_eq!(vt, expected_result);
    }

    #[test]
    fn validate_with_input_works() {
        let output_ref = mock_output_ref(0, 0);

        ExternalityBuilder::default()
            .with_utxo(output_ref.clone(), Bogus, true)
            .build()
            .execute_with(|| {
                let input = Input {
                    output_ref,
                    redeemer: Vec::new(),
                };

                let tx = TestTransactionBuilder::default()
                    .with_input(input)
                    .build(true, false);

                let vt = TestExecutive::validate_tuxedo_transaction(&tx).unwrap();

                let expected_result = ValidTransactionBuilder::default().into();

                assert_eq!(vt, expected_result);
            });
    }

    #[test]
    fn validate_with_peek_works() {
        let output_ref = mock_output_ref(0, 0);

        ExternalityBuilder::default()
            .with_utxo(output_ref.clone(), Bogus, true)
            .build()
            .execute_with(|| {
                let tx = TestTransactionBuilder::default()
                    .with_peek(output_ref)
                    .build(true, false);

                let vt = TestExecutive::validate_tuxedo_transaction(&tx).unwrap();

                let expected_result = ValidTransactionBuilder::default().into();

                assert_eq!(vt, expected_result);
            });
    }

    #[test]
    fn validate_with_output_works() {
        ExternalityBuilder::default().build().execute_with(|| {
            let output = Output {
                payload: Bogus.into(),
                verifier: TestVerifier { verifies: false },
            };
            let tx = TestTransactionBuilder::default()
                .with_output(output)
                .build(true, false);

            // This is a real transaction, so we need to calculate a real OutputRef
            let tx_hash = BlakeTwo256::hash_of(&tx.encode());
            let output_ref = OutputRef { tx_hash, index: 0 };

            let vt = TestExecutive::validate_tuxedo_transaction(&tx).unwrap();

            let expected_result = ValidTransactionBuilder::default()
                .and_provides(output_ref)
                .into();

            assert_eq!(vt, expected_result);
        });
    }

    #[test]
    fn validate_with_missing_input_works() {
        ExternalityBuilder::default().build().execute_with(|| {
            let output_ref = mock_output_ref(0, 0);
            let input = Input {
                output_ref: output_ref.clone(),
                redeemer: Vec::new(),
            };

            let tx = TestTransactionBuilder::default()
                .with_input(input)
                .build(true, false);

            let vt = TestExecutive::validate_tuxedo_transaction(&tx).unwrap();

            let expected_result = ValidTransactionBuilder::default()
                .and_requires(output_ref)
                .into();

            assert_eq!(vt, expected_result);
        });
    }

    #[test]
    fn validate_with_missing_peek_works() {
        ExternalityBuilder::default().build().execute_with(|| {
            let output_ref = mock_output_ref(0, 0);

            let tx = TestTransactionBuilder::default()
                .with_peek(output_ref.clone())
                .build(true, false);

            let vt = TestExecutive::validate_tuxedo_transaction(&tx).unwrap();

            let expected_result = ValidTransactionBuilder::default()
                .and_requires(output_ref)
                .into();

            assert_eq!(vt, expected_result);
        });
    }

    #[test]
    fn validate_with_duplicate_input_fails() {
        let output_ref = mock_output_ref(0, 0);

        ExternalityBuilder::default()
            .with_utxo(output_ref.clone(), Bogus, false)
            .build()
            .execute_with(|| {
                let input = Input {
                    output_ref,
                    redeemer: Vec::new(),
                };

                let tx = TestTransactionBuilder::default()
                    .with_input(input.clone())
                    .with_input(input)
                    .build(true, false);

                let result = TestExecutive::validate_tuxedo_transaction(&tx);

                assert_eq!(result, Err(UtxoError::DuplicateInput));
            });
    }

    #[test]
    fn validate_with_duplicate_peek_works() {
        // Peeking at the same input twice is considered valid. However, wallets should do their best
        // not to construct such transactions whenever possible because it makes the transactions space inefficient.

        let output_ref = mock_output_ref(0, 0);

        ExternalityBuilder::default()
            .with_utxo(output_ref.clone(), Bogus, false)
            .build()
            .execute_with(|| {
                let tx = TestTransactionBuilder::default()
                    .with_peek(output_ref.clone())
                    .with_peek(output_ref)
                    .build(true, false);

                let vt = TestExecutive::validate_tuxedo_transaction(&tx).unwrap();

                let expected_result = ValidTransactionBuilder::default().into();

                assert_eq!(vt, expected_result);
            });
    }

    #[test]
    fn validate_with_unsatisfied_verifier_fails() {
        let output_ref = mock_output_ref(0, 0);

        ExternalityBuilder::default()
            .with_utxo(output_ref.clone(), Bogus, false)
            .build()
            .execute_with(|| {
                let input = Input {
                    output_ref,
                    redeemer: Vec::new(),
                };

                let tx = TestTransactionBuilder::default()
                    .with_input(input)
                    .build(true, false);

                let result = TestExecutive::validate_tuxedo_transaction(&tx);

                assert_eq!(result, Err(UtxoError::VerifierError));
            });
    }

    #[test]
    fn validate_with_pre_existing_output_fails() {
        // This test requires a transaction to create an output at a location where
        // an output already exists. This could happen in the wild when two transactions
        // don't have inputs and have the same outputs. I initially couldn't think of how
        // this could happen.

        // First we create the transaction that will be submitted in the test.
        let output = Output {
            payload: Bogus.into(),
            verifier: TestVerifier { verifies: false },
        };
        let tx = TestTransactionBuilder::default()
            .with_output(output)
            .build(true, false);

        // Now calculate the output ref that the transaction creates so we can pre-populate the state.
        let tx_hash = BlakeTwo256::hash_of(&tx.encode());
        let output_ref = OutputRef { tx_hash, index: 0 };

        ExternalityBuilder::default()
            .with_utxo(output_ref, Bogus, false)
            .build()
            .execute_with(|| {
                let result = TestExecutive::validate_tuxedo_transaction(&tx);
                assert_eq!(result, Err(UtxoError::PreExistingOutput));
            });
    }

    #[test]
    fn validate_with_constraint_error_fails() {
        ExternalityBuilder::default().build().execute_with(|| {
            let tx = TestTransactionBuilder::default().build(false, false);

            let vt = TestExecutive::validate_tuxedo_transaction(&tx);

            assert_eq!(vt, Err(UtxoError::ConstraintCheckerError(())));
        });
    }

    #[test]
    fn apply_empty_works() {
        ExternalityBuilder::default().build().execute_with(|| {
            let tx = TestTransactionBuilder::default().build(true, false);

            let vt = TestExecutive::apply_tuxedo_transaction(tx);

            assert_eq!(vt, Ok(()));
        });
    }

    #[test]
    fn apply_with_missing_input_fails() {
        ExternalityBuilder::default().build().execute_with(|| {
            let output_ref = mock_output_ref(0, 0);
            let input = Input {
                output_ref: output_ref.clone(),
                redeemer: Vec::new(),
            };

            let tx = TestTransactionBuilder::default()
                .with_input(input)
                .build(true, false);

            let vt = TestExecutive::apply_tuxedo_transaction(tx);

            assert_eq!(vt, Err(UtxoError::MissingInput));
        });
    }

    #[test]
    fn apply_with_missing_peek_fails() {
        ExternalityBuilder::default().build().execute_with(|| {
            let output_ref = mock_output_ref(0, 0);

            let tx = TestTransactionBuilder::default()
                .with_peek(output_ref)
                .build(true, false);

            let vt = TestExecutive::apply_tuxedo_transaction(tx);

            assert_eq!(vt, Err(UtxoError::MissingInput));
        });
    }

    #[test]
    fn update_storage_consumes_input() {
        let output_ref = mock_output_ref(0, 0);

        ExternalityBuilder::default()
            .with_utxo(output_ref.clone(), Bogus, true)
            .build()
            .execute_with(|| {
                let input = Input {
                    output_ref: output_ref.clone(),
                    redeemer: Vec::new(),
                };

                let tx = TestTransactionBuilder::default()
                    .with_input(input)
                    .build(true, false);

                // Commit the tx to storage
                TestExecutive::update_storage(tx);

                // Check whether the Input is still in storage
                assert!(!sp_io::storage::exists(&output_ref.encode()));
            });
    }

    #[test]
    fn update_storage_adds_output() {
        ExternalityBuilder::default().build().execute_with(|| {
            let output = Output {
                payload: Bogus.into(),
                verifier: TestVerifier { verifies: false },
            };

            let tx = TestTransactionBuilder::default()
                .with_output(output.clone())
                .build(true, false);

            let tx_hash = BlakeTwo256::hash_of(&tx.encode());
            let output_ref = OutputRef { tx_hash, index: 0 };

            // Commit the tx to storage
            TestExecutive::update_storage(tx);

            // Check whether the Output has been written to storage and the proper value is stored
            let stored_bytes = sp_io::storage::get(&output_ref.encode()).unwrap();
            let stored_value = Output::decode(&mut &stored_bytes[..]).unwrap();
            assert_eq!(stored_value, output);
        });
    }

    #[test]
    fn open_block_works() {
        let header = TestHeader {
            parent_hash: H256::repeat_byte(5),
            number: 5,
            state_root: H256::repeat_byte(6),
            extrinsics_root: H256::repeat_byte(7),
            digest: Default::default(),
        };

        ExternalityBuilder::default().build().execute_with(|| {
            // Call open block which just writes the header to storage
            TestExecutive::open_block(&header);

            // Fetch the header back out of storage
            let retrieved_header = sp_io::storage::get(HEADER_KEY)
                .and_then(|d| TestHeader::decode(&mut &*d).ok())
                .expect("Open block should have written a header to storage");

            // Make sure the header that came out is the same one that went in.
            assert_eq!(retrieved_header, header);
        });
    }

    #[test]
    fn apply_valid_extrinsic_work() {
        ExternalityBuilder::default().build().execute_with(|| {
            let tx = TestTransactionBuilder::default().build(true, false);

            let apply_result = TestExecutive::apply_extrinsic(tx.clone());

            // Make sure the returned result is Ok
            assert_eq!(apply_result, Ok(Ok(())));

            // Make sure the transaction is noted in storage
            let noted_extrinsics = sp_io::storage::get(EXTRINSIC_KEY)
                .and_then(|d| <Vec<Vec<u8>>>::decode(&mut &*d).ok())
                .unwrap_or_default();

            assert_eq!(noted_extrinsics, vec![tx.encode()]);
        });
    }

    #[test]
    fn apply_invalid_extrinsic_rejects() {
        ExternalityBuilder::default().build().execute_with(|| {
            let tx = TestTransactionBuilder::default().build(false, false);

            let apply_result = TestExecutive::apply_extrinsic(tx.clone());

            // Make sure the returned result is an error
            assert!(apply_result.is_err());

            // TODO Do we actually want to note transactions that ultimately reject?
            // Make sure the transaction is noted in storage
            let noted_extrinsics = sp_io::storage::get(EXTRINSIC_KEY)
                .and_then(|d| <Vec<Vec<u8>>>::decode(&mut &*d).ok())
                .unwrap_or_default();

            assert_eq!(noted_extrinsics, vec![tx.encode()]);
        });
    }

    #[test]
    fn close_block_works() {
        let parent_hash = H256::repeat_byte(5);
        let block_number = 6;
        let extrinsic = vec![1, 2, 3];
        ExternalityBuilder::default()
            .with_pre_header(parent_hash, block_number)
            .with_noted_extrinsic(extrinsic.clone())
            .build()
            .execute_with(|| {
                let returned_header = TestExecutive::close_block();

                // Make sure the header is as we expected
                let raw_state_root = &sp_io::storage::root(StateVersion::V1)[..];
                let state_root = H256::decode(&mut &raw_state_root[..]).unwrap();
                let expected_header = TestHeader {
                    parent_hash,
                    number: block_number,
                    state_root,
                    extrinsics_root: BlakeTwo256::ordered_trie_root(
                        vec![extrinsic],
                        StateVersion::V0,
                    ),
                    digest: Default::default(),
                };

                assert_eq!(returned_header, expected_header);

                // Make sure the transient storage has been removed
                assert!(!sp_io::storage::exists(HEADER_KEY));
                assert!(!sp_io::storage::exists(EXTRINSIC_KEY));
            });
    }

    #[test]
    fn execute_empty_block_works() {
        ExternalityBuilder::default().build().execute_with(|| {
            let b = TestBlock {
                header: TestHeader {
                    parent_hash: H256::zero(),
                    number: 6,
                    state_root: array_bytes::hex_n_into_unchecked(
                        "858174d563f845dbb4959ea64816bd8409e48cc7e65db8aa455bc98d61d24071",
                    ),
                    extrinsics_root: array_bytes::hex_n_into_unchecked(
                        "03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314",
                    ),
                    digest: Default::default(),
                },
                extrinsics: Vec::new(),
            };

            TestExecutive::execute_block(b);
        });
    }

    #[test]
    fn execute_block_with_transaction_works() {
        ExternalityBuilder::default().build().execute_with(|| {
            let b = TestBlock {
                header: TestHeader {
                    parent_hash: H256::zero(),
                    number: 6,
                    state_root: array_bytes::hex_n_into_unchecked(
                        "858174d563f845dbb4959ea64816bd8409e48cc7e65db8aa455bc98d61d24071",
                    ),
                    extrinsics_root: array_bytes::hex_n_into_unchecked(
                        "d609af1c51521f5891054014cf667619067a93f4bca518b398f5a39aeb270cca",
                    ),
                    digest: Default::default(),
                },
                extrinsics: vec![TestTransactionBuilder::default().build(true, false)],
            };

            TestExecutive::execute_block(b);
        });
    }

    #[test]
    #[should_panic(expected = "ConstraintCheckerError(())")]
    fn execute_block_invalid_transaction() {
        ExternalityBuilder::default().build().execute_with(|| {
            let b = TestBlock {
                header: TestHeader {
                    parent_hash: H256::zero(),
                    number: 6,
                    state_root: array_bytes::hex_n_into_unchecked(
                        "858174d563f845dbb4959ea64816bd8409e48cc7e65db8aa455bc98d61d24071",
                    ),
                    extrinsics_root: array_bytes::hex_n_into_unchecked(
                        "03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314",
                    ),
                    digest: Default::default(),
                },
                extrinsics: vec![TestTransactionBuilder::default().build(false, false)],
            };

            TestExecutive::execute_block(b);
        });
    }

    #[test]
    #[should_panic(expected = "state root mismatch")]
    fn execute_block_state_root_mismatch() {
        ExternalityBuilder::default().build().execute_with(|| {
            let b = TestBlock {
                header: TestHeader {
                    parent_hash: H256::zero(),
                    number: 6,
                    state_root: H256::zero(),
                    extrinsics_root: array_bytes::hex_n_into_unchecked(
                        "03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314",
                    ),
                    digest: Default::default(),
                },
                extrinsics: Vec::new(),
            };

            TestExecutive::execute_block(b);
        });
    }

    #[test]
    #[should_panic(expected = "extrinsics root mismatch")]
    fn execute_block_extrinsic_root_mismatch() {
        ExternalityBuilder::default().build().execute_with(|| {
            let b = TestBlock {
                header: TestHeader {
                    parent_hash: H256::zero(),
                    number: 6,
                    state_root: array_bytes::hex_n_into_unchecked(
                        "858174d563f845dbb4959ea64816bd8409e48cc7e65db8aa455bc98d61d24071",
                    ),
                    extrinsics_root: H256::zero(),
                    digest: Default::default(),
                },
                extrinsics: Vec::new(),
            };

            TestExecutive::execute_block(b);
        });
    }

    #[test]
    fn execute_block_inherent_only_works() {
        ExternalityBuilder::default().build().execute_with(|| {
            let b = TestBlock {
                header: TestHeader {
                    parent_hash: H256::zero(),
                    number: 6,
                    state_root: array_bytes::hex_n_into_unchecked(
                        "858174d563f845dbb4959ea64816bd8409e48cc7e65db8aa455bc98d61d24071",
                    ),
                    extrinsics_root: array_bytes::hex_n_into_unchecked(
                        "799fc6d36f68fc83ae3408de607006e02836181e91701aa3a8021960b1f3507c",
                    ),
                    digest: Default::default(),
                },
                extrinsics: vec![TestTransactionBuilder::default().build(true, true)],
            };

            TestExecutive::execute_block(b);
        });
    }

    #[test]
    fn execute_block_inherent_first_works() {
        ExternalityBuilder::default().build().execute_with(|| {
            let b = TestBlock {
                header: TestHeader {
                    parent_hash: H256::zero(),
                    number: 6,
                    state_root: array_bytes::hex_n_into_unchecked(
                        "858174d563f845dbb4959ea64816bd8409e48cc7e65db8aa455bc98d61d24071",
                    ),
                    extrinsics_root: array_bytes::hex_n_into_unchecked(
                        "bf3e98799022bee8f0a55659af5f498717736ae012d2aff6274cdb7c2b0d78e9",
                    ),
                    digest: Default::default(),
                },
                extrinsics: vec![
                    TestTransactionBuilder::default().build(true, true),
                    TestTransactionBuilder::default().build(true, false),
                ],
            };

            TestExecutive::execute_block(b);
        });
    }

    #[test]
    #[should_panic(
        expected = "Tried to execute opening inherent after switching to non-inherents."
    )]
    fn execute_block_inherents_must_be_first() {
        ExternalityBuilder::default().build().execute_with(|| {
            let b = TestBlock {
                header: TestHeader {
                    parent_hash: H256::zero(),
                    number: 6,
                    state_root: array_bytes::hex_n_into_unchecked(
                        "858174d563f845dbb4959ea64816bd8409e48cc7e65db8aa455bc98d61d24071",
                    ),
                    extrinsics_root: array_bytes::hex_n_into_unchecked(
                        "df64890515cd8ef5a8e736248394f7c72a1df197bd400a4e31affcaf6e051984",
                    ),
                    digest: Default::default(),
                },
                extrinsics: vec![
                    TestTransactionBuilder::default().build(true, false),
                    TestTransactionBuilder::default().build(true, true),
                ],
            };

            TestExecutive::execute_block(b);
        });
    }

    #[test]
    #[should_panic(
        expected = "Tried to execute opening inherent after switching to non-inherents."
    )]
    fn execute_block_inherents_must_all_be_first() {
        ExternalityBuilder::default().build().execute_with(|| {
            let b = TestBlock {
                header: TestHeader {
                    parent_hash: H256::zero(),
                    number: 6,
                    state_root: array_bytes::hex_n_into_unchecked(
                        "858174d563f845dbb4959ea64816bd8409e48cc7e65db8aa455bc98d61d24071",
                    ),
                    extrinsics_root: array_bytes::hex_n_into_unchecked(
                        "0x36601deae36de127b974e8498e118e348a50aa4aa94bc5713e29c56e0d37e44f",
                    ),
                    digest: Default::default(),
                },
                extrinsics: vec![
                    TestTransactionBuilder::default().build(true, true),
                    TestTransactionBuilder::default().build(true, false),
                    TestTransactionBuilder::default().build(true, true),
                ],
            };

            TestExecutive::execute_block(b);
        });
    }
}
