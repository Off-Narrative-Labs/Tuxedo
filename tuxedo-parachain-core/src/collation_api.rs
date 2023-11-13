//! Tuxedo's implementation of the CollectCollationInfoApi.
//! It is pretty basic and just returns the encoded ehader along with some empty data.
//! It will get more complex and interesting when we start to support XCM or parachain runtime upgrades.

use cumulus_primitives_core::{relay_chain::HeadData, CollationInfo};
use parity_scale_codec::Encode;
use sp_api::BlockT;
use sp_std::vec::Vec;
use tuxedo_core::Executive;

use crate::{GetRelayParentNumberStorage, RelayParentNumberStorage};

/// An extension trait that allows us to implement more methods on tuxedo-core's executive.
pub trait ParachainExecutiveExtension<Header> {
    fn collect_collation_info(header: &Header) -> cumulus_primitives_core::CollationInfo;
}

impl<B: BlockT, V, C> ParachainExecutiveExtension<B::Header> for Executive<B, V, C> {
    fn collect_collation_info(header: &B::Header) -> cumulus_primitives_core::CollationInfo {
        // The implementation here is simple. Most of the fields are related to xcm and parachain runtime upgrades,
        // neither or which are supported in the PoC, so they are left blank.

        // Get the relay parent number out of storage so we can advance the hrmp watermark
        let hrmp_watermark = RelayParentNumberStorage::get();

        // The final field allows us to specify head data. We will do the boring / standard / default / original
        // thing which is to just directly encode the block header.
        // The cumulus collator and FRAME pallets allow for custom head data, which seems to be motivated only
        // by the solo to para migration path, so I will put that all off as well. For more details see
        // https://github.com/paritytech/cumulus/pull/825 and https://github.com/paritytech/cumulus/pull/882
        CollationInfo {
            upward_messages: Vec::new(),
            horizontal_messages: Vec::new(),
            new_validation_code: None,
            processed_downward_messages: 0,
            hrmp_watermark,
            head_data: HeadData(header.encode()),
        }
    }
}
