//! TODO explanation this type

use crate::{
    types::{Output, OutputRef},
    verifier::Verifier,
    LOG_TARGET,
};
use parity_scale_codec::{Decode, Encode};
use sp_std::marker::PhantomData;

pub struct TransparentUtxoSet<Verifier>(PhantomData<Verifier>);

impl<V: Verifier> TransparentUtxoSet<V> {
    /// Fetch a utxo from the set.
    pub fn peek_utxo(output_ref: &OutputRef) -> Option<Output<V>> {
        sp_io::storage::get(&output_ref.encode()).and_then(|d| Output::decode(&mut &*d).ok())
    }

    /// Consume a Utxo from the set.
    pub fn consume_utxo(output_ref: &OutputRef) -> Option<Output<V>> {
        // TODO do we even need to read the stored value here? The only place we call this
        // is from `update_storage` and we don't use the value there.
        let maybe_output = Self::peek_utxo(output_ref);
        sp_io::storage::clear(&output_ref.encode());
        maybe_output
    }

    /// Add a utxo into the set.
    /// This will overwrite any utxo that already exists at this OutputRef. It should never be the
    /// case that there are collisions though. Right??
    pub fn store_utxo(output_ref: OutputRef, output: &Output<V>) {
        let key = output_ref.encode();
        log::debug!(
            target: LOG_TARGET,
            "Storing UTXO at key: {:?}",
            sp_core::hexdisplay::HexDisplay::from(&key)
        );
        sp_io::storage::set(&key, &output.encode());
    }
}
