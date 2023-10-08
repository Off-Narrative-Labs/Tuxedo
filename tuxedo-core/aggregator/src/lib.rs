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
    let inner_types = variant_type_pairs.map(|(_v, t)| t);

    let output = quote! {
        // First keep the original code in tact
        #original_code

        // Now write all the From impls
        #(
            impl From<#inner_types> for #outer_type {
                fn from(b: #inner_types) -> Self {
                    Self::#variants(b)
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
/// trait for each of the inner types. Then it implements the `ConstraintChecker` trait for this
/// enum by delegating to an inner type.
///
/// In order to implement the `ConstraintChecker` trait, this macro must declare a few additional associated
/// aggregate types including an aggregate error enum and an aggregate accumulator struct, and this accumulator's
/// associated value type.
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

    let mut accumulator_type_name = outer_type.to_string();
    accumulator_type_name.push_str("Accumulator");
    let accumulator_type = Ident::new(&accumulator_type_name, outer_type.span());

    let mut accumulator_value_type_name = outer_type.to_string();
    accumulator_value_type_name.push_str("AccumulatorValueType");
    let accumulator_value_type = Ident::new(&accumulator_value_type_name, outer_type.span());

    let vis = ast.vis;
    let inner_types = inner_types.clone();
    let inner_types2 = inner_types.clone();
    let inner_types3 = inner_types.clone();
    let inner_types4 = inner_types.clone();
    let inner_types5 = inner_types.clone();
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
        /// It is an aggregate error type associated with the aggregate constraint checker.
        /// It has a variant that encapsulates the error type of each individual constituent constraint checker.
        ///
        /// This type is accessible downstream as `<OuterConstraintChecker as ConstraintChecker>::Error`
        #[derive(Debug)]
        #vis enum #error_type {
            #(
                #variants(<#inner_types as tuxedo_core::ConstraintChecker<#verifier>>::Error),
            )*
        }

        /// This type is generated by the `#[tuxedo_constraint_checker]` macro.
        /// It is an aggregate accumulator type associated with the aggregate constraint checker.
        ///
        /// This type is accessible downstream as `<OuterConstraintChecker as ConstraintChecker>::Error`
        #[derive(Debug)]
        #vis struct #accumulator_type;

        /// This type is generated by the `#[tuxedo_constraint_checker]` macro.
        /// It is an aggregate enum with a variant for the associated value type for each constituent accumulator.
        #[derive(Debug)]
        // TODO conflicting impls here too? I'm really not getting this error
        // Supposedly it conflicts with this from core: impl<T> From<T> for T;
        // #[tuxedo_core::aggregate]
        #vis enum #accumulator_value_type {
            #(
                #variants2(<<#inner_types2 as tuxedo_core::ConstraintChecker<#verifier>>::Accumulator as tuxedo_core::constraint_checker::Accumulator>::ValueType),
            )*
        }

        impl tuxedo_core::constraint_checker::Accumulator for #accumulator_type {
            type ValueType = #accumulator_value_type;

            fn initial_value(intermediate: Self::ValueType) -> Self::ValueType {
                match intermediate {
                    #(
                        Self::ValueType::#variants3(inner) => Self::ValueType::#variants3(<<#inner_types3 as tuxedo_core::ConstraintChecker<#verifier>>::Accumulator as tuxedo_core::constraint_checker::Accumulator>::initial_value(inner)),
                    )*
                }
            }

            fn key_path(intermediate: Self::ValueType) -> & 'static str {
                match intermediate {
                    #(
                        Self::ValueType::#variants4(inner) => <<#inner_types4 as tuxedo_core::ConstraintChecker<#verifier>>::Accumulator as tuxedo_core::constraint_checker::Accumulator>::key_path(inner),
                    )*
                }
            }

            fn accumulate(acc: Self::ValueType, next: Self::ValueType) -> Result<Self::ValueType, ()> {
                match (acc, next) {
                    #(
                        (Self::ValueType::#variants5(inner_acc), Self::ValueType::#variants5(inner_next)) => {
                            <<#inner_types5 as tuxedo_core::ConstraintChecker<#verifier>>::Accumulator as tuxedo_core::constraint_checker::Accumulator>::accumulate(inner_acc, inner_next)
                                .map(|inner_result| {
                                    Self::ValueType::#variants5(inner_result)
                                })
                        }
                    )*
                }
            }
        }

        impl tuxedo_core::ConstraintChecker<#verifier> for #outer_type {
            type Error = #error_type;
            type Accumulator = #accumulator_type;

            fn check (
                &self,
                inputs: &[tuxedo_core::types::Output<#verifier>],
                peeks: &[tuxedo_core::types::Output<#verifier>],
                outputs: &[tuxedo_core::types::Output<#verifier>],
            ) -> Result<tuxedo_core::constraint_checker::ConstraintCheckingSuccess<<Self::Accumulator as tuxedo_core::constraint_checker::Accumulator>::ValueType>, Self::Error> {
                match self {
                    #(
                        Self::#variants6(inner) => inner.check(inputs, peeks, outputs)
                        .map(|old| {
                            //TODO I would really rather have an into or from impl for ConstraintCheckingSuccess, but that's just won't compile
                            tuxedo_core::constraint_checker::ConstraintCheckingSuccess {
                                priority: old.priority,
                                accumulator_value: old.accumulator_value.into(),
                            }
                        })
                        .map_err(|e| Self::Error::#variants6(e)),
                    )*
                }
            }
        }
    };

    output.into()
}
