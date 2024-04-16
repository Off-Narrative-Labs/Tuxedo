//! Macros to derive common traits without bounds
//!
//! Copied from frame support

mod clone_no_bound;
mod debug_no_bound;
mod default_no_bound;

use proc_macro::TokenStream;

/// Derive [`Clone`] but do not bound any generic. Docs are at `frame_support::CloneNoBound`.
#[proc_macro_derive(CloneNoBound)]
pub fn derive_clone_no_bound(input: TokenStream) -> TokenStream {
    clone_no_bound::derive_clone_no_bound(input)
}

/// Derive [`Debug`] but do not bound any generics. Docs are at `frame_support::DebugNoBound`.
#[proc_macro_derive(DebugNoBound)]
pub fn derive_debug_no_bound(input: TokenStream) -> TokenStream {
    debug_no_bound::derive_debug_no_bound(input)
}

/// derive `Default` but do no bound any generic. Docs are at `frame_support::DefaultNoBound`.
#[proc_macro_derive(DefaultNoBound, attributes(default))]
pub fn derive_default_no_bound(input: TokenStream) -> TokenStream {
    default_no_bound::derive_default_no_bound(input)
}
