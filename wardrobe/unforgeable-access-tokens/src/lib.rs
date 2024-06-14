//! An unforgeable access token is an NFT whose purpose is to grant access to some
//! on-chain functionality to the bearer.
//! 
//! For example, some chains have the ability to pay bounties to users, modify balances,
//! or even upgrade the code of the runtime itself. None of these functionalities should
//! be exposed to the general public.
//! 
//! An unforgeable token can be created at genesis or through a transaction. Each new
//! unforgeable token has a unique id based on the block hash and the creation transaction's
//! index in the block. This prevents the same token from ever being created twice.
//! 
//! ## Managing Ownership
//! 
//! This piece very little in terms of managing ownership of unforgeable tokens.
//! That is because unforgeable tokens can be managed using the same kinds of verifiers or
//! on-chain daos that any other token is managed with. The most obvious way is Tuxedo's verifiers.
//! 
//! For a simple example, consider a privileged address who should be the only one to access
//! the unforgeable token. You can achieve this by protecting the unforgeable token with simple
//! signature checking verifier. Or if you want something more akin to a council, you could use
//! a multisig verifier.
//! 
//! In order to change the sudo account or update the multisig members, a single constraint
//! checker called `BumpToken` exists. It consumes a single unforgeable token and re-creates
//! the same token. This allows the token holder to swap verifiers when necessary.
//! 
//! ## Composition with other constraint checkers
//! 
//! This piece does not provide much functionality itself. It only provides the ability to create
//! and bump unforgeable tokens. In order for those tokens to be useful, this piece should
//! be composed with one or more additional pieces...

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData}, ensure, SimpleConstraintChecker
};

// #[cfg(test)]
// mod tests;

/// A simple non-fungible token that can not be forged
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct UnforgeableToken{
    /// Sequential serial number for each unforgeable name created.
    serial_number: u32,
}

impl UtxoData for UnforgeableToken {
    const TYPE_ID: [u8; 4] = *b"unfo";
}

/// The counter for the serial number of each created unforgeable token.
/// 
/// If you want to allow creating unforgeable tokens after genesis, this
/// must be present in the genesis config. No new tokens can be created
/// without peeking at this one.
/// 
/// This token should be unique in the runtime.
/// 
/// For now we require a total ordering over the creation of new unforgeable tokens.
/// This is how we guarantee that you never create two with the same id.
/// I have a suspicion this could be improved with the use of splittable / mergable
/// pseudo random number generators.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct UnforgeableTokenFactory {
    /// The serial number of the next unforgeable token that will be created.
    pub next_serial: u32,
}

impl UtxoData for UnforgeableTokenFactory {
    const TYPE_ID: [u8; 4] = *b"unff";
}

/// Reasons that the sudo token constraint checkers may fail
#[derive(Debug, Eq, PartialEq)]
pub enum UnforgeableTokenError {
    // Bumping

    /// No inputs were presented in the transaction. But the sudo token must be consumed.
    NoInputs,
    /// The first input to the transaction must be the sudo token, but it was not.
    InputIsNotUnforgeableToken,
    /// 
    NoOutput,
    ///
    OutputIsNotUnforgeableToken,
    ///
    NoFirstOutput,

    // Creating

    /// You have not consumed the proper unforgeable token factory to create a new unforgeable token.
    NoFactoryPresent,
    
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// Allows updating the verifier that is protecting a particular unforgeable token.
pub struct BumpUnforgeableToken;

impl SimpleConstraintChecker for BumpUnforgeableToken {
    type Error = UnforgeableTokenError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        peek_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // ensure one properly typed input
        // unsure one properly typed output
        // ensure input equals output
        todo!()
    }
}

// TODO is this every actually necessary?
// Is it better to make another pice do the creation through the help of a trait?
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
/// Allows a user to create a new unforgeable token by calling this transaction.
/// 
/// Not all runtimes will want to expose this functionality to users.
/// In simple cases it is sufficient to have a small number of unforgeable tokens
/// created at genesis.
pub struct CreateUnforgeableToken;

impl SimpleConstraintChecker for CreateUnforgeableToken {
    type Error = UnforgeableTokenError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        peek_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // ensure one input that is a factory
        // unsure first output is the properly updated factory
        // ensure other output is the correct new unforgeable token
        todo!()
    }
}


/// DO NOT USE IN PRODUCTION!!!!!!!!
/// 
/// Allows a user to forge an unforgeable token. This could be useful for testing
/// purposes. This is also a really useful transaction type to study to help new
/// users understand where the security of unforgeable access tokens comes from.
/// 
/// Take note of the differences between this transaction and the normal creation
/// transactions. That one requires the use of a factory which guarantees the tokens
/// are unique. This one does not have any factory to confirm the serial numbers are
/// unique, and thus the tokens are forgeable.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct ForgeUnforgeableToken;

impl SimpleConstraintChecker for ForgeUnforgeableToken {
    type Error = UnforgeableTokenError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        peek_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // ensure no inputs
        // ensure one properly typed output
        todo!()
    }
}