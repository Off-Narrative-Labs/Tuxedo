//! Utilities for working with inherents in Tuxedo based chains
//!
//! # Background on inherents
//!
//! Inherents are a Substrate feature that allows block authors to insert some data from the environment
//! into the body of the block. Inherents are similar to pre-runtime digests which are persisted in the block header,
//! but they differ because inherents go into the block body.
//!
//! Some classic usecases are a block timestamp, information about the relay chain (if the current chain is a parachain),
//! or information about who should receive the block reward.
//!
//! # Complexities in UTXO chains
//!
//! In account based systems, the classic way to use an inherent is that the block author calls the runtime providing the current timestamp.
//! The runtime returns an extrinsic that will set the timestamp on-chain. The block author includes that inherent extrinsic in the block author.
//! The purpose for the extra round-trip to the runtime is to facilitate runtime upgrades, I believe. Then when the extrinsic executes it, for example,
//! overwrites the timestamp in a dedicated storage item.
//!
//! In UTXO chains, there are no storage items, and all state is local to a UTXO. This is the case with, for example, the timestamp as well.
//! This means that when the author calls into the runtime with a timestamp, the transaction that is returned must include the correct reference
//! to the UTXO that contained the previous best timestamp. This is the crux of the problem. There is no easy way to know the location of
//! the previous timestamp in the utxo-space from inside the runtime.
//!
//! # Scraping the Parent Block
//!
//! The solution is to provide the entirety of the previous block to the runtime when asking it to construct inherents.
//! This module provides an inherent data provider that does just this. Any Tuxedo runtime that uses inherents (At least ones,
//! like I've described above), they need to first include this foundational inherent data provider that provvides the previous
//! block so that the Tuxedo executive can scrape it to find the output references of the previous inherent transactions.
//!
//! # Inherent-Per-Block Cadence Assumption
//!
//! This entire process assumes that the previous state that needs to be consumed exists in the immediate parent block.
//! This is only guaranteed if the inherent is included in every single block. Currently all the usecases I mentioned above
//! meet this assumption, and this seems to be the way that inehrents are used in general.
//!
//! If we find that there are compelling usecases for a more flexible cadence, some options to explore include:
//! 1. Use a custom authoring trait which we are already considering for end-of-block inherents anyway.
//! 2. Use the auxiliary storage to keep track of the utxo refs on a block-by-block basis
//!
//! I'm starting to think that inherents are always going to be tied to block cadence. If this is not the case,
//! I would argue that the task of inserting data should not be tied to block authorship, and instead left to some kind of on-chain
//! game or dao that anyone can participate in.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_inherents::{
    CheckInherentsResult, InherentData, InherentIdentifier, IsFatalError, MakeFatalError,
};
use sp_runtime::traits::Block as BlockT;
use sp_std::{vec, vec::Vec};

use crate::{types::Transaction, ConstraintChecker, Verifier};

/// An inherent identifier for the Tuxedo parent block inherent
pub const PARENT_INHERENT_IDENTIFIER: InherentIdentifier = *b"prnt_blk";

/// An inherent data provider that inserts the previous block into the inherent data.
/// This data does NOT go into an extrinsic.
pub struct ParentBlockInherentDataProvider<Block>(pub Block);

#[cfg(feature = "std")]
impl<B> sp_std::ops::Deref for ParentBlockInherentDataProvider<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[cfg(feature = "std")]
#[async_trait::async_trait]
impl<B: BlockT> sp_inherents::InherentDataProvider for ParentBlockInherentDataProvider<B> {
    async fn provide_inherent_data(
        &self,
        inherent_data: &mut InherentData,
    ) -> Result<(), sp_inherents::Error> {
        inherent_data.put_data(PARENT_INHERENT_IDENTIFIER, &self.0)
    }

    async fn try_handle_error(
        &self,
        identifier: &InherentIdentifier,
        error: &[u8],
    ) -> Option<Result<(), sp_inherents::Error>> {
        if identifier == &PARENT_INHERENT_IDENTIFIER {
            println!("UH OH! INHERENT ERROR!!!!!!!!!!!!!!!!!!!!!!");
            Some(Err(sp_inherents::Error::Application(Box::from(
                String::decode(&mut &error[..]).ok()?,
            ))))
        } else {
            None
        }
    }
}

/// Tuxedo's interface around Substrate's concept of inherents.
///
/// Tuxedo assumes that each inherent will appear exactly once in each block.
/// It is recommended that inherent constraint checkers use their Accumulator to verify this
/// at the end of each block.
///
/// This interface is stricter and more structured, and therefore simpler than FRAME's.
pub trait TuxedoInherent<V: Verifier, C: ConstraintChecker<V>>: Sized + TypeInfo {
    type Error: Encode + IsFatalError;

    const INHERENT_IDENTIFIER: InherentIdentifier;

    /// Create the inherent extrinsic to insert into a block that is being authored locally.
    /// The inherent data is supplied by the authoring node.
    fn create_inherent(
        authoring_inherent_data: &InherentData,
        previous_inherent: Transaction<V, C>,
    ) -> Transaction<V, C>;

    /// Perform off-chain pre-execution checks on the inherents.
    /// The inherent data is supplied by the importing node.
    /// The inherent data available here is not guaranteed to be the
    /// same as what is available at authoring time.
    fn check_inherent(
        importing_inherent_data: &InherentData,
        inherent: Transaction<V, C>,
        results: &mut CheckInherentsResult,
    );
}

impl<V: Verifier, C: ConstraintChecker<V>> TuxedoInherent<V, C> for () {
    type Error = MakeFatalError<()>;
    const INHERENT_IDENTIFIER: InherentIdentifier = *b"no_inher";

    fn create_inherent(_: &InherentData, _: Transaction<V, C>) -> Transaction<V, C> {
        panic!("Attempting to create an inherent for a constraint checker that does not support inherents.")
    }

    fn check_inherent(_: &InherentData, _: Transaction<V, C>, _: &mut CheckInherentsResult) {
        panic!("Attemping to perform inherent checks for a constraint checker that does not support inherents.")
    }
}

//TODO I didn't intend for this trait to be public.
// But I can't use it in the public constraint checker trait if it isn't public.
/// Almost identical to TuxedoInherent, but allows returning multiple
/// exrinsics (as aggregate runtimes will need to) and removes the
/// requirement that the generic outer constraint checker be buildable
/// from `Self` so we can implement it for ().
pub trait InherentInternal<V: Verifier, C: ConstraintChecker<V>>: Sized + TypeInfo {
    type Error: Encode + IsFatalError;

    /// Create the inherent extrinsic to insert into a block that is being authored locally.
    /// The inherent data is supplied by the authoring node.
    fn create_inherents(
        authoring_inherent_data: &InherentData,
        previous_inherents: Vec<Transaction<V, C>>,
    ) -> Vec<Transaction<V, C>>;

    /// Perform off-chain pre-execution checks on the inherents.
    /// The inherent data is supplied by the importing node.
    /// The inherent data available here is not guaranteed to be the
    /// same as what is available at authoring time.
    fn check_inherent(
        importing_inherent_data: &InherentData,
        inherent: Transaction<V, C>,
        results: &mut CheckInherentsResult,
    );
}

impl<V: Verifier, C: ConstraintChecker<V>, T: TuxedoInherent<V, C>> InherentInternal<V, C> for T {
    type Error = <T as TuxedoInherent<V, C>>::Error;

    fn create_inherents(
        authoring_inherent_data: &InherentData,
        previous_inherents: Vec<Transaction<V, C>>,
    ) -> Vec<Transaction<V, C>> {
        if previous_inherents.len() > 1 {
            panic!("Authoring a leaf inherent constraint checker, but multiple previous inherents were supplied.")
        }

        // This is how the first block hack manifests in the core
        // TODO remove this if statement once genesis inherents are supported.
        if previous_inherents.is_empty() {
            return Vec::new();
        }

        let previous_inherent = previous_inherents.get(0).expect(
            "Authoring a leaf inherent constraint checker, but no previous inherent was supplied.",
        ).clone();

        // This is the magic. We just take the single transaction from the individual piece
        // and put it into a vec so it can be aggregated.
        vec![<T as TuxedoInherent<V, C>>::create_inherent(
            authoring_inherent_data,
            previous_inherent,
        )]
    }

    fn check_inherent(
        importing_inherent_data: &InherentData,
        inherent: Transaction<V, C>,
        results: &mut CheckInherentsResult,
    ) {
        <T as TuxedoInherent<V, C>>::check_inherent(importing_inherent_data, inherent, results)
    }
}
