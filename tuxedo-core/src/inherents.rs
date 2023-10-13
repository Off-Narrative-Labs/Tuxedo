//! APIs and utilities for working with Substrate's Inherents in Tuxedo based chains.
//!
//! # Substrate inherents
//!
//! Inherents are a Substrate feature that allows block authors to insert some transactions directly
//! into the body of the block. Inherents are similar to pre-runtime digests which allow authors to
//! insert info into the block header. However inherents go in the block body and therefore must be transactions.
//!
//! Classic usecases for inherents are injecting and updating environmental information such as a block timestamp,
//! information about the relay chain (if the current chain is a parachain), or information about who should receive the block reward.
//!
//! In order to allow the runtime to construct such transactions while keeping the cleint opaque, there are special APIs
//! for creating inherents and performing off-chain validation of inherents. That's right, inherents also offer
//! a special API to have their environmental data checked off-chain before the block is executed.
//!
//! # Complexities in UTXO chains
//!
//! In account based systems, the classic way to use an inherent is that the block inserts a transaction providing some data like a timestamp.
//! When the extrinsic executed it, overwrites the previously stored timestamp in a dedicated storage item.
//!
//! In UTXO chains, there are no storage items, and all state is local to a UTXO. This is the case with, for example, the timestamp as well.
//! This means that when the author calls into the runtime with a timestamp, the transaction that is returned must include the correct reference
//! to the UTXO that contained the previous best timestamp. This is the crux of the problem: there is no easy way to know the location of
//! the previous timestamp in the utxo-space from inside the runtime.
//!
//! # Scraping the Parent Block
//!
//! The solution is to provide the entirety of the previous block to the runtime when asking it to construct inherents.
//! This module provides an inherent data provider that does just this. Any Tuxedo runtime that uses inherents (At least ones
//! that update environmental data), needs to include this foundational previous block inherent data provider that provvides
//! so that the Tuxedo executive can scrape it to find the output references of the previous inherent transactions.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::H256;
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
#[cfg(feature = "std")]
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

/// Tuxedo's controlled interface around Substrate's concept of inherents.
///
/// This interface assumes that each inherent will appear exactly once in each block.
/// This will be verified off-chain by nodes before block execution begins.
///
/// This interface is stricter and more structured, and therefore simpler than FRAME's.
/// If you need to do something more powerful (which you probably don't) and you
/// understand exactly how Substrate's block authoring and Tuxedo's piece aggregation works
/// (which you probably don't) you can directly implement the `InherentInternal` trait
/// which is more powerful (and dangerous).
pub trait TuxedoInherent<V, C: ConstraintChecker<V>>: Sized {
    type Error: Encode + IsFatalError;

    const INHERENT_IDENTIFIER: InherentIdentifier;

    /// Create the inherent extrinsic to insert into a block that is being authored locally.
    /// The inherent data is supplied by the authoring node.
    fn create_inherent(
        authoring_inherent_data: &InherentData,
        // The option represents the so-called "first block hack".
        // We need a way to initialize the chain with a first inherent on block one
        // where there is no previous inherent. Once we introduce genesis extrinsics, this can be removed.
        previous_inherent: Option<(Transaction<V, C>, H256)>,
    ) -> Transaction<V, C>;

    /// Perform off-chain pre-execution checks on the inherent.
    /// The inherent data is supplied by the importing node.
    /// The inherent data available here is not guaranteed to be the
    /// same as what is available at authoring time.
    fn check_inherent(
        importing_inherent_data: &InherentData,
        inherent: Transaction<V, C>,
        results: &mut CheckInherentsResult,
    );
}

/// Almost identical to TuxedoInherent, but allows returning multiple extrinsics
/// (as aggregate runtimes  will need to) and removes the requirement that the generic
/// outer constraint checker be buildable from `Self` so we can implement it for ().
///
/// If you are trying to implement some complex inherent logic that requires the interaction of
/// multiple inherents, or features a variable number of inherents in each block, you might be
/// able to express it by implementing this trait, but such designs are probably too complicated.
/// Think long and hard before implementing this trait directly.
pub trait InherentInternal<V, C: ConstraintChecker<V>>: Sized {
    /// Create the inherent extrinsic to insert into a block that is being authored locally.
    /// The inherent data is supplied by the authoring node.
    fn create_inherents(
        authoring_inherent_data: &InherentData,
        previous_inherents: Vec<(Transaction<V, C>, H256)>,
    ) -> Vec<Transaction<V, C>>;

    /// Perform off-chain pre-execution checks on the inherents.
    /// The inherent data is supplied by the importing node.
    /// The inherent data available here is not guaranteed to be the
    /// same as what is available at authoring time.
    fn check_inherents(
        importing_inherent_data: &InherentData,
        inherents: Vec<Transaction<V, C>>,
        results: &mut CheckInherentsResult,
    );
}

/// An adapter to transform structured Tuxedo inherents into the more general and powerful
/// InherentInternal trait.
#[derive(Debug, Default, TypeInfo, Clone, Copy)]
pub struct TuxedoInherentAdapter<T>(T);

impl<V: Verifier, C: ConstraintChecker<V>, T: TuxedoInherent<V, C> + 'static> InherentInternal<V, C>
    for TuxedoInherentAdapter<T>
{
    fn create_inherents(
        authoring_inherent_data: &InherentData,
        previous_inherents: Vec<(Transaction<V, C>, H256)>,
    ) -> Vec<Transaction<V, C>> {
        if previous_inherents.len() > 1 {
            panic!("Authoring a leaf inherent constraint checker, but multiple previous inherents were supplied.")
        }

        let previous_inherent = previous_inherents.get(0).cloned();

        vec![<T as TuxedoInherent<V, C>>::create_inherent(
            authoring_inherent_data,
            previous_inherent,
        )]
    }

    fn check_inherents(
        importing_inherent_data: &InherentData,
        inherents: Vec<Transaction<V, C>>,
        results: &mut CheckInherentsResult,
    ) {
        if inherents.is_empty() {
            results
                .put_error(
                    *b"12345678",
                    &MakeFatalError::from(
                        "Tuxedo inherent expected exactly one inherent extrinsic but found zero",
                    ),
                )
                .expect("Should be able to put an error.");
            return;
        } else if inherents.len() > 1 {
            results
                .put_error(*b"12345678", &MakeFatalError::from("Tuxedo inherent expected exactly one inherent extrinsic but found multiple"))
                .expect("Should be able to put an error.");
            return;
        }
        let inherent = inherents
            .get(0)
            .expect("We already checked the bounds.")
            .clone();
        <T as TuxedoInherent<V, C>>::check_inherent(importing_inherent_data, inherent, results)
    }
}

impl<V, C: ConstraintChecker<V>> InherentInternal<V, C> for () {
    fn create_inherents(
        _: &InherentData,
        _: Vec<(Transaction<V, C>, H256)>,
    ) -> Vec<Transaction<V, C>> {
        Vec::new()
    }

    fn check_inherents(
        _: &InherentData,
        inherents: Vec<Transaction<V, C>>,
        _: &mut CheckInherentsResult,
    ) {
        // Inherents should always be empty for this stub implementation. Not just in valid blocks, but as an invariant.
        // The way we determined which inherents got here is by matching on the constraint checker.
        assert!(
            inherents.is_empty(),
            "inherent extrinsic was passed to check inherents stub implementation."
        )
    }
}
