use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemEnum};

#[proc_macro_attribute]
pub fn aggregate(attrs: TokenStream, body: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(body as ItemEnum);
    let original_code = ast.clone();

    // Uncomment this to inspect the ast of the original code.
    // eprintln!("{:#?}", ast);

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

    // The macro supports automatically implementing Tuxedo-related traits.
    // Currently two traits (Verifier and ConstraintChecker) are supported,
    // as well as no trait auto-implementation. More traits may be supported in the future.
    let mut tuxedo_trait_impl = quote! {};

    if !attrs.is_empty() {
        let variants = variants.clone();
        let attrs_tree = parse_macro_input!(attrs as Ident);

        tuxedo_trait_impl = if ident_is_named(&attrs_tree, "Verifier") {
            quote! {
                impl tuxedo_core::Verifier for #outer_type {
                    fn verify(&self, simplified_tx: &[u8], redeemer: &[u8]) -> bool {
                        match self {
                            #(
                                Self::#variants(inner) => inner.verify(simplified_tx, redeemer),
                            )*
                        }
                    }
                }
            }
        } else if ident_is_named(&attrs_tree, "ConstraintChecker") {
            let vis = ast.vis;
            let mut error_type_name = outer_type.to_string();
            error_type_name.push_str("Error");
            let error_type = Ident::new(&error_type_name, outer_type.span());
            let inner_types = inner_types.clone();
            let variants2 = variants.clone();
            quote! {

                /// This type is generated by the `#[aggregate(ConstraintChecker)]` macro.
                /// It is an aggregate error type for the errors of each individual checker.
                ///
                /// This type is accessible downstream as `<OuterConstraintChecker as ConstraintChecker>::Error`
                #[derive(Debug)]
                #vis enum #error_type {
                    #(
                        #variants(<#inner_types as tuxedo_core::ConstraintChecker>::Error),
                    )*
                }

                impl tuxedo_core::ConstraintChecker for #outer_type {
                    type Error = #error_type;

                    fn check<V: tuxedo_core::Verifier>(
                        &self,
                        inputs: &[tuxedo_core::types::Output<V>],
                        outputs: &[tuxedo_core::types::Output<V>],
                    ) -> Result<TransactionPriority, Self::Error> {
                        match self {
                            #(
                                Self::#variants2(inner) => inner.check(inputs, outputs).map_err(|e| Self::Error::#variants2(e)),
                            )*
                        }
                    }
                }
            }
        } else {
            //TODO, how to use the correct span, which is `attrs_tree.span()`?
            // maybe this will help https://stackoverflow.com/questions/54392702
            panic!("Auto-derive trait supplied is invalid. Only \"Verifier\" and \"ConstraintChecker\" are allowed");
        };
    }

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

        // Now implement the Tuxedo trait requested, if any
        #tuxedo_trait_impl
    };

    output.into()
}

/// This helper function tests whether an Ident has the given name.
/// This is basically a work around for the fact that the Ident type doesn't
/// provide access to the string name. So instead we construct a new Ident
/// and copy in the original span, then check for equality on the entire span.
fn ident_is_named(original: &Ident, name: &str) -> bool {
    let named = Ident::new(name, original.span());
    original == &named
}
