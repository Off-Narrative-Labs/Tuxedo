//! This macro is copied from cumulus-pallet-parachain-system-proc-macro crate
//! and modified slightly to fit Tuxedo's needs.

use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, Token,
};

/// Provides an identifier that is a safe way to refer to the crate tuxedo_core within the macro
fn crate_() -> Result<Ident, Error> {
    match crate_name("tuxedo-parachain-core") {
        Ok(FoundCrate::Itself) => Ok(syn::Ident::new("tuxedo_parachain_core", Span::call_site())),
        Ok(FoundCrate::Name(name)) => Ok(Ident::new(&name, Span::call_site())),
        Err(e) => Err(Error::new(Span::call_site(), e)),
    }
}

struct RegisterValidateBlockInput {
    pub block: Ident,
    _comma1: Token![,],
    pub verifier: Ident,
    _comma2: Token![,],
    pub constraint_checker: Ident,
}

impl Parse for RegisterValidateBlockInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed = Self {
            block: input.parse()?,
            _comma1: input.parse()?,
            verifier: input.parse()?,
            _comma2: input.parse()?,
            constraint_checker: input.parse()?,
        };

        if !input.is_empty() {
            return Err(Error::new(
                input.span(),
                "Expected exactly three parameters: Block, Verifier, ConstraintChecker.",
            ));
        }

        Ok(parsed)
    }
}

#[proc_macro]
pub fn register_validate_block(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Extract the paths to the parts from the runtime developer's input
    // I will likely need to revise or simplify the fields that are passed in.
    // I hope to only use the exposed runtime APIs here, not some custom trait impls. (if possible)
    let input: RegisterValidateBlockInput = match syn::parse(input) {
        Ok(t) => t,
        Err(e) => return e.into_compile_error().into(),
    };

    let block = input.block.clone();
    let verifier = input.verifier.clone();
    let constraint_checker = input.constraint_checker.clone();

    // A way to refer to the tuxedo_parachain_core crate from within the macro.
    let crate_ = match crate_() {
        Ok(c) => c,
        Err(e) => return e.into_compile_error().into(),
    };

    //TODO We need to check inherents. At least the timestamp one, and maybe also the parachain one?
    // https://github.com/Off-Narrative-Labs/Tuxedo/issues/144
    // But I think the parachain one is handled already.
    // To start the hack, we will just not check them at all. Fewer places to panic XD
    // let check_inherents = match check_inherents {
    // 	Some(_check_inherents) => {
    // 		quote::quote! { #_check_inherents }
    // 	},
    // 	None => {
    // 		quote::quote! {
    // 			#crate_::DummyCheckInherents<<#runtime as #crate_::validate_block::GetRuntimeBlockType>::RuntimeBlock>
    // 		}
    // 	},
    // };

    if cfg!(not(feature = "std")) {
        quote::quote! {
            #[doc(hidden)]
            mod parachain_validate_block {
                use super::*;

                #[no_mangle]
                unsafe fn validate_block(arguments: *mut u8, arguments_len: usize) -> u64 {
                    // There is some complex low-level shared-memory stuff implemented here.
                    // It is basically a wrapper around the validate block implementation
                    // that handles extracting params and returning results via shared memory.

                    // Setp 1. Extract the arguments from shared memory

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
                        #block,
                        #verifier,
                        #constraint_checker,
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
    }
    .into()
}
