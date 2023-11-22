//! Allow collators to include information about the relay chain, parachain, and their relationship, via an inherent.
//!
//! This piece is necessary if the runtime is going to work as a parachain.
//!
//! In each block, the block author must include a single `SetParachainInfo` transaction that consumes the
//! corresponding UTXO that was created in the previous block, and creates a new one with updated parachain info.
//! This is quite similar to how the timestamp inherent works, except that in this case we are consuming the previous
//! input directly instead of peeking. This decision may be revisitied if keeping the info around would be useful.
//!
//! ## Comparison with Cumulus Pallet Parachain System
//!
//! This is similar to FRAME's pallet parachain system, although this piece is only responsible for the inherent flow
//! while that pallet is responsible for most of the core parachain requirements including the validate block function.
//!
//! ## Hack Warning
//!
//! Like the timestamp piece, this piece currently abuses the UpForGrabs verifier.
//! This should be replaced with an Unspendable verifier and an eviction workflow.

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use cumulus_primitives_parachain_inherent::{ParachainInherentData, INHERENT_IDENTIFIER};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_inherents::{CheckInherentsResult, InherentData};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::{vec, vec::Vec};
// We get all the Tuxedo core stuff through the re-export so we don't risk crossed versions.
use tuxedo_parachain_core::ParachainInherentDataUtxo;
use tuxedo_parachain_core::{
    tuxedo_core::{
        ensure,
        inherents::{TuxedoInherent, TuxedoInherentAdapter},
        support_macros::{CloneNoBound, DebugNoBound, DefaultNoBound},
        types::{Input, Output, OutputRef, Transaction},
        verifier::UpForGrabs,
        ConstraintChecker, Verifier,
    },
    SetRelayParentNumberStorage,
};

#[cfg(test)]
mod tests;

/// A piece-wide target for logging
const LOG_TARGET: &str = "parachain-info";

/// Options to configure the timestamp piece when it is aggregated or used in a runtime.
pub trait ParachainPieceConfig {
    //TODO Consider whetther including this config item is useful or wise. It is just an idea I had
    // and I'm scribbling it here so I don't forget it.
    //
    // Also, there is currently a value in the chainspec as well, and it is duplicated.
    // This duplication of info on the client side and runtime side was a problem in the original Cumulus design as well.
    /// The Parachain Id associated with this parachain
    const PARA_ID: u32 = 2_000;

    /// A means of setting an ambiently available relay parent number. This value WILL be used when
    /// the colaltor calls the colaltion API after the block is authored and also in validate_block.
    /// Additionally, it MAY be used by any other pieces in the runtime who have access to it.
    type SetRelayParentNumberStorage: SetRelayParentNumberStorage;
}

/// Reasons that setting or cleaning up the parachain info may go wrong.
#[derive(Debug, Eq, PartialEq)]
pub enum ParachainError {
    /// UTXO data has an unexpected type
    BadlyTyped,
    /// When attempting to set a new parachain info, you have not included any output.
    MissingNewInfo,
    /// Multiple outputs were specified while setting the parachain info, but exactly one is required.
    ExtraOutputs,
    /// No previous parachain info was consumed in this transaction, but at consuming the previous utxo is required.
    MissingPreviousInfo,
    /// Multiple inputs were specified while setting the parachain info, but exacctly one is required.
    ExtraInputs,
    /// The new relay chain block number is expected to be higher than the previous, but that is not the case.
    RelayBlockNotIncreasing,
}

/// A constraint checker for the simple act of including new parachain information.
///
/// This is expected to be performed through an inherent, and to happen exactly once per block.
///
/// This transaction comsumes a single input which is the previous parachain info,
/// And it creates a new output which is the current parachain info.
#[derive(
    Serialize,
    Deserialize,
    Encode,
    Decode,
    DebugNoBound,
    DefaultNoBound,
    PartialEq,
    Eq,
    CloneNoBound,
    TypeInfo,
)]
#[scale_info(skip_type_params(T))]
pub struct SetParachainInfo<T>(PhantomData<T>);

impl<T: ParachainPieceConfig + 'static, V: Verifier + From<UpForGrabs>> ConstraintChecker<V>
    for SetParachainInfo<T>
{
    type Error = ParachainError;
    type InherentHooks = TuxedoInherentAdapter<Self>;

    fn check(
        &self,
        input_data: &[Output<V>],
        _peek_data: &[Output<V>],
        output_data: &[Output<V>],
    ) -> Result<TransactionPriority, Self::Error> {
        log::debug!(
            target: LOG_TARGET,
            "Checking onchain constraints for SetParachainInfo."
        );

        // Make sure there is exactly one output which is the current parachain info
        ensure!(!output_data.is_empty(), Self::Error::MissingNewInfo);
        ensure!(output_data.len() == 1, Self::Error::ExtraOutputs);
        let current: ParachainInherentData = output_data[0]
            .payload
            .extract::<ParachainInherentDataUtxo>()
            .map_err(|_| Self::Error::BadlyTyped)?
            .into();

        // SIDE EFFECT: Write the relay parent number to storage to use later in the collation info api
        T::SetRelayParentNumberStorage::set(current.validation_data.relay_parent_number);

        // Make sure there is exactly one input which is the previous parachain info
        ensure!(!input_data.is_empty(), Self::Error::MissingPreviousInfo);
        ensure!(input_data.len() == 1, Self::Error::ExtraInputs);
        let previous: ParachainInherentData = input_data[0]
            .payload
            .extract::<ParachainInherentDataUtxo>()
            .map_err(|_| Self::Error::BadlyTyped)?
            .into();

        // Make sure the relay chain block height is strictly increasing.
        // In frame this logic is generic and it doesn't have to be so strict.
        // But for now I'll start simple.
        ensure!(
            current.validation_data.relay_parent_number
                > previous.validation_data.relay_parent_number,
            Self::Error::RelayBlockNotIncreasing,
        );

        // We may need to put a log on the block header at some point.
        // Frame does this. However, it seems this design is not fully fleshed out in cumulus itself.
        // FIXME https://github.com/Off-Narrative-Labs/Tuxedo/issues/147 for more context and info.

        // 1. Maybe we should validate the parent head data?
        //would require a method in core to expose the header (or at least it's hash)
        // But I'm not sure if this is necessary or if it is already handled somewhere else.
        // assert_eq!(
        //     hash(header_from_core_method),
        //     current.validation_data.parent_head,
        // );

        Ok(0)
    }

    fn is_inherent(&self) -> bool {
        true
    }
}

impl<V: Verifier + From<UpForGrabs>, T: ParachainPieceConfig + 'static> TuxedoInherent<V, Self>
    for SetParachainInfo<T>
{
    // Same error type as in frame
    type Error = sp_inherents::MakeFatalError<()>;
    const INHERENT_IDENTIFIER: sp_inherents::InherentIdentifier = INHERENT_IDENTIFIER;

    fn create_inherent(
        authoring_inherent_data: &InherentData,
        (_previous_inherent, previous_id): (Transaction<V, Self>, H256),
    ) -> Transaction<V, Self> {
        let current_info: ParachainInherentData = authoring_inherent_data
            .get_data(&INHERENT_IDENTIFIER)
            .expect("Inherent data should decode properly")
            .expect("Parachain inherent data should be present.");

        log::debug!(
            target: LOG_TARGET,
            "parachain inherent data while creating inherent: {:?}", current_info
        );

        // The first task is to construct the input that we will consume.
        // We are given the entire previous inherent in case we need data from it or need to scrape the outputs.
        // But our transactions are simple enough that we know we just need the one and only output.
        let output_ref = OutputRef {
            tx_hash: previous_id,
            // There is always 1 output, so we know right where to find it.
            index: 0,
        };

        let input = Input {
            output_ref,
            redeemer: Vec::new(),
        };

        let new_output = Output {
            payload: ParachainInherentDataUtxo::from(current_info).into(),
            verifier: UpForGrabs.into(),
        };

        let t = Transaction {
            inputs: vec![input],
            peeks: Vec::new(),
            outputs: vec![new_output],
            checker: Self::default(),
        };

        log::debug!(
            target: LOG_TARGET,
            "created inherent transaction {:?}.", t
        );

        t
    }

    fn check_inherent(
        _importing_inherent_data: &InherentData,
        _inherent: Transaction<V, Self>,
        _result: &mut CheckInherentsResult,
    ) {
        log::debug!(
            target: LOG_TARGET,
            "In check_inherents for parachain inherent. No actual off-chain checks are required."
        );

        //TODO The debug message above reflects the current design which is a copy of basti's design.
        // I think the process of checking this inherent should be accessible through some abstract interface in the end.
    }

    #[cfg(feature = "std")]
    fn genesis_transactions() -> Vec<Transaction<V, Self>> {
        let payload = new_data_from_relay_parent_number(0).into();

        vec![Transaction {
            inputs: Vec::new(),
            peeks: Vec::new(),
            outputs: vec![Output {
                payload,
                verifier: UpForGrabs.into(),
            }],
            checker: Self::default(),
        }]
    }
}

#[cfg(feature = "std")]
fn new_data_from_relay_parent_number(relay_parent_number: u32) -> ParachainInherentDataUtxo {
    let sproof_builder = cumulus_test_relay_sproof_builder::RelayStateSproofBuilder::default();
    //TODO consider changing the para_id here. For sure do it if we keep the piece config item.

    let (relay_parent_storage_root, relay_chain_state_proof) =
        sproof_builder.into_state_root_and_proof();

    ParachainInherentData {
        validation_data: cumulus_primitives_core::PersistedValidationData {
            parent_head: Default::default(),
            relay_parent_number,
            relay_parent_storage_root,
            max_pov_size: Default::default(),
        },
        relay_chain_state: relay_chain_state_proof,
        downward_messages: Default::default(),
        horizontal_messages: Default::default(),
    }
    .into()
}
