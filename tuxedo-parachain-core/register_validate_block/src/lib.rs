//! This macro is copied from cumulus-pallet-parachain-system-proc-macro crate
//! and modified slightly to fit Tuxedo's needs.

use proc_macro2::{Literal, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, Token,
};

/// Provides an identifier that is a safe way to refer to the crate tuxedo_parachain_core within the macro
fn crate_() -> Result<Ident, Error> {
    match crate_name("tuxedo-parachain-core") {
        Ok(FoundCrate::Itself) => Ok(syn::Ident::new("tuxedo_parachain_core", Span::call_site())),
        Ok(FoundCrate::Name(name)) => Ok(Ident::new(&name, Span::call_site())),
        Err(e) => Err(Error::new(Span::call_site(), e)),
    }
}

struct RegisterValidateBlockInput {
    pub verifier: Ident,
    _comma1: Token![,],
    pub inner_constraint_checker: Ident,
    _comma2: Token![,],
    pub para_id: Literal,
}

impl Parse for RegisterValidateBlockInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed = Self {
            verifier: input.parse()?,
            _comma1: input.parse()?,
            inner_constraint_checker: input.parse()?,
            _comma2: input.parse()?,
            para_id: input.parse()?,
        };

        if !input.is_empty() {
            return Err(Error::new(
                input.span(),
                "Expected exactly three parameters: Verifier, InnerConstraintChecker, ParaId.",
            ));
        }

        Ok(parsed)
    }
}

//TODO rename the macro
#[proc_macro]
pub fn register_validate_block(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Extract the paths to the parts from the runtime developer's input
    // I will likely need to revise or simplify the fields that are passed in.
    // I hope to only use the exposed runtime APIs here, not some custom trait impls. (if possible)
    let input: RegisterValidateBlockInput = match syn::parse(input) {
        Ok(t) => t,
        Err(e) => return e.into_compile_error().into(),
    };

    let verifier = input.verifier.clone();
    let inner_constraint_checker = input.inner_constraint_checker.clone();
    let para_id = input.para_id.clone();

    // A way to refer to the tuxedo_parachain_core crate from within the macro.
    let crate_ = match crate_() {
        Ok(c) => c,
        Err(e) => return e.into_compile_error().into(),
    };

    // Implementation of Polkadot's validate_block function. Inspired by Basti's frame version:
    // https://github.com/paritytech/polkadot-sdk/blob/0becc45b/cumulus/pallets/parachain-system/proc-macro/src/lib.rs#L93-L153
    let validate_block_func = if cfg!(not(feature = "std")) {
        quote::quote! {
            #[doc(hidden)]
            mod parachain_validate_block {
                use super::*;

                #[no_mangle]
                unsafe fn validate_block(arguments: *mut u8, arguments_len: usize) -> u64 {
                    // There is some complex low-level shared-memory stuff implemented here.
                    // It is basically a wrapper around the validate block implementation
                    // that handles extracting params and returning results via shared memory.

                    // Step 1. Extract the arguments from shared memory
                    // We convert the `arguments` into a boxed slice and then into `Bytes`.
                    let args = #crate_::sp_std::boxed::Box::from_raw(
                        #crate_::sp_std::slice::from_raw_parts_mut(
                            arguments,
                            arguments_len,
                        )
                    );
                    let args = #crate_::bytes::Bytes::from(args);

                    // Then we decode from these bytes the `MemoryOptimizedValidationParams`.
                    let params = #crate_::decode_from_bytes::<
                        #crate_::MemoryOptimizedValidationParams
                    >(args).expect("Invalid arguments to `validate_block`.");

                    // Step 2: Call the actual validate_block implementation
                    let res = #crate_::validate_block::validate_block::<
                        #verifier,
                        ParachainConstraintChecker,
                    >(params);

                    // Step 3: Write the return value back into the shared memory
                    let return_pointer = #crate_::polkadot_parachain_primitives::write_result(&res);

                    return_pointer
                }
            }
        }
    } else {
        // If we are building to std, we don't include this validate_block entry point at all
        quote::quote!()
    };

    // Write the piece config and the `ParachainConstraintChecker` enum.
    let parachain_constraint_checker_enum = quote::quote! {
        #[derive(PartialEq, Eq, Clone)]
        pub struct RuntimeParachainConfig;
        impl parachain_piece::ParachainPieceConfig for RuntimeParachainConfig {
            // Use the para ID 2_000 which is the first available in the rococo-local runtime.
            // This is the default value, so this could be omitted, but explicit is better.
            const PARA_ID: u32 = #para_id;

            type SetRelayParentNumberStorage = tuxedo_parachain_core::RelayParentNumberStorage;
        }

        /// The Outer / Aggregate Constraint Checker for the Parachain runtime.
        ///
        /// It is comprized of two individual checkers:
        ///   First, the parachain inherent piece
        ///   Second, the constraint checker from the normal Tuxedo Template Runtime.
        ///
        /// That second checker, the normal tuxedo template runtime, is itself an aggregate
        /// constraint checker aggregated from individual pieces such as money, amoeba, and others.
        /// Therefore, this crate shows:
        ///   Generally, how to perform recursive aggregation of constraint checkers.
        ///   Specifically, how to elegantly transform a sovereign runtime into a parachain runtime by wrapping.
        #[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
        #[tuxedo_constraint_checker]
        pub enum ParachainConstraintChecker {
            /// All other calls are delegated to the normal Tuxedo Template Runtime.
            Inner(#inner_constraint_checker),

            /// Set some parachain related information via an inherent extrinsic.
            ParachainInfo(InherentAdapter<parachain_piece::SetParachainInfo<RuntimeParachainConfig>>),
        }

        // We provide a way for the relay chain validators to extract the parachain inherent data from
        // a raw transaction.
        impl #crate_::ParachainConstraintChecker for ParachainConstraintChecker {

            fn is_parachain(&self) -> bool {
                // TODO does this still match as expected when self is a reference?
                matches!(self, Self::ParachainInfo(_))
            }
        }
    };

    // The final output is the `ParachainConstraintChecker` plus the `validate_block` function.

    quote::quote! {
        #validate_block_func

        #parachain_constraint_checker_enum
    }
    .into()
}
