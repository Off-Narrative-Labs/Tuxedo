use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemEnum};

/// Automatically implements `From` for each type in an aggregate type enum.
///
/// The supplied enum should have a single unnamed type parameter for each variant.
/// And the type for each variant should be unique in the enum.
///
/// The macro generates all the `From` implementations automatically.
#[proc_macro_attribute]
pub fn aggregate(_: TokenStream, body: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(body as ItemEnum);
    let original_code = ast.clone();

    let outer_type = ast.ident;
    let variant_type_pairs = ast.variants.iter().map(|variant| {
        // Make sure there is only a single field, and if not, give a helpful error
        assert!(
            variant.fields.len() == 1,
            "Each variant must have a single unnamed field"
        );
        (
            variant.ident.clone(),
            variant
                .fields
                .iter()
                .next()
                .expect("exactly one field per variant")
                .ty
                .clone(),
        )
    });
    let variants = variant_type_pairs.clone().map(|(v, _t)| v);
    let variants2 = variants.clone();
    let inner_types = variant_type_pairs.map(|(_v, t)| t);
    let inner_types2 = inner_types.clone();

    let output = quote! {
        // First keep the original code in tact
        #original_code

        // Now write all the wrapping From impls
        #(
            impl From<#inner_types> for #outer_type {
                fn from(b: #inner_types) -> Self {
                    Self::#variants(b)
                }
            }
        )*

        // Finally write all the un-wrapping From impls
        #(
            impl From<#outer_type> for #inner_types2 {
                fn from(a: #outer_type) -> Self {
                    if let #outer_type::#variants2(b) = a {
                        b
                    } else {
                        panic!("wrong type or something...")
                    }
                }
            }
        )*
    };

    output.into()
}

/// This macro treats the supplied enum as an aggregate verifier. As such, it implements the `From`
/// trait for eah of the inner types. Then it implements the `Verifier` trait for this type for this
/// enum by delegating to an inner type.
#[proc_macro_attribute]
pub fn tuxedo_verifier(_: TokenStream, body: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(body as ItemEnum);
    let original_code = ast.clone();

    let outer_type = ast.ident;
    let variants = ast.variants.into_iter().map(|v| v.ident);

    let output = quote! {

        // Preserve the original enum, and write the From impls
        #[tuxedo_core::aggregate]
        #original_code

        impl tuxedo_core::Verifier for #outer_type {
            fn verify(&self, simplified_tx: &[u8], redeemer: &[u8]) -> bool {
                match self {
                    #(
                        Self::#variants(inner) => inner.verify(simplified_tx, redeemer),
                    )*
                }
            }
        }
    };
    output.into()
}

/// This macro treats the supplied enum as an aggregate constraint checker. As such, it implements the `From`
/// trait for eah of the inner types. Then it implements the `ConstraintChecker` trait for this type for this
/// enum by delegating to an inner type.
///
/// It also declares an associated error type. The error type has a variant for each inner constraint checker,
/// just like this original enum. however, the contained values in the error enum are of the corresponding types
/// for the inner constraint checker.
#[proc_macro_attribute]
pub fn tuxedo_constraint_checker(attrs: TokenStream, body: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(body as ItemEnum);
    let verifier = parse_macro_input!(attrs as Ident);
    let original_code = ast.clone();

    let outer_type = ast.ident;
    let variant_type_pairs = ast.variants.iter().map(|variant| {
        // Make sure there is only a single field, and if not, give a helpful error
        assert!(
            variant.fields.len() == 1,
            "Each variant must have a single unnamed field"
        );
        (
            variant.ident.clone(),
            variant
                .fields
                .iter()
                .next()
                .expect("exactly one field per variant")
                .ty
                .clone(),
        )
    });
    let variants = variant_type_pairs.clone().map(|(v, _t)| v);
    let inner_types = variant_type_pairs.map(|(_v, t)| t);

    // Set up the names of the new associated types.
    let mut error_type_name = outer_type.to_string();
    error_type_name.push_str("Error");
    let error_type = Ident::new(&error_type_name, outer_type.span());

    let mut inherent_hooks_name = outer_type.to_string();
    inherent_hooks_name.push_str("InherentHooks");
    let inherent_hooks = Ident::new(&inherent_hooks_name, outer_type.span());

    let vis = ast.vis;

    // TODO there must be a better way to do this, right?
    let inner_types2 = inner_types.clone();
    let inner_types3 = inner_types.clone();
    let inner_types4 = inner_types.clone();
    let inner_types6 = inner_types.clone();
    let inner_types7 = inner_types.clone();
    let variants2 = variants.clone();
    let variants3 = variants.clone();
    let variants4 = variants.clone();
    let variants5 = variants.clone();
    let variants6 = variants.clone();

    let output = quote! {
        // Preserve the original enum, and write the From impls
        #[tuxedo_core::aggregate]
        #original_code


        /// This type is generated by the `#[tuxedo_constraint_checker]` macro.
        /// It is a combined error type for the errors of each individual checker.
        ///
        /// This type is accessible downstream as `<OuterConstraintChecker as ConstraintChecker>::Error`
        #[derive(Debug)]
        #vis enum #error_type {
            #(
                #variants(<#inner_types as tuxedo_core::ConstraintChecker<#verifier>>::Error),
            )*
        }

        /// This type is generated by the `#[tuxedo_constraint_checker]` macro.
        /// It is a combined set of inherent hooks for the inherent hooks of each individual checker.
        ///
        /// This type is accessible downstream as `<OuterConstraintChecker as ConstraintChecker>::InherentHooks`
        #[derive(Debug, scale_info::TypeInfo)]
        #vis enum #inherent_hooks {
            #(
                #variants2(<#inner_types2 as tuxedo_core::ConstraintChecker<#verifier>>::InherentHooks),
            )*
        }

        impl tuxedo_core::inherents::InherentInternal<#verifier, #outer_type> for #inherent_hooks {

            fn create_inherents(
                authoring_inherent_data: &InherentData,
                previous_inherents: Vec<(tuxedo_core::types::Transaction<#verifier, #outer_type>, sp_core::H256)>,
            ) -> Vec<tuxedo_core::types::Transaction<#verifier, #outer_type>>  {

                let mut all_inherents = Vec::new();

                #(
                    {
                        // Filter the previous inherents down to just the ones that came from this piece
                        let previous_inherents = previous_inherents
                            .iter()
                            .filter_map(|(tx, hash)| {
                                match tx.checker {
                                    #outer_type::#variants3(ref inner_checker) => Some((tx.transform::<#inner_types3>(), *hash )),
                                    _ => None,
                                }
                            })
                            .collect();

                        let inherents = <#inner_types3 as tuxedo_core::ConstraintChecker<#verifier>>::InherentHooks::create_inherents(authoring_inherent_data, previous_inherents)
                            .iter()
                            .map(|tx| tx.transform::<#outer_type>())
                            .collect::<Vec<_>>();
                        all_inherents.extend(inherents);
                    }
                )*

                // Return the aggregate of all inherent extrinsics from all constituent constraint checkers.
                all_inherents
            }

            fn check_inherents(
                importing_inherent_data: &sp_inherents::InherentData,
                inherents: Vec<tuxedo_core::types::Transaction<#verifier, #outer_type>>,
                result: &mut sp_inherents::CheckInherentsResult,
            ) {
                #(
                    let relevant_inherents: Vec<tuxedo_core::types::Transaction<#verifier, #inner_types4>> = inherents
                        .iter()
                        .filter_map(|tx| {
                            match tx.checker {
                                #outer_type::#variants4(ref inner_checker) => Some(tx.transform::<#inner_types4>()),
                                _ => None,
                            }
                        })
                        .collect();

                    <#inner_types4 as tuxedo_core::ConstraintChecker<#verifier>>::InherentHooks::check_inherents(importing_inherent_data, relevant_inherents, result);

                    // According to https://paritytech.github.io/polkadot-sdk/master/sp_inherents/struct.CheckInherentsResult.html
                    // "When a fatal error occurs, all other errors are removed and the implementation needs to abort checking inherents."
                    if result.fatal_error() {
                        return;
                    }
                )*
            }

            #[cfg(feature = "std")]
            fn genesis_transactions() -> Vec<tuxedo_core::types::Transaction<#verifier, #outer_type>> {
                let mut all_transactions: Vec<tuxedo_core::types::Transaction<#verifier, #outer_type>> = Vec::new();

                #(
                    let transactions = <<#inner_types6 as tuxedo_core::ConstraintChecker<#verifier>>::InherentHooks as tuxedo_core::inherents::InherentInternal<#verifier, #inner_types6>>::genesis_transactions();
                    all_transactions.extend(
                        transactions
                            .into_iter()
                            .map(|tx| tx.transform::<#outer_type>())
                            .collect::<Vec<_>>()
                    );
                )*

                all_transactions
            }

        }

        impl tuxedo_core::ConstraintChecker<#verifier> for #outer_type {
            type Error = #error_type;

            type InherentHooks = #inherent_hooks;

            fn check (
                &self,
                inputs: &[tuxedo_core::types::Output<#verifier>],
                peeks: &[tuxedo_core::types::Output<#verifier>],
                outputs: &[tuxedo_core::types::Output<#verifier>],
            ) -> Result<TransactionPriority, Self::Error> {
                match self {
                    #(
                        Self::#variants5(inner) => inner.check(inputs, peeks, outputs).map_err(|e| Self::Error::#variants5(e)),
                    )*
                }
            }

            fn is_inherent(&self) -> bool {
                match self {
                    #(
                        Self::#variants6(inner) => <#inner_types7 as tuxedo_core::ConstraintChecker<#verifier>>::is_inherent(inner),
                    )*
                }

            }

        }
    };

    output.into()
}
