//! An Order Book Decentralized Exchange.
//! 
//! Allows users to place trade orders offering a certain amount of
//! one token asking a certain amount of another token in exchange.
//! 
//! Also allows matching sets of compatible orders together.
//! Orders can be matched as long as every ask is fulfilled.
//! 
//! This piece is instantiable and parameterized in two tokens.
//! If you want multiple trading pairs, then you will need multiple
//! instances of this piece.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_std::prelude::*;
use tuxedo_core::{
    Verifier,
    dynamic_typing::{DynamicallyTypedData, DynamicTypingError, UtxoData},
    ensure,
    traits::Cash,
    SimpleConstraintChecker,
    support_macros::{CloneNoBound, DebugNoBound, DefaultNoBound},
};

// TODO Order type


// TODO Error Type


// TODO MakeOrder SimpleConstraintChecker


// TODO MatchOrder ConstraintChecker
