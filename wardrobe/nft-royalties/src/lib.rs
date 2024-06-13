//! Simple NFTs with on-chain royalties for the creator (or other beneficiary).
//!
//! The purpose of this piece is to show how to implement payments to specific
//! users while keeping a strong verifier / constraint checker separation.
//!
//! ## Creating NFTs
//! Anyone can create an NFT at any time. The creator specifies metadata,
//! an owner, and a royalty rate and beneficiary. Both the owner and the royalty
//! beneficiary can be the creator.
//!
//! The creation transaction specifies the royalty beneficiary specifically by
//! creating a unique token that they privately own. This token can later be
//! used to claim royalty payments.
//!
//! ## Transferring NFTs
//! There is a dedicated extrinsic to transfer the NFT to a new owner.
//! The exchange of cash is built in to discourage out-of-band payments.
//! For the transfer to be valid a correct royalty amount must be paid.
//!
//! It is always possible to bypass the royalty payment when the exchange
//! happens between trusted parties, by making the bulk of the payment in
//! an unrelated (And possibly even off-chain) cash transaction, reporting
//! only a small amount in the official sale transaction. This happens in
//! real life as well, for example, when stating a low purchase price for
//! a used car in order to avoid taxes.
//!
//! ## Claiming royalties
//! The process for claiming royalty payments relies on an access token.
//! When the creator makes the NFT they are create a beneficiary token that represents
//! the right to claim royalty payments. After the NFT has been traded one
//! or more times, and some royalty payments have been made, the beneficiary
//! can submit a transaction claiming all of those royalty payments and
//! consolidating them into one or more coins that they own.
//!
//! ## Deferred claiming and object capabilities
//! This process is inspired by the object capability paradigm. It may
//! initially seem unintuitive to those who are coming from the account model
//! and are familiar with the access control lists that are ubiquitous there.
//!
//! This technique is well suited for the UTXO model, and Tuxedo Tailors should
//! study it.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, SimpleConstraintChecker,
};

// #[cfg(test)]
// mod tests;

/// A universal creator. It must be present for the creation of each and every NFT.
/// It's job is to ensure a unique id for each NFT.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct UniversalCreator {
    next_id: u32,
}

impl UtxoData for UniversalCreator {
    const TYPE_ID: [u8; 4] = *b"uc__";
}

/// A simple opaque NFT. Any special meaning it has is in its metadata.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Nft {
    /// The NFTs metadata. This is to be interpreted by the application.
    /// No meaning or structure is assumed by this piece.
    pub metadata: Vec<u8>,
    /// The globally unique identifier for this NFT among all NFTs managed by this piece.
    pub id: u32,
    /// The flat royalty payment amount.
    /// TODO it would be more realistic to have a percentage, but going into fixed point here would be distracting.
    pub royalty_amount: u32,
}

impl UtxoData for Nft {
    const TYPE_ID: [u8; 4] = *b"NFT!";
}

/// A token that represents the right to claim roylaties for
/// future transfers of a particular NFT.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct RoyaltyRights {
    /// The id of the NFT to which this token grants royalty rights.
    id: u32,
}

impl UtxoData for RoyaltyRights {
    const TYPE_ID: [u8; 4] = *b"rltR";
}

/// A not-yet-claimed royalty payment.
///
/// These are created during each and every transfer of the NFT.
/// Royalty payments may later be claimed by the holder of the rights
/// token. Claims may be batched.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct RoyaltyPayment {
    /// The id of the NFT that generated this royalty payment.
    id: u32,
    /// The amount of money held in this payment.
    value: u32,
}

impl UtxoData for RoyaltyPayment {
    const TYPE_ID: [u8; 4] = *b"rltP";
}

/// Reasons that the NFT constraint checkers may fail
#[derive(Debug, Eq, PartialEq)]
pub enum NftError {
    /// An input data has the wrong type.
    BadlyTypedInput,
    /// An output data has the wrong type.
    BadlyTypedOutput,

    /// TODO
    TODO,
}

/// Create a new NFT.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct CreateNft;

impl SimpleConstraintChecker for CreateNft {
    type Error = NftError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Check the universal creator is the first input and output with the correct ids.
        // Check the NFT has the correct ID.
        // Check that the royalty rights claim token has the right id.
        todo!()
    }
}

/// Transfer an NFT paying the appropriate royalties.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct NftTransfer;

impl SimpleConstraintChecker for NftTransfer {
    type Error = NftError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure the first input and output are the NFT unchanged.
        // (The NFT is unchanged at the _constraint checker_ level, but most likely the verifier changed.)
        
        // Check that the correct royalty payment has been made.

        // Check that the input and output amounts and royalty payment add up.
        todo!()
    }
}

/// As the beneficiary of an NFT, collect one or more of your royalties.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct CollectRoyalty;

impl SimpleConstraintChecker for CollectRoyalty {
    type Error = NftError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Make sure the first input and output are the same royalty rights.
        // Make sure that each payment is for the correct NFT and sum values.
        // Make sure that remaining outputs are coins with value less than or equal to the royalty amount.

        todo!()
    }
}
