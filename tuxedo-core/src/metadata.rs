//! A simple type to use as metadata. For now the metadata just communicates whether we
//! are dealing with a parachain or not.

use parity_scale_codec::{Decode, Encode};
#[derive(Default, Debug, Encode, Decode)]
pub struct TuxedoMetadata {
    /// Placeholder for the scale info type registry that will hopefully eventually go here.
    _registry: (),
    /// Indicator of whether this chain is a parachain or not.
    parachain: bool,
}

impl TuxedoMetadata {
    pub fn new_parachain() -> Self {
        Self {
            _registry: (),
            parachain: true,
        }
    }

    pub fn is_parachain(&self) -> bool {
        self.parachain
    }
}
