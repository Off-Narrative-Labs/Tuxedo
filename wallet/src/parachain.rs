//! Parachain compatibility for the template wallet.
//!
//! The wallet is not intended to support a wide variety of chains, but it is able
//! to support both the sovereign and parachain template nodes. There are a few types
//! necessary to make this work.
//!
//! We don't want the wallet to depend on the parachain runtime which has a huge
//! dependency graph itself. So a few types are duplicated here.

use parity_scale_codec::{Decode, Encode};
use runtime::OuterConstraintChecker;
use tuxedo_core::SimpleConstraintChecker;

/// We don't want the wallet to depend on the huge parachain codebase,
/// So we just recreate this one little type here.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum ParachainConstraintChecker {
    Normal(OuterConstraintChecker),
    Parachain,
}

impl SimpleConstraintChecker for ParachainConstraintChecker {
    type Error = ();

    fn check(
        &self,
        _: &[tuxedo_core::dynamic_typing::DynamicallyTypedData],
        _: &[tuxedo_core::dynamic_typing::DynamicallyTypedData],
        _: &[tuxedo_core::dynamic_typing::DynamicallyTypedData],
        _: &[tuxedo_core::dynamic_typing::DynamicallyTypedData],
    ) -> Result<sp_runtime::transaction_validity::TransactionPriority, Self::Error> {
        todo!()
    }
}

impl From<OuterConstraintChecker> for ParachainConstraintChecker {
    fn from(c: OuterConstraintChecker) -> Self {
        ParachainConstraintChecker::Normal(c)
    }
}
