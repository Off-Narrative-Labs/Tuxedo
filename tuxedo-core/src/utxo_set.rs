//! These three functions comprize what I sketched out in the UtxoSet trait in element the other day
//! For now, this is complicated enough, so I'll just leave them here. In the future it may be wise to
//! abstract away the utxo set. Especially if we start doing zk stuff and need the nullifiers.

use sp_std::marker::PhantomData;
use crate::{redeemer::Redeemer, types::{OutputRef, Output}};
use parity_scale_codec::{Encode, Decode};

pub struct TransparentUtxoSet<Redeemer>(PhantomData<Redeemer>);

impl<R: Redeemer> TransparentUtxoSet<R> {
    /// Fetch a utxo from the set.
    pub fn peek_utxo(output_ref: &OutputRef) -> Option<Output<R>> {
        sp_io::storage::get(&output_ref.encode()).and_then(|d| Output::decode(&mut &*d).ok())
    }

    /// Consume a Utxo from the set.
    pub fn consume_utxo(output_ref: &OutputRef) -> Option<Output<R>> {
        // TODO do we even need to read the stored value here? The only place we call this
        // is from `update_storage` and we don't use the value there.
        let maybe_output = Self::peek_utxo(output_ref);
        sp_io::storage::clear(&output_ref.encode());
        maybe_output
    }

    /// Add a utxo into the set.
    /// This will overwrite any utxo that already exists at this OutputRef. It should never be the
    /// case that there are collisions though. Right??
    pub fn store_utxo(output_ref: OutputRef, output: &Output<R>) {
        sp_io::storage::set(&output_ref.encode(), &output.encode());
    }
}
