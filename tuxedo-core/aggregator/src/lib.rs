use proc_macro::{TokenStream};
use quote::quote;
use syn::{parse_macro_input, ItemEnum, Ident};

#[proc_macro_attribute]
pub fn aggregate(attrs: TokenStream, body: TokenStream) -> TokenStream {

    let ast = parse_macro_input!(body as ItemEnum);
    // TODO get the type name to impl the trait
    // let type_name = ast.ident.into();
    let original_code = ast.clone();

    // Uncomment this to inspect the ast of the original code.
    // eprintln!("{:#?}", ast);

    let outer_type = ast.ident;
    let variant_type_pairs = ast
        .variants
        .iter()
        .map(|variant| {
            // Make sure there is only a single field, and if not, give a helpful error
            assert!(variant.fields.len() == 1, "Each variant must have a single unnamed field");
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
    let types = variant_type_pairs.map(|(_v, t)| t);

    // The macro supports automatically implementing Tuxedo-related traits.
    // Currently two traits (Verifier and ConstraintChecker) are supported,
    // as well as no trait auto-implementation. More traits may be supported in the future.
    let mut tuxedo_trait_impl = quote!{};
    
    if !attrs.is_empty() {
        let attrs_tree = parse_macro_input!(attrs as Ident);
        tuxedo_trait_impl = if ident_is_named(&attrs_tree, "Verifier") {
                println!("TODO implement verifier");
                quote!{
                    todo!("impl Verifier for the type");
                }
            } else if ident_is_named(&attrs_tree, "ConstraintChecker") {
                println!("TODO implement ConstraintChecker");
                quote!{
                    todo!("impl ConstraintChecker for the type");
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
            impl From<#types> for #outer_type {
                fn from(b: #types) -> Self {
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