//! This module  contains the definition of the TuxedoBlockBuilder runtime api.
//! 
//! Currently this api is intended to be supplimentary to the standard Substrate BlockBuilder
//! api, but depending on the direction the standard trait evolves, this Tuxedo-specific trait
//! may eventually become a complete replacement for the standard block builder trait.
//! See https://github.com/polkadot-fellows/RFCs/pull/13

use sp_api::{BlockT, decl_runtime_apis};
use sp_inherents::InherentData;

decl_runtime_apis! {
    /// A runtime API for Tuxedo chains to coordinate end of block inherents.
    /// This trait is supplementary to the standard Substrate Block Builder api.
    pub trait TuxedoBlockBuilder {
        /// Generate the for the end of the block inherent extrinsics.
        /// The inherent data will vary from chain to chain.
		fn closing_inherent_extrinsics(
			inherent: InherentData,
		) -> sp_std::vec::Vec<<Block as BlockT>::Extrinsic>;
    }
}