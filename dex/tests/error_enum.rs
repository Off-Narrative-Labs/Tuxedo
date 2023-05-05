use dex::DexError::{self, *};
use tuxedo_core::dynamic_typing::DynamicTypingError;


#[test]
fn error_enum_has_right_variants_for_making_orders() {
    fn _match_outer_constraint_checker(e: DexError) {
        match e {
            TypeError => (),
            OrderMissing => (),
            TooManyOutputsWhenMakingOrder => (),
            NotEnoughCollateralToOpenOrder => (),
            // We allow the possibility of more variants existing because some
            // learners will have brainstormed errors related to matching orders.
            //
            // Learners who only included these four will not have any other patterns
            // and we don't want to throw a warning, so we suppress it.
            #[allow(unreachable_patterns)]
            _ => (),
        }
    }
}

#[test]
fn from_dynamic_typing_error_is_implemented_properly_for_decoding_failed() {
    let dte = DynamicTypingError::DecodingFailed;
    let de: DexError = dte.into();
    assert_eq!(de, DexError::TypeError);
}

#[test]
fn from_dynamic_typing_error_is_implemented_properly_for_wrong_type() {
    let dte = DynamicTypingError::WrongType;
    let de: DexError = dte.into();
    assert_eq!(de, DexError::TypeError);
}