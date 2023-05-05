use dex::DexError::{self, *};


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