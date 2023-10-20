//! This macro is copied from cumulus-pallet-parachain-system-proc-macro crate
//! and modified slightly to fit Tuxedo's needs.

use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token, Error, Ident, Path,
};

mod keywords {
    syn::custom_keyword!(Runtime);
    syn::custom_keyword!(BlockExecutor);
    syn::custom_keyword!(CheckInherents);
}

struct Input {
    runtime: Path,
    block_executor: Path,
    check_inherents: Option<Path>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let mut runtime = None;
        let mut block_executor = None;
        let mut check_inherents = None;

        fn parse_inner<KW: Parse + Spanned>(
            input: ParseStream,
            result: &mut Option<Path>,
        ) -> Result<(), Error> {
            let kw = input.parse::<KW>()?;

            if result.is_none() {
                input.parse::<token::Eq>()?;
                *result = Some(input.parse::<Path>()?);
                if input.peek(token::Comma) {
                    input.parse::<token::Comma>()?;
                }

                Ok(())
            } else {
                Err(Error::new(kw.span(), "Is only allowed to be passed once"))
            }
        }

        while !input.is_empty() || runtime.is_none() || block_executor.is_none() {
            let lookahead = input.lookahead1();

            if lookahead.peek(keywords::Runtime) {
                parse_inner::<keywords::Runtime>(input, &mut runtime)?;
            } else if lookahead.peek(keywords::BlockExecutor) {
                parse_inner::<keywords::BlockExecutor>(input, &mut block_executor)?;
            } else if lookahead.peek(keywords::CheckInherents) {
                parse_inner::<keywords::CheckInherents>(input, &mut check_inherents)?;
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(Self {
            runtime: runtime.expect("Everything is parsed before; qed"),
            block_executor: block_executor.expect("Everything is parsed before; qed"),
            check_inherents,
        })
    }
}

/// Provides an identifier that is a safe way to refer to the crate tuxedo_core within the macro
fn crate_() -> Result<Ident, Error> {
    match crate_name("tuxedo-core") {
        Ok(FoundCrate::Itself) => Ok(syn::Ident::new("tuxedo_core", Span::call_site())),
        Ok(FoundCrate::Name(name)) => Ok(Ident::new(&name, Span::call_site())),
        Err(e) => Err(Error::new(Span::call_site(), e)),
    }
}

#[proc_macro]
pub fn register_validate_block(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Extract the paths to the parts from the runtime developer's input
    // I will likely need to revise or simplify the fields that are passed in.
    // I hope to only use the exposed runtime APIs here, not some custom trait impls. (if possible)
    let Input {
        runtime,
        block_executor,
        check_inherents,
    } = match syn::parse(input) {
        Ok(t) => t,
        Err(e) => return e.into_compile_error().into(),
    };

    // A way to refer to the tuxedo_core crate from within the macro.
    let crate_ = match crate_() {
        Ok(c) => c,
        Err(e) => return e.into_compile_error().into(),
    };

    //TODO We need to check inherents. At least the timestamp one, and maybe also the parachain one?
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
                    let args = #crate_::validate_block::sp_std::boxed::Box::from_raw(
                        #crate_::validate_block::sp_std::slice::from_raw_parts_mut(
                            arguments,
                            arguments_len,
                        )
                    );
                    let args = #crate_::validate_block::bytes::Bytes::from(args);

                    // Then we decode from these bytes the `MemoryOptimizedValidationParams`.
                    let params = #crate_::validate_block::decode_from_bytes::<
                        #crate_::validate_block::MemoryOptimizedValidationParams
                    >(args).expect("Invalid arguments to `validate_block`.");

                    // Step 2: Call the actual validate_block implementation
                    let res = #crate_::validate_block::implementation::validate_block::<
                        <#runtime as #crate_::validate_block::GetRuntimeBlockType>::RuntimeBlock,
                        #block_executor,
                        #runtime,
                        #check_inherents,
                    >(params);

                    // Step 3: Write the return value back into the shared memory
                    #crate_::validate_block::polkadot_parachain_primitives::write_result(&res)
                }
            }
        }
    } else {
        // If we are building to std, we don't include this validate_block entry point at all
        quote::quote!()
    }
    .into()
}
