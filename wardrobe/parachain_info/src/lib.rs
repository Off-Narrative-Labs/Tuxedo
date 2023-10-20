//! Allow collators to include information about the relay chain, parachain, and their relationship via an inherent.
//!
//! This piece is necessary if the runtime is going to work as a parachain.
//!
//! In each block, the block author must include a single `SetParachainInfo` transaction that consumes the
//! corresponding UTXO that was created in the previous block, and creates a new one with updated parachain info.
//! This is quite similar to how the timestamp inherent works, except that in this case we are consuming the previous
//! input directly instead of peeking. This decision is to keep things simple to get started. It may be revisitied if
//! keeping this info around would be useful.
//!
//! ## Comparison with Cumulus Pallet Parachain System
//!
//! This is similar to FRAME's pallet parachain system, although this piece is only responsible for the inherent flow
//! while that pallet is responsible for most of the core parachain requirements including the validate block function
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
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure,
    inherents::{TuxedoInherent, TuxedoInherentAdapter},
    support_macros::{CloneNoBound, DebugNoBound, DefaultNoBound},
    types::{Output, OutputRef, Transaction},
    verifier::UpForGrabs,
    ConstraintChecker, Verifier,
};

#[cfg(test)]
mod tests;

/// A piece-wide target for logging
const LOG_TARGET: &str = "parachain-info";

// We are not creating our own data struct here. This one is imported from cumulus.
impl UtxoData for ParachainInherentData {
    const TYPE_ID: [u8; 4] = *b"para";
}

/// Options to configure the timestamp piece in your runtime.
/// Currently we only need access to a block number.
pub trait ParachainPieceConfig {
    //TODO consider whether we will need this at all after the first block hack is removed
    /// A means of getting the current block height.
    /// Probably this will be the Tuxedo Executive
    fn block_height() -> u32;

    //TODO Consider whetther including this config item is useful or wise. It is just an idea I had
    // and I'm scribbling it here so I don't forget it.
    //
    // Also, there is currently a value in the chainspec as well, and it is duplicated.
    // This duplication of info on the client side and runtime side was a problem in the original Cumulus design as well.
    /// The Parachain Id associated with this parachain
    const ParaId: u32 = 2_000;
}

/// Reasons that setting or cleaning up the parachain info may go wrong.
#[derive(Debug, Eq, PartialEq)]
pub enum ParachainInfoError {
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
    type Error = ParachainInfoError;
    type InherentHooks = TuxedoInherentAdapter<Self>;

    fn check(
        &self,
        input_data: &[tuxedo_core::types::Output<V>],
        peek_data: &[tuxedo_core::types::Output<V>],
        output_data: &[tuxedo_core::types::Output<V>],
    ) -> Result<TransactionPriority, Self::Error> {
        log::debug!(
            target: LOG_TARGET,
            "Checking constraints for SetParachainInfo."
        );

        // Make sure there is exactly one input which is the previous parachain info
        ensure!(!input_data.is_empty(), Self::Error::MissingPreviousInfo,);
        ensure!(input_data.len() == 1, Self::Error::ExtraInputs,);
        let previous = output_data[0]
            .payload
            .extract::<ParachainInherentData>()
            .map_err(|_| Self::Error::BadlyTyped)?;

        // Make sure there is exactly one output which is the current parachain info
        ensure!(!output_data.is_empty(), Self::Error::MissingNewInfo);
        ensure!(output_data.len() == 1, Self::Error::MissingNewInfo,);
        let current = output_data[0]
            .payload
            .extract::<ParachainInherentData>()
            .map_err(|_| Self::Error::BadlyTyped)?;

        // Make sure the relay chain block height is strictly increasing.
        // In frame this logic is generic and it doesn't have to be so strict.
        // But for now I'll start simple.
        ensure!(
            current.validation_data.relay_parent_number
                > previous.validation_data.relay_parent_number,
            Self::Error::RelayBlockNotIncreasing,
        );

        // TODO There may be a lot more checks to make here. For now this is where I'll leave it.

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
        previous_inherent: Option<(Transaction<V, Self>, H256)>,
    ) -> tuxedo_core::types::Transaction<V, Self> {
        let current_info = authoring_inherent_data
            .get_data(&INHERENT_IDENTIFIER)
            .expect("Inherent data should decode properly")
            .expect("Parachain inherent data should be present.");

        log::debug!(
            target: LOG_TARGET,
            "parachain inherent data while creating inherent: {current_info}"
        );

        let mut inputs = Vec::new();
        match (previous_inherent, T::block_height()) {
            (None, 1) => {
                // This is the first block hack case.
                // We don't need any inputs, so just do nothing.
            }
            (None, _) => panic!("Attemping to construct parachain inherent with no previous inherent (and not block 1)."),
            (Some((_previous_inherent, previous_id)), _) => {
                // This is the the normal case. We create a full previous to peek at.

                // We are given the entire previous inherent in case we need data from it or need to scrape the outputs.
                // But out transactions are simple enough that we know we just need the one and only output.
                inputs.push(OutputRef {
                    tx_hash: previous_id,
                    // There is always 1 output, so we know right where to find it.
                    index: 0,
                });
            }
        }

        let new_output = Output {
            payload: current_info.into(),
            verifier: UpForGrabs.into(),
        };

        Transaction {
            inputs,
            peeks: Vec::new(),
            outputs: vec![new_output],
            checker: Self::default(),
        }
    }

    fn check_inherent(
        importing_inherent_data: &InherentData,
        inherent: Transaction<V, Self>,
        result: &mut CheckInherentsResult,
    ) {
        log::debug!(
            target: LOG_TARGET,
            "In check_inherents for parachain inherent. No actual off-chain checks are required."
        );
    }
}
